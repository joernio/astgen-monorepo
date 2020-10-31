const babelParser = require("@babel/parser");
const DockerfileParser = require("dockerfile-ast").DockerfileParser;

const { join } = require("path");
const fs = require("fs");
const path = require("path");

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
 * Convert a single JS/TS file to AST
 */
const toJSAst = (file) => {
  const ast = babelParser.parse(
    fs.readFileSync(file, "utf-8"),
    babelParserOptions
  );
  return ast;
};
exports.toJSAst = toJSAst;

/**
 * Generate AST for JavaScript or TypeScript
 */
const createJSAst = async (options) => {
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
 * Write AST data to a json file
 */
const writeAstFile = (file, ast, options) => {
  const relativePath = file.replace(
    new RegExp("^" + options.src + path.sep),
    ""
  );
  const outAstFile = path.join(options.output, relativePath + ".json");
  const data = {
    fullName: file,
    relativeName: relativePath,
    ast: ast,
  };
  fs.mkdirSync(path.dirname(outAstFile), { recursive: true });
  fs.writeFileSync(outAstFile, JSON.stringify(data, null, "  "));
  console.log("Converted", relativePath, "to", outAstFile);
};

/**
 * Convert a single dockerfile to ast
 */
const toDockerAst = (file) => {
  const dockerfile = DockerfileParser.parse(fs.readFileSync(file, "utf-8"));
  const instructions = dockerfile.getInstructions();
  const dataList = [];
  for (let instruction of instructions) {
    const instructionRange = instruction.instructionRange;
    const data = {
      type: instruction.getKeyword(),
      start: instructionRange.start.line,
      end: instructionRange.end.line,
      loc: instructionRange,
      arguments: instruction.getArguments(),
      argumentsRange: instruction.getArgumentsRange(),
      argumentsContent: instruction.getArgumentsContent(),
      expandedArguments: instruction.getExpandedArguments(),
      variables: instruction.getVariables(),
    };
    dataList.push(data);
  }
  return {
    type: "file",
    errors: [],
    program: { type: "Dockerfile", sourceType: "Dockerfile", body: dataList },
  };
};
exports.toDockerAst = toDockerAst;

/**
 * Generate AST for dockerfile
 */
const createDockerAst = (options) => {
  const dockerfiles = getAllFiles(options.src, "Dockerfile");
  for (const file of dockerfiles) {
    try {
      const ast = toDockerAst(file);
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
    return createDockerAst(options);
  }
};

/**
 * Method to start the ast generation process
 *
 * @args options CLI arguments
 */
const start = async (options) => {
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
    case "docker":
      return createDockerAst(options);
    default:
      return await createXAst(options);
  }
};
exports.start = start;
