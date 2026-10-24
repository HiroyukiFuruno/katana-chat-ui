import AppKit
import Foundation

enum ReferenceError: Error, CustomStringConvertible {
    case blocked(String)
    case failed(String)

    var description: String {
        switch self {
        case let .blocked(message), let .failed(message): message
        }
    }
}

final class TraceWriter {
    private let handle: FileHandle

    init(path: String) throws {
        guard path.hasPrefix("/") else { throw ReferenceError.failed("KCU_TRACE_PATH must be absolute") }
        let url = URL(fileURLWithPath: path)
        guard FileManager.default.fileExists(atPath: url.deletingLastPathComponent().path) else {
            throw ReferenceError.failed("trace parent directory does not exist")
        }
        guard FileManager.default.createFile(atPath: url.path, contents: nil) else {
            throw ReferenceError.failed("cannot create native reference trace")
        }
        handle = try FileHandle(forWritingTo: url)
    }

    func write(_ value: [String: Any]) {
        do {
            let data = try JSONSerialization.data(withJSONObject: value)
            try handle.write(contentsOf: data)
            try handle.write(contentsOf: Data([0x0a]))
            try handle.synchronize()
        } catch {
            fputs("native reference trace write failed: \(error)\n", stderr)
            exit(1)
        }
    }

    func close() { try? handle.close() }
}

final class ReferenceTextView: NSTextView {
    let trace: TraceWriter
    private let referenceTextStorage: NSTextStorage
    private var lastPublishedText = ""
    private var commandSequence = 0

    init(frame: NSRect, trace: TraceWriter) {
        self.trace = trace
        let storage = NSTextStorage()
        let layoutManager = NSLayoutManager()
        let container = NSTextContainer(size: frame.size)
        container.widthTracksTextView = true
        container.heightTracksTextView = true
        storage.addLayoutManager(layoutManager)
        layoutManager.addTextContainer(container)
        referenceTextStorage = storage
        super.init(frame: frame, textContainer: container)
        isEditable = true
        isSelectable = true
        isRichText = false
        allowsUndo = false
        font = NSFont.systemFont(ofSize: 14)
        textColor = .labelColor
        drawsBackground = true
        backgroundColor = .textBackgroundColor
        textContainerInset = NSSize(width: 8, height: 8)
        string = ""
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) { fatalError("init(coder:) is unavailable") }

    override var acceptsFirstResponder: Bool { true }

    override func draw(_ dirtyRect: NSRect) {
        super.draw(dirtyRect)
        guard string.isEmpty else { return }
        let attributes: [NSAttributedString.Key: Any] = [
            .font: font ?? NSFont.systemFont(ofSize: 14),
            .foregroundColor: NSColor.secondaryLabelColor
        ]
        let rect = NSRect(x: textContainerInset.width,
                          y: textContainerInset.height,
                          width: bounds.width - textContainerInset.width * 2,
                          height: 22)
        ("Ask anything" as NSString).draw(in: rect, withAttributes: attributes)
    }

    override func setMarkedText(_ string: Any, selectedRange: NSRange, replacementRange: NSRange) {
        writeNativeEvent("setMarkedText", text: textValue(string), before: nativeState())
        super.setMarkedText(string, selectedRange: selectedRange, replacementRange: replacementRange)
        writeNativeEvent("setMarkedText.after", text: self.string, before: nativeState())
    }

    override func unmarkText() {
        writeNativeEvent("unmarkText", text: self.string, before: nativeState())
        super.unmarkText()
        let state = nativeState()
        writeNativeEvent("unmarkText.after", text: self.string, before: state)
        publishCommittedTextIfNeeded(state: state, source: "unmarkText")
    }

    override func insertText(_ string: Any, replacementRange: NSRange) {
        writeNativeEvent("insertText", text: textValue(string), before: nativeState())
        super.insertText(string, replacementRange: replacementRange)
        let state = nativeState()
        writeNativeEvent("insertText.after", text: self.string, before: state)
        publishCommittedTextIfNeeded(state: state, source: "insertText")
    }

    private func publishCommittedTextIfNeeded(state: [String: Any], source: String) {
        guard state["hasMarkedText"] as? Bool == false, string != lastPublishedText else { return }
        lastPublishedText = string
        commandSequence += 1
        let quoted = String(reflecting: string)
        trace.write(["kind": "command", "seq": commandSequence,
                     "command": "DraftChanged(\(quoted))", "draft": string,
                     "source": "NSTextInputClient.\(source)"])
    }

    private func writeNativeEvent(_ event: String, text: String, before state: [String: Any]) {
        trace.write(["kind": "native-ime", "event": event, "text": text,
                     "state": state])
    }

    private func nativeState() -> [String: Any] {
        let range = markedRange()
        return ["hasMarkedText": hasMarkedText(),
                "markedRange": ["location": range.location, "length": range.length],
                "firstResponder": window?.firstResponder === self,
                "string": string]
    }

    private func textValue(_ value: Any) -> String {
        if let value = value as? NSAttributedString { return value.string }
        if let value = value as? NSString { return value as String }
        return String(describing: value)
    }
}

final class ReferenceApplicationDelegate: NSObject, NSApplicationDelegate {
    private let trace: TraceWriter
    private var window: NSWindow?

    init(trace: TraceWriter) { self.trace = trace }

    func applicationDidFinishLaunching(_ notification: Notification) {
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 1280, height: 900),
                              styleMask: [.titled, .closable, .miniaturizable, .resizable],
                              backing: .buffered, defer: false)
        window.title = "katana-chat-ui harness appkit"
        window.center()
        let view = ReferenceTextView(frame: NSRect(x: 360, y: 80, width: 560, height: 88), trace: trace)
        view.autoresizingMask = [.minXMargin, .maxXMargin]
        window.contentView = NSView(frame: NSRect(origin: .zero, size: window.contentLayoutRect.size))
        window.contentView?.addSubview(view)
        self.window = window
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        window.makeFirstResponder(view)
        trace.write(["kind": "reference-ready", "title": window.title,
                     "window": ["width": 1280, "height": 900],
                     "textView": ["x": 360, "y": 80, "width": 560, "height": 88],
                     "firstResponder": window.firstResponder === view])
    }

    func applicationWillTerminate(_ notification: Notification) { trace.close() }
}

@main
struct MacOSAppKitReferenceMain {
    static func main() {
        do {
            guard ProcessInfo.processInfo.environment["GITHUB_ACTIONS"] == "true",
                  ProcessInfo.processInfo.environment["RUNNER_OS"] == "macOS" else {
                throw ReferenceError.blocked("must run only on a GitHub Actions macOS runner")
            }
            guard let path = ProcessInfo.processInfo.environment["KCU_TRACE_PATH"] else {
                throw ReferenceError.failed("KCU_TRACE_PATH is required")
            }
            let trace = try TraceWriter(path: path)
            let app = NSApplication.shared
            app.setActivationPolicy(.regular)
            let delegate = ReferenceApplicationDelegate(trace: trace)
            app.delegate = delegate
            withExtendedLifetime(delegate) { app.run() }
        } catch let error as ReferenceError {
            fputs("macOS AppKit reference \(error)\n", stderr)
            if case .blocked = error { exit(77) }
            exit(1)
        } catch {
            fputs("macOS AppKit reference \(error)\n", stderr)
            exit(1)
        }
    }
}
