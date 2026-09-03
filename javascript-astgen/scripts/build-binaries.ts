#!/usr/bin/env bun

/**
 * Cross-compile astgen for all release targets in parallel.
 * Outputs binaries to the project root; names match those expected by the
 * release workflow (astgen-linux-x64, astgen-linux-arm64, astgen-macos-x64,
 * astgen-macos-arm64, astgen-win-x64, astgen-win-arm64).
 */

export {}

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
            compile: {target, outfile},
            minify: true,
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
