import Foundation

enum NativeTextTestError: Error, CustomStringConvertible {
    case assertion(String)

    var description: String {
        switch self { case let .assertion(message): message }
    }
}

@main
struct NativeTextTests {
    static func main() {
        let output = CommandLine.arguments.dropFirst().first(where: { $0.hasPrefix("--result=") })
            .map { URL(fileURLWithPath: String($0.dropFirst("--result=".count))) }
            ?? URL(fileURLWithPath: "/tmp/kcu-native-text-test-result.json")
        do {
            try runAll()
            try write(["status": "passed", "exit": 0], to: output)
            print("native text tests passed: \(output.path)")
            exit(0)
        } catch {
            try? write(["status": "failed", "exit": 1, "error": String(describing: error)], to: output)
            fputs("native text tests failed: \(error)\n", stderr)
            exit(1)
        }
    }

    private static func runAll() throws {
        let old = try line(["kind": "native-preedit", "text": "日本語", "callback_uptime": 4.0, "native_has_marked": true])
        let fresh = try line(["kind": "native-preedit", "text": "日本語", "callback_uptime": 12.0, "native_has_marked": true])
        let events = try NativeTextEvent.parseJSONL(old + fresh)
        try expect(NativeTextEvent.selectFresh(events, baselineEventCount: 1, baselineUptime: 10,
                                               kind: .preedit, expectedTexts: ["日本語"], expectedMarked: true)?.text == "日本語",
                   "fresh exact preedit must be selected")
        try expect(NativeTextEvent.selectFresh(events, baselineEventCount: 2, baselineUptime: 12,
                                               kind: .preedit, expectedTexts: ["日本語"], expectedMarked: true) == nil,
                   "stale callback must not be selected")

        let newerWrong = try line(["kind": "native-preedit", "text": "日本語訳", "callback_uptime": 13.0, "native_has_marked": true])
        let newerWrongEvents = try NativeTextEvent.parseJSONL(old + fresh + newerWrong)
        try expect(NativeTextEvent.selectFresh(newerWrongEvents, baselineEventCount: 1, baselineUptime: 10,
                                               kind: .preedit, expectedTexts: ["日本語"], expectedMarked: true) == nil,
                   "newer unexpected text must not fall back to older exact text")
        let newerWrongState = try line(["kind": "native-preedit", "text": "日本語", "callback_uptime": 13.0, "native_has_marked": false])
        let newerWrongStateEvents = try NativeTextEvent.parseJSONL(old + fresh + newerWrongState)
        try expect(NativeTextEvent.selectFresh(newerWrongStateEvents, baselineEventCount: 1, baselineUptime: 10,
                                               kind: .preedit, expectedTexts: ["日本語"], expectedMarked: true) == nil,
                   "newer wrong marked state must not fall back to older exact text")
        try expect(NativeTextEvent.latestFresh(newerWrongStateEvents, baselineEventCount: 1, baselineUptime: 10,
                                               kind: .preedit, expectedMarked: true) == nil,
                   "latest candidate selection must not fall back across a marked state change")
        let repeated = try line(["kind": "native-preedit", "text": "日本語", "callback_uptime": 12.0, "native_has_marked": true]) +
            line(["kind": "native-preedit", "text": "日本語", "callback_uptime": 12.0, "native_has_marked": true])
        let repeatedEvents = try NativeTextEvent.parseJSONL(repeated)
        try expect(NativeTextEvent.selectFresh(repeatedEvents, baselineEventCount: 0, baselineUptime: 10,
                                               kind: .preedit, expectedTexts: ["日本語"], expectedMarked: true)?.recordIndex == 1,
                   "equal uptime must select the later JSONL record deterministically")

        let duplicate = try line(["kind": "native-insert", "text": "日本語", "callback_uptime": 20.0, "native_has_marked": false]) +
            line(["kind": "native-insert", "text": "日本語", "callback_uptime": 21.0, "native_has_marked": false])
        try expect(NativeTextEvent.selectFresh(try NativeTextEvent.parseJSONL(duplicate), baselineEventCount: 0,
                                               baselineUptime: 0, kind: .insert, expectedTexts: ["日本語"], expectedMarked: false)?.recordIndex == 1,
                   "latest duplicate insert must be selected for caller-side exact-count validation")

        try expectThrows(line(["kind": "native-unmark", "callback_uptime": 22.0, "native_has_marked": false]),
                         "missing text must fail")
        try expectThrows(line(["kind": "native-unmark", "text": 123, "callback_uptime": 22.0, "native_has_marked": false]),
                         "wrong text type must fail")
        try expectThrows(line(["kind": "native-unmark", "text": "", "callback_uptime": true, "native_has_marked": false]),
                         "numeric boolean uptime must fail")
        try expectThrows(line(["kind": "native-unmark", "text": "", "callback_uptime": 22.0, "native_has_marked": 0]),
                         "numeric boolean marked state must fail")
        let tooLong = String(repeating: "a", count: 129)
        try expectThrows(line(["kind": "native-preedit", "text": tooLong, "callback_uptime": 22.0, "native_has_marked": true]),
                         "oversized text must fail")
        try expectThrows(Data(#"{"kind":"native-preedit","text":"日本語","callback_uptime":22,"native_has_marked":true"#.utf8) + Data([0x0a]),
                         "malformed complete JSON must fail")
        let partial = fresh + Data(#"{"kind":"native-preedit","text":"日本語"}"#.utf8)
        try expect(NativeTextEvent.parseJSONL(partial).count == 1, "partial final line must be ignored")
        let ignored = try line(["kind": "key-processed", "text": "日本語", "callback_uptime": 30.0, "native_has_marked": true])
        try expect(NativeTextEvent.parseJSONL(ignored).isEmpty, "other trace kinds must be ignored")
    }

    private static func line(_ dictionary: [String: Any]) throws -> Data {
        try JSONSerialization.data(withJSONObject: dictionary) + Data([0x0a])
    }

    private static func expect(_ condition: Bool, _ message: String) throws {
        guard condition else { throw NativeTextTestError.assertion(message) }
    }

    private static func expectThrows(_ data: Data, _ message: String) throws {
        do {
            _ = try NativeTextEvent.parseJSONL(data)
            throw NativeTextTestError.assertion(message)
        } catch is NativeTextEvent.ParseError { return }
        catch is NativeKeyAcknowledgement.ParseError { return }
    }

    private static func write(_ value: [String: Any], to path: URL) throws {
        let data = try JSONSerialization.data(withJSONObject: value, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: path, options: .atomic)
    }
}
