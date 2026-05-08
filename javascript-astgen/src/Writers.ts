import * as path from "node:path"

import * as babelParser from "@babel/parser"

import Options from "./Options"
import {TypeMap} from "./TscUtils"
import * as Logger from "./Logger"
import {WriteSink} from "./WriteSink"

/**
 * Writes the AST (Abstract Syntax Tree) data of a source file to a JSON file.
 *
 * The output file is created in the output directory specified in the options,
 * preserving the relative path structure from the source directory. The AST data
 * is serialized using a writer that handles circular references.
 *
 * @param file - The absolute path to the source file.
 * @param ast - The Babel ParseResult object representing the AST of the file.
 * @param options - Configuration options containing source and output directories.
 * @param sink - Write target. Production code passes `FsWriteSink`; tests can
 *               supply an in-memory implementation.
 */
export async function writeAstFile(
    file: string,
    ast: babelParser.ParseResult,
    options: Options,
    sink: WriteSink,
): Promise<void> {
    const relativePath: string = path.relative(options.src, file)
    const outAstFile: string = path.join(options.output, relativePath + ".json")
    const data = {
        fullName: file,
        relativeName: relativePath,
        ast: ast,
    }
    await sink.ensureDir(path.dirname(outAstFile))
    await sink.writeAstJson(outAstFile, data)
    Logger.info("Converted AST for", relativePath, "to", outAstFile)
}

/**
 * Writes TypeScript type information to a JSON file.
 *
 * @param file - The absolute path to the source file.
 * @param seenTypes - The `TypeMap` containing type information to be written.
 * @param options - Configuration options containing source and output directories.
 * @param sink - Write target.
 */
export async function writeTypesFile(
    file: string,
    seenTypes: TypeMap,
    options: Options,
    sink: WriteSink,
): Promise<void> {
    const relativePath: string = path.relative(options.src, file)
    const outTypeFile: string = path.join(options.output, relativePath + ".typemap")
    await sink.ensureDir(path.dirname(outTypeFile))
    await sink.writeTypeMapJson(outTypeFile, seenTypes)
    Logger.info("Converted types for", relativePath, "to", outTypeFile)
}
