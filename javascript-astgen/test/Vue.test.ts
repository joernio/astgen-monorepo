import * as fs from "node:fs"
import * as path from "node:path"

import {setupTestFixture} from "./helpers"

describe("astgen Vue integration", () => {
    it("emits an AST JSON for a single .vue file when type=vue", async () => {
        const code = `<template>
<div class="hello">{{ greeting }}</div>
</template>
<script>
export default {
  name: 'Hello',
  data() {
    return { greeting: 'world' }
  }
}
</script>
<style>
.hello { color: red; }
</style>
`

        await setupTestFixture(
            code,
            "App.vue",
            {type: "vue", tsTypes: false},
            (tmpDir) => {
                const out = path.join(tmpDir, "ast_out", "App.vue.json")
                expect(fs.existsSync(out)).toBe(true)

                const parsed = JSON.parse(fs.readFileSync(out, "utf-8"))
                expect(parsed.relativeName).toBe("App.vue")
                // The script body is preserved as a JS AST and should contain at least one statement
                expect(parsed.ast?.program?.body?.length).toBeGreaterThan(0)
                // No `.typemap` is produced for Vue files
                expect(fs.existsSync(path.join(tmpDir, "ast_out", "App.vue.typemap"))).toBe(false)
            },
        )
    })

    it("skips Vue files when type=ts (only .ts/.js extensions are picked up)", async () => {
        const code = "<template><div></div></template>"

        await setupTestFixture(
            code,
            "App.vue",
            {type: "ts", tsTypes: false},
            (tmpDir) => {
                expect(fs.existsSync(path.join(tmpDir, "ast_out", "App.vue.json"))).toBe(false)
            },
        )
    })
})
