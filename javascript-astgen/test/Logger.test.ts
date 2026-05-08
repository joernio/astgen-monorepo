import * as path from "node:path"

import {MemoryLogger, setupTestFixture} from "./helpers"
import * as Defaults from "../src/Defaults"
import * as LoggerModule from "../src/Logger"
import {SourceDirNotReadableError} from "../src/Errors"
import start from "../src/Pipeline"

describe("Logger seam", () => {
    let logger: MemoryLogger

    beforeEach(() => {
        logger = new MemoryLogger()
        logger.install()
    })

    afterEach(() => {
        logger.uninstall()
    })

    it("warns when an EMSCRIPTEN-tagged file is encountered", async () => {
        const code = "// EMSCRIPTEN_START_ASM\nconst x = 1;"
        await setupTestFixture(code, "main.js", {tsTypes: false}, () => {
            expect(logger.warnedWith("EMSCRIPTEN", "main.js")).toBe(true)
        })
    })

    it("warns when a file exceeds the LOC limit", async () => {
        const code = Array(Defaults.MAX_LOC_IN_FILE + 1).fill("const x = 1;").join("\n")
        await setupTestFixture(code, "huge.ts", {tsTypes: false}, () => {
            expect(logger.warnedWith("more than", String(Defaults.MAX_LOC_IN_FILE))).toBe(true)
        })
    })

    it("warns when a file contains a line that exceeds MAX_LINE_LENGTH", async () => {
        const longLine = "const x = \"" + "a".repeat(Defaults.MAX_LINE_LENGTH + 1) + "\";"
        await setupTestFixture(`const y = 1;\n${longLine}\n`, "longline.ts", {tsTypes: false}, () => {
            expect(logger.warnedWith("exceeds", String(Defaults.MAX_LINE_LENGTH))).toBe(true)
        })
    })

    it("warns when a file exceeds MAX_FILE_SIZE_BYTES", async () => {
        const code = "x".repeat(Defaults.MAX_FILE_SIZE_BYTES + 1)
        await setupTestFixture(code, "huge.ts", {tsTypes: false}, () => {
            expect(logger.warnedWith("exceeds maximum file size of", String(Defaults.MAX_FILE_SIZE_BYTES))).toBe(true)
        })
    })

    it("emits an info message for each successfully written AST file", async () => {
        await setupTestFixture("const x = 1;", "main.ts", {tsTypes: false}, () => {
            expect(logger.infoedWith("Converted AST for", "main.ts")).toBe(true)
        })
    })
})

describe("start() error handling", () => {
    it("throws SourceDirNotReadableError instead of calling process.exit", async () => {
        await expect(
            start({
                src: path.join("/", "definitely", "does", "not", "exist", String(Date.now())),
                output: "/tmp/ignored",
                recurse: true,
                tsTypes: false,
                "exclude-file": [],
            }),
        ).rejects.toBeInstanceOf(SourceDirNotReadableError)
    })
})

describe("setLogger / resetLogger", () => {
    it("setLogger returns the previous logger so callers can restore it", () => {
        const a: LoggerModule.Logger = {info: jest.fn(), warn: jest.fn(), error: jest.fn()}
        const b: LoggerModule.Logger = {info: jest.fn(), warn: jest.fn(), error: jest.fn()}
        const original = LoggerModule.setLogger(a)
        try {
            const previous = LoggerModule.setLogger(b)
            expect(previous).toBe(a)
        } finally {
            LoggerModule.setLogger(original)
        }
    })
})
