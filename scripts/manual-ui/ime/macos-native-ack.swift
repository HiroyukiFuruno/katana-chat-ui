import Foundation
import CoreFoundation

struct NativeKeyAcknowledgement: Equatable {
    let keyCode: UInt16
    let eventTimestamp: Double
    let handlerEndUptime: Double
    let nativeHasMarked: Bool

    enum ParseError: Error, CustomStringConvertible {
        case malformedJSON(line: Int)
        case missingField(line: Int, field: String)
        case invalidField(line: Int, field: String)

        var description: String {
            switch self {
            case let .malformedJSON(line): "malformed JSON on complete line \(line)"
            case let .missingField(line, field): "missing field \(field) on line \(line)"
            case let .invalidField(line, field): "invalid field \(field) on line \(line)"
            }
        }
    }

    static func parseJSONL(_ data: Data) throws -> [Self] {
        var acknowledgements: [Self] = []
        for (index, dictionary) in try parseCompleteRecords(data).enumerated() {
            if let acknowledgement = try parse(dictionary, line: index + 1) {
                acknowledgements.append(acknowledgement)
            }
        }
        return acknowledgements
    }

    static func parseCompleteRecords(_ data: Data) throws -> [[String: Any]] {
        let lines = data.split(separator: 0x0a, omittingEmptySubsequences: false)
        let completeCount = data.last == 0x0a ? lines.count : max(lines.count - 1, 0)
        var records: [[String: Any]] = []
        for (index, rawLine) in lines.prefix(completeCount).enumerated() {
            let line = rawLine.last == 0x0d ? rawLine.dropLast() : rawLine[...]
            guard !line.isEmpty else { continue }
            guard let object = try? JSONSerialization.jsonObject(with: Data(line)),
                  let dictionary = object as? [String: Any] else {
                throw ParseError.malformedJSON(line: index + 1)
            }
            records.append(dictionary)
        }
        return records
    }

    static func parseAndSelect(
        _ data: Data,
        expectedKeyCode: UInt16,
        pressedAtUptime: Double,
        releasedAtUptime: Double
    ) throws -> Self? {
        try selectLatest(
            parseJSONL(data),
            expectedKeyCode: expectedKeyCode,
            pressedAtUptime: pressedAtUptime,
            releasedAtUptime: releasedAtUptime
        )
    }

    static func selectLatest(
        _ acknowledgements: [Self],
        expectedKeyCode: UInt16,
        pressedAtUptime: Double,
        releasedAtUptime: Double
    ) -> Self? {
        acknowledgements
            .filter {
                $0.keyCode == expectedKeyCode
                    && $0.eventTimestamp >= pressedAtUptime
                    && $0.handlerEndUptime >= releasedAtUptime
            }
            .max {
                ($0.eventTimestamp, $0.handlerEndUptime) < ($1.eventTimestamp, $1.handlerEndUptime)
            }
    }

    private static func parse(_ dictionary: [String: Any], line: Int) throws -> Self? {
        guard let kind = string(dictionary, key: "kind"), kind == "key-processed" else {
            return nil
        }
        guard let eventType = string(dictionary, key: "event_type") else {
            throw ParseError.missingField(line: line, field: "event_type")
        }
        guard eventType == "keyUp" else { return nil }
        let keyCode = try unsignedInteger(dictionary, key: "key_code", line: line)
        let eventTimestamp = try finiteDouble(dictionary, key: "event_timestamp", line: line)
        let handlerEndUptime = try finiteDouble(dictionary, key: "handler_end_uptime", line: line)
        let clientMatchesWindow = try boolean(dictionary, key: "client_matches_window", line: line)
        guard clientMatchesWindow else { return nil }
        let nativeHasMarked = try boolean(dictionary, key: "native_has_marked", line: line)
        return Self(
            keyCode: keyCode,
            eventTimestamp: eventTimestamp,
            handlerEndUptime: handlerEndUptime,
            nativeHasMarked: nativeHasMarked
        )
    }

    private static func string(_ dictionary: [String: Any], key: String) -> String? {
        dictionary[key] as? String
    }

    private static func required(_ dictionary: [String: Any], key: String, line: Int) throws -> Any {
        guard let value = dictionary[key] else {
            throw ParseError.missingField(line: line, field: key)
        }
        return value
    }

    private static func boolean(_ dictionary: [String: Any], key: String, line: Int) throws -> Bool {
        let value = try required(dictionary, key: key, line: line)
        guard let number = value as? NSNumber,
              CFGetTypeID(number) == CFBooleanGetTypeID() else {
            throw ParseError.invalidField(line: line, field: key)
        }
        return number.boolValue
    }

    private static func finiteDouble(_ dictionary: [String: Any], key: String, line: Int) throws -> Double {
        guard let value = try required(dictionary, key: key, line: line) as? NSNumber else {
            throw ParseError.invalidField(line: line, field: key)
        }
        guard CFGetTypeID(value) != CFBooleanGetTypeID() else {
            throw ParseError.invalidField(line: line, field: key)
        }
        let result = value.doubleValue
        guard result.isFinite else {
            throw ParseError.invalidField(line: line, field: key)
        }
        return result
    }

    private static func unsignedInteger(_ dictionary: [String: Any], key: String, line: Int) throws -> UInt16 {
        guard let value = try required(dictionary, key: key, line: line) as? NSNumber else {
            throw ParseError.invalidField(line: line, field: key)
        }
        guard CFGetTypeID(value) != CFBooleanGetTypeID() else {
            throw ParseError.invalidField(line: line, field: key)
        }
        let result = value.doubleValue
        guard result.isFinite, result >= 0, result.rounded() == result, result <= Double(UInt16.max) else {
            throw ParseError.invalidField(line: line, field: key)
        }
        return UInt16(result)
    }
}
