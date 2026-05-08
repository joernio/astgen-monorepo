/**
 * Runtime configuration consumed by [Pipeline.start](./Pipeline.ts).
 *
 * Field names follow the CLI flag spelling (kebab-case for `exclude-file` /
 * `exclude-regex`) so the parsed yargs result can be dropped in directly.
 * The interface is **not part of any public programmatic API** — it exists
 * only so test code and the CLI share the same shape.
 */
export default interface Options {
    /**
     * Absolute path to the source directory to scan. Resolved by the CLI
     * before being passed to {@link Pipeline.start}.
     */
    src: string,

    /**
     * Absolute path to the directory that receives generated `.json` AST and
     * `.typemap` files. Defaults to `<src>/ast_out` when the CLI flag is
     * omitted.
     */
    output: string,

    /**
     * Project type override (`js`, `ts`, `nodejs`, `vue`, ...). When omitted
     * the pipeline auto-detects: it processes the directory as a JS/TS project
     * if `package.json` or `rush.json` is present, else it logs a warning and
     * still falls back to JS/TS processing.
     */
    type?: string,

    /**
     * When true, descend into subdirectories. When false, only the top-level
     * of `src` is scanned.
     */
    recurse: boolean,

    /**
     * When true (the default), the TypeScript compiler is invoked after the
     * AST pass to produce a `.typemap` per source file. Disable to halve
     * runtime when only the AST is needed.
     */
    tsTypes: boolean,

    /**
     * Files or directories to exclude. Each entry is interpreted relative to
     * `src` (or absolute) and excludes either an exact file path or any path
     * under that directory. Repeatable on the CLI: `--exclude-file a.js
     * --exclude-file build/`.
     */
    "exclude-file": string[],

    /**
     * Optional case-insensitive regex matched against the absolute path of
     * each candidate file or directory. Files/directories that match are
     * skipped. Invalid regexes are dropped with a warning at parse time.
     */
    "exclude-regex"?: RegExp
}
