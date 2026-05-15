import * as path from "node:path"

import * as babelParser from "@babel/parser"

import Options from "./Options"
import {TypeMap} from "./TscUtils"
import * as Logger from "./Logger"
import {WriteSink} from "./WriteSink"
import {outputPathFor} from "./FileUtils"

export type AstFile = {
    fullName: string
    relativeName: string
    ast: babelParser.ParseResult
}

/**
 * Wire format for the per-file `<source>.json` output. Defined here (the writer
 * owns the schema) and reused by [AstWorker.ts](./AstWorker.ts) so workers
 * write the same shape without hardcoding the field names.
 */
export function buildAstFile(file: string, relativeName: string, ast: babelParser.ParseResult): AstFile {
    return {fullName: file, relativeName, ast}
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
    const {relativePath, outputPath} = outputPathFor(options.src, options.output, file, ".typemap")
    await sink.ensureDir(path.dirname(outputPath))
    await sink.writeTypeMapJson(outputPath, seenTypes)
    Logger.info("Converted types for", relativePath, "to", outputPath)
}
