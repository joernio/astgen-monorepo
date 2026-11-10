import * as fs from "node:fs"
import * as path from "node:path"

import start from "../src/Pipeline"
import {setupProject, withTmpDir, writeFiles} from "./helpers"

const CODE = "const x = 1;"

/** Runs astgen over `names` and returns the subset that produced an AST. */
async function parsedFiles(names: string[]): Promise<string[]> {
    const parsed: string[] = []
    await setupProject(
        names.map((name) => ({name, code: CODE})),
        {tsTypes: false},
        (tmpDir) => {
            for (const name of names) {
                if (fs.existsSync(path.join(tmpDir, "ast_out", name + ".json"))) parsed.push(name)
            }
        },
    )
    return parsed.sort()
}

describe("default file skip rules", () => {
    it("skips test, spec, mock, minified and declaration files", async () => {
        const skipped = [
            "a.test.js",
            "a.spec.ts",
            "a.mock.js",
            "a.e2e.js",
            "a.min.js",
            "a-min.js",
            "a.d.ts",
            "a-test.js",
            "a_test.js",
            "test.js",
            "Test.js",
            "spec.ts",
            "webpack.conf.js",
            "conf.js",
        ]
        expect(await parsedFiles([...skipped, "main.js"])).toEqual(["main.js"])
    })

    it("skips bundler output", async () => {
        const skipped = [
            "chunk-vendors.js",
            "chunk-vendors.1a2b3c.js",
            "app~.js",
            "app~1a2b3c.js",
            "main.chunk.js",
            "bundle.js",
            "app.bundle.js",
            "app.min.map.js",
        ]
        expect(await parsedFiles([...skipped, "main.js"])).toEqual(["main.js"])
    })

    it("skips build and linter tooling configuration", async () => {
        const skipped = [
            "jest.config.js",
            "jest.config.ts",
            "webpack.config.js",
            "webpack.dev.js",
            "vue.config.js",
            "babel.config.js",
            "rollup.config.js",
            "rollup.config.mjs",
            "eslint-local-rules.js",
            "Gruntfile.js",
            "src/Gruntfile.js",
        ]
        expect(await parsedFiles([...skipped, "main.js"])).toEqual(["main.js"])
    })

    it("skips generated type files", async () => {
        const skipped = ["api.types.js", "api.cjs.js", "api.t.ts"]
        expect(await parsedFiles([...skipped, "main.js"])).toEqual(["main.js"])
    })

    it("keeps files that merely resemble ignored names", async () => {
        const kept = [
            "webpack.js",
            "rollup.ts",
            "types.js",
            "config.js",
            "bundler.js",
            "chunk.js",
            "src/app.ts",
            "latest.js",
            "contest.ts",
            "attest.js",
            "protest.js",
            "respec.js",
            "userTest.js",
            "reconf.js",
            "mybundle.js",
            "myjest.config.js"
        ]
        expect(await parsedFiles(kept)).toEqual([...kept].sort())
    })

    it("only matches the file name, not the surrounding path", async () => {
        const kept = ["webpack.stuff/main.js", "jest.config.d/main.js", "my.chunk/main.js"]
        expect(await parsedFiles(kept)).toEqual([...kept].sort())
    })
})

describe("exclude paths and symlinked source directories", () => {
    /**
     * Creates `real/` with a few sources and `link` pointing at it, then runs astgen with `srcName` as the source
     * directory and the given exclude paths, and returns the generated ASTs relative to the output directory.
     * A junction is used so that the test also works on Windows without elevated privileges.
     */
    async function parsedWithSymlink(
        srcName: "real" | "link",
        excludes: (realDir: string, linkDir: string) => string[],
    ): Promise<string[]> {
        let parsed: string[] = []
        await withTmpDir(async (tmpDir) => {
            const realDir = fs.realpathSync(path.join(tmpDir))
            const projectDir = path.join(realDir, "real")
            const linkDir = path.join(realDir, "link")
            writeFiles(projectDir, [
                {name: "a.js", code: CODE},
                {name: "b.js", code: CODE},
                {name: path.join("sub", "c.js"), code: CODE},
            ])
            fs.symlinkSync(projectDir, linkDir, "junction")
            const output = path.join(realDir, "out")
            await start({
                src: srcName === "real" ? projectDir : linkDir,
                output,
                type: "js",
                recurse: true,
                tsTypes: false,
                "exclude-file": excludes(projectDir, linkDir),
            })
            parsed = ["a.js", "b.js", path.join("sub", "c.js")].filter((name) =>
                fs.existsSync(path.join(output, name + ".json")),
            )
        })
        return parsed.sort()
    }

    it("excludes a file given by its real path when the source is reached through a symlink", async () => {
        expect(await parsedWithSymlink("link", (real) => [path.join(real, "a.js")])).toEqual(
            ["b.js", path.join("sub", "c.js")].sort(),
        )
    })

    it("excludes a file given by a symlinked path when the source is the real directory", async () => {
        expect(await parsedWithSymlink("real", (_, link) => [path.join(link, "a.js")])).toEqual(
            ["b.js", path.join("sub", "c.js")].sort(),
        )
    })

    it("excludes a directory given by its real path when the source is reached through a symlink", async () => {
        expect(await parsedWithSymlink("link", (real) => [path.join(real, "sub")])).toEqual(["a.js", "b.js"])
    })

    it("keeps excluding by the path the source was given with", async () => {
        expect(await parsedWithSymlink("link", (_, link) => [path.join(link, "a.js")])).toEqual(
            ["b.js", path.join("sub", "c.js")].sort(),
        )
    })

    it("does not exclude siblings of an excluded file", async () => {
        expect(await parsedWithSymlink("link", (real) => [path.join(real, "sub", "other.js")])).toEqual(
            ["a.js", "b.js", path.join("sub", "c.js")].sort(),
        )
    })
})
