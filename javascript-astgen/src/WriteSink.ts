import * as fs from "node:fs"

import {TypeMap} from "./TscUtils"
import * as JsonUtils from "./JsonUtils"

/**
 * Abstraction over the side-effecting writes performed by the AST/typemap
 * pipeline. Production code uses {@link FsWriteSink}, which mirrors the
 * historical behaviour (mkdir + buffered streaming JSON writes via
 * {@link JsonUtils}). Tests can supply an in-memory implementation to assert on
 * generated output without touching the filesystem.
 *
 * Why an interface and not a bag of free functions:
 * - keeps `Writers.ts` agnostic of `fs` so it is reachable from unit tests
 * - allows future sinks (e.g. tar/zip output, stdout streaming) to slot in
 *   without changing the pipeline.
 */
export interface WriteSink {
    /**
     * Idempotently ensures `dir` exists. Implementations are expected to
     * deduplicate concurrent calls for the same path (see {@link DirCache} in
     * [Writers.ts](./Writers.ts)).
     */
    ensureDir(dir: string): Promise<void>

    /**
     * Writes the AST JSON for a single source file. `data` may contain circular
     * references — the default implementation handles them via
     * {@link JsonUtils.writeJsonStreamCircular}.
     */
    writeAstJson(filePath: string, data: unknown): Promise<void>

    /**
     * Writes the typemap JSON for a single source file. Keys in `map` are
     * packed (start, end) positions; the default implementation decodes them to
     * `"start:end"` string keys via {@link JsonUtils.writeMapToJsonFile}.
     */
    writeTypeMapJson(filePath: string, map: TypeMap): Promise<void>
}

/**
 * Default {@link WriteSink} backed by `node:fs` and {@link JsonUtils}.
 *
 * Output directories are deduplicated: two concurrent writes targeting the same
 * directory issue at most one `mkdir -p` syscall. On mkdir failure the inflight
 * slot is cleared (via `.finally`) so a transient error (e.g. EMFILE under high
 * concurrency) does not poison the cache and permanently fail every subsequent
 * write to that directory.
 */
export class FsWriteSink implements WriteSink {
    private readonly created = new Set<string>()
    private readonly inflight = new Map<string, Promise<void>>()

    async ensureDir(dir: string): Promise<void> {
        if (this.created.has(dir)) return
        let pending = this.inflight.get(dir)
        if (!pending) {
            pending = fs.promises.mkdir(dir, {recursive: true})
                .then(() => {
                    this.created.add(dir)
                })
                .finally(() => {
                    this.inflight.delete(dir)
                })
            this.inflight.set(dir, pending)
        }
        await pending
    }

    async writeAstJson(filePath: string, data: unknown): Promise<void> {
        JsonUtils.writeJsonStreamCircular(filePath, data)
    }

    async writeTypeMapJson(filePath: string, map: TypeMap): Promise<void> {
        JsonUtils.writeMapToJsonFile(filePath, map)
    }
}
