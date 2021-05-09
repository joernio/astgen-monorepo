import babelParser from "@babel/parser";
import { DockerfileParser } from "dockerfile-ast";
import { execFileSync } from "child_process";
import vueCompiler from "vue-template-compiler";
import * as svelteCompiler from "svelte/compiler";
import markdownParser from "mdast-util-from-markdown";
import yamlParser from "yaml-language-server-parser";
import { fromXml } from "xast-util-from-xml";
import { toXast } from "hast-util-to-xast";
import unified from "unified";
import parse from "rehype-parse";

import path, { join } from "path";
import fs from "fs";

const IGNORE_DIRS = [
  "node_modules",
  "venv",
  "docs",
  "test",
  "tests",
  "e2e",
  "examples",
  "cypress",
];

const IGNORE_FILE_PATTERN = new RegExp("(conf|test|spec)\\.(js|ts)$", "i");

const getAllFiles = (dir, extn, files, result, regex) => {
  files = files || fs.readdirSync(dir);
  result = result || [];
  regex = regex || new RegExp(`\\${extn}$`);

  for (let i = 0; i < files.length; i++) {
    if (IGNORE_FILE_PATTERN.test(files[i])) {
      continue;
    }
    let file = join(dir, files[i]);
    if (fs.statSync(file).isDirectory()) {
      // Ignore directories
      const dirName = path.basename(file);
      if (
        dirName.startsWith(".") ||
        IGNORE_DIRS.includes(dirName.toLowerCase())
      ) {
        continue;
      }
      try {
        result = getAllFiles(file, extn, fs.readdirSync(file), result, regex);
      } catch (error) {
        continue;
      }
    } else {
      if (regex.test(file)) {
        result.push(file);
      }
    }
  }
  return result;
};

const babelParserOptions = {
  sourceType: "module",
  allowImportExportEverywhere: true,
  allowAwaitOutsideFunction: true,
  allowReturnOutsideFunction: true,
  errorRecovery: true,
  plugins: [
    "optionalChaining",
    "classProperties",
    "decorators-legacy",
    "exportDefaultFrom",
    "doExpressions",
    "numericSeparator",
    "dynamicImport",
    "jsx",
    "typescript",
  ],
};

/**
 * Return paths to all (j|tsx?) files.
 */
const getAllSrcJSAndTSFiles = (src) =>
  Promise.all([
    getAllFiles(src, ".js"),
    getAllFiles(src, ".jsx"),
    getAllFiles(src, ".ts"),
    getAllFiles(src, ".tsx"),
  ]);

/**
 * Return paths to all yaml files.
 */
const getAllYamlFiles = (src) =>
  Promise.all([getAllFiles(src, ".yml"), getAllFiles(src, ".yaml")]);

/**
 * Return paths to all html files.
 */
const getAllHtmlFiles = (src) =>
  Promise.all([getAllFiles(src, ".htm"), getAllFiles(src, ".html"), getAllFiles(src, ".svg")]);

/**
 * Convert a single JS/TS file to AST
 */
export const toJSAst = (file) => {
  const ast = babelParser.parse(
    fs.readFileSync(file, "utf-8"),
    babelParserOptions
  );
  return ast;
};

/**
 * Convert a single vue file to AST
 */
export const toVueAst = (file) => {
  const astObj = vueCompiler.compile(fs.readFileSync(file, "utf-8"));
  if (astObj && astObj.ast) {
    return astObj.ast;
  }
  return undefined;
};

/**
 * Convert a single svelte file to AST
 */
export const toSvelteAst = (file) => {
  const astObj = svelteCompiler.parse(fs.readFileSync(file, "utf-8"), {
    filename: file,
  });
  return astObj;
};

/**
 * Convert a single yaml file to AST
 */
export const toYamlAst = (file) => {
  const astObj = yamlParser.load(fs.readFileSync(file, "utf-8"), {
    filename: file,
    ignoreDuplicateKeys: true,
    allowAnyEscape: true,
  });
  return astObj;
};

/**
 * Convert a single markdown file to AST
 */
export const toMarkdownAst = (file) => {
  const astObj = markdownParser(fs.readFileSync(file, "utf-8"));
  return astObj;
};

/**
 * Convert a single xml file to AST
 */
export const toXmlAst = (file) => {
  const astObj = fromXml(fs.readFileSync(file, "utf-8"));
  return astObj;
};

/**
 * Convert a single html file to AST
 */
export const toHtmlAst = (file) => {
  const hast = unified().use(parse).parse(fs.readFileSync(file, "utf-8"));
  const astObj = toXast(hast);
  return astObj;
};

/**
 * Generate AST for JavaScript or TypeScript
 */
export const createJSAst = async (options) => {
  try {
    const errFiles = [];
    const promiseMap = await getAllSrcJSAndTSFiles(options.src);
    const srcFiles = promiseMap.flatMap((d) => d);
    for (const file of srcFiles) {
      try {
        const ast = toJSAst(file);
        writeAstFile(file, ast, options);
      } catch (err) {
        console.error(file, err.message);
        errFiles.push(file);
      }
    }
  } catch (err) {
    console.error(err);
  }
};

/**
 * Generate AST for .vue files
 */
export const createVueAst = async (options) => {
  const srcFiles = getAllFiles(options.src, ".vue");
  for (const file of srcFiles) {
    try {
      const ast = toVueAst(file);
      if (ast) {
        writeAstFile(file, ast, options);
      }
    } catch (err) {
      console.error(file, err.message);
    }
  }
};

/**
 * Generate AST for .svelte files
 */
export const createSvelteAst = async (options) => {
  const srcFiles = getAllFiles(options.src, ".svelte");
  for (const file of srcFiles) {
    try {
      const ast = toSvelteAst(file);
      if (ast) {
        writeAstFile(file, ast, options);
      }
    } catch (err) {
      console.error(file, err.message);
    }
  }
};

/**
 * Generate AST for Yaml
 */
export const createYamlAst = async (options) => {
  try {
    const errFiles = [];
    const promiseMap = await getAllYamlFiles(options.src);
    const srcFiles = promiseMap.flatMap((d) => d);
    for (const file of srcFiles) {
      try {
        const ast = toYamlAst(file);
        writeAstFile(file, ast, options);
      } catch (err) {
        console.error(file, err.message);
        errFiles.push(file);
      }
    }
  } catch (err) {
    console.error(err);
  }
};

/**
 * Generate AST for html
 */
export const createHtmlAst = async (options) => {
  try {
    const errFiles = [];
    const promiseMap = await getAllHtmlFiles(options.src);
    const srcFiles = promiseMap.flatMap((d) => d);
    for (const file of srcFiles) {
      try {
        const ast = toHtmlAst(file);
        writeAstFile(file, ast, options);
      } catch (err) {
        console.error(file, err.message);
        errFiles.push(file);
      }
    }
  } catch (err) {
    console.error(err);
  }
};

/**
 * Generate AST for .md files
 */
export const createMarkdownAst = async (options) => {
  const srcFiles = getAllFiles(options.src, ".md");
  for (const file of srcFiles) {
    try {
      const ast = toMarkdownAst(file);
      if (ast) {
        writeAstFile(file, ast, options);
      }
    } catch (err) {
      console.error(file, err.message);
    }
  }
};

/**
 * Generate AST for .xml files
 */
export const createXmlAst = async (options) => {
  const srcFiles = getAllFiles(options.src, ".xml");
  for (const file of srcFiles) {
    try {
      const ast = toXmlAst(file);
      if (ast) {
        writeAstFile(file, ast, options);
      }
    } catch (err) {
      console.error(file, err.message);
    }
  }
};

/**
 * Deal with cyclic reference in json
 */
const getCircularReplacer = () => {
  const seen = new WeakSet();
  return (key, value) => {
    if (typeof value === "object" && value !== null) {
      if (seen.has(value)) {
        return;
      }
      seen.add(value);
    }
    return value;
  };
};

/**
 * Write AST data to a json file
 */
export const writeAstFile = (file, ast, options) => {
  const relativePath = file.replace(new RegExp("^" + options.src + "/"), "");
  const outAstFile = path.join(options.output, relativePath + ".json");
  const data = {
    fullName: file,
    relativeName: relativePath,
    ast: ast,
  };
  fs.mkdirSync(path.dirname(outAstFile), { recursive: true });
  fs.writeFileSync(
    outAstFile,
    JSON.stringify(data, getCircularReplacer(), "  ")
  );
  console.log("Converted", relativePath, "to", outAstFile);
};

/**
 * Convert a single dockerfile to ast
 */
export const toDockerAst = (file, content) => {
  const dockerfile = DockerfileParser.parse(
    file ? fs.readFileSync(file, "utf-8") : content
  );
  const instructions = dockerfile.getInstructions();
  const dataList = [];
  for (const instruction of instructions) {
    const instructionRange = instruction.instructionRange;
    const keyword = instruction.getKeyword();
    const argumentsContent = instruction.getArgumentsContent();
    let instructionAst = {};
    let flags = [];
    let metadata = {};
    // Create AST for RUN arguments
    if (keyword === "RUN") {
      instructionAst = toBashAst(file, argumentsContent);
    }
    if (keyword === "COPY") {
      flags = instruction.getFromFlag();
    }
    if (keyword === "FROM") {
      metadata = {
        image: instruction.getImage() || "",
        imageTag: instruction.getImageTag() || "",
        imageDigest: instruction.getImageDigest() || "",
        registry: instruction.getRegistry() || "",
        buildStage: instruction.getBuildStage() || "",
        platform: instruction.getPlatformFlag() || "",
      };
    }
    if (keyword === "HEALTHCHECK") {
      metadata = {
        subcommand: instruction.getSubcommand(),
      };
    }

    const data = {
      type: keyword,
      start: instructionRange.start.line,
      end: instructionRange.end.line,
      loc: instructionRange,
      arguments: instruction.getArguments(),
      argumentsRange: instruction.getArgumentsRange(),
      argumentsContent,
      expandedArguments: instruction.getExpandedArguments(),
      variables: instruction.getVariables(),
      flags: flags,
      metadata: metadata,
      instructionAst,
    };
    dataList.push(data);
  }
  return {
    type: "file",
    errors: [],
    program: { type: "Dockerfile", sourceType: "Dockerfile", body: dataList },
  };
};

/**
 * Generate AST for dockerfile
 */
export const createDockerAst = (options) => {
  const dockerfiles = getAllFiles(options.src, "Dockerfile.*");
  for (const file of dockerfiles) {
    try {
      const ast = toDockerAst(file);
      writeAstFile(file, ast, options);
    } catch (err) {
      console.error(file, err.message);
    }
  }
};

/**
 * Convert a single bash file to ast
 */
export const toBashAst = (file, content) => {
  if (file && !content) {
    content = fs.readFileSync(file, "utf-8");
  }
  let astString = "{}";
  try {
    astString = execFileSync("shfmt", ["-tojson"], {
      input: content,
      encoding: "utf-8",
    });
  } catch (err) {
    console.log("Check if shfmt is installed and available in PATH");
    console.error(err);
  }
  return {
    type: "file",
    errors: [],
    program: { type: "bash", sourceType: "bash", body: JSON.parse(astString) },
  };
};

/**
 * Generate AST for bash script
 */
export const createBashAst = (options) => {
  const shfiles = getAllFiles(options.src, ".sh");
  for (const file of shfiles) {
    try {
      const content = fs.readFileSync(file, "utf-8");
      const ast = toBashAst(file, content);
      writeAstFile(file, ast, options);
    } catch (err) {
      console.error(file, err.message);
    }
  }
};

const createXAst = async (options) => {
  const src_dir = options.src;
  try {
    fs.accessSync(src_dir, fs.constants.R_OK);
  } catch (err) {
    console.error(src_dir, "is invalid");
    process.exit(1);
  }
  const { projectType } = options;
  // node.js - package.json
  if (
    fs.existsSync(path.join(src_dir, "package.json")) ||
    fs.existsSync(path.join(src_dir, "rush.json"))
  ) {
    return await createJSAst(options);
  }
  // Dockerfile
  if (fs.existsSync(path.join(src_dir, "Dockerfile"))) {
    createDockerAst(options);
  }
  createBashAst(options);
};

/**
 * Method to start the ast generation process
 *
 * @args options CLI arguments
 */
export const start = async (options) => {
  let { type } = options;
  if (!type) {
    type = "";
  }
  type = type.toLowerCase();
  switch (type) {
    case "nodejs":
    case "js":
    case "javascript":
    case "typescript":
    case "ts":
      return await createJSAst(options);
    case "vue":
      return await createVueAst(options);
    case "svelte":
      return await createSvelteAst(options);
    case "yaml":
      return await createYamlAst(options);
    case "htm":
    case "svg":  
    case "html":
      return await createHtmlAst(options);
    case "docker":
      return createDockerAst(options);
    case "md":
    case "markdown":
      return createMarkdownAst(options);
    case "xml":
      return createXmlAst(options);
    case "bash":
    case "sh":
      return createBashAst(options);
    default:
      return await createXAst(options);
  }
};
