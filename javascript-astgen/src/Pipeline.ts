import * as path from "node:path"

import Options from "./Options"
import * as Defaults from "./Defaults"
import * as FileUtils from "./FileUtils"
import * as Logger from "./Logger"
import * as Writers from "./Writers"
import {getErrorMessage, SourceDirNotReadableError} from "./Errors"
import {FsWriteSink, WriteSink} from "./WriteSink"
import TscUtils from "./TscUtils"
import {PARSER_JS, PARSER_VUE, type ParserKind} from "./AstWorker"
import {defaultPoolSize, WorkerPool} from "./WorkerPool"

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
        Logger.warn("Retrieving types :", getErrorMessage(err))
        return undefined
    }
}

function parserFor(filePath: string): ParserKind {
    return filePath.endsWith(Defaults.VUE_EXTENSION) ? PARSER_VUE : PARSER_JS
}

/**
 * Dispatches every matching source file to the worker pool. Each worker runs
 * read+validate+parse+write end-to-end so the multi-MB ParseResult never
 * crosses the thread boundary — only the per-file Done/Error envelope does.
 * Peak memory stays bounded by pool size × per-worker single-file buffering.
 *
 * The returned `filePaths` list feeds the type-extraction phase (run on the
 * main thread) when `tsTypes` is enabled.
 *
 * @param options - Configuration options controlling source location and output.
 * @param sink - Write target. Used for `ensureDir` only; AST writes go straight
 *               from the worker to disk via JsonUtils.
 * @param extensions - Source extensions to process this run.
 */
async function processAstFilesParallel(
    options: Options,
    sink: WriteSink,
    extensions: string[],
): Promise<string[]> {
    const pool = new WorkerPool(defaultPoolSize())
    const filePaths: string[] = []
    const inflight: Promise<void>[] = []
    try {
        for await (const filePath of FileUtils.pathsWithExtensions(options, extensions)) {
            const {relativePath, outputPath} = FileUtils.outputPathFor(options.src, options.output, filePath, ".json")
            await sink.ensureDir(path.dirname(outputPath))
            const job = {file: filePath, relativePath, outputPath, parser: parserFor(filePath)}
            inflight.push(
                pool.submit(job).then((msg) => {
                    if (msg.kind === "error") {
                        Logger.warn("Parsing", msg.file, ":", msg.message)
                    } else if (msg.skipped) {
                        Logger.warn("Parsing", msg.file, ":", msg.skipped)
                    } else {
                        // Only files that actually produced an AST feed the
                        // type-extraction phase. A file skipped by validateBuffer
                        // (e.g. a huge single-line minified bundle) has no AST, so
                        // running tsc over it would be pointless work — and, on
                        // pathological input, hang the TypeChecker traversal.
                        filePaths.push(filePath)
                        Logger.info("Converted AST for", relativePath, "to", outputPath)
                    }
                }),
            )
        }
        await Promise.all(inflight)
    } finally {
        await pool.shutdown()
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
        try {
            const typeMap = tscUtils.typeMapForFile(filePath)
            if (typeMap.size !== 0) {
                await Writers.writeTypesFile(filePath, typeMap, options, sink)
            }
        } catch (err) {
            Logger.warn("Retrieving types", filePath, ":", getErrorMessage(err))
        }
    }
}

function extensionsFor(type: string, srcDir: string): string[] {
    switch (type) {
        case "nodejs":
        case "js":
        case "javascript":
        case "typescript":
        case "ts":
            return Defaults.JS_TS_EXTENSIONS
        case "vue":
            return [Defaults.VUE_EXTENSION]
        default: {
            const isKnownJsProject =
                FileUtils.fileExistsAndIsReadable(path.join(srcDir, "package.json")) ||
                FileUtils.fileExistsAndIsReadable(path.join(srcDir, "rush.json"))
            if (!isKnownJsProject) {
                Logger.warn("No package.json or rush.json found in", srcDir, "— processing as JS/TS project")
            }
            return Defaults.JS_TS_EXTENSIONS
        }
    }
}

/**
 * Entry point for starting the AST generation process based on the provided options.
 *
 * Per-file failures are caught inside the worker (parse/write errors) or this
 * function (typemap errors) and surfaced as warnings; everything else
 * propagates. The CLI in [astgen.ts](./astgen.ts) translates uncaught errors
 * to a non-zero exit code; tests can assert on them directly.
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

    const type = (options.type || "").toLowerCase()
    const extensions = extensionsFor(type, srcDir)
    const filePaths = await processAstFilesParallel(options, sink, extensions)
    // Type extraction only makes sense for JS/TS projects — TscUtils runs the
    // TypeScript compiler over the input files, which doesn't apply to .vue.
    if (type === "vue") return
    const tscUtils = buildTscUtils(filePaths, options)
    if (tscUtils) {
        await processTypeFiles(filePaths, options, sink, tscUtils)
    }
}
