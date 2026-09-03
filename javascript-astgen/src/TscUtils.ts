import * as Defaults from "./Defaults"

import tsc from "typescript"
import * as path from "node:path"

// tsc resolves its bundled lib.*.d.ts files (Array, Uppercase<T>, etc.) relative to
// `ts.sys.getExecutingFilePath()`. Bun's bundler bakes that path in at compile time
// as the build machine's absolute `node_modules/typescript/lib` path, which does not
// exist on whatever machine eventually runs the compiled binary — silently starving
// the TypeChecker of its standard library (arrays degrade to `{}`, intrinsics like
// `Uppercase<T>` never resolve, and everything otherwise resolvable falls back to
// `any`). Point it at the copy of that directory embedded alongside the binary
// instead (see the matching `assets` entry in scripts/build-binaries.ts). Only the
// directory is used (via getDirectoryPath in tsc), so the filename here is a dummy.
if (Bun.isStandaloneExecutable) {
    const embeddedLibDir = path.join(import.meta.dir, "lib")
    tsc.sys.getExecutingFilePath = () => path.join(embeddedLibDir, "typescript.js")
}

export type TypeMap = Map<number, string>

// Packs (start, end) positions into a single number key.
// Using a Map<number, string> avoids per-entry string allocation; at the scale of a full
// TypeMap (one entry per AST node), packed doubles (~12B each) are ~4x cheaper than
// equivalent "start:end" strings (~50B each) and avoid the N inner-Map overhead of a
// nested Map<number, Map<number, string>>.
//
// POS_SHIFT = 2^26: supports positions up to 64MB per file.
// The MAX_FILE_SIZE_BYTES guard in FileUtils ensures this assumption holds; the
// invariant check below catches future drift if either constant is bumped.
const POS_SHIFT = 0x4000000

if (Defaults.MAX_FILE_SIZE_BYTES >= POS_SHIFT) {
    throw new Error(
        `Invariant violated: MAX_FILE_SIZE_BYTES (${Defaults.MAX_FILE_SIZE_BYTES}) must be < POS_SHIFT (${POS_SHIFT})`
        + ` so that encodePos/decodePos round-trip correctly`,
    )
}

export function encodePos(start: number, end: number): number {
    return start * POS_SHIFT + end
}
export function decodePos(key: number): [number, number] {
    const start = Math.floor(key / POS_SHIFT)
    return [start, key - start * POS_SHIFT]
}

/**
 * Utility class for working with the TypeScript compiler API.
 *
 * `TscUtils` provides methods to analyze TypeScript source files, extract type information,
 * and map AST nodes to their inferred types. It leverages the TypeScript compiler's
 * `Program` and `TypeChecker` to perform type analysis.
 *
 * Main features:
 * - Generates a map of node positions to their type strings for a given file.
 * - Safely converts TypeScript types to string representations.
 * - Identifies signature declarations and function-like nodes.
 */
export default class TscUtils {
    private readonly program: tsc.Program
    private readonly typeChecker: tsc.TypeChecker

    constructor(files: string[]) {
        this.program = tsc.createProgram(files, Defaults.DEFAULT_TSC_OPTIONS)
        this.typeChecker = this.program.getTypeChecker()
    }

    /**
     * Generates a map of node positions to their inferred type strings for a given TypeScript source file.
     *
     * This method traverses the AST of the specified file, analyzes each node using the TypeScript compiler API,
     * and records the type information for relevant nodes. The resulting map keys each node's position as a
     * packed (start, end) number (see {@link encodePos}); writers decode it back to "start:end" on output.
     *
     * @param file - The path to the TypeScript source file to analyze.
     * @returns A `TypeMap` mapping node positions to their inferred type strings.
     */
    typeMapForFile(file: string): TypeMap {
        const seenTypes = new Map<number, string>()

        const addType = (node: tsc.Node): void => {
            if (!this.shouldResolveType(node)) return
            let typeStr: string | null
            if (this.isSignatureDeclaration(node)) {
                const signature = this.typeChecker.getSignatureFromDeclaration(node)
                if (signature) {
                    const returnType: tsc.Type = this.typeChecker.getReturnTypeOfSignature(signature)
                    typeStr = this.safeTypeToString(returnType)
                } else {
                    typeStr = this.safeTypeToString(this.typeChecker.getTypeAtLocation(node))
                }
            } else if (tsc.isFunctionLike(node)) {
                const funcType: tsc.Type = this.typeChecker.getTypeAtLocation(node)
                const funcSignature: tsc.Signature = this.typeChecker.getSignaturesOfType(funcType, tsc.SignatureKind.Call)[0]
                typeStr = funcSignature
                    ? this.safeTypeToString(funcSignature.getReturnType())
                    : this.safeTypeToString(funcType)
            } else {
                typeStr = this.safeTypeToString(this.typeChecker.getTypeAtLocation(node))
            }
            if (typeStr !== null) {
                seenTypes.set(encodePos(node.getStart(), node.getEnd()), typeStr)
            }
        }

        const sourceFile = this.program.getSourceFile(file)
        if (!sourceFile) {
            throw new Error(`TscUtils: source file not present in program: ${file}`)
        }
        this.forEachNode(sourceFile, addType)
        return seenTypes
    }

    private forEachNode(ast: tsc.Node, callback: (node: tsc.Node) => void): void {
        function visit(node: tsc.Node) {
            tsc.forEachChild(node, visit)
            callback(node)
        }

        visit(ast)
    }

    /**
     * Renders a TS type to a Joern-friendly type string. Returns `null` when
     * the type is unhelpful (`any`-equivalent, unresolved, too long, or
     * `unknown`); the caller filters those out of the resulting `TypeMap`.
     *
     * Specific transforms (intentional Joern conventions):
     * - quoted literal types (`"foo"`, `` `bar` ``) → `string`
     * - array suffix types (`Foo[]`) → `__ecma.Array`
     */
    private safeTypeToString(node: tsc.Type): string | null {
        try {
            const tpe: string = this.typeChecker.typeToString(node, undefined, Defaults.DEFAULT_TSC_TYPE_OPTIONS)
            if (tpe.length === 0) return null
            if (tpe.length > Defaults.MAX_TYPE_STRING_LENGTH) return null
            if (tpe === Defaults.UNKNOWN) return null
            if (tpe === Defaults.ANY) return null
            if (tpe.startsWith(Defaults.UNRESOLVED)) return null
            if (Defaults.STRING_REGEX.test(tpe)) return "string"
            if (Defaults.ARRAY_REGEX.test(tpe)) return "__ecma.Array"
            return tpe
        } catch {
            return null
        }
    }

    private isSignatureDeclaration(node: tsc.Node): node is tsc.SignatureDeclaration {
        return tsc.isSetAccessor(node) || tsc.isGetAccessor(node) ||
            tsc.isConstructSignatureDeclaration(node) || tsc.isMethodDeclaration(node) ||
            tsc.isFunctionDeclaration(node) || tsc.isConstructorDeclaration(node)
    }

    private shouldResolveType(node: tsc.Node): boolean {
        const k = node.kind
        if (k === tsc.SyntaxKind.SourceFile) return false
        if (k === tsc.SyntaxKind.EndOfFileToken) return false
        if (k === tsc.SyntaxKind.SyntaxList) return false
        if (k >= tsc.SyntaxKind.FirstKeyword && k <= tsc.SyntaxKind.LastKeyword) return false
        if (k >= tsc.SyntaxKind.FirstPunctuation && k <= tsc.SyntaxKind.LastPunctuation) return false
        if (k === tsc.SyntaxKind.Decorator) return false
        if (k >= tsc.SyntaxKind.FirstStatement && k <= tsc.SyntaxKind.LastStatement) return false
        return true
    }

}

