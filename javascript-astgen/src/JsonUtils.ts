import * as fs from "node:fs"
import {decodePos} from "./TscUtils"
import {getErrorMessage} from "./Errors"

const STREAM_BUFFER_SIZE = 1024 * 1024

/**
 * Opens `filePath`, streams UTF-8 via a rolling ~1MB buffer and `writeSync`,
 * and closes the fd. Does not build one giant string per flush (avoids
 * `chunks.join` over many fragments at each threshold).
 *
 * **Why synchronous I/O?** The pipeline serialises one file at a time on a
 * single async iterator (see [Pipeline.ts](./Pipeline.ts)), so there is no
 * concurrent producer waiting for the event loop. A synchronous `writeSync`
 * loop avoids per-chunk Promise allocation for the common case of a few-MB
 * AST and benchmarks ~30% faster than `fs.promises.write` chunking. If the
 * pipeline ever introduces parallelism, this should be revisited.
 *
 * On failure after the file was created, unlinks it best-effort so partial
 * JSON is not left behind.
 */
function withBufferedWriter(filePath: string, fn: (write: (s: string) => void) => void): void {
    let fd: number | undefined
    try {
        fd = fs.openSync(filePath, "w")
        let buf = ""

        function flush(): void {
            if (buf.length > 0) {
                fs.writeSync(fd as number, buf)
                buf = ""
            }
        }

        function write(s: string): void {
            buf += s
            if (buf.length >= STREAM_BUFFER_SIZE) flush()
        }

        fn(write)
        flush()
    } catch (err) {
        try {
            if (fd !== undefined) {
                fs.closeSync(fd)
                fd = undefined
            }
        } catch {
            /* ignore close errors while handling failure */
        }
        try {
            fs.unlinkSync(filePath)
        } catch {
            /* ignore missing file */
        }
        throw new Error(`Failed to write ${filePath}: ${getErrorMessage(err)}`)
    } finally {
        if (fd !== undefined) {
            fs.closeSync(fd)
        }
    }
}

/**
 * Writes a Map<number, string> as a JSON object directly to a file using buffered streaming,
 * avoiding both the intermediate plain object from Object.fromEntries and the full JSON string.
 * Keys are decoded from packed (start, end) positions to "start:end" strings in the output.
 */
export function writeMapToJsonFile(filePath: string, map: Map<number, string>): void {
    withBufferedWriter(filePath, (write) => {
        write("{")
        let first = true
        for (const [key, value] of map) {
            if (!first) write(",")
            first = false
            const [start, end] = decodePos(key)
            write(`"${start}:${end}"`)
            write(":")
            write(JSON.stringify(value))
        }
        write("}")
    })
}

/**
 * Writes a value as JSON directly to a file using buffered streaming,
 * handling circular references without materializing the full JSON string in memory.
 * Semantics match JSON.stringify with getCircularReplacer:
 * - Circular/duplicate object references are omitted (skipped in objects, null in arrays)
 * - undefined, functions, and symbols are omitted in objects and become null in arrays
 */
export function writeJsonStreamCircular(filePath: string, data: any): void {
    const seen = new WeakSet<object>()

    function shouldSkip(v: any): boolean {
        return v === undefined || typeof v === "function" || typeof v === "symbol" ||
            (typeof v === "object" && v !== null && seen.has(v))
    }

    withBufferedWriter(filePath, (write) => {
        function writeValue(value: any): void {
            if (value === null) { write("null"); return }
            switch (typeof value) {
                case "string":
                    write(JSON.stringify(value))
                    return
                case "number":
                    write(isFinite(value) ? String(value) : "null")
                    return
                case "boolean":
                    write(value ? "true" : "false")
                    return
                case "object":
                    seen.add(value)
                    if (typeof value.toJSON === "function") {
                        const resolved = value.toJSON()
                        if (typeof resolved === "object" && resolved !== null) seen.add(resolved)
                        writeValue(resolved)
                        return
                    }
                    if (Array.isArray(value)) {
                        writeArray(value)
                    } else {
                        writeObject(value)
                    }
                    return
                default:
                    write("null")
                    return
            }
        }

        function writeArray(arr: any[]): void {
            write("[")
            for (let i = 0; i < arr.length; i++) {
                if (i > 0) write(",")
                if (shouldSkip(arr[i])) {
                    write("null")
                } else {
                    writeValue(arr[i])
                }
            }
            write("]")
        }

        function writeObject(obj: object): void {
            write("{")
            let first = true
            for (const key of Object.keys(obj)) {
                const v = (obj as any)[key]
                if (shouldSkip(v)) continue
                if (!first) write(",")
                first = false
                write(JSON.stringify(key))
                write(":")
                writeValue(v)
            }
            write("}")
        }

        writeValue(data)
    })
}
