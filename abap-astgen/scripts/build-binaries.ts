#!/usr/bin/env bun

/**
 * Cross-compile abapgen for all release targets in parallel.
 * Outputs binaries to the project root; names match those expected by the
 * release workflow (abapgen-linux-x64, abapgen-linux-arm64, abapgen-macos-x64,
 * abapgen-macos-arm64, abapgen-win-x64, abapgen-win-arm64).
 */

export {}

const targets = [
    {target: "bun-linux-x64",     outfile: "./abapgen-linux-x64"},
    {target: "bun-linux-arm64",   outfile: "./abapgen-linux-arm64"},
    {target: "bun-darwin-x64",    outfile: "./abapgen-macos-x64"},
    {target: "bun-darwin-arm64",  outfile: "./abapgen-macos-arm64"},
    {target: "bun-windows-x64",   outfile: "./abapgen-win-x64"},
    {target: "bun-windows-arm64", outfile: "./abapgen-win-arm64"},
] as const

console.log(`Building ${targets.length} targets in parallel…`)

const results = await Promise.allSettled(
    targets.map(async ({target, outfile}) => {
        const result = await Bun.build({
            entrypoints: ["./src/parse-abap.ts"],
            compile: {target, outfile},
            // No identifier minification: the JSON `type` field is derived
            // from abaplint statement class names via `constructor.name`,
            // and minifying identifiers mangles them (keepNames does not
            // cover bundled node_modules classes).
            minify: {whitespace: true, syntax: true},
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
