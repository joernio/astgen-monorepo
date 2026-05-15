// Minimal fixed-size worker pool used to parallelise per-file Babel parsing.
//
// Each worker runs the full read+validate+parse+write pipeline for a single
// file (see [AstWorker.ts](./AstWorker.ts)). Only the Done/Error envelope
// crosses the thread boundary; the parsed AST itself is written to disk from
// inside the worker, so the multi-MB tree never has to be `structuredClone`d
// across threads.

import {Worker} from "node:worker_threads"
import * as os from "node:os"
import * as path from "node:path"

import type {Job, WorkerMsg} from "./AstWorker"

export type {Job, WorkerMsg}

type Pending = {
    job: Job
    resolve: (msg: WorkerMsg) => void
}

export function defaultPoolSize(): number {
    // Leave one core for the main thread / IO. Cap at 8 so we don't blow the
    // file descriptor budget on machines with many cores.
    const cores = os.cpus()?.length ?? 1
    return Math.max(1, Math.min(cores - 1, 8))
}

/**
 * Round-robin pool of worker threads.
 *
 * The pool tracks in-flight jobs per worker. `submit(job)` returns a Promise
 * that resolves when *any* worker has finished that specific job. Order of
 * resolution across submitted jobs is therefore not guaranteed — callers that
 * need ordering must impose it themselves.
 */
export class WorkerPool {
    private readonly workers: Worker[] = []
    private readonly inflight: Pending[][] = []
    private nextIdx = 0

    constructor(size: number) {
        // Sibling `AstWorker.js`. `node:worker_threads` cannot load `.ts`, so
        // this requires the project to be built. Production runs from `dist/`;
        // tests use jest's moduleNameMapper to map `../src/X` → `../dist/X`.
        const scriptPath = path.join(__dirname, "AstWorker.js")
        for (let i = 0; i < size; i++) {
            const w = new Worker(scriptPath)
            const queue: Pending[] = []
            this.workers.push(w)
            this.inflight.push(queue)
            w.on("message", (msg: WorkerMsg) => {
                const pending = queue.shift()
                if (pending) pending.resolve(msg)
            })
            w.on("error", (err: Error) => {
                // Drain any remaining pending jobs as worker-level errors so
                // the pipeline doesn't hang on a crashed worker.
                while (queue.length > 0) {
                    const p = queue.shift()!
                    p.resolve({kind: "error", file: p.job.file, message: err.message})
                }
            })
        }
    }

    submit(job: Job): Promise<WorkerMsg> {
        // Pick the worker with the smallest backlog. Avoids piling work on
        // one slow file (e.g. a multi-MB AST) while neighbours sit idle.
        let bestIdx = this.nextIdx
        let bestLen = this.inflight[bestIdx].length
        for (let i = 1; i < this.workers.length; i++) {
            const idx = (this.nextIdx + i) % this.workers.length
            const len = this.inflight[idx].length
            if (len < bestLen) {
                bestIdx = idx
                bestLen = len
            }
        }
        this.nextIdx = (bestIdx + 1) % this.workers.length

        return new Promise<WorkerMsg>((resolve) => {
            this.inflight[bestIdx].push({job, resolve})
            this.workers[bestIdx].postMessage(job)
        })
    }

    async shutdown(): Promise<void> {
        for (const w of this.workers) {
            w.postMessage("shutdown")
        }
        await Promise.all(this.workers.map((w) => w.terminate()))
    }
}
