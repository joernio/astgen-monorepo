import * as path from "node:path"
import * as fs from "node:fs"

import start from "../src/Pipeline"
import {findOffsets, setupProject, setupTestFixture, withTmpDir, writeFiles} from "./helpers"

describe("astgen basic functionality", () => {
    it("emits ast and typemap matching variable types for a CommonJS module", async () => {
        const code = `const somedata = require('../../package.json');
          const foo = "Something";
          const bar = {
            foo
          };
          exports.foo = bar.foo;
          module.exports = bar;`

        await setupTestFixture(code, "main.js", {}, (tmpDir, testFile) => {
            const resultAst = fs.readFileSync(path.join(tmpDir, "ast_out", "main.js.json")).toString()
            expect(resultAst).toContain("\"fullName\":\"" + testFile.replaceAll("\\", "\\\\") + "\"")
            expect(resultAst).toContain("\"relativeName\":\"main.js\"")

            const parsed = JSON.parse(fs.readFileSync(path.join(tmpDir, "ast_out", "main.js.typemap")).toString())
            expect(parsed[findOffsets(code, "'../../package.json'")]).toEqual("string")
            expect(parsed[findOffsets(code, "foo", 0)]).toEqual("string")
            expect(parsed[findOffsets(code, "bar", 0)]).toEqual("{ foo: string; }")
            expect(parsed[findOffsets(code, "foo", 1)]).toEqual("string")
            expect(parsed[findOffsets(code, "exports", 0)]).toEqual("{ foo: any; }")
            expect(parsed[findOffsets(code, "bar", 2)]).toEqual("{ foo: string; }")
            expect(parsed[findOffsets(code, "module.exports = bar")]).toEqual("{ foo: any; }")
        })
    })

    it("should parse a simple js file correctly", async () => {
        const code = "console.log(\"Hello, world!\");"

        await setupTestFixture(code, "main.js", {}, (tmpDir, testFile) => {
            const resultAst = fs.readFileSync(path.join(tmpDir, "ast_out", "main.js.json")).toString()
            expect(resultAst).toContain("\"fullName\":\"" + testFile.replaceAll("\\", "\\\\") + "\"")
            expect(resultAst).toContain("\"relativeName\":\"main.js\"")

            const parsed = JSON.parse(fs.readFileSync(path.join(tmpDir, "ast_out", "main.js.typemap")).toString())
            expect(parsed[findOffsets(code, "console")]).toEqual("Console")
            expect(parsed[findOffsets(code, "log")]).toEqual("(...data: any[]) => void")
            expect(parsed[findOffsets(code, "console.log")]).toEqual("(...data: any[]) => void")
            expect(parsed[findOffsets(code, "\"Hello, world!\"")]).toEqual("string")
            expect(parsed[findOffsets(code, "console.log(\"Hello, world!\")")]).toEqual("void")
        })
    })

    it("should exclude files by relative file path correctly", async () => {
        const code = "console.log(\"Hello, world!\");"
        const config = {tsTypes: false, "exclude-file": ["main.js"]}

        await setupTestFixture(code, "main.js", config, (tmpDir) => {
            expect(fs.existsSync(path.join(tmpDir, "ast_out", "main.js.json"))).toBe(false)
        })
    })

    it("should exclude files by absolute file path correctly", async () => {
        const code = "console.log(\"Hello, world!\");"

        await setupTestFixture(code, "main.js", {tsTypes: false}, (tmpDir) => {
            expect(fs.existsSync(path.join(tmpDir, "ast_out", "main.js.json"))).toBe(false)
        }, (_, testFile) => [testFile])
    })

    it("should exclude files by relative file path with dir correctly", async () => {
        const code = "console.log(\"Hello, world!\");"
        const config = {tsTypes: false, "exclude-file": [path.join("src", "main.js")]}

        await setupTestFixture(code, "src/main.js", config, (tmpDir) => {
            expect(fs.existsSync(path.join(tmpDir, "ast_out", "src", "main.js.json"))).toBe(false)
        })
    })

    it("should exclude files by relative dir path correctly", async () => {
        const code = "console.log(\"Hello, world!\");"
        const config = {tsTypes: false, "exclude-file": ["src"]}

        await setupTestFixture(code, "src/main.js", config, (tmpDir) => {
            expect(fs.existsSync(path.join(tmpDir, "ast_out", "src", "main.js.json"))).toBe(false)
        })
    })

    it("should exclude files by absolute dir path correctly", async () => {
        const code = "console.log(\"Hello, world!\");"

        await setupTestFixture(code, "src/main.js", {tsTypes: false}, (tmpDir) => {
            expect(fs.existsSync(path.join(tmpDir, "ast_out", "src", "main.js.json"))).toBe(false)
        }, (tmpDir) => [path.join(tmpDir, "src")])
    })

    it("should not exclude sibling paths when excluding a directory prefix", async () => {
        await setupProject(
            [
                {name: path.join("src", "app", "main.js"), code: "const x = 1;"},
                {name: path.join("src", "application", "main.js"), code: "const y = 2;"},
            ],
            {tsTypes: false, "exclude-file": [path.join("src", "app")]},
            (tmpDir) => {
                expect(fs.existsSync(path.join(tmpDir, "ast_out", "src", "app", "main.js.json"))).toBe(false)
                expect(fs.existsSync(path.join(tmpDir, "ast_out", "src", "application", "main.js.json"))).toBe(true)
            },
        )
    })

    it("should treat a trailing-separator exclude path the same as a bare path", async () => {
        await setupProject(
            [{name: path.join("src", "app", "main.js"), code: "const x = 1;"}],
            {tsTypes: false, "exclude-file": [path.join("src", "app") + path.sep]},
            (tmpDir) => {
                expect(fs.existsSync(path.join(tmpDir, "ast_out", "src", "app", "main.js.json"))).toBe(false)
            },
        )
    })

    it("should not match anything when an exclude path resolves outside the source root", async () => {
        await setupProject(
            [{name: path.join("src", "main.js"), code: "const x = 1;"}],
            {tsTypes: false, "exclude-file": [path.join("..", "elsewhere")]},
            (tmpDir) => {
                expect(fs.existsSync(path.join(tmpDir, "ast_out", "src", "main.js.json"))).toBe(true)
            },
        )
    })

    it("should exclude files by regex correctly", async () => {
        const code = "console.log(\"Hello, world!\");"
        const config = {tsTypes: false, "exclude-file": [], "exclude-regex": /.*main.*/i}

        await setupTestFixture(code, "main.js", config, (tmpDir) => {
            expect(fs.existsSync(path.join(tmpDir, "ast_out", "main.js.json"))).toBe(false)
        })
    })

    it("should skip files with more than 50000 lines", async () => {
        const code = Array(50001).fill("const x = 1;").join("\n")

        await setupTestFixture(code, "huge.ts", {}, (tmpDir) => {
            expect(fs.existsSync(path.join(tmpDir, "ast_out", "huge.ts.json"))).toBe(false)
        })
    })

    it("should process files with exactly 50000 lines", async () => {
        const code = Array(50000).fill("const x = 1;").join("\n")

        await setupTestFixture(code, "exact-limit.ts", {tsTypes: false}, (tmpDir) => {
            expect(fs.existsSync(path.join(tmpDir, "ast_out", "exact-limit.ts.json"))).toBe(true)
        })
    }, 20000)

    it("should skip files with a line longer than 10000 bytes", async () => {
        const longLine = "const x = \"" + "a".repeat(10001) + "\";"
        const code = `const y = 1;\n${longLine}\nconst z = 2;`

        await setupTestFixture(code, "longline.ts", {}, (tmpDir) => {
            expect(fs.existsSync(path.join(tmpDir, "ast_out", "longline.ts.json"))).toBe(false)
        })
    })

    it("should skip files larger than 5MB", async () => {
        const code = "x".repeat(5 * 1024 * 1024 + 1)

        await setupTestFixture(code, "huge.ts", {}, (tmpDir) => {
            expect(fs.existsSync(path.join(tmpDir, "ast_out", "huge.ts.json"))).toBe(false)
        })
    })

    it("should process files just under all size thresholds", async () => {
        const normalLine = "const x = \"" + "a".repeat(9980) + "\";"
        const code = `${normalLine}\nconst y = 1;`

        await setupTestFixture(code, "borderline.ts", {}, (tmpDir) => {
            expect(fs.existsSync(path.join(tmpDir, "ast_out", "borderline.ts.json"))).toBe(true)
        })
    })

    it("should produce bounded-length type strings for complex union types", async () => {
        // Validates that the pipeline produces short type strings for complex types.
        // TypeScript's own truncation (~160 chars) handles this; the 500-char guard in
        // safeTypeToString is a defense-in-depth layer for future TS version changes.
        const code = `
            type Long = "a1"|"a2"|"a3"|"a4"|"a5"|"a6"|"a7"|"a8"|"a9"|"a10"|
                        "b1"|"b2"|"b3"|"b4"|"b5"|"b6"|"b7"|"b8"|"b9"|"b10"|
                        "c1"|"c2"|"c3"|"c4"|"c5"|"c6"|"c7"|"c8"|"c9"|"c10"|
                        "d1"|"d2"|"d3"|"d4"|"d5"|"d6"|"d7"|"d8"|"d9"|"d10"|
                        "e1"|"e2"|"e3"|"e4"|"e5"|"e6"|"e7"|"e8"|"e9"|"e10"|
                        "f1"|"f2"|"f3"|"f4"|"f5"|"f6"|"f7"|"f8"|"f9"|"f10";
            const x: Long = "a1";
`
        await setupTestFixture(code, "main.ts", {tsTypes: true}, (tmpDir) => {
            const resultTypes = fs.readFileSync(path.join(tmpDir, "ast_out", "main.ts.typemap")).toString()
            const parsed = JSON.parse(resultTypes)
            const values = Object.values(parsed) as string[]
            expect(values.every((v) => v.length <= 500)).toBe(true)
        })
    })

    it("should generate AST and typemap outputs for multiple files with tsTypes enabled", async () => {
        const files = [
            {name: "src/a.ts", code: "const a: string = 'x';"},
            {name: "src/b.ts", code: "const b: number = 42;"},
            {name: "src/c.ts", code: "const c: boolean = true;"},
            {name: "src/d.ts", code: "const d: string = 'y';"},
            {name: "src/e.ts", code: "const e: number = 7;"},
        ]

        await setupProject(files, {type: "ts", tsTypes: true}, (tmpDir) => {
            for (const file of files) {
                const baseName = path.basename(file.name)
                const jsonPath = path.join(tmpDir, "ast_out", "src", `${baseName}.json`)
                const typemapPath = path.join(tmpDir, "ast_out", "src", `${baseName}.typemap`)
                expect(fs.existsSync(jsonPath)).toBe(true)
                expect(fs.existsSync(typemapPath)).toBe(true)

                const ast = JSON.parse(fs.readFileSync(jsonPath, "utf-8"))
                expect(ast.relativeName).toBe(path.join("src", baseName))
                expect(ast.ast?.program?.body?.length).toBeGreaterThan(0)

                const typemap = JSON.parse(fs.readFileSync(typemapPath, "utf-8"))
                expect(Object.keys(typemap).length).toBeGreaterThan(0)
            }
        })
    })

    it("should emit outputs for many files in a single run", async () => {
        await withTmpDir(async (tmpDir) => {
            const files = Array.from({length: 150}, (_, i) => ({
                name: path.join("src", `file-${i}.js`),
                code: `const value${i} = ${i};`,
            }))
            writeFiles(tmpDir, files)

            await start({
                src: tmpDir,
                type: "js",
                output: path.join(tmpDir, "ast_out"),
                recurse: true,
                tsTypes: false,
                "exclude-file": [],
            })

            let outputCount = 0
            for (let i = 0; i < 150; i++) {
                if (fs.existsSync(path.join(tmpDir, "ast_out", "src", `file-${i}.js.json`))) {
                    outputCount++
                }
            }
            expect(outputCount).toBe(150)
        })
    })
})
