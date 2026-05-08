/**
 * Tiny logging seam used throughout astgen.
 *
 * The default implementation forwards to the matching `console.*` method, so the
 * runtime behaviour is identical to direct `console.warn(...)` / `console.error(...)`
 * calls. Tests (or future flags such as `--quiet` or `--json-log`) can call
 * {@link setLogger} to swap the implementation; capture calls; or silence output.
 *
 * Implementation notes:
 * - Module-level singleton instead of threading a logger through every function
 *   signature. The package is CLI-only (see README), so a process-global logger
 *   is fine; there is no embedding scenario where two concurrent loggers coexist.
 * - The signatures match `console.*` (variadic `unknown[]`) so call sites do not
 *   have to change shape when porting from `console.warn(...)`.
 */
export interface Logger {
    info(...args: unknown[]): void

    warn(...args: unknown[]): void

    error(...args: unknown[]): void
}

const consoleLogger: Logger = {
    info: (...args) => console.log(...args),
    warn: (...args) => console.warn(...args),
    error: (...args) => console.error(...args),
}

let current: Logger = consoleLogger

/**
 * Replaces the active logger. Returns the previous logger so callers can restore
 * it after a temporary override (e.g. in test `afterEach`).
 */
export function setLogger(next: Logger): Logger {
    const previous = current
    current = next
    return previous
}

/**
 * Restores the default `console`-backed logger. Convenience for `afterEach`.
 */
export function resetLogger(): void {
    current = consoleLogger
}

export function info(...args: unknown[]): void {
    current.info(...args)
}

export function warn(...args: unknown[]): void {
    current.warn(...args)
}

export function error(...args: unknown[]): void {
    current.error(...args)
}
