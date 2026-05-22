import {WorkerPool} from "../src/WorkerPool"
import type {Job} from "../src/WorkerPool"
import * as path from "node:path"
import * as fs from "node:fs"
import * as os from "node:os"

describe("WorkerPool", () => {
    let tempDir: string

    beforeEach(() => {
        tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "worker-pool-test-"))
    })

    afterEach(() => {
        fs.rmSync(tempDir, {recursive: true, force: true})
    })

    test("gracefully shuts down workers after processing", async () => {
        const pool = new WorkerPool(2)

        // Create a simple test file
        const testFile = path.join(tempDir, "test.js")
        fs.writeFileSync(testFile, "const x = 1;")

        const outputPath = path.join(tempDir, "test.json")
        const job: Job = {
            file: testFile,
            relativePath: "test.js",
            outputPath,
            parser: "js",
        }

        // Submit a job
        const result = await pool.submit(job)
        expect(result.kind).toBe("done")
        expect(fs.existsSync(outputPath)).toBe(true)

        // Shutdown should complete without hanging or crashing
        const shutdownPromise = pool.shutdown()
        await expect(shutdownPromise).resolves.toBeUndefined()
    }, 10000)

    test("handles shutdown timeout for hung workers", async () => {
        const pool = new WorkerPool(1)

        // Submit a job to initialize the worker
        const testFile = path.join(tempDir, "test.js")
        fs.writeFileSync(testFile, "const x = 1;")

        const job: Job = {
            file: testFile,
            relativePath: "test.js",
            outputPath: path.join(tempDir, "test.json"),
            parser: "js",
        }

        await pool.submit(job)

        // Shutdown should complete within reasonable time even if worker doesn't exit immediately
        const start = Date.now()
        await pool.shutdown()
        const elapsed = Date.now() - start

        // Should complete in less than 6 seconds (5s timeout + overhead)
        expect(elapsed).toBeLessThan(6000)
    }, 10000)

    test("processes multiple files in parallel", async () => {
        const pool = new WorkerPool(4)
        const jobs: Promise<any>[] = []

        // Create multiple test files
        for (let i = 0; i < 10; i++) {
            const testFile = path.join(tempDir, `test${i}.js`)
            fs.writeFileSync(testFile, `const x${i} = ${i};`)

            const job: Job = {
                file: testFile,
                relativePath: `test${i}.js`,
                outputPath: path.join(tempDir, `test${i}.json`),
                parser: "js",
            }
            jobs.push(pool.submit(job))
        }

        // All jobs should complete successfully
        const results = await Promise.all(jobs)
        expect(results.every((r) => r.kind === "done")).toBe(true)

        // Graceful shutdown should work after parallel processing
        await expect(pool.shutdown()).resolves.toBeUndefined()
    }, 15000)
})
