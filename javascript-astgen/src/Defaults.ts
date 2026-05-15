import babelParser from "@babel/parser"
import tsc from "typescript"

/**
 * Default name of the per-source output directory. When the user does not pass
 * `-o`, the CLI nests this directory inside `--src` so AST output stays
 * co-located with the project being scanned (see [astgen.ts](./astgen.ts)).
 */
export const DEFAULT_OUTPUT_DIR: string = "ast_out"

/**
 * File extensions treated as JavaScript / TypeScript sources. `xsjs` and
 * `xsjslib` cover SAP HANA XS Engine sources, which are JS-flavoured.
 */
export const JS_TS_EXTENSIONS: string[] = [
    ".js",
    ".jsx",
    ".cjs",
    ".mjs",
    ".xsjs",
    ".xsjslib",
    ".ts",
    ".tsx"
]

export const VUE_EXTENSION = ".vue"

/**
 * Every source extension the pipeline knows how to handle. The worker
 * dispatches on extension to pick between plain Babel and the Vue cleaner +
 * Babel path (see [AstWorker.ts](./AstWorker.ts)).
 */
export const ALL_PARSEABLE_EXTENSIONS: string[] = [...JS_TS_EXTENSIONS, VUE_EXTENSION]

/**
 * Directory basenames that are always skipped during traversal. Grouped here
 * by intent so the rationale survives future edits:
 *
 * - **Dependency / build output**: `node_modules`, `vendor`, `dist`, `build`,
 *   `venv` — never user-authored source.
 * - **Test infrastructure**: `test`, `tests`, `e2e`, `e2e-beta`, `cypress`,
 *   `jest-cache` — too noisy for static analysis and frequently irrelevant.
 * - **Docs / examples**: `docs`, `examples`, `www` — not part of the
 *   shipping codebase.
 * - **Tooling artifacts**: `eslint-rules`, `codemods`, `flow-typed`, `i18n` —
 *   generated or framework-specific scaffolding.
 *
 * Comparison is case-insensitive (the runtime lowercases each name).
 */
export const IGNORE_DIRS: string[] = [
    "node_modules",
    "venv",
    "docs",
    "test",
    "tests",
    "e2e",
    "e2e-beta",
    "examples",
    "cypress",
    "jest-cache",
    "eslint-rules",
    "codemods",
    "flow-typed",
    "i18n",
    "vendor",
    "www",
    "dist",
    "build",
]

/**
 * File-name patterns treated as non-source noise:
 *
 * - `chunk-vendors`, `app~` — Webpack/Vue CLI bundler outputs
 * - `mock` — fixture data
 * - `e2e`, `conf`, `test`, `spec` — test or configuration files outside
 *   IGNORE_DIRS
 * - `[.-]min` — minified bundles (`.min.js`, `-min.js`)
 * - `\.d` — TypeScript declaration files (`*.d.ts`)
 *
 * Matched against the file basename; the extension list mirrors
 * {@link JS_TS_EXTENSIONS}.
 */
export const IGNORE_FILE_PATTERN: RegExp =
    new RegExp("(chunk-vendors|app~|mock|e2e|conf|test|spec|[.-]min|\\.d)\\.(js|jsx|cjs|mjs|xsjs|xsjslib|ts|tsx)$", "i")

/**
 * Skip files with more than this many lines. Avoids spending minutes on
 * generated bundles that slipped past the size guard.
 */
export const MAX_LOC_IN_FILE: number = 50000

/**
 * Skip files larger than 5 MB. Co-tuned with `POS_SHIFT` in
 * [TscUtils.ts](./TscUtils.ts) — see the invariant assertion there.
 */
export const MAX_FILE_SIZE_BYTES: number = 5 * 1024 * 1024

/**
 * Skip files with any single line longer than 10000 bytes. Real source rarely
 * exceeds this; minified or generated content frequently does, and Babel
 * tends to OOM on it.
 */
export const MAX_LINE_LENGTH: number = 10_000

/**
 * Babel plugins enabled in both the primary and fallback configurations.
 * Kept as a separate constant so the two `ParserOptions` blocks below cannot
 * drift apart on plugins they share.
 */
const COMMON_BABEL_PLUGINS: babelParser.ParserPlugin[] = [
    "optionalChaining",
    "classProperties",
    "decorators-legacy",
    "exportDefaultFrom",
    "doExpressions",
    "numericSeparator",
    "dynamicImport",
    "typescript",
]

/**
 * Primary Babel configuration: permissive, accepts JSX and ambiguous module
 * shapes. Tried first by [Parsing.ts](./Parsing.ts).
 */
export const BABEL_PARSER_OPTIONS: babelParser.ParserOptions = {
    sourceType: "unambiguous",
    allowImportExportEverywhere: true,
    allowAwaitOutsideFunction: true,
    allowNewTargetOutsideFunction: true,
    allowReturnOutsideFunction: true,
    allowSuperOutsideMethod: true,
    allowUndeclaredExports: true,
    errorRecovery: true,
    plugins: [...COMMON_BABEL_PLUGINS, "jsx"],
}

/**
 * Fallback Babel configuration used when {@link BABEL_PARSER_OPTIONS} fails.
 * Treats input as an ESM module and disables the JSX plugin (which can cause
 * TSX-vs-TS ambiguity on plain `.ts`). Derived from the primary config so that
 * adding a plugin to {@link COMMON_BABEL_PLUGINS} keeps both in sync.
 */
export const SAFE_BABEL_PARSER_OPTIONS: babelParser.ParserOptions = {
    ...BABEL_PARSER_OPTIONS,
    sourceType: "module",
    allowNewTargetOutsideFunction: undefined,
    allowSuperOutsideMethod: undefined,
    allowUndeclaredExports: undefined,
    plugins: [...COMMON_BABEL_PLUGINS],
}

export const DEFAULT_TSC_OPTIONS: tsc.CompilerOptions = {
    target: tsc.ScriptTarget.ES2020,
    module: tsc.ModuleKind.CommonJS,
    allowJs: true,
    allowUnreachableCode: true,
    allowUnusedLabels: true,
    alwaysStrict: false,
    noUncheckedIndexedAccess: false,
    noPropertyAccessFromIndexSignature: false,
    removeComments: true
}

/**
 * Hard cap on how long a type string can be before it is treated as `any`.
 * TypeScript already truncates at ~160 characters; this guard is defense in
 * depth for future tsc versions that might emit longer strings.
 */
export const MAX_TYPE_STRING_LENGTH: number = 500

/**
 * `TypeFormatFlags` passed to `typeChecker.typeToString`. `InTypeAlias`
 * keeps the textual form closer to user-written aliases (avoids expanding
 * recursive references).
 */
export const DEFAULT_TSC_TYPE_OPTIONS: number = tsc.TypeFormatFlags.InTypeAlias

/**
 * Sentinel for "no usable type". Returned by
 * [TscUtils.safeTypeToString](./TscUtils.ts) when the TypeChecker emits
 * `unknown`, an unresolved type, or a string that exceeds
 * {@link MAX_TYPE_STRING_LENGTH}. The caller filters entries equal to this
 * value out of the resulting `TypeMap`.
 */
export const ANY: string = "any"

/** Literal string emitted by the TypeChecker for `unknown`. */
export const UNKNOWN: string = "unknown"

/**
 * Prefix the TypeChecker uses when it cannot resolve a symbol; treated as
 * a soft failure (mapped to {@link ANY}).
 */
export const UNRESOLVED: string = "/*unresolved*/"

/**
 * Quoted-string literal type matcher (e.g. `"foo"`, `'bar'`, `` `baz` ``).
 * The pipeline collapses any of these to the bare type name `string` so that
 * Joern's type system does not have to reason about the literal value.
 */
export const STRING_REGEX: RegExp = /^["'`].*["'`]$/

/**
 * Array-suffix type matcher (e.g. `Foo[]`). The pipeline collapses these to
 * the Joern-specific `__ecma.Array` type name so consumers can recognise
 * arrays without parsing the element type.
 */
export const ARRAY_REGEX: RegExp = /.+\[]$/
