import Foundation
import CoreFoundation

struct NativeTextEvent: Equatable {
    enum Kind: String, CaseIterable {
        case preedit = "native-preedit"
        case insert = "native-insert"
        case unmark = "native-unmark"
    }

    let kind: Kind
    let text: String
    let uptime: Double
    let nativeHasMarked: Bool
    let recordIndex: Int

    enum ParseError: Error, CustomStringConvertible {
        case missingField(line: Int, field: String)
        case invalidField(line: Int, field: String)
        case duplicate(kind: Kind, text: String)

        var description: String {
            switch self {
            case let .missingField(line, field): "missing field \(field) on line \(line)"
            case let .invalidField(line, field): "invalid field \(field) on line \(line)"
            case let .duplicate(kind, text): "duplicate fresh \(kind.rawValue) text \(text)"
            }
        }
    }

    static func parseJSONL(_ data: Data) throws -> [Self] {
        try NativeKeyAcknowledgement.parseCompleteRecords(data).enumerated().compactMap { index, record in
            try parse(record, line: index + 1, recordIndex: index)
        }
    }

    static func selectFresh(
        _ events: [Self],
        baselineEventCount: Int,
        baselineUptime: Double,
        kind: Kind,
        expectedTexts: Set<String>,
        expectedMarked: Bool
    ) throws -> Self? {
        guard baselineEventCount >= 0, baselineEventCount <= events.count,
              baselineUptime.isFinite else { return nil }
        guard let latest = latestFresh(events, baselineEventCount: baselineEventCount,
                                       baselineUptime: baselineUptime, kind: kind,
                                       expectedMarked: nil) else { return nil }
        guard latest.nativeHasMarked == expectedMarked,
              expectedTexts.contains(latest.text) else { return nil }
        return latest
    }

    static func latestFresh(
        _ events: [Self],
        baselineEventCount: Int,
        baselineUptime: Double,
        kind: Kind,
        expectedMarked: Bool?
    ) -> Self? {
        guard baselineEventCount >= 0, baselineEventCount <= events.count,
              baselineUptime.isFinite else { return nil }
        guard let latest = events.dropFirst(baselineEventCount).filter({
            $0.kind == kind && $0.uptime > baselineUptime
        }).max(by: { ($0.uptime, $0.recordIndex) < ($1.uptime, $1.recordIndex) }) else { return nil }
        guard expectedMarked == nil || latest.nativeHasMarked == expectedMarked else { return nil }
        return latest
    }

    private static func parse(_ record: [String: Any], line: Int, recordIndex: Int) throws -> Self? {
        guard let rawKind = record["kind"] as? String,
              let kind = Kind(rawValue: rawKind) else { return nil }
        guard let text = record["text"] as? String, text.utf16.count <= 128 else {
            if record["text"] == nil { throw ParseError.missingField(line: line, field: "text") }
            throw ParseError.invalidField(line: line, field: "text")
        }
        guard let uptimeValue = record["callback_uptime"] as? NSNumber,
              CFGetTypeID(uptimeValue) != CFBooleanGetTypeID(),
              uptimeValue.doubleValue.isFinite else {
            if record["callback_uptime"] == nil { throw ParseError.missingField(line: line, field: "callback_uptime") }
            throw ParseError.invalidField(line: line, field: "callback_uptime")
        }
        guard let markedValue = record["native_has_marked"] as? NSNumber,
              CFGetTypeID(markedValue) == CFBooleanGetTypeID() else {
            if record["native_has_marked"] == nil { throw ParseError.missingField(line: line, field: "native_has_marked") }
            throw ParseError.invalidField(line: line, field: "native_has_marked")
        }
        return Self(kind: kind, text: text, uptime: uptimeValue.doubleValue,
                    nativeHasMarked: markedValue.boolValue, recordIndex: recordIndex)
    }
}
