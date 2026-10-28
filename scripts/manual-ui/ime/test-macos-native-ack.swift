import Foundation

enum NativeKeyAcknowledgementTestError: Error, CustomStringConvertible {
    case assertion(String)

    var description: String {
        switch self {
        case let .assertion(message): message
        }
    }
}

@main
struct NativeKeyAcknowledgementTests {
    static func main() {
        let output = CommandLine.arguments.dropFirst().first(where: { $0.hasPrefix("--result=") })
            .map { URL(fileURLWithPath: String($0.dropFirst("--result=".count))) }
            ?? URL(fileURLWithPath: "/tmp/kcu-native-key-ack-test-result.json")
        do {
            try runAll()
            try write(["status": "passed", "exit": 0], to: output)
            print("native key acknowledgement tests passed: \(output.path)")
            exit(0)
        } catch {
            try? write(["status": "failed", "exit": 1, "error": String(describing: error)], to: output)
            fputs("native key acknowledgement tests failed: \(error)\n", stderr)
            exit(1)
        }
    }

    private static func runAll() throws {
        let markedTrue = try line([
            "kind": "key-processed", "event_type": "keyUp", "key_code": 36,
            "event_timestamp": 12.0, "handler_end_uptime": 13.0,
            "native_has_marked": true, "client_matches_window": true,
        ])
        let markedFalse = try line([
            "kind": "key-processed", "event_type": "keyUp", "key_code": 36,
            "event_timestamp": 14.0, "handler_end_uptime": 15.0,
            "native_has_marked": false, "client_matches_window": true,
        ])
        let oldAck = try line([
            "kind": "key-processed", "event_type": "keyUp", "key_code": 36,
            "event_timestamp": 8.0, "handler_end_uptime": 9.0,
            "native_has_marked": false, "client_matches_window": true,
        ])
        let data = markedTrue + markedFalse + oldAck
        let latest = try require(
            NativeKeyAcknowledgement.parseAndSelect(
                data,
                expectedKeyCode: 36,
                pressedAtUptime: 10.0,
                releasedAtUptime: 11.0
            ),
            "latest matching acknowledgement is missing"
        )
        try expect(latest.nativeHasMarked == false, "latest acknowledgement must preserve marked=false")
        try expect(
            NativeKeyAcknowledgement.selectLatest(
                try NativeKeyAcknowledgement.parseJSONL(markedTrue),
                expectedKeyCode: 36,
                pressedAtUptime: 10.0,
                releasedAtUptime: 11.0
            )?.nativeHasMarked == true,
            "marked=true acknowledgement must be preserved"
        )
        try expect(
            NativeKeyAcknowledgement.selectLatest(
                try NativeKeyAcknowledgement.parseJSONL(oldAck),
                expectedKeyCode: 36,
                pressedAtUptime: 10.0,
                releasedAtUptime: 11.0
            ) == nil,
            "old acknowledgement must not match"
        )
        try expect(
            try NativeKeyAcknowledgement.parseAndSelect(
                try line(["kind": "key-processed", "event_type": "keyDown", "key_code": 36,
                          "event_timestamp": 12.0, "handler_end_uptime": 13.0,
                          "native_has_marked": false, "client_matches_window": true]),
                expectedKeyCode: 36, pressedAtUptime: 10.0, releasedAtUptime: 11.0
            ) == nil,
            "keyDown must not match"
        )
        try expect(
            try NativeKeyAcknowledgement.parseAndSelect(
                try line(["kind": "key-processed", "event_type": "keyUp", "key_code": 35,
                          "event_timestamp": 12.0, "handler_end_uptime": 13.0,
                          "native_has_marked": false, "client_matches_window": true]),
                expectedKeyCode: 36, pressedAtUptime: 10.0, releasedAtUptime: 11.0
            ) == nil,
            "different key must not match"
        )
        try expectThrows(
            try line(["kind": "key-processed", "event_type": "keyUp", "key_code": 36,
                      "event_timestamp": 12.0, "handler_end_uptime": 13.0,
                      "client_matches_window": true]),
            "missing native_has_marked must fail closed"
        )
        try expectThrows(
            try line(["kind": "key-processed", "event_type": "keyUp", "key_code": 36,
                      "event_timestamp": 12.0, "handler_end_uptime": 13.0,
                      "native_has_marked": false, "client_matches_window": 1]),
            "numeric client mismatch must fail closed"
        )
        let unrelatedClient = try line([
            "kind": "key-processed", "event_type": "keyUp", "key_code": 36,
            "event_timestamp": 12.0, "handler_end_uptime": 13.0,
            "native_has_marked": NSNull(), "client_matches_window": false,
        ])
        try expect(
            try NativeKeyAcknowledgement.parseAndSelect(
                unrelatedClient + markedFalse,
                expectedKeyCode: 36,
                pressedAtUptime: 10.0,
                releasedAtUptime: 11.0
            )?.nativeHasMarked == false,
            "false client with null marked state must be ignored before valid ACK"
        )
        try expectThrows(
            try line([
                "kind": "key-processed", "event_type": "keyUp", "key_code": 36,
                "event_timestamp": 12.0, "handler_end_uptime": 13.0,
                "native_has_marked": NSNull(), "client_matches_window": true,
            ]),
            "true client with null marked state must fail closed"
        )
        try expectThrows(
            try line(["kind": "key-processed", "event_type": "keyUp", "key_code": "36",
                      "event_timestamp": 12.0, "handler_end_uptime": 13.0,
                      "native_has_marked": false, "client_matches_window": true]),
            "unknown key_code field type must fail closed"
        )
        try expectThrows(
            try line(["kind": "key-processed", "event_type": "keyUp", "key_code": 36,
                      "event_timestamp": 12.0, "handler_end_uptime": 13.0,
                      "native_has_marked": 0, "client_matches_window": 1]),
            "numeric boolean fields must fail closed"
        )
        try expectThrows(
            try line(["kind": "key-processed", "event_type": "keyUp", "key_code": 36,
                      "event_timestamp": true, "handler_end_uptime": 13.0,
                      "native_has_marked": false, "client_matches_window": true]),
            "boolean timestamp must fail closed"
        )
        try expectThrows(
            try line(["kind": "key-processed", "event_type": "keyUp", "key_code": true,
                      "event_timestamp": 12.0, "handler_end_uptime": 13.0,
                      "native_has_marked": false, "client_matches_window": true]),
            "boolean key code must fail closed"
        )
        try expectThrows(
            Data(#"{"kind":"key-processed","event_type":"keyUp","key_code":18446744073709551616,"event_timestamp":12,"handler_end_uptime":13,"native_has_marked":false,"client_matches_window":true}"#.utf8) + Data([0x0a]),
            "out-of-range key code must fail closed"
        )
        try expectThrows(
            Data(#"{"kind":"key-processed","event_type":"keyUp"}"#.utf8) + Data([0x0a]),
            "malformed complete JSON must throw"
        )
        let incomplete = markedFalse + Data(#"{"kind":"key-processed"}"#.utf8)
        try expect(
            try NativeKeyAcknowledgement.parseJSONL(incomplete).count == 1,
            "incomplete final line must be ignored"
        )
        try expect(
            try NativeKeyAcknowledgement.parseCompleteRecords(incomplete).count == 1,
            "complete record parser must ignore incomplete final line"
        )
        try expect(
            try NativeKeyAcknowledgement.parseAndSelect(
                Data(), expectedKeyCode: 36, pressedAtUptime: 10.0, releasedAtUptime: 11.0
            ) == nil,
            "empty trace must have no acknowledgement"
        )
    }

    private static func line(_ dictionary: [String: Any]) throws -> Data {
        try JSONSerialization.data(withJSONObject: dictionary) + Data([0x0a])
    }

    private static func require<T>(_ value: T?, _ message: String) throws -> T {
        guard let value else { throw NativeKeyAcknowledgementTestError.assertion(message) }
        return value
    }

    private static func expect(_ condition: Bool, _ message: String) throws {
        guard condition else { throw NativeKeyAcknowledgementTestError.assertion(message) }
    }

    private static func expectThrows(_ data: Data, _ message: String) throws {
        do {
            _ = try NativeKeyAcknowledgement.parseJSONL(data)
            throw NativeKeyAcknowledgementTestError.assertion(message)
        } catch is NativeKeyAcknowledgement.ParseError {
            return
        }
    }

    private static func write(_ value: [String: Any], to path: URL) throws {
        let data = try JSONSerialization.data(withJSONObject: value, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: path, options: .atomic)
    }
}
