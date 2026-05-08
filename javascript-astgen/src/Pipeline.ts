import * as babelParser from "@babel/parser"
import * as path from "node:path"

import Options from "./Options"
import * as Defaults from "./Defaults"
import * as FileUtils from "./FileUtils"
import * as Logger from "./Logger"
import * as Parsing from "./Parsing"
import * as Writers from "./Writers"
import {SourceDirNotReadableError} from "./Errors"
import {FsWriteSink, WriteSink} from "./WriteSink"
import TscUtils from "./TscUtils"

/**
 * Executes an async function and swallows any exception, logging it as a
 * warning instead of propagating. Used at the per-file boundary so that one
 * file's failure does not abort the entire AST generation run.
 *
 * @param errMessage - The error message prefix logged when an exception occurs.
 * @param arg - An optional argument for log identification (typically the file path).
 * @param f - The async function to execute.
 */
async function runOrLogWarning(errMessage: string, arg: string | undefined, f: () => Promise<void>): Promise<void> {
    try {
        await f()
    } catch (err) {
        const detail = err instanceof Error ? err.message : String(err)
        if (arg !== undefined && arg.length > 0) {
            Logger.warn(errMessage, arg, ":", detail)
        } else {
            Logger.warn(errMessage, ":", detail)
        }
    }
}

/**
 * Builds a TscUtils instance to process TypeScript type information for the given files.
 *
 * @returns A TscUtils instance, or undefined if type extraction is disabled,
 *          the files array is empty, or initialization fails.
 */
function buildTscUtils(files: string[], options: Options): TscUtils | undefined {
    if (!options.tsTypes || files.length === 0) return undefined
    try {
        return new TscUtils(files)
    } catch (err) {
        const detail = err instanceof Error ? err.message : String(err)
        Logger.warn("Retrieving types :", detail)
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
 * @param sink - Write target for AST and typemap output.
 */
async function createJSAst(options: Options, sink: WriteSink): Promise<void> {
    const filePaths = await processAstFiles(
        FileUtils.filesWithExtensions(options, Defaults.JS_TS_EXTENSIONS),
        options,
        sink,
    )
    const tscUtils = buildTscUtils(filePaths, options)
    if (tscUtils) {
        await processTypeFiles(filePaths, options, sink, tscUtils)
    }
}

/**
 * Generates Abstract Syntax Trees (ASTs) for all `.vue` files in the specified source directory.
 */
async function createVueAst(options: Options, sink: WriteSink): Promise<void> {
    for await (const file of FileUtils.filesWithExtensions(options, [".vue"])) {
        await runOrLogWarning("Parsing", file.path, async () => {
            await Writers.writeAstFile(file.path, Parsing.toVueAst(file.content), options, sink)
        })
    }
}

async function processAstFiles(
    source: AsyncIterable<FileUtils.FileEntry>,
    options: Options,
    sink: WriteSink,
): Promise<string[]> {
    const filePaths: string[] = []
    for await (const file of source) {
        filePaths.push(file.path)
        await runOrLogWarning("Parsing", file.path, async () => {
            const ast: babelParser.ParseResult = Parsing.codeToJsAst(file.content)
            await Writers.writeAstFile(file.path, ast, options, sink)
        })
    }
    return filePaths
}

async function processTypeFiles(
    filePaths: string[],
    options: Options,
    sink: WriteSink,
    tscUtils: TscUtils,
): Promise<void> {
    for (const filePath of filePaths) {
        await runOrLogWarning("Retrieving types", filePath, async () => {
            const typeMap = tscUtils.typeMapForFile(filePath)
            if (typeMap.size !== 0) {
                await Writers.writeTypesFile(filePath, typeMap, options, sink)
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
 */
async function createXAst(options: Options, sink: WriteSink): Promise<void> {
    const srcDir: string = options.src
    const isKnownJsProject =
        FileUtils.fileExistsAndIsReadable(path.join(srcDir, "package.json")) ||
        FileUtils.fileExistsAndIsReadable(path.join(srcDir, "rush.json"))
    if (!isKnownJsProject) {
        Logger.warn("No package.json or rush.json found in", srcDir, "— processing as JS/TS project")
    }
    return await createJSAst(options, sink)
}

/**
 * Entry point for starting the AST generation process based on the provided options.
 *
 * Exceptions raised by sub-pipelines (other than per-file errors swallowed by
 * {@link runOrLogWarning}) propagate to the caller. The CLI in
 * [astgen.ts](./astgen.ts) translates them to a non-zero exit code; tests can
 * assert on them directly.
 *
 * @param options - Configuration options and CLI arguments controlling source location, output, and processing type.
 * @param sink - Optional write target. Defaults to a fresh {@link FsWriteSink}; tests can pass an in-memory sink.
 * @returns A Promise that resolves when the AST generation process is complete.
 * @throws {SourceDirNotReadableError} when `options.src` does not exist or is not readable.
 */
export default async function start(options: Options, sink: WriteSink = new FsWriteSink()): Promise<void> {
    const srcDir = options.src
    if (!FileUtils.fileExistsAndIsReadable(srcDir)) {
        throw new SourceDirNotReadableError(srcDir)
    }

    const type: string = (options.type || "").toLowerCase()
    switch (type) {
        case "nodejs":
        case "js":
        case "javascript":
        case "typescript":
        case "ts":
            return await createJSAst(options, sink)
        case "vue":
            return await createVueAst(options, sink)
        default:
            return await createXAst(options, sink)
    }
}
