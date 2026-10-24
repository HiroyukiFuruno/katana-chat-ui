import ApplicationServices
import AppKit
import Carbon
import CoreGraphics
import Foundation

enum InputMenuError: Error, CustomStringConvertible {
    case blocked(String)
    case failed(String)

    var description: String {
        switch self {
        case let .blocked(message), let .failed(message): message
        }
    }
}

private let inputMenuIdentifiers = Set([
    "com.apple.menuextra.inputmenu",
    "com.apple.menuextra.textinput",
    "com.apple.inputmethod.menu"
])
private let inputMenuTitles = Set(["Input Menu", "Input menu", "Text Input", "入力メニュー"])
private let hiraganaTitles = Set(["Hiragana", "ひらがな"])
private let maximumDepth = 8
private let maximumNodes = 1_000
private let pollTimeout: TimeInterval = 30

private struct AXRecord {
    let element: AXUIElement
    let ownerBundle: String
    let role: String?
    let title: String?
    let identifier: String?
    let description: String?
    let depth: Int

    var json: [String: Any] {
        ["owner_bundle": ownerBundle, "role": role ?? (NSNull() as Any), "title": title ?? (NSNull() as Any),
         "identifier": identifier ?? (NSNull() as Any),
         "description": description ?? (NSNull() as Any), "depth": depth]
    }
}

private final class AXCollector {
    var records: [AXRecord] = []
    let ownerBundle: String

    init(ownerBundle: String) { self.ownerBundle = ownerBundle }

    func collect(_ element: AXUIElement, depth: Int) {
        guard depth <= maximumDepth, records.count < maximumNodes,
              !records.contains(where: { CFEqual($0.element, element) }) else { return }
        let record = AXRecord(element: element, ownerBundle: ownerBundle,
                              role: stringAttribute(element, kAXRoleAttribute),
                              title: stringAttribute(element, kAXTitleAttribute),
                              identifier: stringAttribute(element, kAXIdentifierAttribute),
                              description: stringAttribute(element, kAXDescriptionAttribute),
                              depth: depth)
        records.append(record)
        guard depth < maximumDepth, records.count < maximumNodes else { return }
        var children: CFTypeRef?
        guard AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &children) == .success,
              let childArray = children as? [AXUIElement] else { return }
        for child in childArray {
            collect(child, depth: depth + 1)
            if records.count >= maximumNodes { return }
        }
    }
}

private func stringAttribute(_ element: AXUIElement, _ attribute: String) -> String? {
    var value: CFTypeRef?
    guard AXUIElementCopyAttributeValue(element, attribute as CFString, &value) == .success else { return nil }
    return value as? String
}

private func writeJSON(_ value: Any, to url: URL) throws {
    let data = try JSONSerialization.data(withJSONObject: value, options: [.prettyPrinted, .sortedKeys])
    try data.write(to: url, options: .atomic)
}

private func requireCI() throws {
    guard ProcessInfo.processInfo.environment["GITHUB_ACTIONS"] == "true",
          ProcessInfo.processInfo.environment["RUNNER_OS"] == "macOS" else {
        throw InputMenuError.blocked("input menu automation requires a GitHub Actions macOS runner")
    }
    guard AXIsProcessTrusted() else {
        throw InputMenuError.blocked("Accessibility permission is unavailable")
    }
}

private func requireExpectedHostFrontmost(_ pid: pid_t) throws {
    guard NSWorkspace.shared.frontmostApplication?.processIdentifier == pid else {
        throw InputMenuError.failed("expected host is not frontmost")
    }
}

private func menuApplications() -> [NSRunningApplication] {
    NSWorkspace.shared.runningApplications.filter { application in
        guard let bundleID = application.bundleIdentifier else { return false }
        return bundleID == "com.apple.TextInputMenuAgent" ||
            bundleID == "com.apple.systemuiserver"
    }
}

private func collectMenuRecords() throws -> (records: [AXRecord], applications: [[String: Any]]) {
    let applications = menuApplications()
    guard !applications.isEmpty else {
        throw InputMenuError.failed("TextInputMenuAgent/SystemUIServer is not running")
    }
    var records: [AXRecord] = []
    var applicationJSON: [[String: Any]] = []
    for application in applications {
        let root = AXUIElementCreateApplication(application.processIdentifier)
        AXUIElementSetMessagingTimeout(root, 2)
        let collector = AXCollector(ownerBundle: application.bundleIdentifier ?? "")
        collector.collect(root, depth: 0)
        for attribute in [kAXMenuBarAttribute, kAXExtrasMenuBarAttribute] {
            var menuBar: CFTypeRef?
            if AXUIElementCopyAttributeValue(root, attribute as CFString, &menuBar) == .success,
               let menuBar, CFGetTypeID(menuBar) == AXUIElementGetTypeID() {
                collector.collect(unsafeBitCast(menuBar, to: AXUIElement.self), depth: 1)
            }
        }
        records.append(contentsOf: collector.records)
        applicationJSON.append([
            "pid": application.processIdentifier,
            "bundle_identifier": application.bundleIdentifier ?? "",
            "localized_name": application.localizedName ?? "",
            "records": collector.records.map(\.json)
        ])
    }
    return (records, applicationJSON)
}

private func saveSnapshot(_ applications: [[String: Any]], stage: String, directory: URL) throws {
    try writeJSON(["stage": stage, "applications": applications,
                   "maximum_depth": maximumDepth, "maximum_nodes": maximumNodes],
                  to: directory.appendingPathComponent("input-menu-ax-\(stage).json"))
}

private func currentInputSourceName() throws -> String {
    guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
          let value = TISGetInputSourceProperty(source, kTISPropertyLocalizedName) else {
        throw InputMenuError.failed("current input source name is unavailable")
    }
    let name = Unmanaged<CFString>.fromOpaque(value).takeUnretainedValue() as String
    guard !name.isEmpty else { throw InputMenuError.failed("current input source name is empty") }
    return name
}

private func inputMenuCandidates(_ records: [AXRecord], currentSourceName: String) -> [AXRecord] {
    records.filter { record in
        guard record.role == kAXMenuBarItemRole else { return false }
        // Native13の実AX記録では、入力status itemだけが無題で現在の方式名をdescriptionに持つ。
        let currentSourceItem = record.ownerBundle == "com.apple.TextInputMenuAgent" &&
            record.title == nil && record.description == currentSourceName
        return currentSourceItem ||
            (record.identifier.map(inputMenuIdentifiers.contains) ?? false) ||
            (record.title.map(inputMenuTitles.contains) ?? false) ||
            (record.description.map(inputMenuTitles.contains) ?? false)
    }
}

private func hiraganaCandidates(_ records: [AXRecord]) -> [AXRecord] {
    records.filter { record in
        record.role == kAXMenuItemRole && record.title.map(hiraganaTitles.contains) == true
    }
}

private func clickUnique(_ candidates: [AXRecord], description: String, directory: URL) throws {
    guard candidates.count == 1, let candidate = candidates.first else {
        throw InputMenuError.failed("\(description) candidate count is \(candidates.count), expected exactly one")
    }
    var enabled: CFTypeRef?
    guard AXUIElementCopyAttributeValue(candidate.element, kAXEnabledAttribute as CFString, &enabled) == .success,
          let enabled, CFGetTypeID(enabled) == CFBooleanGetTypeID(),
          CFBooleanGetValue(unsafeBitCast(enabled, to: CFBoolean.self)) else {
        throw InputMenuError.failed("AX candidate is not enabled for \(description)")
    }
    var rawPosition: CFTypeRef?
    var rawSize: CFTypeRef?
    let positionStatus = AXUIElementCopyAttributeValue(candidate.element, kAXPositionAttribute as CFString, &rawPosition)
    let sizeStatus = AXUIElementCopyAttributeValue(candidate.element, kAXSizeAttribute as CFString, &rawSize)
    var actionNames: CFArray?
    let actionStatus = AXUIElementCopyActionNames(candidate.element, &actionNames)
    guard positionStatus == .success, sizeStatus == .success,
          let rawPosition, let rawSize,
          CFGetTypeID(rawPosition) == AXValueGetTypeID(), CFGetTypeID(rawSize) == AXValueGetTypeID() else {
        throw InputMenuError.failed("AX bounds unavailable for \(description): position=\(positionStatus.rawValue) size=\(sizeStatus.rawValue)")
    }
    var position = CGPoint.zero
    var size = CGSize.zero
    guard AXValueGetValue(unsafeBitCast(rawPosition, to: AXValue.self), .cgPoint, &position),
          AXValueGetValue(unsafeBitCast(rawSize, to: AXValue.self), .cgSize, &size),
          position.x.isFinite, position.y.isFinite, size.width.isFinite, size.height.isFinite,
          size.width > 0, size.height > 0 else {
        throw InputMenuError.failed("AX bounds are invalid for \(description)")
    }
    let point = CGPoint(x: position.x + size.width / 2, y: position.y + size.height / 2)
    guard CGDisplayBounds(CGMainDisplayID()).contains(point),
          let down = CGEvent(mouseEventSource: nil, mouseType: .leftMouseDown,
                             mouseCursorPosition: point, mouseButton: .left),
          let up = CGEvent(mouseEventSource: nil, mouseType: .leftMouseUp,
                           mouseCursorPosition: point, mouseButton: .left) else {
        throw InputMenuError.failed("visible menu click is unavailable for \(description)")
    }
    try writeJSON(["candidate": candidate.json, "method": "CGEvent mouse down/up at AX bounds",
                   "position": [position.x, position.y], "size": [size.width, size.height],
                   "click_point": [point.x, point.y], "action_names_status": actionStatus.rawValue,
                   "action_names": (actionNames as? [String]) ?? []],
                  to: directory.appendingPathComponent("input-menu-click-\(description.replacingOccurrences(of: " ", with: "-")).json"))
    down.setIntegerValueField(.mouseEventClickState, value: 1)
    up.setIntegerValueField(.mouseEventClickState, value: 1)
    down.post(tap: .cghidEventTap)
    up.post(tap: .cghidEventTap)
}

private func waitForHiragana(_ expectedHostPID: pid_t, inputMenu: AXRecord, directory: URL) throws -> AXRecord {
    var latestApplications: [[String: Any]] = []
    let deadline = Date().addingTimeInterval(pollTimeout)
    while Date() < deadline {
        let frontPID = NSWorkspace.shared.frontmostApplication?.processIdentifier
        guard frontPID == expectedHostPID || menuApplications().contains(where: { $0.processIdentifier == frontPID }) else {
            throw InputMenuError.failed("unexpected foreground application while the input menu is open")
        }
        let collected = try collectMenuRecords()
        latestApplications = collected.applications
        let subtree = AXCollector(ownerBundle: inputMenu.ownerBundle)
        subtree.collect(inputMenu.element, depth: 0)
        let candidates = hiraganaCandidates(subtree.records)
        if candidates.count == 1 {
            try saveSnapshot(latestApplications, stage: "opened", directory: directory)
            return candidates[0]
        }
        if candidates.count > 1 {
            throw InputMenuError.failed("Hiragana candidate is ambiguous")
        }
        RunLoop.main.run(until: Date().addingTimeInterval(0.05))
    }
    try saveSnapshot(latestApplications, stage: "opened-timeout", directory: directory)
    throw InputMenuError.failed("Hiragana menu item did not appear before timeout")
}

func selectJapaneseFromInputMenu(expectedHostPID: pid_t, artifactDirectory: URL) throws {
    try requireCI()
    guard FileManager.default.fileExists(atPath: artifactDirectory.path) else {
        throw InputMenuError.failed("artifact directory does not exist")
    }
    try requireExpectedHostFrontmost(expectedHostPID)
    let before = try collectMenuRecords()
    try saveSnapshot(before.applications, stage: "before", directory: artifactDirectory)
    let inputCandidates = inputMenuCandidates(before.records, currentSourceName: try currentInputSourceName())
    try clickUnique(inputCandidates, description: "input menu", directory: artifactDirectory)
    guard let inputMenu = inputCandidates.first else { throw InputMenuError.failed("input menu is unavailable") }
    let hiragana = try waitForHiragana(expectedHostPID, inputMenu: inputMenu, directory: artifactDirectory)
    try clickUnique([hiragana], description: "Hiragana", directory: artifactDirectory)
    let after = try collectMenuRecords()
    try saveSnapshot(after.applications, stage: "after", directory: artifactDirectory)
}
