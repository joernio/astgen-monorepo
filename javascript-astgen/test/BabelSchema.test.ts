import * as fs from "node:fs"
import * as os from "node:os"
import * as path from "node:path"

import {buildSchema} from "../src/BabelSchema"
import {writeJsonStreamCircular} from "../src/JsonUtils"
import * as Parsing from "../src/Parsing"
import {checkAst, newConformance} from "./schemaConformance"

const FIXTURE_DIR = path.join(import.meta.dir, "fixtures", "syntax")

/** The parse result, serialised by the very writer astgen uses and read back: exactly what consumers see. */
function parseToJson(code: string): { ast: unknown; errors: number } {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), "schema-"))
    try {
        const out = path.join(dir, "ast.json")
        const parsed = Parsing.codeToJsAst(code)
        writeJsonStreamCircular(out, parsed)
        return {ast: JSON.parse(fs.readFileSync(out, "utf8")), errors: (parsed as { errors?: unknown[] }).errors?.length ?? 0}
    } finally {
        fs.rmSync(dir, {recursive: true, force: true})
    }
}

function fixtures(): { name: string; code: string }[] {
    return fs.readdirSync(FIXTURE_DIR)
        .filter((f) => f.endsWith(".fixture"))
        .sort()
        .map((f) => ({name: f, code: fs.readFileSync(path.join(FIXTURE_DIR, f), "utf8")}))
}

/** Real-world sources that ship with the repo: our own TypeScript and a CJS bundle. */
function realWorld(): { name: string; code: string }[] {
    const root = path.join(import.meta.dir, "..")
    const files = [
        ...fs.readdirSync(path.join(root, "src")).filter((f) => f.endsWith(".ts")).map((f) => path.join("src", f)),
        ...fs.readdirSync(path.join(root, "test")).filter((f) => f.endsWith(".test.ts")).map((f) => path.join("test", f)),
        path.join("node_modules", "@babel", "parser", "lib", "index.js"),
        path.join("node_modules", "yargs", "build", "lib", "yargs-factory.js"),
    ]
    for (const f of files) if (!fs.existsSync(path.join(root, f))) throw new Error(`Real-world input is missing: ${f}`)
    // Missing inputs must fail loudly: they carry coverage the fixtures do not (see the fixtures test below).
    return files
        .map((f) => ({name: f, code: fs.readFileSync(path.join(root, f), "utf8")}))
}

describe("buildSchema", () => {
    const schema = buildSchema()

    it("covers the full @babel/types node set plus comments", () => {
        const types = new Set(schema.nodes.map((n) => n.type))
        for (const t of ["Identifier", "File", "Program", "ClassMethod", "TSInterfaceDeclaration", "JSXElement", "CommentBlock", "CommentLine"]) {
            expect(types.has(t)).toBe(true)
        }
        expect(schema.nodes.length).toBeGreaterThan(250)
        expect(schema.babelTypesVersion).toMatch(/^\d+\.\d+\.\d+/)
    })

    it("is deterministic", () => {
        expect(JSON.stringify(buildSchema().nodes)).toEqual(JSON.stringify(schema.nodes))
    })

    it("sorts nodes by name and never lists a type twice", () => {
        const names = schema.nodes.map((n) => n.type)
        expect(names).toEqual([...names].sort())
        expect(new Set(names).size).toBe(names.length)
    })

    it("never lets a node type and an alias share a name", () => {
        const types = new Set(schema.nodes.map((n) => n.type))
        expect([...schema.aliasMembers.keys()].filter((a) => types.has(a))).toEqual([])
    })

    it("only references node types and aliases that exist", () => {
        const known = new Set([...schema.nodes.map((n) => n.type), ...schema.aliasMembers.keys()])
        const unknown: string[] = []
        const check = (where: string, t: import("../src/BabelSchema").ValueType): void => {
            if (t.kind === "node") t.types.filter((n) => !known.has(n)).forEach((n) => unknown.push(`${where}: ${n}`))
            if (t.kind === "array") check(where, t.element)
        }
        for (const n of schema.nodes) for (const f of n.fields) check(`${n.type}.${f.name}`, f.type)
        expect(unknown).toEqual([])
    })

    it("drops union members that an alias in the same union already covers", () => {
        const classMethod = schema.nodes.find((n) => n.type === "ClassMethod")!
        const key = classMethod.fields.find((f) => f.name === "key")!.type
        expect(key.kind).toBe("node")
        const types = (key as { kind: "node"; types: string[] }).types
        // Identifier and the literals are Expressions, so `Expression` alone stands for them.
        expect(types).toContain("Expression")
        expect(types).not.toContain("Identifier")
        expect(types).not.toContain("StringLiteral")
    })
})

describe("parser output conforms to the schema", () => {
    const schema = buildSchema()

    for (const {name, code} of [...fixtures(), ...realWorld()]) {
        it(name, () => {
            const result = newConformance()
            checkAst(schema, parseToJson(code).ast, name, result)
            expect(result.nodes).toBeGreaterThan(0)
            expect(result.violations).toEqual([])
        })
    }
})

describe("syntax fixtures", () => {
    const schema = buildSchema()

    // Only the recovery fixture is meant to produce parse errors.
    const EXPECTED_ERRORS: Record<string, number> = {"recovery.js.fixture": 5}

    it("parse with the expected number of recovered errors", () => {
        const actual = Object.fromEntries(fixtures().map(({name, code}) => [name, parseToJson(code).errors]))
        const expected = Object.fromEntries(fixtures().map(({name}) => [name, EXPECTED_ERRORS[name] ?? 0]))
        expect(actual).toEqual(expected)
    })

    // Node types no fixture contains because the parser cannot produce them with its configuration (src/Defaults.ts):
    // Flow is not enabled, and a few syntaxes depend on plugins or options that are off. Everything else must be covered,
    // so that every generated accessor is exercised. Adding a fixture for one of these (or an upgrade that makes one
    // reachable) means removing it from here; a new node type means adding a fixture.
    const UNREACHABLE = `
        AnyTypeAnnotation ArgumentPlaceholder ArrayTypeAnnotation BindExpression BooleanLiteralTypeAnnotation
        BooleanTypeAnnotation ClassImplements DecimalLiteral DeclareClass DeclareExportAllDeclaration
        DeclareExportDeclaration DeclareFunction DeclareInterface DeclareModule DeclareModuleExports DeclareOpaqueType
        DeclareTypeAlias DeclareVariable DeclaredPredicate EmptyTypeAnnotation EnumBooleanBody EnumBooleanMember
        EnumDeclaration EnumDefaultedMember EnumNumberBody EnumNumberMember EnumStringBody EnumStringMember EnumSymbolBody
        ExistsTypeAnnotation FunctionTypeAnnotation FunctionTypeParam GenericTypeAnnotation ImportExpression
        IndexedAccessType InferredPredicate InterfaceDeclaration InterfaceExtends InterfaceTypeAnnotation
        IntersectionTypeAnnotation MixedTypeAnnotation ModuleExpression Noop NullLiteralTypeAnnotation
        NullableTypeAnnotation NumberLiteralTypeAnnotation NumberTypeAnnotation ObjectTypeAnnotation ObjectTypeCallProperty
        ObjectTypeIndexer ObjectTypeInternalSlot ObjectTypeProperty ObjectTypeSpreadProperty OpaqueType
        OptionalIndexedAccessType ParenthesizedExpression PipelineBareFunction PipelinePrimaryTopicReference
        PipelineTopicExpression Placeholder QualifiedTypeIdentifier RecordExpression StringLiteralTypeAnnotation
        StringTypeAnnotation SymbolTypeAnnotation TSEnumBody TSTemplateLiteralType ThisTypeAnnotation TopicReference
        TupleExpression TupleTypeAnnotation TypeAlias TypeAnnotation TypeCastExpression TypeParameter
        TypeParameterDeclaration TypeParameterInstantiation TypeofTypeAnnotation UnionTypeAnnotation V8IntrinsicIdentifier
        Variance VoidPattern VoidTypeAnnotation
    `.split(/\s+/).filter(Boolean).sort()

    it("contain every node type the parser can produce", () => {
        const result = newConformance()
        for (const {name, code} of fixtures()) checkAst(schema, parseToJson(code).ast, name, result)
        const unseen = schema.nodes.map((n) => n.type).filter((type) => !result.seen.has(type))
        expect(unseen).toEqual(UNREACHABLE)
    })

    // Fields of reachable node types that no fixture contains: legacy or Flow-only (`predicate`, `variance`,
    // `assertions`, ...), set by plugins that are off (`decorators` on patterns, `phase`, `module`), or only ever
    // built, never parsed (`optional` on calls, `File.tokens`), or always `null` (`id` on methods and arrow functions). The generated accessors for these are therefore
    // only exercised through their empty path.
    const UNEXERCISED_FIELDS = `
        ArrayPattern.optional ArrayPattern.decorators ArrowFunctionExpression.expression
        ArrowFunctionExpression.predicate ArrowFunctionExpression.id AssignmentPattern.typeAnnotation
        AssignmentPattern.optional AssignmentPattern.decorators CallExpression.typeArguments CallExpression.optional
        ClassAccessorProperty.optional ClassAccessorProperty.variance ClassDeclaration.mixins ClassExpression.typeParameters ClassExpression.superTypeParameters
        ClassExpression.implements ClassExpression.decorators ClassExpression.mixins ClassMethod.abstract
        ClassMethod.access ClassMethod.id ClassPrivateMethod.abstract ClassPrivateMethod.accessibility
        ClassPrivateMethod.override ClassPrivateMethod.computed ClassPrivateMethod.optional ClassPrivateMethod.access
        ClassPrivateMethod.decorators ClassPrivateMethod.id ClassPrivateProperty.decorators
        ClassPrivateProperty.variance ClassProperty.variance ExportAllDeclaration.assertions
        ExportNamedDeclaration.assertions File.tokens FunctionDeclaration.declare FunctionDeclaration.predicate
        FunctionExpression.predicate ImportDeclaration.assertions ImportDeclaration.module ImportDeclaration.phase
        JSXElement.selfClosing JSXOpeningElement.typeArguments MemberExpression.optional NewExpression.typeArguments
        ObjectMethod.decorators ObjectMethod.id ObjectPattern.optional ObjectPattern.decorators
        ObjectProperty.decorators OptionalCallExpression.typeArguments RestElement.optional RestElement.decorators
        TSDeclareMethod.override TSDeclareMethod.access TSDeclareMethod.decorators TSDeclareMethod.id
        TSEnumDeclaration.initializer TSEnumDeclaration.body TSImportType.options TSParameterProperty.decorators
        TSPropertySignature.kind VariableDeclarator.definite
    `.split(/\s+/).filter(Boolean)

    it("exercise every field of every reachable node type but a known few", () => {
        const result = newConformance()
        for (const {name, code} of fixtures()) checkAst(schema, parseToJson(code).ast, name, result)
        const unexercised = schema.nodes
            .filter((node) => result.seen.has(node.type))
            .flatMap((node) => node.fields.map((f) => `${node.type}.${f.name}`))
            .filter((key) => !result.present.has(key))
        expect(unexercised).toEqual(UNEXERCISED_FIELDS)
    })
})
