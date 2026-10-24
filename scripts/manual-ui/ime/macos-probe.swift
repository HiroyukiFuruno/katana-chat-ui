import AppKit
import ApplicationServices
import Carbon
import CoreGraphics
import CoreFoundation
import CryptoKit
import Foundation
import Vision

enum ProbeError: Error, CustomStringConvertible {
    case usage(String)
    case blocked(String)
    case failed(String)

    var description: String {
        switch self {
        case let .usage(message), let .blocked(message), let .failed(message): message
        }
    }
}

struct Arguments {
    let binary: URL
    let artifacts: URL
    let host: String
    let keyHoldMilliseconds: Int

    init() throws {
        let values = Array(CommandLine.arguments.dropFirst())
        guard [6, 8].contains(values.count),
              values[0] == "--binary",
              values[2] == "--artifacts",
              values[4] == "--host",
              ["egui", "floem", "gpui", "appkit", "egui-reference"].contains(values[5]) else {
            throw ProbeError.usage("usage: macos-probe --binary ABS --artifacts ABS --host HOST [--key-hold-ms {0|50}]")
        }
        if values.count == 8 {
            guard values[6] == "--key-hold-ms", ["0", "50"].contains(values[7]) else {
                throw ProbeError.usage("--key-hold-ms must be 0 or 50")
            }
        }
        keyHoldMilliseconds = values.count == 8 && values[7] == "50" ? 50 : 0
        guard values[1].hasPrefix("/"), values[3].hasPrefix("/") else {
            throw ProbeError.usage("--binary and --artifacts must be absolute paths")
        }
        binary = URL(fileURLWithPath: values[1]).standardizedFileURL
        artifacts = URL(fileURLWithPath: values[3]).standardizedFileURL
        host = values[5]
        guard FileManager.default.isExecutableFile(atPath: binary.path) else {
            throw ProbeError.failed("binary is not executable: \(binary.path)")
        }
        guard !FileManager.default.fileExists(atPath: artifacts.path) else {
            throw ProbeError.failed("artifacts directory must not already exist: \(artifacts.path)")
        }
        let expectedBinary = host == "egui-reference" ? "ime_reference" : "manual-host-\(host)"
        guard binary.lastPathComponent == expectedBinary else {
            throw ProbeError.failed("binary name must be \(expectedBinary)")
        }
    }
}

final class Probe {
    private let args: Arguments
    private var result: [String: Any] = ["status": "running"]
    private var child: Process?
    private var originalInputSource: TISInputSource?
    private var selectedInputSourceID = ""
    private var composerPoint = CGPoint.zero
    private var composerCapture = ""
    private var inputRowCenter: CGFloat = 0
    private var inputRowHeight: CGFloat = 0
    private var postedKeys: [[String: Any]] = []
    private var nativeTrace: URL?
    private var nativePrepareRequest: URL?
    private var confirmationMode = "one"

    init(args: Arguments) { self.args = args }

    func run() throws {
        guard ProcessInfo.processInfo.environment["GITHUB_ACTIONS"] == "true",
              ProcessInfo.processInfo.environment["RUNNER_OS"] == "macOS" else {
            throw ProbeError.blocked("must run only on a GitHub Actions macOS runner")
        }
        confirmationMode = ProcessInfo.processInfo.environment["KCU_IME_CONFIRMATION"] ?? "one"
        guard ["one", "observed"].contains(confirmationMode) else {
            throw ProbeError.usage("KCU_IME_CONFIRMATION must be one or observed")
        }
        try FileManager.default.createDirectory(at: args.artifacts, withIntermediateDirectories: false)
        let state = args.artifacts.appendingPathComponent("state", isDirectory: true)
        try FileManager.default.createDirectory(at: state, withIntermediateDirectories: false)
        let trace = args.artifacts.appendingPathComponent("\(args.host)-host.jsonl")
        result["host"] = args.host
        result["confirmation_mode"] = confirmationMode
        result["key_hold_milliseconds"] = args.keyHoldMilliseconds
        result["binary"] = args.binary.path
        result["artifacts"] = args.artifacts.path
        result["trace"] = trace.path
        result["started_at"] = ISO8601DateFormatter().string(from: Date())

        do {
            try permissionPreflight()
            try launchHost(state: state, trace: trace)
            try focusOwnWindow()
            try clickComposer()
            try captureInitialComposer()
            let baseline = try completeCommands(at: trace)
            try selectJapanese()
            try assertOwnHostFrontmost()
            try captureInitialComposer()
            try prepareNativeInputContext()
            let activation = ProcessInfo.processInfo.environment["KCU_IME_ACTIVATION_KEY"] ?? "none"
            guard ["none", "kana"].contains(activation) else {
                throw ProbeError.usage("KCU_IME_ACTIVATION_KEY must be none or kana")
            }
            result["native_input_activation"] = activation
            if activation == "kana" {
                try sendKey(CGKeyCode(kVK_JIS_Kana))
            }
            try sendTextKeys([45, 34, 4, 31, 45, 5, 31])
            let preedit = try waitForPhase(name: "preedit", trace: trace, baseline: baseline,
                                            requiredOCR: ["にほんご", "日本語"],
                                            requireCommit: false)
            try requireNativeMarked(true, keyCode: 31, phase: "preedit")
            try sendKey(49)
            let conversion = try waitForPhase(name: "conversion", trace: trace, baseline: baseline,
                                               requiredOCR: ["日本語"], requireCommit: false)
            try requireNativeMarked(true, keyCode: 49, phase: "conversion")
            let beforeFirstReturn = try completeCommands(at: trace)
            result["commands_before_first_return"] = Array(beforeFirstReturn.dropFirst(baseline.count))
            guard beforeFirstReturn == baseline else {
                throw ProbeError.failed("host command arrived before the first confirmation Return")
            }
            try sendKey(36)
            result["confirmation_return_count"] = 1
            if confirmationMode == "observed" {
                guard let firstAcknowledgement = try waitForNativeKey(keyCode: 36, phase: "first-return") else {
                    throw ProbeError.blocked("observed confirmation requires native key-processing acknowledgement")
                }
                let firstReturn = try waitForPhase(name: "first-return", trace: trace, baseline: beforeFirstReturn,
                                                  requiredOCR: ["日本語"], requireCommit: false, permitObservedCommit: true)
                result["first_return_phase"] = firstReturn.json
                let beforeAdditionalReturn = Array(try completeCommands(at: trace).dropFirst(beforeFirstReturn.count))
                result["commands_before_additional_return"] = beforeAdditionalReturn
                guard beforeAdditionalReturn.isEmpty || beforeAdditionalReturn == ["DraftChanged(\"日本語\")"] else {
                    throw ProbeError.failed("unexpected command arrived before the additional Return")
                }
                if firstAcknowledgement.nativeHasMarked {
                    guard beforeAdditionalReturn.isEmpty else {
                        throw ProbeError.failed("draft committed while the native input client remained marked")
                    }
                    try sendKey(36)
                    result["confirmation_return_count"] = 2
                    try requireNativeMarked(false, keyCode: 36, phase: "final-return")
                }
            } else if ProcessInfo.processInfo.environment["KCU_REFERENCE_EXTRA_RETURN"] == "1" {
                guard args.host == "appkit" else {
                    throw ProbeError.usage("extra Return diagnosis is limited to the standard AppKit reference")
                }
                let remainedMarked = try referenceRemainsMarkedAfterReturn(trace: trace)
                result["reference_first_return_remained_marked"] = remainedMarked
                if remainedMarked {
                    try sendKey(36)
                    result["confirmation_return_count"] = 2
                }
                result["reference_extra_return_sent"] = remainedMarked
            }
            let committed = try waitForPhase(name: "committed", trace: trace, baseline: baseline,
                                              requiredOCR: ["日本語"], requireCommit: true)
            guard committed.commands.filter({ $0.contains("DraftChanged") }) == ["DraftChanged(\"日本語\")"],
                  !committed.commands.contains(where: { $0.contains("Submit") }) else {
                throw ProbeError.failed("commit trace is not exactly one DraftChanged(日本語) with no Submit")
            }
            let committedBaseline = try completeCommands(at: trace)
            try sendTextKeys([17, 14, 1, 32, 17, 31])
            let cancelPreedit = try waitForPhase(name: "cancel-preedit", trace: trace, baseline: committedBaseline,
                                                  requiredOCR: ["日本語てすと", "日本語テスト"], requireCommit: false)
            try requireNativeMarked(true, keyCode: 31, phase: "cancel-preedit")
            try sendKey(53)
            try sendKey(53)
            result["cancel_key_sequence"] = ["Escape", "Escape"]
            try captureDiagnosticImage(name: "after-cancel-keys")
            try requireNativeMarked(false, keyCode: 53, phase: "cancelled")
            let cancelled = try waitForPhase(name: "cancelled", trace: trace, baseline: committedBaseline,
                                              requiredOCR: ["日本語"], requireCommit: false)
            guard cancelled.commands.isEmpty,
                  cancelPreedit.commands.filter({ $0.contains("DraftChanged") }).isEmpty else {
                throw ProbeError.failed("Escape changed the committed draft or emitted DraftChanged")
            }
            result["status"] = "passed"
            result["phases"] = ["preedit": preedit.json, "conversion": conversion.json,
                                 "committed": committed.json, "cancel_preedit": cancelPreedit.json,
                                 "cancelled": cancelled.json]
        } catch {
            result["status"] = (error as? ProbeError).map { if case .blocked = $0 { return "blocked" }; return "failed" } ?? "failed"
            result["error"] = String(describing: error)
            if child?.isRunning == true {
                do { try captureDiagnosticImage(name: "failure") }
                catch { result["failure_capture_error"] = String(describing: error) }
            }
            throw error
        }
    }

    func finish() {
        if let child, child.isRunning { child.terminate() }
        let deadline = Date().addingTimeInterval(5)
        while child?.isRunning == true && Date() < deadline {
            RunLoop.current.run(until: Date().addingTimeInterval(0.1))
        }
        if let child, child.isRunning { kill(child.processIdentifier, SIGKILL); child.waitUntilExit() }
        if let source = originalInputSource { _ = TISSelectInputSource(source) }
        result["finished_at"] = ISO8601DateFormatter().string(from: Date())
        let output = args.artifacts.appendingPathComponent("\(args.host)-result.json")
        if let data = try? JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys]) {
            try? data.write(to: output, options: .atomic)
        }
    }

    private func permissionPreflight() throws {
        try runnerPreflight()
    }

    private func configureNativeObserver(_ process: Process) throws {
        guard let path = ProcessInfo.processInfo.environment["KCU_NATIVE_IME_OBSERVER_DYLIB"] else {
            guard confirmationMode != "observed" else {
                throw ProbeError.blocked("observed confirmation requires the native observer library")
            }
            return
        }
        guard path.hasPrefix("/"), FileManager.default.isReadableFile(atPath: path) else {
            throw ProbeError.usage("native observer library must be a readable absolute path")
        }
        let events = args.artifacts.appendingPathComponent("\(args.host)-native-ime.jsonl")
        let request = args.artifacts.appendingPathComponent("native-ime-prepare.request")
        nativeTrace = events
        nativePrepareRequest = request
        process.environment?["DYLD_INSERT_LIBRARIES"] = path
        process.environment?["KCU_NATIVE_IME_EXPECTED_BINARY"] = args.binary.lastPathComponent
        process.environment?["KCU_NATIVE_IME_TRACE_PATH"] = events.path
        process.environment?["KCU_NATIVE_IME_PREPARE_PATH"] = request.path
        for key in ["GITHUB_ACTIONS", "RUNNER_OS"] {
            process.environment?[key] = ProcessInfo.processInfo.environment[key]
        }
        result["native_observer"] = ["library": path, "library_sha256": sha256(URL(fileURLWithPath: path)),
                                     "trace": events.path, "prepare_request": request.path]
    }

    private func nativeRecords() throws -> [[String: Any]] {
        guard let nativeTrace, FileManager.default.fileExists(atPath: nativeTrace.path) else { return [] }
        return try NativeKeyAcknowledgement.parseCompleteRecords(Data(contentsOf: nativeTrace))
    }

    private func exactNativeBool(_ value: Any?) -> Bool? {
        guard let number = value as? NSNumber, CFGetTypeID(number) == CFBooleanGetTypeID() else { return nil }
        return number.boolValue
    }

    private func prepareNativeInputContext() throws {
        guard let request = nativePrepareRequest else { return }
        try Data().write(to: request, options: .withoutOverwriting)
        let deadline = Date().addingTimeInterval(30)
        while Date() < deadline {
            let records = try nativeRecords()
            if let error = records.last(where: { $0["kind"] as? String == "mode-error" }) {
                result["native_mode_error"] = error
                throw ProbeError.blocked("native client input-context preparation failed")
            }
            if let ready = records.last(where: { $0["kind"] as? String == "mode-ready" }) {
                guard ready["selected_input_source"] as? String == "com.apple.inputmethod.Kotoeri.RomajiTyping.Japanese",
                      exactNativeBool(ready["selected_matches"]) == true,
                      exactNativeBool(ready["current_client_matches"]) == true,
                      exactNativeBool(ready["client_matches_window"]) == true,
                      exactNativeBool(ready["app_active"]) == true,
                      exactNativeBool(ready["window_key"]) == true,
                      exactNativeBool(ready["window_visible"]) == true,
                      exactNativeBool(ready["actual_has_marked"]) == false else {
                    throw ProbeError.failed("native client did not acknowledge a fresh Japanese input context")
                }
                result["native_mode_ready"] = ready
                try assertOwnHostFrontmost()
                return
            }
            guard child?.isRunning == true else { throw ProbeError.failed("host exited before native input-context readiness") }
            RunLoop.current.run(until: Date().addingTimeInterval(0.1))
        }
        throw ProbeError.blocked("native input-context readiness was not acknowledged")
    }

    private func waitForNativeKey(keyCode: UInt16, phase: String) throws -> NativeKeyAcknowledgement? {
        guard let nativeTrace else { return nil }
        guard let posted = postedKeys.last, posted["key_code"] as? Int == Int(keyCode),
              let pressed = posted["pressed_at_uptime"] as? Double,
              let released = posted["released_at_uptime"] as? Double else {
            throw ProbeError.failed("native key acknowledgement has no matching posted key")
        }
        let deadline = Date().addingTimeInterval(30)
        while Date() < deadline {
            let data = try Data(contentsOf: nativeTrace)
            if let acknowledgement = try NativeKeyAcknowledgement.parseAndSelect(
                data, expectedKeyCode: keyCode, pressedAtUptime: pressed, releasedAtUptime: released
            ) {
                result["native_ack_\(phase)"] = ["key_code": Int(acknowledgement.keyCode),
                                                 "event_timestamp": acknowledgement.eventTimestamp,
                                                 "handler_end_uptime": acknowledgement.handlerEndUptime,
                                                 "native_has_marked": acknowledgement.nativeHasMarked]
                return acknowledgement
            }
            guard child?.isRunning == true else { throw ProbeError.failed("host exited before native key acknowledgement") }
            RunLoop.current.run(until: Date().addingTimeInterval(0.1))
        }
        throw ProbeError.failed("native keyUp processing was not acknowledged for phase \(phase)")
    }

    private func requireNativeMarked(_ expected: Bool, keyCode: UInt16, phase: String) throws {
        if let acknowledgement = try waitForNativeKey(keyCode: keyCode, phase: phase),
           acknowledgement.nativeHasMarked != expected {
            throw ProbeError.failed("native marked state is incorrect for phase \(phase)")
        }
    }

    private func launchHost(state: URL, trace: URL) throws {
        let process = Process()
        process.executableURL = args.binary
        process.currentDirectoryURL = state
        process.environment = ["PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "KCU_TRACE_PATH": trace.path]
        // OSの入力方式はrunner本来のprofileを使い、アプリ固有状態はcurrentDirectoryへ分離する。
        for key in ["HOME", "TMPDIR", "LANG", "LC_ALL", "__CF_USER_TEXT_ENCODING"] {
            process.environment?[key] = ProcessInfo.processInfo.environment[key]
        }
        result["application_state_root"] = state.path
        if ["appkit", "egui-reference"].contains(args.host) {
            // 親で確認済みの公開CI識別子だけを比較器へ引き継ぎ、認証環境は渡さない。
            for key in ["GITHUB_ACTIONS", "RUNNER_OS"] {
                process.environment?[key] = ProcessInfo.processInfo.environment[key]
            }
        }
        if args.host == "appkit", let source = ProcessInfo.processInfo.environment["KCU_REFERENCE_CONTEXT_SOURCE"] {
            guard source == "com.apple.inputmethod.Kotoeri.RomajiTyping.Japanese" else {
                throw ProbeError.usage("unsupported reference context source")
            }
            process.environment?["KCU_REFERENCE_CONTEXT_SOURCE"] = source
        }
        try configureNativeObserver(process)
        guard let childEnvironment = process.environment else {
            throw ProbeError.failed("isolated native environment is unavailable")
        }
        result["native_environment_keys"] = childEnvironment.keys.sorted()
        let log = FileHandle.standardError
        process.standardOutput = log
        process.standardError = log
        try process.run()
        child = process
    }

    private func focusOwnWindow() throws {
        let (_, _, origin, extent) = try waitForOwnWindow()
        composerPoint = CGPoint(x: origin.x + extent.width / 2, y: origin.y + extent.height - 120)
        // 小さいrunner画面では入力欄が中央半分より広くなるため、可視window幅を採取する。
        let captureLeft = max(0, origin.x)
        let captureRight = min(origin.x + extent.width, NSScreen.main?.frame.maxX ?? origin.x + extent.width)
        guard captureRight > captureLeft else { throw ProbeError.failed("host window is outside the visible display") }
        composerCapture = "\(Int(captureLeft)),\(Int(origin.y + extent.height - 180)),\(Int(captureRight - captureLeft)),140"
        result["window"] = ["position": [origin.x, origin.y], "size": [extent.width, extent.height],
                             "composer_point": [composerPoint.x, composerPoint.y]]
        result["composer_capture_rect"] = composerCapture
    }

    private func selectJapanese() throws {
        try assertOwnHostFrontmost(requireInputSource: false)
        result["input_activation_order"] = "focus and capture empty composer, select Japanese, verify focused empty composer again"
        originalInputSource = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue()
        let list = TISCreateInputSourceList(nil, true)?.takeRetainedValue() as NSArray? ?? []
        var sources: [String: TISInputSource] = [:]
        var availableIDs: [String] = []
        for case let source as TISInputSource in list {
            if let id = inputSourceID(source) {
                availableIDs.append(id)
                sources[id] = source
            }
        }
        result["input_source_ids"] = availableIDs
        let preferred = ["com.apple.inputmethod.Kotoeri.RomajiTyping.Japanese", "com.apple.inputmethod.Kotoeri.Japanese"]
        guard let sourceID = preferred.first(where: { sources[$0] != nil }),
              let source = sources[sourceID] else { throw ProbeError.blocked("Kotoeri Japanese input source is unavailable") }
        let parentID = String(sourceID.dropLast(".Japanese".count))
        guard let parent = sources[parentID] else { throw ProbeError.blocked("Kotoeri parent input method is unavailable") }
        // Carbonの仕様上、入力modeを選択する前に親の入力方式を有効にする必要がある。
        let parentStatus = TISEnableInputSource(parent)
        result["parent_input_method"] = ["id": parentID, "enable_status": parentStatus]
        guard parentStatus == noErr else { throw ProbeError.blocked("parent TISEnableInputSource failed: \(parentStatus)") }
        let enableStatus = TISEnableInputSource(source)
        result["mode_enable_status"] = enableStatus
        guard enableStatus == noErr else { throw ProbeError.blocked("mode TISEnableInputSource failed: \(enableStatus)") }
        if ProcessInfo.processInfo.environment["KCU_IME_ACTIVATION_MENU"] == "1" {
            guard let child else { throw ProbeError.failed("input menu has no target host") }
            // 背景TIS選択を先行させず、本来の入力メニューの経路だけを比較する。
            try selectJapaneseFromInputMenu(expectedHostPID: child.processIdentifier, artifactDirectory: args.artifacts)
            let currentID = try waitForSelectedInputSource(expected: sourceID)
            selectedInputSourceID = currentID
            result["selected_input_source"] = currentID
            result["input_source_activation_method"] = "system-input-menu"
            try focusOwnWindow()
            return
        }
        let selectStatus = TISSelectInputSource(source)
        result["mode_select_status"] = selectStatus
        guard selectStatus == noErr else { throw ProbeError.blocked("TISSelectInputSource failed: \(selectStatus)") }
        let activationMethod = ProcessInfo.processInfo.environment["KCU_IME_ACTIVATION_METHOD"] ?? "tis"
        guard ["tis", "shortcut"].contains(activationMethod) else {
            throw ProbeError.usage("KCU_IME_ACTIVATION_METHOD must be tis or shortcut")
        }
        result["input_source_activation_method"] = activationMethod
        if activationMethod == "shortcut" {
            guard let romanSource = sources["com.apple.keylayout.US"] else {
                throw ProbeError.blocked("US input source is unavailable for shortcut comparison")
            }
            let romanStatus = TISSelectInputSource(romanSource)
            result["shortcut_initial_roman_status"] = romanStatus
            guard romanStatus == noErr else { throw ProbeError.blocked("cannot select US before input shortcut") }
            _ = try waitForSelectedInputSource(expected: "com.apple.keylayout.US")
            try sendInputSourceShortcut()
        }
        let currentID = try waitForSelectedInputSource(expected: inputSourceID(source) ?? "")
        selectedInputSourceID = currentID
        result["selected_input_source"] = currentID
    }

    private func waitForOwnWindow() throws -> (AXUIElement, [AXUIElement], CGPoint, CGSize) {
        guard let child else { throw ProbeError.failed("host process was not launched") }
        let deadline = Date().addingTimeInterval(30)
        while Date() < deadline {
            guard child.isRunning else { throw ProbeError.failed("host exited before its window appeared") }
            guard let app = NSRunningApplication(processIdentifier: child.processIdentifier) else {
                RunLoop.current.run(until: Date().addingTimeInterval(0.1))
                continue
            }
            _ = app.activate(options: [])
            if NSWorkspace.shared.frontmostApplication?.processIdentifier == child.processIdentifier {
                let application = AXUIElementCreateApplication(child.processIdentifier)
                var windows: CFTypeRef?
                if AXUIElementCopyAttributeValue(application, kAXWindowsAttribute as CFString, &windows) == .success,
                   let windowArray = windows as? [AXUIElement], !windowArray.isEmpty {
                    var position: CFTypeRef?
                    var size: CFTypeRef?
                    if AXUIElementCopyAttributeValue(windowArray[0], kAXPositionAttribute as CFString, &position) == .success,
                       AXUIElementCopyAttributeValue(windowArray[0], kAXSizeAttribute as CFString, &size) == .success,
                       let position, let size {
                        let positionValue = position as! AXValue
                        let sizeValue = size as! AXValue
                        var origin = CGPoint.zero
                        var extent = CGSize.zero
                        if AXValueGetValue(positionValue, .cgPoint, &origin),
                           AXValueGetValue(sizeValue, .cgSize, &extent), extent.width > 0, extent.height > 120 {
                            return (application, windowArray, origin, extent)
                        }
                    }
                }
            }
            RunLoop.current.run(until: Date().addingTimeInterval(0.1))
        }
        throw ProbeError.failed("timed out waiting for own host frontmost AX window")
    }

    private func waitForSelectedInputSource(expected: String) throws -> String {
        let deadline = Date().addingTimeInterval(30)
        while Date() < deadline {
            if let current = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
               let currentID = inputSourceID(current), currentID == expected { return currentID }
            RunLoop.current.run(until: Date().addingTimeInterval(0.1))
        }
        throw ProbeError.blocked("selected input source did not become current")
    }

    private func assertOwnHostFrontmost(requireInputSource: Bool = true) throws {
        guard let child, child.isRunning,
              NSWorkspace.shared.frontmostApplication?.processIdentifier == child.processIdentifier else {
            throw ProbeError.failed("own host is not frontmost")
        }
        guard requireInputSource else { return }
        guard let current = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
              inputSourceID(current) == selectedInputSourceID else {
            throw ProbeError.failed("selected input source changed before input event")
        }
    }

    private func inputSourceID(_ source: TISInputSource) -> String? {
        guard let pointer = TISGetInputSourceProperty(source, kTISPropertyInputSourceID) else { return nil }
        return Unmanaged<CFString>.fromOpaque(pointer).takeUnretainedValue() as String
    }

    private func clickComposer() throws {
        try assertOwnHostFrontmost(requireInputSource: false)
        guard let event = CGEvent(mouseEventSource: nil, mouseType: .leftMouseDown,
                                  mouseCursorPosition: composerPoint, mouseButton: .left) else {
            throw ProbeError.failed("cannot create composer click")
        }
        event.post(tap: .cghidEventTap)
        guard let up = CGEvent(mouseEventSource: nil, mouseType: .leftMouseUp,
                               mouseCursorPosition: composerPoint, mouseButton: .left) else {
            throw ProbeError.failed("cannot create composer click release")
        }
        up.post(tap: .cghidEventTap)
        try assertOwnHostFrontmost(requireInputSource: false)
    }

    private func sendTextKeys(_ keys: [CGKeyCode]) throws { for key in keys { try sendKey(key) } }

    private func sendInputSourceShortcut() throws {
        try assertOwnHostFrontmost(requireInputSource: false)
        guard let modifierDown = CGEvent(keyboardEventSource: nil, virtualKey: CGKeyCode(kVK_Control), keyDown: true),
              let modifierUp = CGEvent(keyboardEventSource: nil, virtualKey: CGKeyCode(kVK_Control), keyDown: false) else {
            throw ProbeError.failed("cannot create input-source shortcut modifier")
        }
        modifierDown.type = .flagsChanged
        modifierDown.flags = .maskControl
        modifierUp.type = .flagsChanged
        modifierUp.flags = []
        modifierDown.post(tap: .cghidEventTap)
        defer { modifierUp.post(tap: .cghidEventTap) }
        try sendKey(49, flags: .maskControl, requireInputSource: false)
        result["input_source_shortcut"] = "Control down, Space down/up, Control up"
    }

    private func sendKey(_ key: CGKeyCode, flags: CGEventFlags = [], requireInputSource: Bool = true) throws {
        try assertOwnHostFrontmost(requireInputSource: requireInputSource)
        guard let down = CGEvent(keyboardEventSource: nil, virtualKey: key, keyDown: true) else {
            throw ProbeError.failed("cannot create key event \(key)")
        }
        down.flags = flags
        let pressedAt = ProcessInfo.processInfo.systemUptime
        down.timestamp = CGEventTimestamp(pressedAt * 1_000_000_000)
        down.post(tap: .cghidEventTap)
        // キーの押下時間をdown/up間で再現し、瞬時のtapとの差を同一runnerで比較する。
        if args.keyHoldMilliseconds > 0 {
            Thread.sleep(forTimeInterval: Double(args.keyHoldMilliseconds) / 1000)
        }
        guard let up = CGEvent(keyboardEventSource: nil, virtualKey: key, keyDown: false) else {
            throw ProbeError.failed("cannot create key release \(key)")
        }
        up.flags = flags
        // 子はpost呼出し中にも処理を終え得るため、ACKの下限は呼出し直前とする。
        let releasedAt = ProcessInfo.processInfo.systemUptime
        up.timestamp = CGEventTimestamp(releasedAt * 1_000_000_000)
        up.post(tap: .cghidEventTap)
        let releasePostReturnedAt = ProcessInfo.processInfo.systemUptime
        postedKeys.append(["key_code": Int(key), "uptime": ProcessInfo.processInfo.systemUptime,
                           "modifier_flags": flags.rawValue,
                           "cg_timestamp_seconds": Double(down.timestamp) / 1_000_000_000,
                           "release_timestamp_seconds": Double(up.timestamp) / 1_000_000_000,
                           "pressed_at_uptime": pressedAt,
                           "released_at_uptime": releasedAt,
                           "release_post_returned_at_uptime": releasePostReturnedAt,
                           "actual_hold_milliseconds": (releasedAt - pressedAt) * 1000,
                           "keyboard_type": down.getIntegerValueField(.keyboardEventKeyboardType)])
        result["posted_keys"] = postedKeys
        guard (releasedAt - pressedAt) * 1000 >= Double(args.keyHoldMilliseconds) else {
            throw ProbeError.failed("native key hold ended before its requested duration")
        }
    }

    private func captureDiagnosticImage(name: String) throws {
        let image = args.artifacts.appendingPathComponent("\(args.host)-\(name).png")
        guard try runTool("/usr/sbin/screencapture", ["-x", image.path]) == 0 else {
            throw ProbeError.failed("diagnostic screenshot failed for \(name)")
        }
        result["image_\(name)"] = ["path": image.path, "sha256": sha256(image)]
    }

    private struct Phase {
        let json: [String: Any]
        let commands: [String]
    }

    private func captureInitialComposer() throws {
        let image = args.artifacts.appendingPathComponent("\(args.host)-initial-composer.png")
        let deadline = Date().addingTimeInterval(30)
        while Date() < deadline {
            guard try runTool("/usr/sbin/screencapture", ["-x", "-R", composerCapture, image.path]) == 0 else {
                throw ProbeError.failed("initial composer capture failed")
            }
            let snapshot = try recognize(image: image)
            if let placeholder = snapshot.observations.first(where: { $0.normalized == "askanything" }) {
                inputRowCenter = placeholder.boundingBox.midY
                inputRowHeight = placeholder.boundingBox.height
                result["initial_composer"] = ["image": image.path,
                                               "ocr_all": snapshot.allJSON,
                                               "filtered_input_body": placeholder.normalized,
                                               "filtered_input_bounds": [placeholder.boundingBoxJSON]]
                return
            }
            RunLoop.current.run(until: Date().addingTimeInterval(0.1))
        }
        throw ProbeError.failed("Ask anything placeholder did not become visible in the composer row")
    }

    private func waitForPhase(name: String, trace: URL, baseline: [String], requiredOCR: [String], requireCommit: Bool,
                              permitObservedCommit: Bool = false) throws -> Phase {
        let image = args.artifacts.appendingPathComponent("\(args.host)-\(name).png")
        let composerImage = args.artifacts.appendingPathComponent("\(args.host)-\(name)-composer.png")
        let deadline = Date().addingTimeInterval(30)
        while Date() < deadline {
            guard child?.isRunning == true else { throw ProbeError.failed("host exited while waiting for phase \(name)") }
            let screenshotStatus = try runTool("/usr/sbin/screencapture", ["-x", image.path])
            guard screenshotStatus == 0, FileManager.default.fileExists(atPath: image.path) else {
                throw ProbeError.failed("screencapture failed for phase \(name)")
            }
            guard try runTool("/usr/sbin/screencapture", ["-x", "-R", composerCapture, composerImage.path]) == 0 else {
                throw ProbeError.failed("composer capture failed for phase \(name)")
            }
            let snapshot = try recognize(image: composerImage)
            let body = snapshot.filteredBody(rowCenter: inputRowCenter, baselineHeight: inputRowHeight)
            let commands = try completeCommands(at: trace).dropFirst(baseline.count)
            let commandList = Array(commands)
            result["last_observed_phase"] = [
                "name": name, "ocr_all": snapshot.allJSON, "filtered_input_body": body,
                "filtered_input_bounds": snapshot.filteredBoundsJSON(rowCenter: inputRowCenter, baselineHeight: inputRowHeight),
                "commands": commandList, "image": image.path, "composer_image": composerImage.path
            ]
            let hasOCR = snapshot.matchesExpected(requiredOCR, rowCenter: inputRowCenter, baselineHeight: inputRowHeight)
            let hasCommit = commandList.contains("DraftChanged(\"日本語\")")
            let commitCount = commandList.filter { $0 == "DraftChanged(\"日本語\")" }.count
            if permitObservedCommit && !commandList.isEmpty && commandList != ["DraftChanged(\"日本語\")"] {
                throw ProbeError.failed("confirmation observation emitted an unexpected command")
            }
            if !requireCommit && !permitObservedCommit && commandList.contains(where: { $0.contains("DraftChanged") || $0.contains("Submit") }) {
                throw ProbeError.failed("phase \(name) emitted commit or Submit before the expected commit")
            }
            if hasOCR && (!requireCommit || (hasCommit && commitCount == 1)) {
                return Phase(json: ["ocr_all": snapshot.allJSON,
                                    "filtered_input_body": body,
                                    "filtered_input_bounds": snapshot.filteredBoundsJSON(rowCenter: inputRowCenter, baselineHeight: inputRowHeight),
                                    "image": image.path,
                                    "composer_image": composerImage.path,
                                    "image_sha256": sha256(image), "commands": commandList], commands: commandList)
            }
            RunLoop.current.run(until: Date().addingTimeInterval(0.1))
        }
        throw ProbeError.failed("timed out waiting for phase \(name)")
    }

    private func completeCommands(at path: URL) throws -> [String] {
        guard let data = try? Data(contentsOf: path), let text = String(data: data, encoding: .utf8) else { return [] }
        let lines = text.split(separator: "\n", omittingEmptySubsequences: false)
        var commands: [String] = []
        for (index, line) in lines.enumerated() {
            if line.isEmpty { continue }
            if index == lines.count - 1 && !text.hasSuffix("\n") { break }
            guard let object = try JSONSerialization.jsonObject(with: Data(line.utf8)) as? [String: Any] else {
                throw ProbeError.failed("malformed complete host trace JSON")
            }
            guard let kind = object["kind"] as? String else { continue }
            if kind == "command" {
                guard let command = object["command"] as? String else { throw ProbeError.failed("command trace record has no command") }
                commands.append(command)
            }
        }
        return commands
    }

    private func referenceRemainsMarkedAfterReturn(trace: URL) throws -> Bool {
        let deadline = Date().addingTimeInterval(30)
        while Date() < deadline {
            let text = try String(contentsOf: trace, encoding: .utf8)
            let completeLines = text.split(separator: "\n", omittingEmptySubsequences: false).dropLast()
            for line in completeLines.reversed() where !line.isEmpty {
                guard let object = try JSONSerialization.jsonObject(with: Data(line.utf8)) as? [String: Any] else {
                    throw ProbeError.failed("malformed reference key trace")
                }
                if object["kind"] as? String == "key-event", object["phase"] as? String == "after",
                   object["keyCode"] as? Int == 36,
                   let state = object["state"] as? [String: Any], let marked = state["hasMarkedText"] as? Bool {
                    return marked
                }
            }
            guard child?.isRunning == true else { throw ProbeError.failed("reference exited before Return acknowledgement") }
            RunLoop.current.run(until: Date().addingTimeInterval(0.1))
        }
        throw ProbeError.failed("reference did not acknowledge the first Return")
    }

    private func recognize(image: URL) throws -> OCRSnapshot {
        let request = VNRecognizeTextRequest()
        request.recognitionLanguages = ["ja-JP", "en-US"]
        request.recognitionLevel = .accurate
        request.usesLanguageCorrection = false
        try VNImageRequestHandler(url: image, options: [:]).perform([request])
        let observations = request.results?.compactMap { observation -> OCRObservation? in
            guard let candidate = observation.topCandidates(1).first else { return nil }
            return OCRObservation(text: candidate.string,
                                  normalized: normalizeOCR(candidate.string),
                                  boundingBox: observation.boundingBox)
        } ?? []
        return OCRSnapshot(observations: observations)
    }


    private func runTool(_ path: String, _ arguments: [String]) throws -> Int32 {
        let process = Process(); process.executableURL = URL(fileURLWithPath: path); process.arguments = arguments
        try process.run(); process.waitUntilExit(); return process.terminationStatus
    }

    private func sha256(_ path: URL) -> String {
        guard let data = try? Data(contentsOf: path) else { return "" }
        return SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined()
    }
}

func runnerPreflight() throws {
    guard ProcessInfo.processInfo.environment["GITHUB_ACTIONS"] == "true",
          ProcessInfo.processInfo.environment["RUNNER_OS"] == "macOS" else {
        throw ProbeError.blocked("must run only on a GitHub Actions macOS runner")
    }
    guard CGPreflightPostEventAccess() else { throw ProbeError.blocked("PostEvent TCC permission unavailable") }
    guard AXIsProcessTrusted() else { throw ProbeError.blocked("Accessibility TCC permission unavailable") }
    guard CGPreflightScreenCaptureAccess() else { throw ProbeError.blocked("ScreenCapture TCC permission unavailable") }
}

@main
struct MacOSIMEProbeMain {
    static func main() {
        do {
            if CommandLine.arguments.count == 3, CommandLine.arguments[1] == "--preflight" {
                // ローカル実行では権限照会もファイル書込みも行わない。
                guard ProcessInfo.processInfo.environment["GITHUB_ACTIONS"] == "true",
                      ProcessInfo.processInfo.environment["RUNNER_OS"] == "macOS" else {
                    throw ProbeError.blocked("must run only on a GitHub Actions macOS runner")
                }
                let output = URL(fileURLWithPath: CommandLine.arguments[2])
                do {
                    try runnerPreflight()
                    try JSONSerialization.data(withJSONObject: ["status": "passed"]).write(to: output)
                } catch {
                    try JSONSerialization.data(withJSONObject: ["status": "blocked", "error": String(describing: error)]).write(to: output)
                    throw error
                }
                exit(0)
            }
            let probe = try Probe(args: Arguments())
            defer { probe.finish() }
            try probe.run()
        } catch let error as ProbeError {
            fputs("macOS IME probe \(error)\n", stderr)
            if case .blocked = error { exit(77) }
            exit(1)
        } catch {
            fputs("macOS IME probe \(error)\n", stderr)
            exit(1)
        }
    }
}
