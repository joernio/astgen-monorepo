import Options from "./Options"
import * as Defaults from "./Defaults"
import * as Logger from "./Logger"

import * as fs from "node:fs"
import * as path from "node:path"

export type FileEntry = { path: string; content: string }

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

// Validates a file's content (EMSCRIPTEN marker, line length, total LOC) after it
// has been read. The size guard is performed earlier from stats to avoid reading
// oversized files. Returns the content if valid, or null with a warning otherwise.
async function readAndValidateContent(fileWithDir: string): Promise<string | null> {
    const content = await fs.promises.readFile(fileWithDir, "utf-8")
    if (content.includes("// EMSCRIPTEN_START_ASM")) {
        Logger.warn("Parsing", fileWithDir, ":", "File skipped as it contains EMSCRIPTEN code")
        return null
    }
    let lineStart = 0
    let lineCount = 0
    for (let i = 0; i <= content.length; i++) {
        if (i === content.length || content[i] === "\n") {
            if (i - lineStart > Defaults.MAX_LINE_LENGTH) {
                Logger.warn(fileWithDir, "line", lineCount + 1, "exceeds", Defaults.MAX_LINE_LENGTH, "bytes")
                return null
            }
            if (++lineCount > Defaults.MAX_LOC_IN_FILE) {
                Logger.warn(fileWithDir, "more than", Defaults.MAX_LOC_IN_FILE, "lines of code")
                return null
            }
            lineStart = i + 1
        }
    }
    return content
}

// Cheap shared iterator used by filesWithExtensions. Applies name-based filters
// during the readdirp walk and the size guard from stats before content reads.
//
// `options.src` is normalized via path.resolve so the invariant "fullPath is
// absolute" is explicit and doesn't depend on readdirp's internal behavior.
async function* iterateMatchingEntries(
    options: Options,
    extensions: string[],
): AsyncGenerator<{ path: string }> {
    const dir = path.resolve(options.src)
    const excludeRules = buildExcludeRules(options)
    // Dynamic import for ESM-only package.
    const {readdirp} = await import('readdirp')
    const stream = readdirp(dir, {
        root: dir,
        fileFilter: (f) => !ignoreFileByName(excludeRules, f.basename, f.fullPath, extensions),
        directoryFilter: (d) => !ignoreDirectory(excludeRules, d.basename, d.fullPath),
        lstat: true,
        alwaysStat: true,
        depth: options.recurse ? undefined : 0,
    })
    for await (const entry of stream) {
        const stats = entry.stats as fs.Stats
        if (stats.size > Defaults.MAX_FILE_SIZE_BYTES) {
            Logger.warn(entry.fullPath, "exceeds maximum file size of", Defaults.MAX_FILE_SIZE_BYTES, "bytes")
            continue
        }
        yield {path: entry.fullPath}
    }
}

/**
 * Asynchronously yields source file entries (path + content) matching the
 * given extensions and passing every exclusion and content-validation rule.
 * Reads each file once and yields immediately.
 *
 * @param options - The options object containing source directory and exclusion patterns.
 * @param extensions - An array of file extensions to include (e.g., ['.js', '.ts']).
 * @returns An async generator that yields FileEntry objects for matching files.
 */
export async function* filesWithExtensions(options: Options, extensions: string[]): AsyncGenerator<FileEntry> {
    for await (const entry of iterateMatchingEntries(options, extensions)) {
        const content = await readAndValidateContent(entry.path)
        if (content !== null) {
            yield {path: entry.path, content}
        }
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
