const path = require('node:path');
const os = require('node:os');
const fs = require('node:fs');
const { spawnSync } = require('node:child_process');

const parser = path.join(__dirname, '..', 'parse-abap.js');

function runFixture(code, filename, assertFn) {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'abapgen-tests-'));
    const srcDir = path.join(tmpDir, 'src');
    const outDir = path.join(tmpDir, 'out');
    fs.mkdirSync(srcDir, { recursive: true });
    fs.writeFileSync(path.join(srcDir, filename), code);

    const result = spawnSync('node', [parser, srcDir, outDir], { encoding: 'utf8' });
    try {
        assertFn({ tmpDir, srcDir, outDir, result });
    } finally {
        fs.rmSync(tmpDir, { recursive: true, force: true });
    }
}

function readJsonOutput(outDir, abapFilename) {
    const jsonName = abapFilename.replace(/\.abap$/, '.json');
    return JSON.parse(fs.readFileSync(path.join(outDir, jsonName), 'utf8'));
}

describe('abapgen basic functionality', () => {
    it('should emit JSON for a simple class definition', () => {
        const code = `CLASS zcl_simple DEFINITION PUBLIC.
  PUBLIC SECTION.
    METHODS greet
      IMPORTING iv_name TYPE string
      RETURNING VALUE(rv_result) TYPE string.
ENDCLASS.
CLASS zcl_simple IMPLEMENTATION.
  METHOD greet.
    rv_result = iv_name.
  ENDMETHOD.
ENDCLASS.
`;
        runFixture(code, 'zcl_simple.clas.abap', ({ outDir, result }) => {
            expect(result.status).toBe(0);
            expect(result.stdout).toMatch(/^OK /m);

            const ast = readJsonOutput(outDir, 'zcl_simple.clas.abap');
            expect(ast.file).toBe('zcl_simple.clas.abap');
            expect(ast.objectType).toBe('CLAS');
            expect(Array.isArray(ast.statements)).toBe(true);
            expect(ast.statements.length).toBeGreaterThan(0);
        });
    });

    it('should record statement types and tokens', () => {
        const code = `REPORT z_hello.
WRITE 'hello'.
`;
        runFixture(code, 'z_hello.prog.abap', ({ outDir }) => {
            const ast = readJsonOutput(outDir, 'z_hello.prog.abap');
            const types = ast.statements.map(s => s.type);
            expect(types).toContain('Report');
            expect(types).toContain('Write');

            const write = ast.statements.find(s => s.type === 'Write');
            expect(write.tokens.map(t => t.str)).toEqual(['WRITE', "'hello'", '.']);
        });
    });

    it('should include row/col position info for each statement', () => {
        const code = `REPORT z_pos.
WRITE 'x'.
`;
        runFixture(code, 'z_pos.prog.abap', ({ outDir }) => {
            const ast = readJsonOutput(outDir, 'z_pos.prog.abap');
            for (const stmt of ast.statements) {
                expect(stmt.start).toEqual(expect.objectContaining({
                    row: expect.any(Number),
                    col: expect.any(Number)
                }));
                expect(stmt.end).toEqual(expect.objectContaining({
                    row: expect.any(Number),
                    col: expect.any(Number)
                }));
            }
            const write = ast.statements.find(s => s.type === 'Write');
            expect(write.start.row).toBe(2);
        });
    });

    it('should process every .abap file in the input directory', () => {
        const codeA = `REPORT z_a.\n`;
        const codeB = `REPORT z_b.\n`;
        const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'abapgen-tests-'));
        const srcDir = path.join(tmpDir, 'src');
        const outDir = path.join(tmpDir, 'out');
        fs.mkdirSync(srcDir, { recursive: true });
        fs.writeFileSync(path.join(srcDir, 'z_a.prog.abap'), codeA);
        fs.writeFileSync(path.join(srcDir, 'z_b.prog.abap'), codeB);

        try {
            const result = spawnSync('node', [parser, srcDir, outDir], { encoding: 'utf8' });
            expect(result.status).toBe(0);
            expect(fs.existsSync(path.join(outDir, 'z_a.prog.json'))).toBe(true);
            expect(fs.existsSync(path.join(outDir, 'z_b.prog.json'))).toBe(true);
        } finally {
            fs.rmSync(tmpDir, { recursive: true, force: true });
        }
    });

    it('should skip non-.abap files in the input directory', () => {
        const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'abapgen-tests-'));
        const srcDir = path.join(tmpDir, 'src');
        const outDir = path.join(tmpDir, 'out');
        fs.mkdirSync(srcDir, { recursive: true });
        fs.writeFileSync(path.join(srcDir, 'note.txt'), 'not abap');
        fs.writeFileSync(path.join(srcDir, 'z_valid.prog.abap'), 'REPORT z_valid.\n');

        try {
            const result = spawnSync('node', [parser, srcDir, outDir], { encoding: 'utf8' });
            expect(result.status).toBe(0);
            expect(fs.existsSync(path.join(outDir, 'note.json'))).toBe(false);
            expect(fs.existsSync(path.join(outDir, 'z_valid.prog.json'))).toBe(true);
        } finally {
            fs.rmSync(tmpDir, { recursive: true, force: true });
        }
    });

    it('should accept a single .abap file as input (not just a directory)', () => {
        const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'abapgen-tests-'));
        const outDir = path.join(tmpDir, 'out');
        const inputFile = path.join(tmpDir, 'z_single.prog.abap');
        fs.writeFileSync(inputFile, 'REPORT z_single.\n');

        try {
            const result = spawnSync('node', [parser, inputFile, outDir], { encoding: 'utf8' });
            expect(result.status).toBe(0);
            expect(fs.existsSync(path.join(outDir, 'z_single.prog.json'))).toBe(true);
        } finally {
            fs.rmSync(tmpDir, { recursive: true, force: true });
        }
    });

    it('should exit non-zero when called without arguments', () => {
        const result = spawnSync('node', [parser], { encoding: 'utf8' });
        expect(result.status).not.toBe(0);
        expect(result.stderr).toMatch(/Usage/);
    });

    it('should exit non-zero when called with input but no output dir', () => {
        const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'abapgen-tests-'));
        try {
            const result = spawnSync('node', [parser, tmpDir], { encoding: 'utf8' });
            expect(result.status).not.toBe(0);
            expect(result.stderr).toMatch(/Usage/);
        } finally {
            fs.rmSync(tmpDir, { recursive: true, force: true });
        }
    });

    it('should fail (non-zero exit) when the input path does not exist', () => {
        const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'abapgen-tests-'));
        const missing = path.join(tmpDir, 'does-not-exist');
        const outDir = path.join(tmpDir, 'out');
        try {
            const result = spawnSync('node', [parser, missing, outDir], { encoding: 'utf8' });
            expect(result.status).not.toBe(0);
            expect(result.stderr).toMatch(/ENOENT/);
        } finally {
            fs.rmSync(tmpDir, { recursive: true, force: true });
        }
    });

    it('should not crash on an empty .abap file', () => {
        runFixture('', 'z_empty.prog.abap', ({ outDir, result }) => {
            expect(result.status).toBe(0);
            expect(result.stderr).not.toMatch(/TypeError|SyntaxError|ReferenceError/);
            // Either OK (empty statements array) or ERR (no object) — both fine.
            expect(result.stdout).toMatch(/^(OK|ERR) /m);
        });
    });

    it('should exit cleanly when the input dir only contains non-.abap files', () => {
        const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'abapgen-tests-'));
        const srcDir = path.join(tmpDir, 'src');
        const outDir = path.join(tmpDir, 'out');
        fs.mkdirSync(srcDir, { recursive: true });
        fs.writeFileSync(path.join(srcDir, 'readme.md'), '# not abap');
        fs.writeFileSync(path.join(srcDir, 'config.yaml'), 'key: value');

        try {
            const result = spawnSync('node', [parser, srcDir, outDir], { encoding: 'utf8' });
            expect(result.status).toBe(0);
            expect(fs.readdirSync(outDir)).toEqual([]);
        } finally {
            fs.rmSync(tmpDir, { recursive: true, force: true });
        }
    });

    it('should parse FORM routines', () => {
        const code = `REPORT z_form.
FORM greet USING iv_name TYPE string.
  WRITE iv_name.
ENDFORM.
`;
        runFixture(code, 'z_form.prog.abap', ({ outDir }) => {
            const ast = readJsonOutput(outDir, 'z_form.prog.abap');
            const types = ast.statements.map(s => s.type);
            expect(types).toContain('Form');
            expect(types).toContain('EndForm');
        });
    });

    it('should exit cleanly and emit no JSON when the input directory has no .abap files', () => {
        const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'abapgen-tests-'));
        const srcDir = path.join(tmpDir, 'src');
        const outDir = path.join(tmpDir, 'out');
        fs.mkdirSync(srcDir, { recursive: true });

        try {
            const result = spawnSync('node', [parser, srcDir, outDir], { encoding: 'utf8' });
            expect(result.status).toBe(0);
            expect(fs.existsSync(outDir)).toBe(true);
            expect(fs.readdirSync(outDir)).toEqual([]);
        } finally {
            fs.rmSync(tmpDir, { recursive: true, force: true });
        }
    });

    it('should not crash when given a filename that does not map to an ABAP object', () => {
        // @abaplint identifies object types from the filename suffix (.prog.abap,
        // .clas.abap, .intf.abap, etc). A bare `.abap` without a recognized
        // suffix exercises the !obj/!file branch in the parser, which should
        // print ERR and keep going rather than crash.
        const code = `REPORT z_x.\n`;
        runFixture(code, 'random_name.abap', ({ outDir, result }) => {
            expect(result.status).toBe(0);
            // Either ERR (fell through no-object branch) or OK (abaplint inferred
            // it as a program) — both are acceptable, what matters is: no crash,
            // no output in the broken-object case.
            expect(result.stdout).toMatch(/^(OK|ERR) /m);
            expect(result.stderr).not.toMatch(/TypeError|SyntaxError|ReferenceError/);
        });
    });

    it('should preserve UTF-8 characters in token strings', () => {
        const code = `REPORT z_utf8.
WRITE 'héllo wörld — €'.
`;
        runFixture(code, 'z_utf8.prog.abap', ({ outDir }) => {
            const ast = readJsonOutput(outDir, 'z_utf8.prog.abap');
            const write = ast.statements.find(s => s.type === 'Write');
            expect(write).toBeDefined();
            const literal = write.tokens.map(t => t.str).find(s => s.startsWith("'"));
            expect(literal).toBe("'héllo wörld — €'");
        });
    });

    it('should set objectType to INTF for interface definitions', () => {
        const code = `INTERFACE zif_thing PUBLIC.
  METHODS do_it.
ENDINTERFACE.
`;
        runFixture(code, 'zif_thing.intf.abap', ({ outDir }) => {
            const ast = readJsonOutput(outDir, 'zif_thing.intf.abap');
            expect(ast.objectType).toBe('INTF');
            const types = ast.statements.map(s => s.type);
            expect(types).toContain('Interface');
            expect(types).toContain('EndInterface');
        });
    });

    it('should emit Method statements for class method implementations', () => {
        const code = `CLASS zcl_m DEFINITION PUBLIC.
  PUBLIC SECTION.
    METHODS do_it.
ENDCLASS.
CLASS zcl_m IMPLEMENTATION.
  METHOD do_it.
    DATA lv_x TYPE i.
    lv_x = 1.
  ENDMETHOD.
ENDCLASS.
`;
        runFixture(code, 'zcl_m.clas.abap', ({ outDir }) => {
            const ast = readJsonOutput(outDir, 'zcl_m.clas.abap');
            const types = ast.statements.map(s => s.type);
            expect(types).toContain('ClassDefinition');
            expect(types).toContain('MethodDef');
            expect(types).toContain('ClassImplementation');
            expect(types).toContain('MethodImplementation');
            expect(types).toContain('EndMethod');
            expect(types).toContain('Data');
        });
    });
});
