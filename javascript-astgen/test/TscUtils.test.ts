import * as path from "node:path"

import TscUtils, {decodePos, encodePos} from "../src/TscUtils"
import * as Defaults from "../src/Defaults"
import {FileFixture, withTmpDir, writeFiles} from "./helpers"

describe("encodePos / decodePos", () => {
    it("round-trips representative positions", () => {
        const cases: [number, number][] = [
            [0, 0],
            [0, 1],
            [1, 0],
            [12, 27],
            [1234, 5678],
            [Defaults.MAX_FILE_SIZE_BYTES - 1, Defaults.MAX_FILE_SIZE_BYTES - 1],
        ]
        for (const [s, e] of cases) {
            const key = encodePos(s, e)
            expect(decodePos(key)).toEqual([s, e])
        }
    })

    it("round-trips a randomized sample within the supported range", () => {
        // POS_SHIFT (2^26) is more than 12x MAX_FILE_SIZE_BYTES (5MB), so any
        // position within a permitted file fits comfortably below the shift.
        const max = Defaults.MAX_FILE_SIZE_BYTES
        for (let i = 0; i < 200; i++) {
            const s = Math.floor(Math.random() * max)
            const e = Math.floor(Math.random() * max)
            const key = encodePos(s, e)
            expect(decodePos(key)).toEqual([s, e])
        }
    })

    it("produces distinct keys for distinct (start, end) pairs near the boundary", () => {
        const max = Defaults.MAX_FILE_SIZE_BYTES - 1
        expect(encodePos(max, 0)).not.toEqual(encodePos(0, max))
        expect(encodePos(max, max)).not.toEqual(encodePos(max - 1, max))
    })
})

describe("typeMapForFile determinism", () => {
    // Cross-file union mixing literal members with class types declared in
    // other (global-scope) files. Without stableTypeOrdering in
    // Defaults.DEFAULT_TSC_OPTIONS, tsc orders union members by
    // encounter-order type IDs, so the rendered string flips depending on the
    // program's root file order and on which file the checker is queried for
    // first. The Pipeline feeds files to TscUtils in sorted order, but these
    // tests deliberately shuffle both orders to prove the output is immune.
    const files: FileFixture[] = [
        {name: "g1.ts", code: `class C1 { a: string }\n`},
        {name: "g2.ts", code: `class C2 { b: number }\nconst z = new C2()\n`},
        {name: "u.ts", code: `type T = "x" | C1 | "y" | C2\nconst t: T = "x"\n`},
    ]

    it("renders identical type strings for any program and query order", async () => {
        await withTmpDir(async (tmpDir) => {
            writeFiles(tmpDir, files)
            const abs = (name: string): string => path.join(tmpDir, name)
            const rootOrders: string[][] = [
                ["g1.ts", "g2.ts", "u.ts"],
                ["u.ts", "g2.ts", "g1.ts"],
                ["g2.ts", "u.ts", "g1.ts"],
            ]
            const queryOrders: string[][] = [
                ["g2.ts", "u.ts"],
                ["u.ts", "g2.ts"],
            ]

            const rendered: string[] = []
            for (const roots of rootOrders) {
                for (const query of queryOrders) {
                    const utils = new TscUtils(roots.map(abs))
                    let uMap: Map<number, string> = new Map()
                    for (const f of query) {
                        const m = utils.typeMapForFile(abs(f))
                        if (f === "u.ts") uMap = m
                    }
                    // Sort entries so the comparison does not rely on
                    // traversal/insertion order.
                    rendered.push(JSON.stringify([...uMap].sort((a, b) => a[0] - b[0])))
                }
            }
            for (const r of rendered) {
                expect(r).toBe(rendered[0])
            }
        })
    })

    it("renders union members in canonical content order", async () => {
        await withTmpDir(async (tmpDir) => {
            writeFiles(tmpDir, files)
            const utils = new TscUtils(files.map((f) => path.join(tmpDir, f.name)))
            const map = utils.typeMapForFile(path.join(tmpDir, "u.ts"))
            const code = files[2].code
            const start = code.indexOf("t: T")
            expect(map.get(encodePos(start, start + 1))).toBe(`"x" | "y" | C1 | C2`)
        })
    })
})
