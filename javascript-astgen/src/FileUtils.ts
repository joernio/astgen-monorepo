import Options from "./Options"
import * as Defaults from "./Defaults"
import * as Logger from "./Logger"
import {getErrorMessage} from "./Errors"

import * as fs from "node:fs"
import * as path from "node:path"
import {readdirp} from "readdirp"

/**
 * Computes the per-file output path layout. Both `relativePath` (from `src`,
 * for log messages) and `outputPath` (joined under `output`, with the extra
 * suffix appended) are returned together so callers don't recompute either —
 * a single source file flows through ensureDir, the writer, and the log line
 * with one `path.relative` call.
 */
export function outputPathFor(
    src: string,
    output: string,
    file: string,
    suffix: string,
): { relativePath: string; outputPath: string } {
    const relativePath = path.relative(src, file)
    const outputPath = path.join(output, relativePath + suffix)
    return {relativePath, outputPath}
}

/**
 * Compiled exclusion rules for a single AST generation run.
 *
 * - `prefixPaths` are matched as **directory prefixes** (i.e. any candidate that
 *   resolves under one of these is excluded). Used by {@link ignoreDirectory}.
 * - `exactPaths` are matched **only as exact-equal absolute paths** by
 *   {@link ignoreFileByName}. A path can appear in both sets — files added via
 *   `--exclude-file` populate both so a single rule can exclude either a file
 *   or a directory tree without forcing the user to disambiguate.
 * - `regex` is applied to absolute paths of both files and directories.
 */
type ExcludeRules = {
    prefixPaths: string[]
    exactPaths: Set<string>
    regex?: RegExp
}

// Cached at module load. Defaults.IGNORE_DIRS is a constant; safe to memoize.
const IGNORE_DIRS_SET = new Set(Defaults.IGNORE_DIRS.map((d) => d.toLowerCase()))

function resolveExcludePath(srcDir: string, excludePath: string): string {
    return path.resolve(path.isAbsolute(excludePath) ? excludePath : path.join(srcDir, excludePath))
}

function buildExcludeRules(options: Options): ExcludeRules {
    const srcDir = path.resolve(options.src)
    const prefixPaths: string[] = []
    const exactPaths = new Set<string>()
    for (const excludePath of options["exclude-file"]) {
        const resolved = resolveExcludePath(srcDir, excludePath)
        prefixPaths.push(resolved)
        exactPaths.add(resolved)
    }
    return {prefixPaths, exactPaths, regex: options["exclude-regex"]}
}

// Note: path comparison is case-sensitive. On macOS/Windows (case-insensitive
// filesystems) an exclude path with a different case than the on-disk path
// will not match. Accept this trade-off rather than per-OS lowercasing.
function pathIsInDirectory(candidatePath: string, directoryPath: string): boolean {
    const relativePath = path.relative(directoryPath, candidatePath)
    return relativePath === "" || (!relativePath.startsWith("..") && !path.isAbsolute(relativePath))
}

function ignoreDirectory(rules: ExcludeRules, dirName: string, fullPath: string): boolean {
    return dirName.startsWith(".") ||
        dirName.startsWith("__") ||
        rules.prefixPaths.some((ignoredDir) => pathIsInDirectory(fullPath, ignoredDir)) ||
        rules.regex?.test(fullPath) ||
        IGNORE_DIRS_SET.has(dirName.toLowerCase())
}

function ignoreFileByName(
    rules: ExcludeRules,
    fileName: string,
    fullPath: string,
    extensions: string[],
): boolean {
    return !extensions.some((e: string) => fileName.endsWith(e)) ||
        fileName.startsWith(".") ||
        fileName.startsWith("__") ||
        Defaults.IGNORE_FILE_PATTERN.test(fileName) ||
        rules.exactPaths.has(fullPath) ||
        (rules.regex?.test(fullPath) ?? false)
}

const NEWLINE_BYTE = "\n".charCodeAt(0)
const EMSCRIPTEN_MARKER = Buffer.from("// EMSCRIPTEN_START_ASM")

export type ValidationResult =
    | { ok: true; content: string }
    | { ok: false; reason: string }

/**
 * Validates a file's bytes (EMSCRIPTEN marker, line length, total LOC) before
 * paying the UTF-8 decode cost. Files that fail validation (typically minified
 * bundles that slipped past the size guard) never become a JS string, which
 * avoids a multi-MB heap allocation in the rejection path.
 *
 * Pure and synchronous so it can run on the main thread or inside a worker
 * (see [AstWorker.ts](./AstWorker.ts)) without dragging in Logger or fs.
 * `reason` does not include the file path; callers should prefix that
 * themselves when surfacing the message.
 */
export function validateBuffer(buf: Buffer): ValidationResult {
    if (buf.indexOf(EMSCRIPTEN_MARKER) !== -1) {
        return {ok: false, reason: "File skipped as it contains EMSCRIPTEN code"}
    }
    let lineStart = 0
    let lineCount = 0
    // Buffer.indexOf is native (memchr) — much faster than a per-byte JS loop
    // on multi-MB files. A miss (-1) doubles as the tail-line check, so the
    // text after the last newline needs no separate code path. The tail counts
    // as a line: a MAX_LOC_IN_FILE+1-line file has MAX_LOC_IN_FILE newlines.
    while (true) {
        const newline = buf.indexOf(NEWLINE_BYTE, lineStart)
        const lineEnd = newline === -1 ? buf.length : newline
        if (lineEnd - lineStart > Defaults.MAX_LINE_LENGTH) {
            return {ok: false, reason: `line ${lineCount + 1} exceeds ${Defaults.MAX_LINE_LENGTH} bytes`}
        }
        if (++lineCount > Defaults.MAX_LOC_IN_FILE) {
            return {ok: false, reason: `more than ${Defaults.MAX_LOC_IN_FILE} lines of code`}
        }
        if (newline === -1) break
        lineStart = newline + 1
    }
    return {ok: true, content: buf.toString("utf-8")}
}

/**
 * Yields absolute paths of source files that survive name-based filters and
 * the size guard. Callers (the worker pool, see [Pipeline.ts](./Pipeline.ts))
 * are responsible for reading and validating bytes themselves so the multi-MB
 * Buffer stays off the main thread.
 *
 * `options.src` is normalized via `path.resolve` so the invariant "fullPath
 * is absolute" is explicit and doesn't depend on readdirp's internal behavior.
 */
export async function* pathsWithExtensions(
    options: Options,
    extensions: string[],
): AsyncGenerator<string> {
    const dir = path.resolve(options.src)
    const excludeRules = buildExcludeRules(options)
    // Run readdirp in dirent mode (alwaysStat omitted) so the walk does not
    // stat every entry — readdirp applies fileFilter AFTER stat, so enabling
    // alwaysStat would stat every node_modules JSON etc. before the
    // name-based filter rejects it. We stat ourselves below, only for entries
    // that survive the name filter, purely to enforce MAX_FILE_SIZE_BYTES
    // before the worker reads the file.
    const stream = readdirp(dir, {
        fileFilter: (f) => !ignoreFileByName(excludeRules, f.basename, f.fullPath, extensions),
        directoryFilter: (d) => !ignoreDirectory(excludeRules, d.basename, d.fullPath),
        depth: options.recurse ? undefined : 0,
    })
    for await (const entry of stream) {
        let size: number
        try {
            size = (await fs.promises.stat(entry.fullPath)).size
        } catch (err) {
            Logger.info("Parsing", entry.fullPath, ":", "unable to stat due to", getErrorMessage(err))
            continue
        }
        if (size > Defaults.MAX_FILE_SIZE_BYTES) {
            Logger.info("Parsing", entry.fullPath, ":", "exceeds maximum file size of", Defaults.MAX_FILE_SIZE_BYTES, "bytes")
            continue
        }
        yield entry.fullPath
    }
}

/**
 * Checks if the folder or file at the given path exists and is readable.
 *
 * @param path - The path to the folder or file to check.
 * @returns True if the folder or file exists and is readable; false otherwise.
 */
export function fileExistsAndIsReadable(path: string): boolean {
    try {
        fs.accessSync(path, fs.constants.R_OK)
        return true
    } catch (err) {
        return false
    }
}
