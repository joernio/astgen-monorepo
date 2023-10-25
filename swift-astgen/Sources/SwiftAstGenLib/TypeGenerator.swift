import CodeGeneration

struct TypeGenerator {

  struct ReturnTypeAndCast {
    let returnType: String
    let cast: String
  }

  private static func type(for child: Child) -> String {
    switch child.kind {
    case .node(let kind):
      return "\(kind.syntaxType)"
    case .nodeChoices(let choices):
      let choicesDescriptions = choices.map { type(for: $0) }
      return "\(choicesDescriptions.joined(separator: " | "))"
    case .collection(let kind, _, _, _):
      return "\(kind.syntaxType)"
    case .token(_, _, _):
      return "SwiftToken"
    }
  }

  static func returnTypeAndCast(for child: Child) -> ReturnTypeAndCast {
    let childType = type(for: child)
    let isOptional = child.isOptional
    let returnType = isOptional ? "Option[\(childType)]" : "\(childType)"
    let cast =
      isOptional ? ".map(_.asInstanceOf[\(childType)])" : ".head.asInstanceOf[\(childType)]"
    return ReturnTypeAndCast(returnType: returnType, cast: cast)
  }

  static func returnTypeAndCast(for collection: CollectionNode) -> ReturnTypeAndCast {
    let collectionType: String
    if let onlyElement = collection.elementChoices.only {
      collectionType = "\(onlyElement.syntaxType)"
    } else {
      collectionType =
        "\(collection.elementChoices.map { "\($0.syntaxType)" }.joined(separator: " | "))"
    }
    let returnType = "Seq[\(collectionType)]"
    let cast = ".map(_.asInstanceOf[\(collectionType)])"
    return ReturnTypeAndCast(returnType: returnType, cast: cast)
  }

}
