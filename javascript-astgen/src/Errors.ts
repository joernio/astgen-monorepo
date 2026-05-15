/**
 * Typed errors thrown by the AST generation pipeline.
 *
 * The CLI entry point ([astgen.ts](./astgen.ts)) is responsible for translating
 * these into process exit codes; library code itself never calls `process.exit`,
 * which keeps the pipeline drivable from tests.
 */

export class AstgenError extends Error {
    constructor(message: string) {
        super(message)
        this.name = new.target.name
    }
}

/**
 * Extracts a human-readable message from an unknown thrown value. Used at every
 * `catch` site that surfaces the error to the user (logger or worker envelope)
 * to keep the `Error` vs non-`Error` branch out of the call sites.
 */
export function getErrorMessage(err: unknown): string {
    return err instanceof Error ? err.message : String(err)
}

/**
 * Thrown when the configured `--src` directory does not exist or is not readable.
 */
export class SourceDirNotReadableError extends AstgenError {
    readonly srcDir: string

    constructor(srcDir: string) {
        super(`Source directory does not exist or is not readable: ${srcDir}`)
        this.srcDir = srcDir
    }
}
