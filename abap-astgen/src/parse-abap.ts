#!/usr/bin/env bun
// Usage: parse-abap.ts <input-dir> <output-dir>
// Dumps raw @abaplint/core statements as JSON. All interpretation happens
// downstream (e.g. in AbapJsonParser.scala in the joern abap2cpg frontend).

import * as fs from "node:fs"
import * as path from "node:path"
import {ABAPFile, ABAPObject, MemoryFile, Registry} from "@abaplint/core"

const [, , inputArg, outputDir] = process.argv
if (!inputArg || !outputDir) {
    process.stderr.write("Usage: parse-abap.ts <input-dir> <output-dir>\n")
    process.exit(1)
}

fs.mkdirSync(outputDir, {recursive: true})

function* walkAbap(dir: string, relPrefix = ""): Generator<[string, string]> {
    for (const entry of fs.readdirSync(dir, {withFileTypes: true})) {
        const abs = path.join(dir, entry.name)
        const rel = relPrefix ? path.join(relPrefix, entry.name) : entry.name
        if (entry.isDirectory()) {
            yield* walkAbap(abs, rel)
        } else if (entry.isFile() && entry.name.endsWith(".abap")) {
            yield [abs, rel]
        }
    }
}

const pairs: [string, string][] = fs.statSync(inputArg).isDirectory()
    ? [...walkAbap(inputArg)]
    : [[inputArg, path.basename(inputArg)]]

for (const [absPath, relPath] of pairs) {
    const relName = path.basename(relPath)
    try {
        const reg = new Registry()
        reg.addFile(new MemoryFile(relName, fs.readFileSync(absPath, "utf8")))
        reg.parse()

        const obj = reg.getFirstObject()
        // ABAPObjects sequence their files (includes etc.); other object types
        // only expose unordered getFiles(). Either way we need an ABAPFile to
        // read statements from.
        const candidate = obj instanceof ABAPObject ? obj.getSequencedFiles()[0] : obj?.getFiles()[0]
        const file = candidate instanceof ABAPFile ? candidate : undefined
        if (!obj || !file) {
            process.stdout.write(`ERR ${absPath}\n`)
            continue
        }

        const statements = file.getStatements().map(s => ({
            type: s.get().constructor.name,
            tokens: s.getTokens().map(t => ({str: t.getStr()})),
            start: {row: s.getStart().getRow(), col: s.getStart().getCol()},
            end: {row: s.getEnd().getRow(), col: s.getEnd().getCol()},
        }))

        const outPath = path.join(outputDir, relPath.replace(/\.abap$/, ".json"))
        fs.mkdirSync(path.dirname(outPath), {recursive: true})
        fs.writeFileSync(outPath, JSON.stringify({file: relName, objectType: obj.getType(), statements}))
        process.stdout.write(`OK ${outPath}\n`)
    } catch (e) {
        process.stderr.write(`Error: ${absPath}: ${(e as Error).message}\n`)
        process.stdout.write(`ERR ${absPath}\n`)
    }
}
