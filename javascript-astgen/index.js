const babelParser = require("@babel/parser");
const DockerfileParser = require("dockerfile-ast").DockerfileParser;
const { execFileSync } = require("child_process");

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
  const relativePath = file.replace(new RegExp("^" + options.src + "/"), "");
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
const toDockerAst = (file, content) => {
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
exports.toDockerAst = toDockerAst;

/**
 * Generate AST for dockerfile
 */
const createDockerAst = (options) => {
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
const toBashAst = (file, content) => {
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
exports.toBashAst = toBashAst;

/**
 * Generate AST for bash script
 */
const createBashAst = (options) => {
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
    case "bash":
    case "sh":
      return createBashAst(options);
    default:
      return await createXAst(options);
  }
};
exports.start = start;
