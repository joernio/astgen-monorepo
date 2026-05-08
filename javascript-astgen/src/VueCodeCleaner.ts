const vueCleaningRegex = /<\/*script.*>|<style[\s\S]*style>|<\/*br>/ig
const vueTemplateRegex = /(<template.*>)([\s\S]*)(<\/template>)/ig
const vueCommentRegex = /<!--[\s\S]*?-->/ig
const vueBindRegex = /(:\[)(\S*?)(])/ig
const vuePropRegex = /\s([.:@])(\S*?=)/ig
const vueOpenImgTag = /(<img)((?!>)[\s\S]+?)( [^\/]>)/ig

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
 * - **Templates**: prop prefixes (`:`, `@`, `.`) become spaces, dotted prop
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
            grB.replace(vuePropRegex, function (_: string, grA: string, grB: string): string {
                return " " + grA.replace(/[.:@]/g, " ") + grB.replaceAll(".", "-")
            })
                .replace(vueOpenImgTag, function (_: string, grA: string, grB: string, grC: string): string {
                    return grA + grB + grC.replace(" >", "/>")
                })
                .replaceAll("{{", "{ ")
                .replaceAll("}}", " }") +
            grC
    })
}
