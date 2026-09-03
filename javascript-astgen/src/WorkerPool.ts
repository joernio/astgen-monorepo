// Minimal fixed-size worker pool used to parallelise per-file Babel parsing.
//
// Each worker runs the full read+validate+parse+write pipeline for a single
// file (see AstWorker.ts). Only the Done/Error envelope crosses the thread
// boundary; the parsed AST itself is written to disk from inside the worker,
// so the multi-MB tree never has to be `structuredClone`d across threads.
//
// Bun's global `Worker` covers all runtime contexts (`bun run`, `bun test`,
// compiled binary); only the entrypoint specifier differs:
//
//   Compiled binary (`bun build --compile`): modules are embedded in a virtual
//   filesystem (`/$bunfs/`). The specifier must be a literal string so the
//   bundler statically rewrites it at compile time. `import.meta.url` does NOT
//   work there because the main entrypoint URL (`/$bunfs/root/<binary-name>`)
//   has no `.ts` extension, so relative resolution yields the wrong virtual
//   FS key.
//
//   `bun run` / `bun test`: absolute `file://` URL via `import.meta.url`; Bun
//   transpiles the `.ts` source directly.

import * as os from "node:os"

import type {Job, WorkerMsg} from "./AstWorker"

export type {Job, WorkerMsg}

type Pending = {
    job: Job
    resolve: (msg: WorkerMsg) => void
}

export function defaultPoolSize(): number {
    // Leave one core for the main thread / IO. Cap at 8 because each worker
    // owns its own JSC heap and can hold a multi-MB ParseResult in memory; past
    // ~8 the marginal speedup from extra parallelism is dwarfed by the memory
    // and worker-startup overhead.
    const cores = os.cpus()?.length ?? 1
    return Math.max(1, Math.min(cores - 1, 8))
}

function spawnWorker(queue: Pending[], markDead: () => void): Worker {
    const w = Bun.isStandaloneExecutable
        ? new Worker("./AstWorker.ts")
        : new Worker(new URL("./AstWorker.ts", import.meta.url).href)
    // One job in flight per reply, in FIFO order, so replies correlate with
    // the pending queue without message IDs. Bun queues messages posted before
    // the worker is ready, so submit() can post immediately after construction.
    w.addEventListener("message", (ev: MessageEvent<WorkerMsg>) => {
        const pending = queue.shift()
        if (pending) pending.resolve(ev.data)
    })
    w.addEventListener("error", (ev: ErrorEvent) => {
        // processOne in AstWorker catches all per-file errors, so an "error"
        // event means the worker itself is broken (e.g. module-load failure in
        // a compiled binary). It will never reply again: mark it dead so
        // submit() stops routing jobs to it — posting into the void would hang
        // the pipeline's Promise.all.
        markDead()
        while (queue.length > 0) {
            const p = queue.shift()!
            p.resolve({kind: "error", file: p.job.file, message: ev.message})
        }
    })
    return w
}

/**
 * Fixed-size pool of worker threads.
 *
 * The pool tracks in-flight jobs per worker. `submit(job)` returns a Promise
 * that resolves when *any* worker has finished that specific job. Order of
 * resolution across submitted jobs is therefore not guaranteed — callers that
 * need ordering must impose it themselves.
 */
export class WorkerPool {
    private readonly workers: Worker[] = []
    private readonly inflight: Pending[][] = []
    private readonly dead: boolean[] = []
    private nextIdx = 0

    constructor(size: number) {
        for (let i = 0; i < size; i++) {
            const queue: Pending[] = []
            this.inflight.push(queue)
            this.dead.push(false)
            this.workers.push(spawnWorker(queue, () => {
                this.dead[i] = true
            }))
        }
    }

    submit(job: Job): Promise<WorkerMsg> {
        // Pick the live worker with the smallest backlog to avoid piling work
        // onto one slow file while neighbours sit idle.
        let bestIdx = -1
        let bestLen = Infinity
        for (let i = 0; i < this.workers.length; i++) {
            const idx = (this.nextIdx + i) % this.workers.length
            if (this.dead[idx]) continue
            const len = this.inflight[idx].length
            if (len < bestLen) {
                bestIdx = idx
                bestLen = len
            }
        }
        if (bestIdx === -1) {
            // Fail fast instead of posting a job nobody will ever answer.
            return Promise.resolve({kind: "error", file: job.file, message: "no live workers left in pool"})
        }
        this.nextIdx = (bestIdx + 1) % this.workers.length

        return new Promise<WorkerMsg>((resolve) => {
            this.inflight[bestIdx].push({job, resolve})
            this.workers[bestIdx].postMessage(job)
        })
    }

    async shutdown(): Promise<void> {
        // Callers drain all submitted jobs before shutting down (Pipeline
        // awaits Promise.all(inflight) first), so terminate() cannot lose work.
        const exitPromises = this.workers.map((w) =>
            new Promise<void>((resolve) => {
                const timeout = setTimeout(resolve, 5000)
                w.addEventListener("close", () => {
                    clearTimeout(timeout)
                    resolve()
                }, {once: true})
            })
        )
        for (const w of this.workers) {
            w.terminate()
        }
        await Promise.all(exitPromises)
    }
}
