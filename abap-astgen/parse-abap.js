#!/usr/bin/env node
// Usage: parse-abap.js <input-dir> <output-dir>
// Dumps raw @abaplint/core statements as JSON. All interpretation happens
// downstream (e.g. in AbapJsonParser.scala in the joern abap2cpg frontend).

const fs = require('fs');
const path = require('path');
const { Registry, MemoryFile } = require('@abaplint/core');

const [,, inputArg, outputDir] = process.argv;
if (!inputArg || !outputDir) {
  process.stderr.write('Usage: parse-abap.js <input-dir> <output-dir>\n');
  process.exit(1);
}

fs.mkdirSync(outputDir, { recursive: true });

function* walkAbap(dir, relPrefix = '') {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const abs = path.join(dir, entry.name);
    const rel = relPrefix ? path.join(relPrefix, entry.name) : entry.name;
    if (entry.isDirectory()) {
      yield* walkAbap(abs, rel);
    } else if (entry.isFile() && entry.name.endsWith('.abap')) {
      yield [abs, rel];
    }
  }
}

const pairs = fs.statSync(inputArg).isDirectory()
  ? [...walkAbap(inputArg)]
  : [[inputArg, path.basename(inputArg)]];

for (const [absPath, relPath] of pairs) {
  const relName = path.basename(relPath);
  try {
    const reg = new Registry();
    reg.addFile(new MemoryFile(relName, fs.readFileSync(absPath, 'utf8')));
    reg.parse();

    const obj  = [...reg.getObjects()][0];
    const file = obj && (obj.getSequencedFiles ? obj.getSequencedFiles()[0] : obj.getFiles()[0]);
    if (!obj || !file) { process.stdout.write(`ERR ${absPath}\n`); continue; }

    const statements = file.getStatements().map(s => ({
      type:   s.get().constructor.name,
      tokens: s.getTokens().map(t => ({ str: t.getStr() })),
      start:  { row: s.getStart().getRow(), col: s.getStart().getCol() },
      end:    { row: s.getEnd().getRow(),   col: s.getEnd().getCol() }
    }));

    const outPath = path.join(outputDir, relPath.replace(/\.abap$/, '.json'));
    fs.mkdirSync(path.dirname(outPath), { recursive: true });
    fs.writeFileSync(outPath, JSON.stringify({ file: relName, objectType: obj.getType(), statements }));
    process.stdout.write(`OK ${outPath}\n`);
  } catch (e) {
    process.stderr.write(`Error: ${absPath}: ${e.message}\n`);
    process.stdout.write(`ERR ${absPath}\n`);
  }
}
