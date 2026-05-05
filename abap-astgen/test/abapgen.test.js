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
