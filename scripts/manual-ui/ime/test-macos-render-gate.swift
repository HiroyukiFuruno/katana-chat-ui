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
    }

    private static func expect(_ condition: Bool, _ message: String) throws {
        guard condition else { throw RenderGateTestError.assertion(message) }
    }

    private static func write(_ value: [String: Any], to path: URL) throws {
        let data = try JSONSerialization.data(withJSONObject: value, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: path, options: .atomic)
    }
}
