import Foundation

extension URL {
    /// Returns the path of `self` relative to `base`, or `nil` if either URL is not a file URL.
    ///
    /// Both URLs are standardized and have their symlinks resolved before comparison, and the
    /// returned string uses `/` as separator. The receiver may be located outside `base` — the
    /// result will then start with one or more `..` components.
    ///
    /// - Note: This method is intentionally named `pathRelative(to:)` (rather than `relativePath(from:)`)
    ///   to avoid shadowing the standard-library `URL.relativePath` property.
    func pathRelative(to base: URL) -> String? {
        guard self.isFileURL && base.isFileURL else {
            return nil
        }

        let destComponents = self.standardized.resolvingSymlinksInPath().pathComponents
        let baseComponents = base.standardized.resolvingSymlinksInPath().pathComponents

        var i = 0
        while i < destComponents.count && i < baseComponents.count
            && destComponents[i] == baseComponents[i]
        {
            i += 1
        }

        var relComponents = Array(repeating: "..", count: baseComponents.count - i)
        relComponents.append(contentsOf: destComponents[i...])
        return relComponents.joined(separator: "/")
    }
}
