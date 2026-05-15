#!/usr/bin/env node

import start from "./Pipeline"
import Options from "./Options"
import * as Defaults from "./Defaults"
import * as Logger from "./Logger"
import {getErrorMessage} from "./Errors"
import {VERSION} from "./version"

import * as path from "node:path"
import yargs from "yargs"
import {hideBin} from "yargs/helpers"

async function main(argv: string[]) {
    const parsed = yargs(hideBin(argv))
        .option("src", {
            alias: "i",
            default: ".",
            coerce: (arg: any): string => {
                return path.resolve(arg.toString())
            },
            description: "Source directory",
        })
        .option("output", {
            alias: "o",
            type: "string",
            description:
                `Output directory for generated AST json files (default: <src>/${Defaults.DEFAULT_OUTPUT_DIR})`,
        })
        .option("type", {
            alias: "t",
            type: "string",
            description: "Project type. Default auto-detect",
        })
        .option("recurse", {
            alias: "r",
            default: true,
            type: "boolean",
            description: "Recurse mode suitable for mono-repos",
        })
        .option("tsTypes", {
            default: true,
            type: "boolean",
            description: "Generate type mappings using the Typescript Compiler API",
        })
        .option("exclude-file", {
            default: [],
            type: "string",
            array: true,
            description: "Exclude this file. Can be specified multiple times. Default is empty."
        })
        .option("exclude-regex", {
            coerce: (arg: any): RegExp | undefined => {
                try {
                    return new RegExp(arg.toString(), "i")
                } catch (err) {
                    Logger.warn(`--exclude-regex: ignoring invalid regex ${JSON.stringify(arg)}: ${getErrorMessage(err)}`)
                    return undefined
                }
            },
            description: "Exclude files matching this regex (matches the absolute path)."
        })
        .version(VERSION)
        .help("h").parseSync()

    const args: Options = {
        src: parsed.src,
        output: parsed.output ?? path.join(parsed.src, Defaults.DEFAULT_OUTPUT_DIR),
        type: parsed.type,
        recurse: parsed.recurse,
        tsTypes: parsed.tsTypes,
        "exclude-file": parsed["exclude-file"],
        "exclude-regex": parsed["exclude-regex"],
    }

    try {
        await start(args)
    } catch (e) {
        Logger.error(e)
        process.exit(1)
    }
}

main(process.argv)
