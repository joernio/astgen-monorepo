import {UNIVERSAL_KEYS, type Schema, type ValueType} from "../src/BabelSchema"

/** Aggregated result of validating one or more ASTs against a {@link Schema}. */
export type Conformance = {
    /** Human-readable violations; empty when everything conforms. */
    violations: string[]
    /** Number of nodes checked. */
    nodes: number
    /** `Type.field` -> number of nodes that had the field present and non-null. */
    present: Map<string, number>
    /** `Type` -> number of nodes seen. */
    seen: Map<string, number>
}

export function newConformance(): Conformance {
    return {violations: [], nodes: 0, present: new Map(), seen: new Map()}
}

const MAX_VIOLATIONS = 200

function isNode(v: unknown): v is { type: string; [k: string]: unknown } {
    return typeof v === "object" && v !== null && typeof (v as { type?: unknown }).type === "string"
}

/**
 * Walks `root` guided by `schema` and records every place where the JSON does
 * not match: unknown node types, keys the schema does not list, required
 * fields that are absent or null, and values of the wrong shape.
 *
 * `origin` is only used to label violations (a file name, fixture name, ...).
 */
export function checkAst(schema: Schema, root: unknown, origin: string, into: Conformance): void {
    const nodesByType = new Map(schema.nodes.map((n) => [n.type, n]))

    const fail = (path: string, message: string): void => {
        if (into.violations.length < MAX_VIOLATIONS) into.violations.push(`${origin}: ${path}: ${message}`)
    }

    // Returns an error message, or undefined if `value` fits `type`.
    // Child nodes are *not* descended into here; visit() does that so each node is checked once.
    function fits(value: unknown, type: ValueType): string | undefined {
        switch (type.kind) {
            case "json":
                return undefined
            case "string":
                return typeof value === "string" ? undefined : `expected string, got ${describe(value)}`
            case "boolean":
                return typeof value === "boolean" ? undefined : `expected boolean, got ${describe(value)}`
            case "number":
                return typeof value === "number" ? undefined : `expected number, got ${describe(value)}`
            case "enum":
                return type.values.includes(value as string | boolean)
                    ? undefined
                    : `expected one of ${JSON.stringify(type.values)}, got ${describe(value)}`
            case "node": {
                if (!isNode(value)) return `expected node, got ${describe(value)}`
                const allowed = type.types.some((t) =>
                    t === value.type || schema.aliasMembers.get(t)?.has(value.type) === true)
                return allowed ? undefined : `node type ${value.type} not in [${type.types.join(" | ")}]`
            }
            case "array": {
                if (!Array.isArray(value)) return `expected array, got ${describe(value)}`
                for (let i = 0; i < value.length; i++) {
                    if (value[i] === null) {
                        if (!type.nullableElements) return `element ${i} is null`
                        continue
                    }
                    const err = fits(value[i], type.element)
                    if (err) return `element ${i}: ${err}`
                }
                return undefined
            }
        }
    }

    function describe(v: unknown): string {
        if (v === null) return "null"
        if (Array.isArray(v)) return "array"
        if (isNode(v)) return `node ${v.type}`
        return typeof v
    }

    // Follows the child links the schema knows about, so opaque `json` fields
    // (e.g. File.errors) are never mistaken for nodes.
    function childrenOf(value: unknown, type: ValueType): unknown[] {
        if (type.kind === "node") return isNode(value) ? [value] : []
        if (type.kind === "array" && Array.isArray(value)) return value.flatMap((v) => childrenOf(v, type.element))
        return []
    }

    function visit(node: { type: string; [k: string]: unknown }, path: string): void {
        into.nodes++
        into.seen.set(node.type, (into.seen.get(node.type) ?? 0) + 1)
        const spec = nodesByType.get(node.type)
        if (!spec) {
            fail(path, `unknown node type ${node.type}`)
            return
        }

        const known = new Set(spec.fields.map((f) => f.name))
        for (const key of Object.keys(node)) {
            if (!known.has(key) && !UNIVERSAL_KEYS.has(key)) fail(path, `unexpected key '${key}'`)
        }

        for (const key of ["leadingComments", "trailingComments", "innerComments"]) {
            const comments = node[key]
            if (comments === undefined) continue
            const err = fits(comments, {kind: "array", element: {kind: "node", types: ["Comment"]}, nullableElements: false})
            if (err) fail(`${path}.${key}`, err)
            else for (const c of comments as { type: string }[]) visit(c as never, `${path}.${key}[]`)
        }

        for (const field of spec.fields) {
            const value = node[field.name]
            const fieldPath = `${path}.${field.name}`
            if (value === undefined || value === null) {
                if (!field.optional) fail(fieldPath, `required field is ${value === null ? "null" : "absent"}`)
                continue
            }
            into.present.set(`${node.type}.${field.name}`, (into.present.get(`${node.type}.${field.name}`) ?? 0) + 1)
            const err = fits(value, field.type)
            if (err) {
                fail(fieldPath, err)
                continue
            }
            for (const child of childrenOf(value, field.type)) visit(child as never, `${fieldPath}<${(child as { type: string }).type}>`)
        }
    }

    if (isNode(root)) visit(root, root.type)
    else fail("<root>", "AST root is not a node")
}
