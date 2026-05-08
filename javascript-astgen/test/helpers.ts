import * as fs from "node:fs"
import * as os from "node:os"
import * as path from "node:path"

import Options from "../src/Options"
import start from "../src/Pipeline"
import {Logger, setLogger, resetLogger} from "../src/Logger"

/**
 * Single source file fixture descriptor used by {@link withTmpProject}.
 *
 * `name` is interpreted relative to the temp directory; subdirectories are
 * created automatically.
 */
export type FileFixture = {
    name: string
    code: string
}

/**
 * Creates an isolated tmp directory, runs `body`, and unconditionally removes
 * the directory afterwards. Replaces the inline `mkdtempSync(...) + try/finally`
 * boilerplate that used to be duplicated across every multi-file test.
 */
export async function withTmpDir(body: (tmpDir: string) => Promise<void>): Promise<void> {
    const tmpDir: string = fs.mkdtempSync(path.join(os.tmpdir(), "astgen-tests"))
    try {
        await body(tmpDir)
    } finally {
        fs.rmSync(tmpDir, {recursive: true, force: true})
    }
}

/**
 * Writes a list of files (creating parent dirs as needed) into `dir`.
 */
export function writeFiles(dir: string, files: FileFixture[]): void {
    for (const f of files) {
        const abs = path.join(dir, f.name)
        fs.mkdirSync(path.dirname(abs), {recursive: true})
        fs.writeFileSync(abs, f.code)
    }
}

/**
 * Defaults applied by {@link setupProject} when the caller-supplied overrides
 * leave a field undefined.
 */
const DEFAULT_OPTIONS_BASE: Omit<Options, "src" | "output"> = {
    type: "js",
    recurse: true,
    tsTypes: true,
    "exclude-file": [],
}

/**
 * Single-file convenience wrapper preserving the historical `setupTestFixture`
 * contract used by older tests in this suite.
 */
export async function setupTestFixture(
    code: string,
    filename: string,
    options: Partial<Options>,
    body: (dir: string, testFile: string) => void | Promise<void>,
    excludeFiles: (dir: string, testFile: string) => string[] = () => [],
    runStart: (opts: Options) => Promise<void> = defaultRunStart,
): Promise<void> {
    return withTmpDir(async (tmpDir) => {
        const testFile: string = path.join(tmpDir, filename)
        writeFiles(tmpDir, [{name: filename, code}])

        const opts: Options = {
            ...DEFAULT_OPTIONS_BASE,
            src: tmpDir,
            output: path.join(tmpDir, "ast_out"),
            "exclude-file": excludeFiles(tmpDir, testFile),
            ...options,
        }
        await runStart(opts)
        await body(tmpDir, testFile)
    })
}

/**
 * Multi-file fixture entry-point. Use this for tests that need more than one
 * source file in the project (and therefore cannot use `setupTestFixture`).
 */
export async function setupProject(
    files: FileFixture[],
    options: Partial<Options>,
    body: (dir: string) => void | Promise<void>,
    runStart: (opts: Options) => Promise<void> = defaultRunStart,
): Promise<void> {
    return withTmpDir(async (tmpDir) => {
        writeFiles(tmpDir, files)
        const opts: Options = {
            ...DEFAULT_OPTIONS_BASE,
            src: tmpDir,
            output: path.join(tmpDir, "ast_out"),
            ...options,
        }
        await runStart(opts)
        await body(tmpDir)
    })
}

async function defaultRunStart(opts: Options): Promise<void> {
    await start(opts)
}

/**
 * Returns the `"start:end"` typemap key for the `occurrence`-th appearance of
 * `needle` in `source`. Tests can use this to assert types by name instead of
 * hardcoding byte offsets that drift on whitespace edits.
 *
 * @example
 * const key = findOffsets(code, "console.log") // "0:11"
 * expect(parsed[key]).toEqual("(...data: any[]) => void")
 */
export function findOffsets(source: string, needle: string, occurrence = 0): string {
    let from = 0
    for (let i = 0; i <= occurrence; i++) {
        const idx = source.indexOf(needle, from)
        if (idx === -1) {
            throw new Error(`findOffsets: needle ${JSON.stringify(needle)} not found at occurrence ${occurrence}`)
        }
        if (i === occurrence) return `${idx}:${idx + needle.length}`
        from = idx + 1
    }
    /* unreachable */ throw new Error("findOffsets: loop fell through")
}

/**
 * Captures all calls made through the {@link Logger} so tests can assert on
 * warning/error messages. Call {@link install} in `beforeEach` and
 * {@link uninstall} in `afterEach` (or use {@link withCapturedLogger}).
 */
export class MemoryLogger implements Logger {
    readonly info: jest.Mock = jest.fn()
    readonly warn: jest.Mock = jest.fn()
    readonly error: jest.Mock = jest.fn()

    install(): void {
        setLogger(this)
    }

    uninstall(): void {
        resetLogger()
    }

    /** Concatenates a single call's args the way `console.*` would. */
    private static format(call: unknown[]): string {
        return call.map((c) => (typeof c === "string" ? c : String(c))).join(" ")
    }

    /** Returns true if any captured warning contains every supplied substring. */
    warnedWith(...substrings: string[]): boolean {
        return this.warn.mock.calls.some((call: unknown[]) => {
            const formatted = MemoryLogger.format(call)
            return substrings.every((s) => formatted.includes(s))
        })
    }

    /** Returns true if any captured info call contains every supplied substring. */
    infoedWith(...substrings: string[]): boolean {
        return this.info.mock.calls.some((call: unknown[]) => {
            const formatted = MemoryLogger.format(call)
            return substrings.every((s) => formatted.includes(s))
        })
    }
}
