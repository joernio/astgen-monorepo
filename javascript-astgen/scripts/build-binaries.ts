#!/usr/bin/env bun

/**
 * Cross-compile astgen for all release targets in parallel.
 * Outputs binaries to the project root; names match those expected by the
 * release workflow (astgen-linux-x64, astgen-linux-arm64, astgen-macos-x64,
 * astgen-macos-arm64, astgen-win-x64, astgen-win-arm64).
 */

import * as fs from "node:fs"
import * as path from "node:path"

export {}

// TscUtils.ts points tsc's default-lib resolution at an embedded copy of
// `typescript/lib` (see the `assets` entry below) because the build machine's
// `node_modules/typescript/lib` path baked in by Bun at compile time does not
// exist on whatever machine eventually runs the compiled binary. Stage only
// the `lib.*.d.ts` runtime-library files (~4 MB) into a scratch directory
// rather than embedding the package's `lib/` wholesale (~23 MB, mostly the
// typescript.js/tsc.js compiler bundles, which are already pulled in through
// the normal import graph). The staged directory keeps the `lib` basename so
// it lands at the same embedded path (`<binary>/lib`) that TscUtils.ts reads.
const tsLibSrcDir = path.join(import.meta.dir, "..", "node_modules", "typescript", "lib")
const stagedAssetsDir = path.join(import.meta.dir, "..", ".tslib-assets")
const stagedLibDir = path.join(stagedAssetsDir, "lib")
fs.rmSync(stagedAssetsDir, {recursive: true, force: true})
fs.mkdirSync(stagedLibDir, {recursive: true})
for (const entry of fs.readdirSync(tsLibSrcDir)) {
    if (entry.startsWith("lib.") && entry.endsWith(".d.ts")) {
        fs.copyFileSync(path.join(tsLibSrcDir, entry), path.join(stagedLibDir, entry))
    }
}

const targets = [
    {target: "bun-linux-x64",    outfile: "./astgen-linux-x64"},
    {target: "bun-linux-arm64",  outfile: "./astgen-linux-arm64"},
    {target: "bun-darwin-x64",   outfile: "./astgen-macos-x64"},
    {target: "bun-darwin-arm64", outfile: "./astgen-macos-arm64"},
    {target: "bun-windows-x64",  outfile: "./astgen-win-x64"},
    {target: "bun-windows-arm64", outfile: "./astgen-win-arm64"},
] as const

console.log(`Building ${targets.length} targets in parallel…`)

const results = await Promise.allSettled(
    targets.map(async ({target, outfile}) => {
        const result = await Bun.build({
            entrypoints: ["./src/astgen.ts", "./src/AstWorker.ts"],
            compile: {target, outfile, assets: [stagedLibDir]},
            minify: true,
            // Bytecode-compiled ESM (requires Bun >= 1.4): skips source parsing
            // at startup in the binary and its AstWorker, ~3x faster cold start.
            format: "esm",
            bytecode: true,
        })
        if (!result.success) {
            const logs = result.logs.map(l => l.message).join("\n")
            throw new Error(`${target}: build failed\n${logs}`)
        }
        const suffix = target.startsWith("bun-windows") ? ".exe" : ""
        console.log(`  ✓ ${outfile}${suffix}`)
    })
)

const failures = results.filter(r => r.status === "rejected") as PromiseRejectedResult[]
if (failures.length > 0) {
    for (const f of failures) console.error(f.reason)
    process.exit(1)
}
