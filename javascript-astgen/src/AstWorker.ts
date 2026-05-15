// Worker entrypoint loaded by [WorkerPool.ts](./WorkerPool.ts).
//
// The worker performs a full per-file AST pipeline step:
//   1. Read the file as a Buffer.
//   2. Validate (EMSCRIPTEN marker, line length, LOC) without UTF-8 decoding
//      the rejection path.
//   3. Parse the surviving content — Vue files first run through the Vue
//      cleaner regex pipeline, JS/TS files go straight to Babel.
//   4. Stream the AST JSON straight to disk via JsonUtils.
//
// Doing all of this inside the worker means the multi-MB ParseResult tree is
// never serialized across the thread boundary — only the per-file Done/Error
// envelope is.

import {parentPort} from "node:worker_threads"
import * as fs from "node:fs"

import * as Parsing from "./Parsing"
import * as JsonUtils from "./JsonUtils"
import {validateBuffer} from "./FileUtils"
import {buildAstFile} from "./Writers"
import {getErrorMessage} from "./Errors"

export const PARSER_JS = "js"
export const PARSER_VUE = "vue"
export type ParserKind = typeof PARSER_JS | typeof PARSER_VUE

// `relativePath` and `outputPath` are computed by the dispatcher (see
// [Pipeline.ts](./Pipeline.ts)) and passed in so the same `path.relative` work
// isn't repeated inside the worker. The dispatcher is also responsible for
// ensureDir before submitting, so the output directory already exists here.
export type Job = {
    file: string
    relativePath: string
    outputPath: string
    parser: ParserKind
}

export type Done = { kind: "done"; file: string; skipped?: string }
export type Failure = { kind: "error"; file: string; message: string }
export type WorkerMsg = Done | Failure

function processOne(job: Job): WorkerMsg {
    try {
        const buf = fs.readFileSync(job.file)
        const validation = validateBuffer(buf)
        if (!validation.ok) {
            return {kind: "done", file: job.file, skipped: validation.reason}
        }
        const ast = job.parser === PARSER_VUE
            ? Parsing.toVueAst(validation.content)
            : Parsing.codeToJsAst(validation.content)
        JsonUtils.writeJsonStreamCircular(job.outputPath, buildAstFile(job.file, job.relativePath, ast))
        return {kind: "done", file: job.file}
    } catch (err) {
        return {kind: "error", file: job.file, message: getErrorMessage(err)}
    }
}

if (parentPort !== null) {
    const port = parentPort
    port.on("message", (job: Job | "shutdown") => {
        if (job === "shutdown") {
            port.close()
            return
        }
        port.postMessage(processOne(job))
    })
}
