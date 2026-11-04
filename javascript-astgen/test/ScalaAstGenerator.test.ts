import * as fs from "node:fs"
import * as os from "node:os"
import * as path from "node:path"

import {describe, expect, it} from "bun:test"

import {buildSchema} from "../src/BabelSchema"
import {accessorName, generateScalaAst, renderScalaSource} from "../src/ScalaAstGenerator"

describe("renderScalaSource", () => {
    const schema = buildSchema()
    const source = renderScalaSource(schema)
    const aliasNames = [...schema.aliasMembers.keys()].sort()

    it("is deterministic and wrapped in one object", () => {
        expect(renderScalaSource(buildSchema())).toBe(source)
        expect(source).toContain("object BabelNodeSyntax {")
        expect(source.trimEnd().endsWith("}")).toBe(true)
        expect(source).toContain(`// @babel/types: ${schema.babelTypesVersion}`)
    })

    it("emits one case class and one dispatch entry per node type", () => {
        for (const node of schema.nodes) {
            expect(source).toContain(`final case class ${node.type}(json: Value) extends`)
            expect(source).toContain(`${JSON.stringify(node.type)} -> (json => ${node.type}(json))`)
        }
    })

    it("emits a sealed trait per alias, and nodes extend their aliases", () => {
        for (const alias of aliasNames) expect(source).toContain(`sealed trait ${alias} extends BabelNode`)
        expect(source).toMatch(/final case class ArrayExpression\(json: Value\) extends Expression with Standardized/)
    })

    it("decodes unknown node types to an UnknownNode that belongs to every alias", () => {
        const decl = source.match(/final case class UnknownNode\(json: Value\) extends (.+)/)
        expect(decl).not.toBeNull()
        const parents = decl![1].split(" with ")
        expect(parents[0]).toBe("BabelNode")
        expect(parents.slice(1)).toEqual(aliasNames)
        expect(source).toContain("fold[BabelNode](UnknownNode(json))")
        expect(source).not.toContain("UnsupportedOperationException")
    })

    it("treats the body of a TSModuleDeclaration as optional", () => {
        expect(source).toMatch(/lazy val body: Option\[TSModuleBlock \| TSModuleDeclaration\] = /)
    })

    it("backticks Scala keywords used as field names", () => {
        expect(source).toContain("lazy val `object`:")
        expect(source).toContain("lazy val `abstract`: Option[Boolean]")
    })

    it("models optional, array and sparse-array fields", () => {
        expect(source).toContain("lazy val id: Option[Identifier] = _nodeOpt[Identifier](json, \"id\")")
        expect(source).toContain("lazy val elements: Seq[Option[Expression | SpreadElement]] = _nodeSeqOpt[Expression | SpreadElement](json, \"elements\")")
    })

    it("never lets a node accessor shadow a member of the BabelNode base trait", () => {
        const nodeSection = source.slice(source.indexOf("// MARK: nodes:"))
        const reserved = ["json", "nodeType", "start", "end", "extra", "leadingComments", "trailingComments", "innerComments"]
        const accessors = [...nodeSection.matchAll(/^ {4}lazy val `?(\w+)`?:/gm)].map((m) => m[1])
        expect(accessors.length).toBeGreaterThan(500)
        for (const name of reserved) expect(accessors).not.toContain(name)
    })

    it("decodes numbers JSON cannot represent as absent", () => {
        // `1e999` is written as null; reading it as a required Double would throw.
        expect(source).toMatch(/final case class NumericLiteral[^]*?lazy val value: Option\[Double\] = /)
    })

    it("indexes sequences in constant time and never hashes a subtree", () => {
        expect(source).not.toContain(".toList")
        expect(source).toContain(".toVector")
        expect(source).toContain("override def hashCode: Int = System.identityHashCode(json)")
    })

    it("names the node type and field when a node does not fit", () => {
        expect(source).toContain("required field is missing")
        expect(source).toContain("unexpected node of type")
        expect(source).toContain("lazy val ast: File = _node[File](json, \"ast\")")
    })

    it("types every array accessor as what it reads", () => {
        // Arrays of non-nodes are read raw; the declared type must not promise more.
        expect(source).not.toMatch(/Seq\[(String|Boolean|Double)\] = _valueSeq/)
    })
})

describe("accessorName", () => {
    it("keeps ordinary names", () => {
        expect(accessorName("name")).toBe("name")
    })

    it("renames fields that would shadow a member of the base trait or of case classes", () => {
        for (const name of ["json", "nodeType", "start", "extra", "leadingComments", "copy", "productElementNames", "hashCode"]) {
            expect(accessorName(name)).toBe(`${name}_`)
        }
    })

    it("backticks Scala keywords", () => {
        expect(accessorName("object")).toBe("`object`")
        expect(accessorName("type")).toBe("`type`")
    })
})

describe("generateScalaAst", () => {
    it("writes the file to the given path", () => {
        const dir = fs.mkdtempSync(path.join(os.tmpdir(), "scala-gen-"))
        try {
            const out = path.join(dir, "BabelNodeSyntax.scala")
            generateScalaAst(out)
            expect(fs.readFileSync(out, "utf8")).toBe(renderScalaSource(buildSchema()))
        } finally {
            fs.rmSync(dir, {recursive: true, force: true})
        }
    })
})
