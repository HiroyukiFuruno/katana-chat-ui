import Foundation

enum ObserverTestError: Error, CustomStringConvertible {
    case assertion(String)

    var description: String {
        switch self {
        case let .assertion(message): message
        }
    }
}

@main
struct NativeCommitObserverTests {
    static func main() {
        let output = CommandLine.arguments.dropFirst().first(where: { $0.hasPrefix("--result=") })
            .map { URL(fileURLWithPath: String($0.dropFirst("--result=".count))) }
            ?? URL(fileURLWithPath: "/tmp/kcu-native-commit-observer-test-result.json")
        do {
            try runAll()
            try write(["status": "passed", "exit": 0], to: output)
            print("native commit observer tests passed: \(output.path)")
            exit(0)
        } catch {
            try? write(["status": "failed", "exit": 1, "error": String(describing: error)], to: output)
            fputs("native commit observer tests failed: \(error)\n", stderr)
            exit(1)
        }
    }

    private static func runAll() throws {
        var observer = NativeCommitObserver()
        try expect(observer.observe(text: "日本語", isMarked: true) == nil, "marked Japanese must not commit")
        try expect(observer.observe(text: "日本語", isMarked: false) == nil, "first unmarked poll must not commit")
        try expect(observer.observe(text: "日本語", isMarked: false) == "日本語", "second stable unmarked poll must commit")
        try expect(observer.observe(text: "日本語", isMarked: false) == nil, "repeated committed state must deduplicate")

        var changing = NativeCommitObserver()
        try expect(changing.observe(text: "n", isMarked: false) == nil, "first n poll must wait")
        try expect(changing.observe(text: "ni", isMarked: false) == nil, "text change must reset stability")
        try expect(changing.observe(text: "nihongo", isMarked: false) == nil, "text change must reset stability again")
        try expect(changing.observe(text: "nihongo", isMarked: false) == "nihongo", "nihongo commits only after two stable polls")

        var cancel = NativeCommitObserver()
        _ = cancel.observe(text: "日本語", isMarked: false)
        _ = cancel.observe(text: "日本語", isMarked: false)
        try expect(cancel.observe(text: "日本語てすと", isMarked: true) == nil, "cancel preedit must remain marked")
        try expect(cancel.observe(text: "日本語", isMarked: false) == nil, "restoring committed text must not emit")
        try expect(cancel.observe(text: "日本語", isMarked: false) == nil, "stable restored text must remain deduplicated")

        var deletion = NativeCommitObserver(committedText: "日本語")
        try expect(deletion.observe(text: "", isMarked: false) == nil, "deletion waits for stability")
        try expect(deletion.observe(text: "", isMarked: false) == "", "stable deletion must emit empty text")

        var markReset = NativeCommitObserver()
        try expect(markReset.observe(text: "日本語", isMarked: false) == nil, "mark reset setup waits")
        try expect(markReset.observe(text: "日本語", isMarked: true) == nil, "marking resets stability")
        try expect(markReset.observe(text: "日本語", isMarked: false) == nil, "unmark transition starts a new stability window")
        try expect(markReset.observe(text: "日本語", isMarked: false) == "日本語", "post-reset stable text commits once")
    }

    private static func expect(_ condition: Bool, _ message: String) throws {
        guard condition else { throw ObserverTestError.assertion(message) }
    }

    private static func write(_ value: [String: Any], to path: URL) throws {
        let data = try JSONSerialization.data(withJSONObject: value, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: path, options: .atomic)
    }
}
