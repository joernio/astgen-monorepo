// Renders the Babel AST schema (see [BabelSchema.ts](./BabelSchema.ts)) as a
// single Scala 3 source file, `BabelNodeSyntax.scala`.
//
// The file gives Scala consumers typed accessors over the JSON that astgen
// writes, instead of poking at `json("body")("0")("expression")` by hand. It is
// the counterpart of swift-astgen's `SwiftNodeSyntax.scala` and is produced by
// `astgen --scala-ast-only`.
//
// The output is fully deterministic for a given `@babel/types` version.

import * as fs from "node:fs"

import {buildSchema, type FieldSpec, type NodeSpec, type Schema, type ValueType} from "./BabelSchema"
import * as Logger from "./Logger"
import {VERSION} from "./version"

/** Default location of the generated Scala source file (CWD-relative). */
export const DEFAULT_SCALA_OUTPUT_PATH = "./BabelNodeSyntax.scala"

/** Name of the Scala object that wraps everything. */
const OBJECT_NAME = "BabelNodeSyntax"

/** Scala keywords that can appear as JSON field names, e.g. `object`, `extends`. */
const SCALA_KEYWORDS: ReadonlySet<string> = new Set([
    "abstract", "case", "catch", "class", "def", "do", "else", "enum", "export", "extends", "false", "final",
    "finally", "for", "forSome", "given", "if", "implicit", "import", "lazy", "macro", "match", "new", "null",
    "object", "override", "package", "private", "protected", "return", "sealed", "super", "then", "this",
    "throw", "trait", "true", "try", "type", "val", "var", "while", "with", "yield",
])

/**
 * Names a generated field accessor must not take: members of the generated
 * `BabelNode` base trait, plus what `case class` and `AnyRef` already define.
 */
const RESERVED_MEMBERS: ReadonlySet<string> = new Set([
    // BabelNode
    "json", "nodeType", "start", "end", "startLine", "startColumn", "endLine", "endColumn",
    "leadingComments", "trailingComments", "innerComments", "extra", "isParenthesized",
    // case class / AnyRef
    "copy", "productPrefix", "productArity", "productElement", "productElementName", "productElementNames",
    "productIterator",
    "canEqual", "equals", "hashCode", "toString", "getClass", "clone", "finalize", "notify", "notifyAll", "wait",
    "synchronized", "eq", "ne", "asInstanceOf", "isInstanceOf",
])

/** Scala identifier for the JSON field `name`; never collides with a keyword or a reserved member. */
export function accessorName(name: string): string {
    const safe = RESERVED_MEMBERS.has(name) ? `${name}_` : name
    return SCALA_KEYWORDS.has(safe) ? `\`${safe}\`` : safe
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

function nodeUnion(types: string[]): string {
    return types.length === 1 ? types[0] : types.join(" | ")
}

/** How a ujson value is turned into a Scala scalar: the type, and the checked reader from the preamble that yields it. */
type Scalar = { scala: string; reader: string }

const STRING: Scalar = {scala: "String", reader: "_str"}
const BOOLEAN: Scalar = {scala: "Boolean", reader: "_bool"}
const NUMBER: Scalar = {scala: "Double", reader: "_num"}

/** The scalar `type` decodes to, or `undefined` if consumers get the raw `ujson.Value` instead. */
function scalarOf(type: ValueType): Scalar | undefined {
    switch (type.kind) {
        case "string": return STRING
        case "boolean": return BOOLEAN
        case "number": return NUMBER
        case "enum":
            // mixed enums stay raw, e.g. TSMappedType.readonly: true | false | "+" | "-"
            if (type.values.every((value) => typeof value === "string")) return STRING
            if (type.values.every((value) => typeof value === "boolean")) return BOOLEAN
            return undefined
        default: return undefined
    }
}

/** Scala type of the value an accessor returns for a field of `type`. */
function scalaType(type: ValueType): string {
    if (type.kind === "node") return nodeUnion(type.types)
    if (type.kind === "array") {
        // Only arrays of nodes are decoded further; any other array hands out its raw elements.
        if (type.element.kind !== "node") return "Seq[Value]"
        const element = scalaType(type.element)
        return type.nullableElements ? `Seq[Option[${element}]]` : `Seq[${element}]`
    }
    return scalarOf(type)?.scala ?? "Value"
}

/**
 * Scala expression that reads `field` of the node `json`.
 *
 * Arrays are never optional: an absent or null array reads as empty.
 */
function readExpression(field: FieldSpec): string {
    const {name, type, optional} = field
    const key = JSON.stringify(name)

    if (type.kind === "array") {
        if (type.element.kind !== "node") return `_valueSeq(json, ${key})`
        const element = scalaType(type.element)
        return type.nullableElements ? `_nodeSeqOpt[${element}](json, ${key})` : `_nodeSeq[${element}](json, ${key})`
    }
    if (type.kind === "node") {
        return `${optional ? "_nodeOpt" : "_node"}[${nodeUnion(type.types)}](json, ${key})`
    }
    const reader = scalarOf(type)?.reader
    if (reader === undefined) return optional ? `_opt(json, ${key})` : `_req(json, ${key})`
    return optional ? `_opt(json, ${key}).map(${reader}(json, ${key}, _))` : `${reader}(json, ${key}, _req(json, ${key}))`
}

function fieldType(field: FieldSpec): string {
    const base = scalaType(field.type)
    return field.optional && field.type.kind !== "array" ? `Option[${base}]` : base
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

const I1 = "  "
const I2 = I1 + I1

function header(schema: Schema): string {
    return [
        "// Automatically generated by 'astgen --scala-ast-only'.",
        "// Do not edit directly!",
        `// @babel/types: ${schema.babelTypesVersion}`,
        `// astgen: ${VERSION}`,
        "//",
        "// Alias traits such as `Class`, `Function` or `File` shadow the standard names of the same spelling under",
        "// `import BabelNodeSyntax.*`; import the ones you need by name, or qualify them.",
        "",
        "import scala.collection.immutable.VectorMap",
        "",
        "import ujson.Value",
    ].join("\n")
}

function renderPreamble(): string {
    return `
${I1}/**
${I1} * Decodes one node of astgen's JSON AST into its typed wrapper, dispatching on the \`type\` key.
${I1} *
${I1} * Node types this file does not know (e.g. emitted by a newer parser than the one the file was generated for)
${I1} * are not an error: they decode to [[UnknownNode]], so a single unexpected node does not make the whole file unusable.
${I1} */
${I1}def createBabelNode(json: Value): BabelNode = {
${I2}_nodeTypeMap.get(_typeOf(json)).fold[BabelNode](UnknownNode(json))(_(json))
${I1}}

${I1}/** The per-file envelope \`{"fullName": ..., "relativeName": ..., "ast": File}\` that astgen writes. */
${I1}final case class BabelAstFile(json: Value) {
${I2}lazy val fullName: String = _str(json, "fullName", _req(json, "fullName"))
${I2}lazy val relativeName: String = _str(json, "relativeName", _req(json, "relativeName"))
${I2}lazy val ast: File = _node[File](json, "ast")
${I1}}

${I1}// Field readers. Whatever the JSON looks like, they fail only with a message naming the node type and the field: an
${I1}// \`IllegalStateException\` for a value of the wrong kind, a \`NoSuchElementException\` for a required key that is
${I1}// missing. Nothing from ujson escapes. The \`inline\` ones need the static type to test against; \`T\` may be a union.

${I1}private def _kind(value: Value): String = value match {
${I2}case ujson.Null   => "null"
${I2}case _: ujson.Bool => "a boolean"
${I2}case _: ujson.Num  => "a number"
${I2}case _: ujson.Str  => "a string"
${I2}case _: ujson.Arr  => "an array"
${I2}case _: ujson.Obj  => "an object"
${I1}}

${I1}private def _describe(json: Value): String = json.objOpt.flatMap(_.get("type")).flatMap(_.strOpt).getOrElse("node")

${I1}private def _invalid(json: Value, name: String, expected: String, found: Value): Nothing =
${I2}throw new IllegalStateException(s"\${_describe(json)}.$name: expected $expected, found \${_kind(found)}")

${I1}private def _typeOf(json: Value): String =
${I2}json.objOpt.flatMap(_.get("type")).flatMap(_.strOpt).getOrElse(
${I2}  throw new IllegalStateException(s"expected a node with a string 'type', found \${_kind(json)}")
${I2})

${I1}private def _fields(json: Value): scala.collection.mutable.Map[String, Value] =
${I2}json.objOpt.getOrElse(throw new IllegalStateException(s"expected a node object, found \${_kind(json)}"))

${I1}private def _opt(json: Value, name: String): Option[Value] = _fields(json).get(name).filterNot(_.isNull)

${I1}private def _req(json: Value, name: String): Value =
${I2}_fields(json).getOrElse(name, throw new NoSuchElementException(s"\${_describe(json)}.$name: required field is missing"))

${I1}private def _str(json: Value, name: String, value: Value): String = value.strOpt.getOrElse(_invalid(json, name, "a string", value))

${I1}private def _bool(json: Value, name: String, value: Value): Boolean = value.boolOpt.getOrElse(_invalid(json, name, "a boolean", value))

${I1}private def _num(json: Value, name: String, value: Value): Double = value.numOpt.getOrElse(_invalid(json, name, "a number", value))

${I1}private def _arr(json: Value, name: String, value: Value): Seq[Value] = value.arrOpt.getOrElse(_invalid(json, name, "an array", value)).toSeq

${I1}private def _obj(json: Value, name: String, value: Value): Value = if (value.objOpt.isDefined) value else _invalid(json, name, "an object", value)

${I1}private def _decode(json: Value, name: String, value: Value): BabelNode =
${I2}if (value.isNull) throw new IllegalStateException(s"\${_describe(json)}.$name: expected a node, found null")
${I2}else createBabelNode(_obj(json, name, value))

${I1}private def _unexpected(json: Value, name: String, node: BabelNode): Nothing =
${I2}throw new IllegalStateException(s"\${_describe(json)}.$name: unexpected node of type '\${node.nodeType}'")

${I1}private inline def _cast[T <: BabelNode](json: Value, name: String, node: BabelNode): T = node match {
${I2}case typed: T => typed
${I2}case other    => _unexpected(json, name, other)
${I1}}

${I1}private inline def _node[T <: BabelNode](json: Value, name: String): T =
${I2}_cast[T](json, name, _decode(json, name, _req(json, name)))

${I1}private inline def _nodeOpt[T <: BabelNode](json: Value, name: String): Option[T] =
${I2}_opt(json, name).map(value => _cast[T](json, name, _decode(json, name, value)))

${I1}private inline def _nodeSeq[T <: BabelNode](json: Value, name: String): Seq[T] =
${I2}_opt(json, name).fold(Seq.empty[T])(
${I2}  _arr(json, name, _).iterator.map(value => _cast[T](json, name, _decode(json, name, value))).toVector
${I2})

${I1}private inline def _nodeSeqOpt[T <: BabelNode](json: Value, name: String): Seq[Option[T]] =
${I2}_opt(json, name).fold(Seq.empty[Option[T]])(
${I2}  _arr(json, name, _).iterator.map(value => if (value.isNull) None else Some(_cast[T](json, name, _decode(json, name, value)))).toVector
${I2})

${I1}private def _valueSeq(json: Value, name: String): Seq[Value] =
${I2}_opt(json, name).fold(Seq.empty[Value])(_arr(json, name, _).toVector)
`.replace(/^\n/, "")
}

function renderBaseTrait(): string {
    return `
${I1}/**
${I1} * A node of the AST. Wraps the JSON object that astgen wrote; accessors decode it lazily.
${I1} *
${I1} * Two nodes are equal if they wrap the very same JSON object (so reading a child twice yields equal nodes), and
${I1} * hashing is cheap: neither looks at the content of the subtree.
${I1} */
${I1}sealed trait BabelNode {
${I2}def json: Value

${I2}/** The \`type\` discriminator, e.g. \`Identifier\`. */
${I2}def nodeType: String = _typeOf(json)

${I2}/** Character offsets into the source. */
${I2}lazy val start: Option[Int] = _opt(json, "start").map(_num(json, "start", _).toInt)
${I2}lazy val end: Option[Int] = _opt(json, "end").map(_num(json, "end", _).toInt)

${I2}private def _locField(which: String, name: String): Option[Int] =
${I2}  _opt(json, "loc").flatMap(loc => _opt(_obj(json, "loc", loc), which)).flatMap { pos =>
${I2}    _opt(_obj(json, s"loc.$which", pos), name).map(_num(json, s"loc.$which.$name", _).toInt)
${I2}  }

${I2}/** 1-based line, 0-based column. */
${I2}lazy val startLine: Option[Int] = _locField("start", "line")
${I2}lazy val startColumn: Option[Int] = _locField("start", "column")
${I2}lazy val endLine: Option[Int] = _locField("end", "line")
${I2}lazy val endColumn: Option[Int] = _locField("end", "column")

${I2}lazy val leadingComments: Seq[Comment] = _nodeSeq[Comment](json, "leadingComments")
${I2}lazy val trailingComments: Seq[Comment] = _nodeSeq[Comment](json, "trailingComments")
${I2}lazy val innerComments: Seq[Comment] = _nodeSeq[Comment](json, "innerComments")

${I2}/** Parser extras such as \`raw\`, \`rawValue\` or \`parenthesized\`, in source order. */
${I2}lazy val extra: Map[String, Value] = _opt(json, "extra").fold(VectorMap.empty[String, Value])(extra => VectorMap.from(_fields(_obj(json, "extra", extra))))
${I2}def isParenthesized: Boolean = extra.get("parenthesized").exists(_.boolOpt.contains(true))

${I2}override def equals(other: Any): Boolean = other match {
${I2}  case node: BabelNode => (this eq node) || (json eq node.json)
${I2}  case _                => false
${I2}}
${I2}override def hashCode: Int = System.identityHashCode(json)
${I2}override def toString: String = {
${I2}  def position(name: String) = json.objOpt.flatMap(_.get(name)).flatMap(_.numOpt).fold("?")(_.toInt.toString)
${I2}  s"\${_describe(json)}(\${position("start")}-\${position("end")})"
${I2}}
${I1}}
`.replace(/^\n/, "")
}

function renderAliasTrait(alias: string): string {
    return `${I1}sealed trait ${alias} extends BabelNode`
}

function renderNode(node: NodeSpec): string {
    const parents = node.aliases.length === 0 ? ["BabelNode"] : node.aliases
    const doc = node.aliases.length === 0 ? "" : `${I1}/** Babel \`${node.type}\`. Aliases: ${node.aliases.join(", ")}. */\n`
    const body = node.fields.map((field) =>
        `${I2}lazy val ${accessorName(field.name)}: ${fieldType(field)} = ${readExpression(field)}`)
    const head = `${doc}${I1}final case class ${node.type}(json: Value) extends ${parents.join(" with ")}`
    return body.length === 0 ? head : `${head} {\n${body.join("\n")}\n${I1}}`
}

/**
 * The fallback for node types the schema does not know. It belongs to every alias trait, so it can stand in wherever
 * the decoder expects an alias such as `Expression` or `Statement`.
 *
 * Mind the consequences: an unknown statement also matches `case e: Expression`, so test for `UnknownNode` first if
 * you need to tell them apart. A field typed as a union of concrete node classes (or a single one) cannot hold it;
 * reading such a field fails with an `IllegalStateException` that names the node type and the field.
 */
function renderUnknownNode(aliases: string[]): string {
    const parents = ["BabelNode", ...aliases].join(" with ")
    return `${I1}/** A node whose \`type\` is not a known Babel node type; only the generic \`BabelNode\` members are available. */
${I1}final case class UnknownNode(json: Value) extends ${parents}`
}

/**
 * Renders the complete `BabelNodeSyntax.scala` source for `schema`.
 * Pure: no I/O, no clock, so tests can snapshot it.
 */
export function renderScalaSource(schema: Schema): string {
    const typeMap = schema.nodes.map((n) => `${I2}${JSON.stringify(n.type)} -> (json => ${n.type}(json))`).join(",\n")
    const aliasNames = [...schema.aliasMembers.keys()].sort()
    const aliases = aliasNames.map(renderAliasTrait).join("\n")

    return [
        header(schema),
        "",
        `object ${OBJECT_NAME} {`,
        "",
        renderPreamble(),
        `${I1}private val _nodeTypeMap: Map[String, Value => BabelNode] = Map(\n${typeMap}\n${I1})`,
        "",
        renderBaseTrait(),
        `${I1}// MARK: alias traits:`,
        aliases,
        "",
        `${I1}// MARK: nodes:`,
        schema.nodes.map(renderNode).join("\n\n"),
        "",
        renderUnknownNode(aliasNames),
        "}",
        "",
    ].join("\n")
}

/**
 * Writes `BabelNodeSyntax.scala` to `outputPath`, atomically: a crash never leaves a truncated file behind.
 * @throws if the file cannot be written.
 */
export function generateScalaAst(outputPath: string = DEFAULT_SCALA_OUTPUT_PATH): void {
    const temporary = `${outputPath}.${process.pid}.tmp`
    try {
        fs.writeFileSync(temporary, renderScalaSource(buildSchema()), "utf8")
        fs.renameSync(temporary, outputPath)
    } catch (e) {
        fs.rmSync(temporary, {force: true})
        throw e
    }
    Logger.info("Generated Scala Babel AST in file:", `\`${outputPath}\``)
}
