import {cleanVueCode} from "../src/VueCodeCleaner"

describe("cleanVueCode", () => {
    it("erases comment contents while preserving overall length", () => {
        const input = "<!-- secret -->\nlet x = 1;"
        const output = cleanVueCode(input)
        expect(output).not.toContain("secret")
        expect(output).not.toContain("<!--")
        expect(output).toContain("let x = 1;")
        // every non-whitespace character of the comment is replaced with a single space
        expect(output.length).toBe(input.length)
    })

    it("strips <script> tags around a multi-line body and preserves the body as JS", () => {
        // Note: vueCleaningRegex's `<script.*>` does not span newlines, so the
        // open/close tags only match independently when the body sits on its
        // own line. This is the realistic SFC layout.
        const input = "<script>\nlet x = 1;\n</script>"
        const output = cleanVueCode(input)
        expect(output).not.toContain("<script")
        expect(output).not.toContain("</script>")
        expect(output).toContain("let x = 1;")
    })

    it("erases <style> blocks", () => {
        const input = "<script>\nlet x = 1;\n</script>\n<style>.x { color: red; }</style>"
        const output = cleanVueCode(input)
        expect(output).not.toContain("color: red")
        expect(output).not.toContain("<style")
        expect(output).toContain("let x = 1;")
    })

    it("normalises dynamic [arg] bindings by erasing the brackets and colon", () => {
        const input = "<template>\n<div :[dyn]=\"x\"></div>\n</template>"
        const output = cleanVueCode(input)
        // The :[ and ] become spaces but the inner identifier survives
        expect(output).toContain("dyn")
        expect(output).not.toContain(":[")
        expect(output).not.toContain("]=")
    })

    it("strips colon/at/dot prop prefixes inside templates and rewrites dotted prop names", () => {
        const input = "<template>\n<div :foo.bar=\"x\" @click=\"go\">{{ msg }}</div>\n</template>"
        const output = cleanVueCode(input)
        // `:foo.bar` -> ` foo-bar`; `@click` -> ` click`
        expect(output).toContain(" foo-bar=\"x\"")
        expect(output).toContain(" click=\"go\"")
        expect(output).not.toContain(":foo")
        expect(output).not.toContain("@click")
        // `{{ msg }}` -> `{ msg  }`-style (single curlies with whitespace)
        expect(output).not.toContain("{{")
        expect(output).not.toContain("}}")
        expect(output).toContain("msg")
    })

    it("self-closes unterminated <img> tags inside templates", () => {
        // The cleaner triggers on `<img ... X >` where X is any non-slash char
        // (covers cases where authors leave a stray space before `>`).
        const input = "<template>\n<img src=\"y\"  >\n</template>"
        const output = cleanVueCode(input)
        expect(output).toContain("<img src=\"y\" />")
    })

    it("returns empty string unchanged", () => {
        expect(cleanVueCode("")).toBe("")
    })

    it("leaves plain JS untouched (no Vue tags to clean)", () => {
        const input = "const a = 1;\nfunction f() { return a + 1; }"
        expect(cleanVueCode(input)).toBe(input)
    })
})
