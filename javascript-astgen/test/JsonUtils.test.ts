import * as fs from "node:fs"
import * as os from "node:os"
import * as path from "node:path"

import {writeJsonStreamCircular, writeMapToJsonFile} from "../src/JsonUtils"
import {encodePos} from "../src/TscUtils"

function tmpFile(): string {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), "astgen-jsonutils-"))
    return path.join(dir, "out.json")
}

function readAndCleanup(file: string): string {
    const content = fs.readFileSync(file, "utf-8")
    fs.rmSync(path.dirname(file), {recursive: true, force: true})
    return content
}

describe("writeJsonStreamCircular", () => {
    it("matches JSON.stringify for plain values", () => {
        const f = tmpFile()
        const data = {a: 1, b: "two", c: true, d: null, e: [1, 2, 3]}
        writeJsonStreamCircular(f, data)
        expect(readAndCleanup(f)).toBe(JSON.stringify(data))
    })

    it("omits undefined, function, and symbol values in objects", () => {
        const f = tmpFile()
        const data = {a: 1, b: undefined, c: () => 42, d: Symbol("x"), e: 5}
        writeJsonStreamCircular(f, data)
        expect(readAndCleanup(f)).toBe("{\"a\":1,\"e\":5}")
    })

    it("converts undefined, function, and symbol elements to null in arrays", () => {
        const f = tmpFile()
        writeJsonStreamCircular(f, [1, undefined, () => {}, Symbol("x"), 2])
        expect(readAndCleanup(f)).toBe("[1,null,null,null,2]")
    })

    it("represents non-finite numbers as null (matches JSON.stringify)", () => {
        const f = tmpFile()
        writeJsonStreamCircular(f, {n: NaN, p: Infinity, m: -Infinity, ok: 3.14})
        expect(readAndCleanup(f)).toBe("{\"n\":null,\"p\":null,\"m\":null,\"ok\":3.14}")
    })

    it("breaks circular references by skipping the second visit", () => {
        const f = tmpFile()
        const a: any = {name: "a"}
        const b: any = {name: "b", a}
        a.b = b
        writeJsonStreamCircular(f, a)
        // The cycle a -> b -> a is broken by omitting the second `a`. Specifically,
        // when serializing b.a we have already added `a` to `seen`, so the property is skipped.
        const text = readAndCleanup(f)
        const parsed = JSON.parse(text)
        expect(parsed.name).toBe("a")
        expect(parsed.b.name).toBe("b")
        // The cycle was broken — `b.a` is missing entirely.
        expect("a" in parsed.b).toBe(false)
    })

    it("invokes toJSON when present (Date-like objects)", () => {
        const f = tmpFile()
        const date = new Date("2024-01-02T03:04:05.000Z")
        writeJsonStreamCircular(f, {when: date})
        expect(readAndCleanup(f)).toBe("{\"when\":\"2024-01-02T03:04:05.000Z\"}")
    })

    it("removes the output file when serialization throws mid-stream", () => {
        // Simulate a failure by passing a value whose toJSON throws.
        const f = tmpFile()
        const exploding = {
            toJSON() {
                throw new Error("boom")
            },
        }
        expect(() => writeJsonStreamCircular(f, exploding)).toThrow(/boom/)
        expect(fs.existsSync(f)).toBe(false)
        fs.rmSync(path.dirname(f), {recursive: true, force: true})
    })
})

describe("writeMapToJsonFile", () => {
    it("serializes packed (start,end) keys back to 'start:end' strings", () => {
        const f = tmpFile()
        const map = new Map<number, string>()
        map.set(encodePos(0, 11), "void")
        map.set(encodePos(12, 27), "string")
        writeMapToJsonFile(f, map)
        const parsed = JSON.parse(readAndCleanup(f))
        expect(parsed["0:11"]).toBe("void")
        expect(parsed["12:27"]).toBe("string")
    })

    it("emits a valid empty object for an empty map", () => {
        const f = tmpFile()
        writeMapToJsonFile(f, new Map())
        expect(readAndCleanup(f)).toBe("{}")
    })

    it("escapes special characters in values via JSON.stringify", () => {
        const f = tmpFile()
        const map = new Map<number, string>()
        map.set(encodePos(0, 5), "a \"quoted\" string\nwith newline")
        writeMapToJsonFile(f, map)
        const parsed = JSON.parse(readAndCleanup(f))
        expect(parsed["0:5"]).toBe("a \"quoted\" string\nwith newline")
    })
})
