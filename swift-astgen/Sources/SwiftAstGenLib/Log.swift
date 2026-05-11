import Foundation

/// Lightweight stdout/stderr logger.
///
/// The CLI never aborts on per-file failures; instead, problems are logged through
/// ``Log/warn(_:)`` so they remain visible without affecting exit codes. ``Log/info(_:)``
/// goes to stdout for routine progress output.
enum Log {

    /// Writes `message` followed by a newline to standard output.
    static func info(_ message: @autoclosure () -> String) {
        print(message())
    }

    /// Writes `message` followed by a newline to standard error.
    static func warn(_ message: @autoclosure () -> String) {
        let line = message() + "\n"
        if let data = line.data(using: .utf8) {
            FileHandle.standardError.write(data)
        }
    }
}
