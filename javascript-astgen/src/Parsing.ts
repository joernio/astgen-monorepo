import * as babelParser from "@babel/parser"

import * as Defaults from "./Defaults"
import * as VueCodeCleaner from "./VueCodeCleaner"

/**
 * Converts a JavaScript or TypeScript code string to an Abstract Syntax Tree (AST).
 *
 * The function first attempts to parse the code with standard Babel parser options.
 * If the initial parsing fails (e.g., with experimental syntax), it automatically
 * falls back to a more permissive set of parsing options.
 *
 * @param code - The JavaScript or TypeScript code string to be parsed
 * @returns A Babel ParseResult object representing the AST of the provided code
 * @throws May throw an error if parsing fails with both standard and fallback options
 * @see Defaults.BABEL_PARSER_OPTIONS - The primary parsing configuration
 * @see Defaults.SAFE_BABEL_PARSER_OPTIONS - The fallback parsing configuration
 */
export function codeToJsAst(code: string): babelParser.ParseResult {
    try {
        return babelParser.parse(code, Defaults.BABEL_PARSER_OPTIONS)
    } catch {
        return babelParser.parse(code, Defaults.SAFE_BABEL_PARSER_OPTIONS)
    }
}

/**
 * Converts pre-read Vue file content to an Abstract Syntax Tree (AST).
 *
 * This function cleans the code using the VueCodeCleaner utility to extract and
 * process the script section, then parses the cleaned code into an AST using Babel.
 *
 * @param content - The raw content of the Vue file
 * @returns A Babel ParseResult object representing the AST of the Vue file's script content
 * @throws Will throw an error if parsing fails
 * @see VueCodeCleaner.cleanVueCode - The utility used to extract script content from Vue files
 * @see codeToJsAst - The underlying function used for parsing the extracted code
 */
export function toVueAst(content: string): babelParser.ParseResult {
    return codeToJsAst(VueCodeCleaner.cleanVueCode(content))
}
