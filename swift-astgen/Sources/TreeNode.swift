final class TreeNode: Codable {
  var index: Int
  var name: String
  var tokenKind: String
  var nodeType: String
  var range: Range
  var children: [TreeNode]

  init(tokenKind: String, nodeType: String, range: Range, children: [TreeNode]) {
    self.index = -1
    self.name = ""
    self.tokenKind = tokenKind
    self.nodeType = nodeType
    self.range = range
    self.children = children
  }
}

extension TreeNode: CustomStringConvertible {
  var description: String {
    """
    {
      index: \(name)
      name: \(name)
      tokenKind: \(tokenKind)
      nodeType: \(nodeType)
      range: \(range)
      children: \(String(describing: children))
    }
    """
  }
}

struct Range: Codable, Equatable {
  let startOffset: Int
  let endOffset: Int
  let startLine: Int
  let startColumn: Int
  let endLine: Int
  let endColumn: Int
}

extension Range: CustomStringConvertible {
  var description: String {
    """
    {
      startOffset: \(startOffset)
      endOffset: \(endOffset)
      startLine: \(startLine)
      startColumn: \(startColumn)
      endLine: \(endLine)
      endColumn: \(endColumn)
    }
    """
  }
}
