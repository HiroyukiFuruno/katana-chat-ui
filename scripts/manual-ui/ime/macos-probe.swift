import AppKit
import ApplicationServices
import Carbon
import CoreGraphics
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

    init(args: Arguments) { self.args = args }

    func run() throws {
        guard ProcessInfo.processInfo.environment["GITHUB_ACTIONS"] == "true",
              ProcessInfo.processInfo.environment["RUNNER_OS"] == "macOS" else {
            throw ProbeError.blocked("must run only on a GitHub Actions macOS runner")
        }
        try FileManager.default.createDirectory(at: args.artifacts, withIntermediateDirectories: false)
        let state = args.artifacts.appendingPathComponent("state", isDirectory: true)
        try FileManager.default.createDirectory(at: state, withIntermediateDirectories: false)
        let trace = args.artifacts.appendingPathComponent("\(args.host)-host.jsonl")
        result["host"] = args.host
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
            try selectJapanese()
            try sendKey(CGKeyCode(kVK_JIS_Kana))
            result["native_input_activation"] = "TIS selection followed by native JIS Kana key"
            let baseline = try completeCommands(at: trace)
            try sendTextKeys([45, 34, 4, 31, 45, 5, 31])
            let preedit = try waitForPhase(name: "preedit", trace: trace, baseline: baseline,
                                            requiredOCR: ["にほんご", "日本語"],
                                            requireCommit: false)
            try sendKey(49)
            let conversion = try waitForPhase(name: "conversion", trace: trace, baseline: baseline,
                                               requiredOCR: ["日本語"], requireCommit: false)
            try sendKey(36)
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
            try sendKey(53)
            try sendKey(53)
            result["cancel_key_sequence"] = ["Escape", "Escape"]
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
        result["input_activation_order"] = "focus composer and capture placeholder, then select Japanese"
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
        let selectStatus = TISSelectInputSource(source)
        result["mode_select_status"] = selectStatus
        guard selectStatus == noErr else { throw ProbeError.blocked("TISSelectInputSource failed: \(selectStatus)") }
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

    private func sendKey(_ key: CGKeyCode) throws {
        try assertOwnHostFrontmost()
        guard let down = CGEvent(keyboardEventSource: nil, virtualKey: key, keyDown: true) else {
            throw ProbeError.failed("cannot create key event \(key)")
        }
        down.flags = []
        down.post(tap: .cghidEventTap)
        // キーの押下時間をdown/up間で再現し、瞬時のtapとの差を同一runnerで比較する。
        if args.keyHoldMilliseconds > 0 {
            RunLoop.current.run(until: Date().addingTimeInterval(Double(args.keyHoldMilliseconds) / 1000))
        }
        guard let up = CGEvent(keyboardEventSource: nil, virtualKey: key, keyDown: false) else {
            throw ProbeError.failed("cannot create key release \(key)")
        }
        up.flags = []
        up.post(tap: .cghidEventTap)
        postedKeys.append(["key_code": Int(key), "uptime": ProcessInfo.processInfo.systemUptime,
                           "cg_timestamp_seconds": Double(down.timestamp) / 1_000_000_000,
                           "release_timestamp_seconds": Double(up.timestamp) / 1_000_000_000,
                           "keyboard_type": down.getIntegerValueField(.keyboardEventKeyboardType)])
        result["posted_keys"] = postedKeys
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

    private func waitForPhase(name: String, trace: URL, baseline: [String], requiredOCR: [String], requireCommit: Bool) throws -> Phase {
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
            if !requireCommit && commandList.contains(where: { $0.contains("DraftChanged") || $0.contains("Submit") }) {
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
