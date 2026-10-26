import Foundation

enum RenderGateTestError: Error { case assertion(String) }

@main
struct RenderGateTests {
    static func main() {
        let path = CommandLine.arguments.dropFirst().first(where: { $0.hasPrefix("--result=") })
            .map { URL(fileURLWithPath: String($0.dropFirst("--result=".count))) }
            ?? URL(fileURLWithPath: "/tmp/kcu-render-gate-test-result.json")
        do {
            try runAll()
            try write(["status": "passed", "exit": 0], to: path)
            print("render gate tests passed: \(path.path)")
            exit(0)
        } catch {
            try? write(["status": "failed", "exit": 1, "error": String(describing: error)], to: path)
            fputs("render gate tests failed: \(error)\n", stderr)
            exit(1)
        }
    }

    private static func runAll() throws {
        let screen = CGRect(x: 0, y: 0, width: 1024, height: 768)
        let region = RenderGateRegion.fromContentHeader(contentRect: CGRect(x: 0, y: 60, width: 1024, height: 645),
                                                        screenBounds: screen, width: 1024, height: 768)
        try expect(region.pixelBounds == CGRect(x: 232, y: 65, width: 560, height: 32), "centered content header transform")
        let edge = RenderGateRegion.fromContentHeader(contentRect: CGRect(x: 0, y: 63, width: 1024, height: 642),
                                                       screenBounds: screen, width: 1024, height: 768)
        try expect(edge.pixelBounds == CGRect(x: 232, y: 65, width: 560, height: 32),
                   "content header must transform and clamp")
        let negativeWindow = RenderGateRegion.fromContentHeader(
            contentRect: CGRect(x: -128, y: 60, width: 1024, height: 645),
            screenBounds: screen, width: 1024, height: 768)
        try expect(negativeWindow.pixelBounds == CGRect(x: 104, y: 65, width: 560, height: 32),
                   "negative-origin window must clip safely")
        let baseline = Array(repeating: UInt8(100), count: 4 * 10)
        let same = baseline
        try expect(RenderGate.compare(baseline, same).passes, "identical pixels pass")
        try expect(RenderGate.compare(Array(baseline.dropLast()), same).passes == false, "unequal RGBA lengths fail")
        try expect(RenderGate.compare([1, 2, 3], [1, 2, 3]).passes == false, "non-RGBA length fails")
        var changed = baseline
        changed[0] = 103
        try expect(RenderGate.compare(baseline, changed).passes == false, "changed pixel must fail")
        try expect(RenderGate.compare([], []).passes == false, "empty sample must fail")
        try expect(RenderGate.darkPixelCount([1, 2, 3]) == 0, "malformed dark-pixel sample must fail closed")
        try expect(RenderGateRegion.fromContentHeader(contentRect: CGRect(x: CGFloat.infinity, y: 0, width: 1, height: 1),
                                                       screenBounds: screen, width: 10, height: 10).pixelBounds.isEmpty,
                   "non-finite content rect must produce empty region")
        try expect(RenderGateRegion.fromContentHeader(contentRect: CGRect(x: CGFloat.greatestFiniteMagnitude,
                                                                           y: 0, width: 1, height: 1),
                                                       screenBounds: screen, width: 10, height: 10).pixelBounds.isEmpty,
                   "finite content outside screen must produce empty region")
        try expect(RenderGateRegion.fromContentHeader(contentRect: CGRect(x: 0, y: 0, width: 1, height: 1),
                                                       screenBounds: CGRect(x: CGFloat.nan, y: 0, width: 1, height: 1),
                                                       width: 10, height: 10).pixelBounds.isEmpty,
                   "non-finite screen bounds must produce empty region")
        try expect(RenderGate.darkPixelCount([10, 10, 10, 255, 200, 200, 200, 255]) == 1,
                   "dark pixel count")
        var stability = RenderStability()
        try expect(!stability.observe(token: "a", pixels: baseline), "first frame is not stable")
        try expect(stability.observe(token: "a", pixels: baseline), "same token and pixels stabilize")
        try expect(!stability.observe(token: "b", pixels: baseline), "changed callback resets stability")
        var changedPixels = baseline; changedPixels[0] = 101
        try expect(!stability.observe(token: "b", pixels: changedPixels), "changed pixels reset stability")
        try expect(stability.observe(token: "b", pixels: changedPixels), "changed pixels need a second match")
        try expect(!stability.observe(token: "empty", pixels: []), "empty frame is rejected")
        try frameFreshness(baseline: baseline)
    }

    private static func frameFreshness(baseline: [UInt8]) throws {
        // 同じ古い画像が2枚続いても、native更新後の描画とは限らない。
        var stableOldPixels = RenderStability()
        _ = stableOldPixels.observe(token: "new-native-event", pixels: baseline)
        try expect(stableOldPixels.observe(token: "new-native-event", pixels: baseline),
                   "pixel stability alone accepts an unchanged old frame")
        let old = RuntimeFrameStamp(kind: "ime-state", source: "floem-kuc-runtime-frame", seq: 1,
                                    observed_at: 999, marked: true)
        try expect(!old.followsNative(callbackUptime: 10, captureUptime: 11, captureUnixTime: 1001, expectedMarked: true),
                   "frame before native callback cannot satisfy freshness")
        let current = RuntimeFrameStamp(kind: "ime-state", source: "floem-kuc-runtime-frame", seq: 2,
                                        observed_at: 1000.5, marked: true)
        try expect(current.followsNative(callbackUptime: 10, captureUptime: 11, captureUnixTime: 1001, expectedMarked: true),
                   "matching frame between callback and capture satisfies freshness")
        try expect(!current.followsNative(callbackUptime: 10, captureUptime: 11, captureUnixTime: 1001, expectedMarked: false),
                   "marked state must match the phase")
        try expect(!current.followsNative(callbackUptime: 12, captureUptime: 11, captureUnixTime: 1001, expectedMarked: true),
                   "callback after capture cannot satisfy freshness")
        try expect(!current.followsNative(callbackUptime: 10, captureUptime: 11, captureUnixTime: .nan, expectedMarked: true),
                   "invalid clock cannot satisfy freshness")
        let future = RuntimeFrameStamp(kind: "ime-state", source: "floem-kuc-runtime-frame", seq: 3,
                                       observed_at: 1002, marked: true)
        try expect(!future.followsNative(callbackUptime: 10, captureUptime: 11, captureUnixTime: 1001, expectedMarked: true),
                   "a later frame cannot justify an earlier image")
    }

    private static func expect(_ condition: Bool, _ message: String) throws {
        guard condition else { throw RenderGateTestError.assertion(message) }
    }

    private static func write(_ value: [String: Any], to path: URL) throws {
        let data = try JSONSerialization.data(withJSONObject: value, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: path, options: .atomic)
    }
}
