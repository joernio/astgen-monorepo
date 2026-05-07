import Options from "./Options"
import * as Defaults from "./Defaults"
import * as FileUtils from "./FileUtils"
import * as JsonUtils from "./JsonUtils"
import * as VueCodeCleaner from "./VueCodeCleaner"
import TscUtils, {TypeMap} from "./TscUtils"

import * as babelParser from "@babel/parser"
import * as path from "node:path"
import * as fs from "node:fs"

/**
 * Executes an async function and swallows any exception, logging it as a
 * warning instead of propagating. Used at the per-file boundary so that one
 * file's failure does not abort the entire AST generation run.
 *
 * @param errMessage - The error message prefix logged when an exception occurs.
 * @param arg - An argument that provides better identification in the log (typically the file path).
 * @param f - The async function to execute.
 */
async function runOrLogWarning(errMessage: string, arg: string, f: () => Promise<void>): Promise<void> {
    try {
        await f()
    } catch (err) {
        if (err instanceof Error) {
            console.warn(errMessage, arg, ":", err.message)
        }
    }
}

/**
 * Converts a JavaScript or TypeScript code string to an Abstract Syntax Tree (AST).
 *
 * The function first attempts to parse the code with standard Babel parser options.
 * If the initial parsing fails (e.g., with experimental syntax), it automatically
 * falls back to a more permissive set of parsing options.
 *
 * @param code - The JavaScript or TypeScript code string to be parsed
 * @returns A Babel ParseResult object representing the AST of the provided code
 * @throws May throw an error if parsing fails with both standard and fallback options
 * @see Defaults.BABEL_PARSER_OPTIONS - The primary parsing configuration
 * @see Defaults.SAFE_BABEL_PARSER_OPTIONS - The fallback parsing configuration
 */
function codeToJsAst(code: string): babelParser.ParseResult {
    try {
        return babelParser.parse(code, Defaults.BABEL_PARSER_OPTIONS)
    } catch {
        return babelParser.parse(code, Defaults.SAFE_BABEL_PARSER_OPTIONS)
    }
}

/**
 * Converts pre-read Vue file content to an Abstract Syntax Tree (AST).
 *
 * This function cleans the code using the VueCodeCleaner utility to extract and
 * process the script section, then parses the cleaned code into an AST using Babel.
 *
 * @param content - The raw content of the Vue file
 * @returns A Babel ParseResult object representing the AST of the Vue file's script content
 * @throws Will throw an error if parsing fails
 * @see VueCodeCleaner.cleanVueCode - The utility used to extract script content from Vue files
 * @see codeToJsAst - The underlying function used for parsing the extracted code
 */
function toVueAst(content: string): babelParser.ParseResult {
    return codeToJsAst(VueCodeCleaner.cleanVueCode(content))
}

/**
 * Builds a TscUtils instance to process TypeScript type information for the given files.
 *
 * This function creates a TscUtils object that can be used to extract and analyze
 * TypeScript type information. It only proceeds if type extraction is enabled in
 * the options and there are files to process.
 *
 * @param files - An array of file paths to be analyzed for TypeScript types
 * @param options - Configuration options object that controls the behavior
 * @returns A TscUtils instance if successful, or undefined if type
 *          extraction is disabled, files array is empty, or an error occurs during initialization
 * @see TscUtils - The utility class used for TypeScript type extraction
 */
function buildTscUtils(files: string[], options: Options): TscUtils | undefined {
    if (!options.tsTypes || files.length === 0) return undefined
    try {
        return new TscUtils(files)
    } catch (err) {
        if (err instanceof Error) {
            console.warn("Retrieving types", "", ":", err.message)
        }
        return undefined
    }
}

/**
 * Generates Abstract Syntax Trees (ASTs) for JavaScript and TypeScript source files.
 *
 * Walks matching sources sequentially (`for await`). Each file's contents are parsed
 * and written and then discarded; they are never held for all files at once.
 * Paths are appended to `filePaths` during that pass — when the `tsTypes` option is
 * off the list is unused afterward; when on it feeds a second sequential phase that
 * loads typemaps from disk (`tsc`) without rereading file bodies through this traversal.
 *
 * Peak memory stays bounded primarily by single-file buffering, the JSON writer backlog
 * cap, and (when enabled) incremental type generation rather than buffering every source string.
 *
 * @param options - Configuration options controlling source location, output, and type extraction.
 */
async function createJSAst(options: Options): Promise<void> {
    try {
        const dirCache = new DirCache()
        const filePaths = await processAstFiles(
            FileUtils.filesWithExtensions(options, Defaults.JS_TS_EXTENSIONS),
            options,
            dirCache,
        )
        const tscUtils = buildTscUtils(filePaths, options)
        if (tscUtils) {
            await processTypeFiles(filePaths, options, dirCache, tscUtils)
        }
    } catch (err) {
        console.error(err)
    }
}

/**
 * Generates Abstract Syntax Trees (ASTs) for all `.vue` files in the specified source directory.
 *
 * @param options - Configuration options controlling source location and output directory.
 * @returns A Promise that resolves when all Vue files have been processed.
 */
async function createVueAst(options: Options): Promise<void> {
    const dirCache = new DirCache()
    for await (const file of FileUtils.filesWithExtensions(options, [".vue"])) {
        await runOrLogWarning("Parsing", file.path, async () => {
            await writeAstFile(file.path, toVueAst(file.content), options, dirCache)
        })
    }
}

/**
 * Writes the AST (Abstract Syntax Tree) data of a source file to a JSON file.
 *
 * The output file is created in the output directory specified in the options,
 * preserving the relative path structure from the source directory. The AST data
 * is serialized using a utility that handles circular references.
 * Output directories are created at most once per unique path via `dirCache`.
 *
 * @param file - The absolute path to the source file.
 * @param ast - The Babel ParseResult object representing the AST of the file.
 * @param options - Configuration options containing source and output directories.
 * @param dirCache - Cache that deduplicates concurrent mkdir calls for the same directory.
 */
async function writeAstFile(
    file: string,
    ast: babelParser.ParseResult,
    options: Options,
    dirCache: DirCache,
): Promise<void> {
    const relativePath: string = path.relative(options.src, file)
    const outAstFile: string = path.join(options.output, relativePath + ".json")
    const data = {
        fullName: file,
        relativeName: relativePath,
        ast: ast,
    }
    await dirCache.ensure(path.dirname(outAstFile))
    JsonUtils.writeJsonStreamCircular(outAstFile, data)
    console.log("Converted AST for", relativePath, "to", outAstFile)
}

/**
 * Writes TypeScript type information to a JSON file.
 *
 * @param file - The absolute path to the source file.
 * @param seenTypes - The `TypeMap` containing type information to be written.
 * @param options - Configuration options containing source and output directories.
 * @param dirCache - Cache that deduplicates concurrent mkdir calls for the same directory.
 */
async function writeTypesFile(
    file: string,
    seenTypes: TypeMap,
    options: Options,
    dirCache: DirCache,
): Promise<void> {
    const relativePath: string = path.relative(options.src, file)
    const outTypeFile: string = path.join(options.output, relativePath + ".typemap")
    await dirCache.ensure(path.dirname(outTypeFile))
    JsonUtils.writeMapToJsonFile(outTypeFile, seenTypes)
    console.log("Converted types for", relativePath, "to", outTypeFile)
}

/**
 * Tracks which output directories have been created and deduplicates concurrent
 * mkdir requests. Without this, two concurrent file writes targeting the same
 * directory would each issue a redundant (idempotent) `mkdir -p` syscall.
 *
 * On mkdir failure the inflight slot is cleared (via `.finally`) so a transient
 * error (e.g. EMFILE under high concurrency) does not poison the cache and
 * permanently fail every subsequent write to the same directory.
 */
class DirCache {
    private readonly created = new Set<string>()
    private readonly inflight = new Map<string, Promise<void>>()

    async ensure(dir: string): Promise<void> {
        if (this.created.has(dir)) return
        let pending = this.inflight.get(dir)
        if (!pending) {
            pending = fs.promises.mkdir(dir, {recursive: true})
                .then(() => { this.created.add(dir) })
                .finally(() => { this.inflight.delete(dir) })
            this.inflight.set(dir, pending)
        }
        await pending
    }
}

async function processAstFiles(
    source: AsyncIterable<FileUtils.FileEntry>,
    options: Options,
    dirCache: DirCache,
): Promise<string[]> {
    const filePaths: string[] = []
    for await (const file of source) {
        filePaths.push(file.path)
        await runOrLogWarning("Parsing", file.path, async () => {
            const ast: babelParser.ParseResult = codeToJsAst(file.content)
            await writeAstFile(file.path, ast, options, dirCache)
        })
    }
    return filePaths
}

async function processTypeFiles(
    filePaths: string[],
    options: Options,
    dirCache: DirCache,
    tscUtils: TscUtils,
): Promise<void> {
    for (const filePath of filePaths) {
        await runOrLogWarning("Retrieving types", filePath, async () => {
            const typeMap = tscUtils.typeMapForFile(filePath)
            if (typeMap.size !== 0) {
                await writeTypesFile(filePath, typeMap, options, dirCache)
            }
        })
    }
}

/**
 * Determines the project type in the given source directory and triggers AST generation accordingly.
 *
 * This function checks if the provided source directory contains a `package.json` or `rush.json` file
 * to identify it as a Node.js or JavaScript/TypeScript project. If neither marker is found, it logs
 * a warning and falls back to JS/TS processing rather than exiting.
 *
 * @param options - Configuration options containing the source directory and output settings.
 * @returns A Promise that resolves when AST generation is complete.
 */
async function createXAst(options: Options): Promise<void> {
    const srcDir: string = options.src
    const isKnownJsProject =
        FileUtils.fileExistsAndIsReadable(path.join(srcDir, "package.json")) ||
        FileUtils.fileExistsAndIsReadable(path.join(srcDir, "rush.json"))
    if (!isKnownJsProject) {
        console.warn("No package.json or rush.json found in", srcDir, "— processing as JS/TS project")
    }
    return await createJSAst(options)
}

/**
 * Entry point for starting the AST generation process based on the provided options.
 *
 * @param options - Configuration options and CLI arguments controlling source location, output, and processing type.
 * @returns A Promise that resolves when the AST generation process is complete.
 */
export default async function start(options: Options): Promise<void> {
    const srcDir = options.src
    if (!FileUtils.fileExistsAndIsReadable(srcDir)) {
        console.error("Source directory does not exist or is not readable:", srcDir)
        process.exit(1)
    }

    const type: string = (options.type || "").toLowerCase()
    switch (type) {
        case "nodejs":
        case "js":
        case "javascript":
        case "typescript":
        case "ts":
            return await createJSAst(options)
        case "vue":
            return await createVueAst(options)
        default:
            return await createXAst(options)
    }
}
