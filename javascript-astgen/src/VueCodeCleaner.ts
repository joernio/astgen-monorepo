// Attribute list of an HTML tag; attribute values may contain `>` (e.g. `generic="A extends B<C>"`).
const vueTagAttributes = `(?:\\s+[^\\s"'<>\\/=]+(?:\\s*=\\s*(?:"[^"]*"|'[^']*'))?)*`
// `<script>` / `<style>` tags only count at the start of a line, so that strings in script code
// such as `'<style>'.length` are not mistaken for SFC blocks. Handles multi-line and self-closing tags.
const vueCleaningRegex = new RegExp(
    `(?<=^[ \\t]*)<\\/?script${vueTagAttributes}\\s*>` +
    `|(?<=^[ \\t]*)<style${vueTagAttributes}\\s*\\/>` +
    `|(?<=^[ \\t]*)<style${vueTagAttributes}\\s*>[\\s\\S]*?<\\/style>` +
    `|<\\/*br>`,
    "gim",
)
// Keeps `{{ ... }}` interpolations and whole tags untouched and matches any other `{` / `}`: in text
// content these would be read as JSX expressions.
const vueInterpolationOrTagOrBraceRegex = new RegExp(
    `\\{\\{[\\s\\S]*?\\}\\}|<\\/?[A-Za-z][\\w.:-]*${vueTagAttributes}\\s*\\/?>|[{}]`,
    "g",
)
// The root `<template>` must start a line and be spelled exactly, so generics like `<TemplateFoo>` in script code do not match.
const vueTemplateRegex = /(?<=^[ \t]*)(<template(?:\s.*)?>)([\s\S]*)(<\/template>)/gm
const vueCommentRegex = /<!--[\s\S]*?-->/ig
const vueBindRegex = /(:\[)(\S*?)(])/ig
const vuePropRegex = /\s([.:@])(\S*?=)/ig
const vueOpenTagRegex = /(<[A-Za-z][\w.-]*)((?:\s+[^\s"'<>\/=]+(?:=(?:"[^"]*"|'[^']*'))?)*)/g
const vueAttributeRegex = /(\s+)([^\s"'<>\/=]+)(=(?:"[^"]*"|'[^']*'))?/g
const vueOpenImgTag = /(<img)((?!>)[\s\S]+?)( [^\/]>)/ig

/**
 * Rewrites a single Vue attribute so it is a valid JSX attribute while keeping its length:
 * the `#`, `:`, `@` and `.` prefixes become a space, dynamic names (`[`a.b`]`) lose their
 * brackets, dots become hyphens (`v-model.number`), and a colon before a digit
 * (`v-foo:20`) or a second colon (`update:a:b`) becomes a hyphen. Works for attributes with and without a value.
 */
function cleanVueAttribute(_: string, ws: string, name: string, value: string | undefined): string {
    const stripped = /^[#.:@]/.test(name) ? " " + name.substring(1) : name
    const cleaned = stripped
        .replace(/[\[\]`]/g, " ")
        .replaceAll(".", "-")
        .replace(/:(?=\d)/g, "-")
        .replace(/(:[^:]*):/g, "$1-")
    return ws + cleaned + (value ?? "")
}

/**
 * Cleans and normalizes Vue single-file component (SFC) code so the resulting
 * string parses as plain JavaScript. The output preserves byte offsets where
 * possible (non-source markup is replaced with whitespace of the same length)
 * which keeps Babel's reported positions usable against the original file.
 *
 * Operations performed (in order):
 * - **Comments** (`<!-- ... -->`): non-whitespace replaced with spaces.
 * - **`<script>` / `<style>` / `<br>` tags**: replaced with spaces; a `;` is
 *   appended so the script body stays separable.
 * - **Dynamic bindings** (`:[name]="x"`): brackets and colon spaced out, the
 *   inner identifier survives.
 * - **Templates**: attributes are rewritten into valid JSX names (`#slot`, `@click.stop`, `:flag`, `v-model.number`), prop prefixes (`:`, `@`, `.`) become spaces, dotted prop
 *   names (`foo.bar`) become hyphenated (`foo-bar`), `<img ... X >` is
 *   self-closed to `<img ... />`, and `{{ x }}` interpolation becomes
 *   `{ x  }`.
 *
 * @example
 * cleanVueCode("<!-- secret -->\nlet x = 1;")
 * //=> "                \nlet x = 1;"
 *
 * @example
 * cleanVueCode("<template>\n<div :foo.bar=\"x\">{{ msg }}</div>\n</template>")
 * //=> "<template>\n<div  foo-bar=\"x\">{ msg  }</div>\n</template>"
 *
 * @param code The raw Vue SFC code as a string.
 * @returns The cleaned and normalized code as a string.
 */
export function cleanVueCode(code: string): string {
    return code.replace(vueCommentRegex, function (match: string): string {
        return match.replaceAll(/\S/g, " ")
    }).replace(vueCleaningRegex, function (match: string): string {
        return match.replaceAll(/\S/g, " ").substring(1) + ";"
    }).replace(vueBindRegex, function (_: string, grA: string, grB: string, grC: string): string {
        return grA.replaceAll(/\S/g, " ") +
            grB +
            grC.replaceAll(/\S/g, " ")
    }).replace(vueTemplateRegex, function (_: string, grA: string, grB: string, grC: string): string {
        return grA +
            grB.replace(vueOpenTagRegex, function (_: string, tag: string, attrs: string): string {
                return tag + attrs.replace(vueAttributeRegex, cleanVueAttribute)
            }).replace(vuePropRegex, function (_: string, grA: string, grB: string): string {
                return " " + grA.replace(/[.:@]/g, " ") + grB.replaceAll(".", "-")
            })
                .replace(vueOpenImgTag, function (_: string, grA: string, grB: string, grC: string): string {
                    return grA + grB + grC.replace(" >", "/>")
                })
                .replace(vueInterpolationOrTagOrBraceRegex, (m: string) => (m.length > 1 ? m : " "))
                .replaceAll("{{", "{ ")
                .replaceAll("}}", " }") +
            grC
    })
}
