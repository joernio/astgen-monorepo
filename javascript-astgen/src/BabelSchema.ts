// Builds an in-memory model of the AST shape that `astgen` emits.
//
// The model is derived from the node definitions that ship with `@babel/types`
// (`NODE_FIELDS`, `FLIPPED_ALIAS_KEYS`, ...) plus a small, explicit overlay for
// the places where `@babel/parser`'s output deviates from those definitions
// (they describe how nodes are *built*, the parser has a few extras).
//
// Two consumers share it so they cannot drift apart:
//   - [ScalaAstGenerator.ts](./ScalaAstGenerator.ts) renders it as Scala code.
//   - test/BabelSchema.test.ts validates real parser output against it, which
//     is what keeps the overlay honest across `@babel/parser` upgrades.

import * as babelTypes from "@babel/types"
import babelTypesPackage from "@babel/types/package.json"

/** The shape of the value stored in one field of a node. */
export type ValueType =
    /** A single node. `types` holds node type names and/or alias names (a union). */
    | { kind: "node"; types: string[] }
    /** A JSON array. `nullableElements` is set for sparse arrays like `[a, , b]`. */
    | { kind: "array"; element: ValueType; nullableElements: boolean }
    | { kind: "string" }
    | { kind: "boolean" }
    | { kind: "number" }
    /** One of a fixed set of literal values. */
    | { kind: "enum"; values: (string | boolean)[] }
    /** Anything we do not model further; handed to consumers as raw JSON. */
    | { kind: "json" }

export type FieldSpec = {
    name: string
    type: ValueType
    /** Optional fields may be absent or `null` in the JSON. */
    optional: boolean
}

export type NodeSpec = {
    /** The `type` discriminator in the JSON, e.g. `Identifier`. */
    type: string
    /** Alias groups this node belongs to, e.g. `Expression`. Sorted. */
    aliases: string[]
    fields: FieldSpec[]
}

export type Schema = {
    /** Version of `@babel/types` the schema was derived from. */
    babelTypesVersion: string
    /** Every node type, sorted by name. */
    nodes: NodeSpec[]
    /** Alias name -> concrete node types that belong to it. */
    aliasMembers: ReadonlyMap<string, ReadonlySet<string>>
}

/**
 * Keys every parser-produced node may carry, independent of its type. They are
 * not part of the per-type field lists.
 */
export const UNIVERSAL_KEYS: ReadonlySet<string> = new Set([
    "type", "start", "end", "loc", "range", "extra",
    "leadingComments", "trailingComments", "innerComments",
])

// ---------------------------------------------------------------------------
// Aliases and type constructors
// ---------------------------------------------------------------------------

/** The alias grouping the two comment node types, which only the parser knows. */
export const COMMENT_ALIAS = "Comment"
const COMMENT_TYPES = ["CommentBlock", "CommentLine"]

/**
 * The TypeScript plugin's recovery node for a type annotation where none is allowed, e.g. the argument of
 * `f(x: any[])`. `@babel/types` has no definition for it, so it is added by hand (see `PARSER_ONLY_NODES`).
 */
const TYPE_CAST = "TSTypeCastExpression"

/** Alias name -> member node types. Static data, so it is computed once. */
const ALIAS_MEMBERS: ReadonlyMap<string, ReadonlySet<string>> = new Map([
    ...Object.entries(babelTypes.FLIPPED_ALIAS_KEYS as Record<string, string[]>)
        // It stands wherever an expression does: `f(x: T)`, `(x: T)`, `[x: T]`.
        .map(([alias, members]) => [alias, new Set(alias === "Expression" ? [...members, TYPE_CAST] : members)] as const),
    [COMMENT_ALIAS, new Set(COMMENT_TYPES)],
])

/**
 * A union of node types and/or aliases: deduplicated, sorted, and without
 * concrete types that an alias in the same union already covers.
 */
function nodeOf(...types: string[]): ValueType {
    const unique = [...new Set(types)]
    const covered = (type: string) => unique.some((other) => other !== type && ALIAS_MEMBERS.get(other)?.has(type))
    return {kind: "node", types: unique.filter((type) => !covered(type)).sort()}
}

const json: ValueType = {kind: "json"}
const arrayOf = (element: ValueType, nullableElements = false): ValueType =>
    ({kind: "array", element, nullableElements})
const field = (name: string, type: ValueType, optional = false): FieldSpec => ({name, type, optional})

// ---------------------------------------------------------------------------
// Parser overlay
//
// `@babel/types` describes how nodes are *built*; `@babel/parser` deviates in a
// few places. Each table below documents one kind of deviation.
// test/BabelSchema.test.ts parses a broad corpus against the schema, so a
// missing or wrong entry shows up as a failure, and `buildSchema` rejects
// entries that no longer change anything.
// ---------------------------------------------------------------------------

/** Node types the parser emits that `@babel/types` has no entry for. */
const PARSER_ONLY_NODES: NodeSpec[] = [
    ...COMMENT_TYPES.map((type) => ({type, aliases: [COMMENT_ALIAS], fields: [field("value", {kind: "string"})]})),
    {
        type: TYPE_CAST,
        aliases: ["Expression"],
        fields: [field("expression", nodeOf("Expression")), field("typeAnnotation", nodeOf("TSTypeAnnotation"))],
    },
]

/**
 * The parser puts `id: null` on every function-like node, though only some of them can have a name. The key is part of
 * the output, so it stays modelled (consumers match over all function kinds and read `id` uniformly).
 */
const OPTIONAL_ID = field("id", nodeOf("Identifier"), true)

/** Fields the parser emits, or whose type it changes. Each replaces the same-named `@babel/types` field. */
const PARSER_FIELDS: Record<string, FieldSpec[]> = {
    // The parser attaches its recovered-error list to the root node.
    File: [field("errors", arrayOf(json)), field("comments", arrayOf(nodeOf(COMMENT_ALIAS)), true)],
    ObjectProperty: [field("method", {kind: "boolean"}, true)],
    ObjectMethod: [field("method", {kind: "boolean"}, true), OPTIONAL_ID],
    ArrowFunctionExpression: [OPTIONAL_ID],
    ClassMethod: [OPTIONAL_ID],
    ClassPrivateMethod: [OPTIONAL_ID],
    // `@babel/types` 8 dropped `decorators` from the node, but the parser still attaches them: `@dec() m(): void`.
    TSDeclareMethod: [OPTIONAL_ID, field("decorators", arrayOf(nodeOf("Decorator")), true)],
}

/** Fields `@babel/types` calls required, but the parser leaves out. */
const OPTIONAL_FIELDS = new Set([
    // `class A { #m() {} }`: `@babel/types` requires the flag, but private names cannot be computed.
    "ClassPrivateMethod.computed",
    // `interface I { new?: string }`: the parser leaves the flag out when the key is `new`.
    "TSPropertySignature.computed",
    // `declare module "x";` has no body at all.
    "TSModuleDeclaration.body",
    // `1e999` is Infinity, which JSON cannot represent: astgen writes `null` (the source is in `extra.raw`).
    "NumericLiteral.value",
])

/** Node types the parser puts into a field in addition to what `@babel/types` lists. */
const EXTRA_TYPES: Record<string, string[]> = {
    "TSRestType.typeAnnotation": ["TSNamedTupleMember"], // `[...name: string[]]`
    "ExportDefaultDeclaration.declaration": ["TSInterfaceDeclaration"], // `export default interface I {}`
}

// ---------------------------------------------------------------------------
// Deriving field types from @babel/types validators
// ---------------------------------------------------------------------------

/**
 * Structural view of the validator objects stored in `NODE_FIELDS[...].validate`.
 * They are plain functions decorated with introspection properties.
 */
type Validator = {
    oneOfNodeTypes?: string[]
    oneOfNodeOrValueTypes?: string[]
    oneOf?: (string | boolean)[]
    type?: string
    each?: Validator
    chainOf?: Validator[]
}

const SCALARS: Record<string, ValueType> = {
    string: {kind: "string"}, boolean: {kind: "boolean"}, number: {kind: "number"}, any: json, array: arrayOf(json),
}

function derive(validator?: Validator): ValueType | undefined {
    if (!validator) return undefined
    // Validators in a chain go from general to specific (array -> each, string -> oneOf, ...):
    // the last informative one wins.
    if (validator.chainOf) return validator.chainOf.map(derive).filter((type) => type !== undefined).at(-1)
    if (validator.each) {
        // `null` as an element type marks a sparse array like `[a, , b]`.
        return arrayOf(derive(validator.each) ?? json, validator.each.oneOfNodeOrValueTypes?.includes("null") ?? false)
    }
    if (validator.oneOfNodeTypes) return nodeOf(...validator.oneOfNodeTypes)
    if (validator.oneOfNodeOrValueTypes) return nodeOf(...validator.oneOfNodeOrValueTypes.filter((t) => t !== "null"))
    if (validator.oneOf) return {kind: "enum", values: validator.oneOf}
    return validator.type ? SCALARS[validator.type] : undefined
}

/** Adds `extra` to a node type, or to the elements of an array of nodes. */
function widen(type: ValueType, extra: string[]): ValueType {
    if (type.kind === "node") return nodeOf(...type.types, ...extra)
    if (type.kind === "array") return {...type, element: widen(type.element, extra)}
    throw new Error(`cannot widen a field of kind '${type.kind}'`)
}

// ---------------------------------------------------------------------------
// Schema construction
// ---------------------------------------------------------------------------

type RawField = { optional?: boolean; validate?: Validator }

export function buildSchema(): Schema {
    const nodeFields = babelTypes.NODE_FIELDS as unknown as Record<string, Record<string, RawField>>
    const unusedOptional = new Set(OPTIONAL_FIELDS)
    const unusedExtra = new Set(Object.keys(EXTRA_TYPES))

    const nodes: NodeSpec[] = Object.entries(nodeFields).map(([type, rawFields]) => {
        const fields = new Map<string, FieldSpec>()
        for (const [name, raw] of Object.entries(rawFields)) {
            const key = `${type}.${name}`
            let fieldType = derive(raw.validate) ?? json
            if (unusedExtra.delete(key)) fieldType = widen(fieldType, EXTRA_TYPES[key])
            if (unusedOptional.delete(key) && raw.optional) throw new Error(`${key} is optional already: drop it from OPTIONAL_FIELDS`)
            fields.set(name, field(name, fieldType, raw.optional === true || OPTIONAL_FIELDS.has(key)))
        }
        for (const parserField of PARSER_FIELDS[type] ?? []) {
            if (JSON.stringify(fields.get(parserField.name)) === JSON.stringify(parserField)) {
                throw new Error(`${type}.${parserField.name} matches @babel/types already: drop it from PARSER_FIELDS`)
            }
            fields.set(parserField.name, parserField)
        }
        const aliases = [...ALIAS_MEMBERS].filter(([, members]) => members.has(type)).map(([alias]) => alias).sort()
        return {type, aliases, fields: [...fields.values()]}
    })

    const stale = [...unusedOptional, ...unusedExtra, ...Object.keys(PARSER_FIELDS).filter((type) => !(type in nodeFields))]
    if (stale.length > 0) throw new Error(`The parser overlay refers to unknown fields or node types: ${stale.join(", ")}`)

    return {
        babelTypesVersion: babelTypesPackage.version,
        nodes: [...nodes, ...PARSER_ONLY_NODES].sort((a, b) => (a.type < b.type ? -1 : 1)),
        aliasMembers: ALIAS_MEMBERS,
    }
}
