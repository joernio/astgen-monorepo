import CodeGeneration

/// Renders Scala type expressions for SwiftSyntax ``Child`` and ``CollectionNode`` definitions.
///
/// Used by ``ScalaAstGenerator`` to generate the strongly typed Scala wrapper that downstream
/// consumers (notably the Joern Swift frontend) use to navigate the AST JSON.
enum TypeGenerator {

    /// Pair of Scala return type and the corresponding cast expression to apply when reading
    /// a child node from the JSON children list.
    struct ReturnTypeAndCast {
        let returnType: String
        let cast: String
    }

    /// Returns the Scala type expression for `child`. Choices are joined with ` | `, tokens
    /// always become `SwiftToken`.
    static func type(for child: Child) -> String {
        switch child.kind {
        case .node(let kind):
            return "\(kind.syntaxType)"
        case .nodeChoices(let choices, _):
            let choicesDescriptions = choices.map { type(for: $0) }
            return choicesDescriptions.joined(separator: " | ")
        case .collection(let kind, _, _, _, _):
            return "\(kind.syntaxType)"
        case .token(_, _, _):
            return "SwiftToken"
        }
    }

    /// Returns the Scala return type and matching `.asInstanceOf` cast for `child`.
    static func returnTypeAndCast(for child: Child) -> ReturnTypeAndCast {
        let childType = type(for: child)
        let isOptional = child.isOptional
        let returnType = isOptional ? "Option[\(childType)]" : childType
        let cast =
            isOptional ? ".map(_.asInstanceOf[\(childType)])" : ".head.asInstanceOf[\(childType)]"
        return ReturnTypeAndCast(returnType: returnType, cast: cast)
    }

    /// Returns the Scala element type for a syntax collection.
    /// Multiple element choices are joined with ` | `.
    static func collectionElementType(for collection: CollectionNode) -> String {
        if let onlyElement = collection.elementChoices.only {
            return "\(onlyElement.syntaxType)"
        } else {
            return collection.elementChoices.map { "\($0.syntaxType)" }.joined(separator: " | ")
        }
    }

    /// Returns the Scala return type and matching `.asInstanceOf` cast for `collection`.
    static func returnTypeAndCast(for collection: CollectionNode) -> ReturnTypeAndCast {
        let collectionType = collectionElementType(for: collection)
        let returnType = "Seq[\(collectionType)]"
        let cast = ".map(_.asInstanceOf[\(collectionType)])"
        return ReturnTypeAndCast(returnType: returnType, cast: cast)
    }
}
