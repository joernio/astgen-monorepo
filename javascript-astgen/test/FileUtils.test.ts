import * as fs from "node:fs"
import * as os from "node:os"
import * as path from "node:path"

import {fileExistsAndIsReadable} from "../src/FileUtils"

describe("fileExistsAndIsReadable", () => {
    let tmpDir: string

    beforeEach(() => {
        tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "astgen-fileutils-"))
    })

    afterEach(() => {
        fs.rmSync(tmpDir, {recursive: true, force: true})
    })

    it("returns true for an existing readable file", () => {
        const f = path.join(tmpDir, "a.txt")
        fs.writeFileSync(f, "hi")
        expect(fileExistsAndIsReadable(f)).toBe(true)
    })

    it("returns true for an existing readable directory", () => {
        expect(fileExistsAndIsReadable(tmpDir)).toBe(true)
    })

    it("returns false when the path does not exist", () => {
        expect(fileExistsAndIsReadable(path.join(tmpDir, "nope"))).toBe(false)
    })
})
