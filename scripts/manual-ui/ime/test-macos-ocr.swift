import Foundation

enum OCRTestFailure: Error, CustomStringConvertible {
    case assertion(String)

    var description: String {
        switch self {
        case let .assertion(message): message
        }
    }
}

@main
struct MacOSOCRTests {
    static func main() {
        let resultPath = resultPath()
        do {
            try runAll()
            try writeResult(["status": "passed", "exit": 0], to: resultPath)
            print("macOS OCR pure tests passed: \(resultPath)")
            exit(0)
        } catch {
            try? writeResult(["status": "failed", "exit": 1, "error": String(describing: error)], to: resultPath)
            fputs("macOS OCR pure tests failed: \(error)\n", stderr)
            exit(1)
        }
    }

    private static func runAll() throws {
        try assert(normalizeOCR(" 日本 語 ") == "日本語", "whitespace normalization")

        let eguiRegion = OCRImageRegion.rowRegion(width: 1024, height: 140,
                                                   rowCenter: 0.7127654561268545,
                                                   baselineHeight: 0.11045568557012642)
        try assert(eguiRegion.pixelBounds == CGRect(x: 0, y: 8, width: 1024, height: 65),
                   "Native17 egui ROI must reproduce x0 y8 width1024 height65")
        let referenceRegion = OCRImageRegion.rowRegion(width: 1024, height: 140,
                                                        rowCenter: 0.7926545094626615,
                                                        baselineHeight: 0.13850050426664806)
        try assert(referenceRegion.pixelBounds == CGRect(x: 0, y: 0, width: 1024, height: 62),
                   "Native17 reference ROI must clamp to top edge")
        try assert(OCRImageRegion.rowRegion(width: 12, height: 10, rowCenter: nil, baselineHeight: nil).pixelBounds ==
                   CGRect(x: 0, y: 0, width: 12, height: 10), "missing baseline must use full frame")
        let bottomRegion = OCRImageRegion.rowRegion(width: 20, height: 10, rowCenter: 0.01,
                                                    baselineHeight: 0.2).pixelBounds
        try assert(bottomRegion.minY == 0 && bottomRegion.maxY == 10,
                   "ROI must clamp at both screen edges")

        let cropBox = CGRect(x: 0.25, y: 0.2, width: 0.1, height: 0.2)
        let originalBox = eguiRegion.originalNormalizedBounds(cropBox)
        let roundTrip = CGRect(x: originalBox.minX * eguiRegion.originalWidth - eguiRegion.pixelBounds.minX,
                               y: (1 - originalBox.maxY) * eguiRegion.originalHeight - eguiRegion.pixelBounds.minY,
                               width: originalBox.width * eguiRegion.originalWidth,
                               height: originalBox.height * eguiRegion.originalHeight)
        try assert(abs(roundTrip.minX - cropBox.minX * eguiRegion.pixelBounds.width) < 0.0001 &&
                   abs(roundTrip.minY - (1 - cropBox.maxY) * eguiRegion.pixelBounds.height) < 0.0001 &&
                   abs(roundTrip.width - cropBox.width * eguiRegion.pixelBounds.width) < 0.0001 &&
                   abs(roundTrip.height - cropBox.height * eguiRegion.pixelBounds.height) < 0.0001,
                   "Vision crop bounds must round-trip to original coordinates")

        let duplicate = OCRSnapshot(observations: [
            observation("にほんご", x: 0.1, y: 0.5),
            observation("にほんご", x: 0.4, y: 0.5)
        ])
        try assert(duplicate.filteredBody(rowCenter: 0.55, baselineHeight: 0.1) == "にほんごにほんご",
                   "duplicated preedit must not be silently deduplicated")
        try assert(!duplicate.matchesExpected(["にほんご", "日本語"], rowCenter: 0.55, baselineHeight: 0.1),
                   "duplicated preedit must fail exact required-text matching")

        let mergedDuplicate = OCRSnapshot(observations: [
            observation("にほんにほんご", x: 0.1, y: 0.5)
        ])
        try assert(!mergedDuplicate.matchesExpected(["にほんご", "日本語"], rowCenter: 0.55, baselineHeight: 0.1),
                   "single merged OCR observation must fail exact required-text matching")

        let adjacentRow = OCRSnapshot(observations: [
            observation("日本語", x: 0.1, y: 0.5),
            observation("候補行", x: 0.4, y: 0.72)
        ])
        try assert(adjacentRow.filteredBody(rowCenter: 0.55, baselineHeight: 0.1) == "日本語",
                   "adjacent candidate row must be excluded")

        let reverseX = OCRSnapshot(observations: [
            observation("語", x: 0.4, y: 0.5),
            observation("日本", x: 0.1, y: 0.5)
        ])
        try assert(reverseX.filteredBody(rowCenter: 0.55, baselineHeight: 0.1) == "日本語",
                   "split OCR must be joined in x order")

        let cancelPrefix = OCRSnapshot(observations: [
            observation("てすと", x: 0.4, y: 0.5),
            observation("日本語", x: 0.1, y: 0.5)
        ])
        let cancelBody = cancelPrefix.filteredBody(rowCenter: 0.55, baselineHeight: 0.1)
        try assert(cancelBody == "日本語てすと", "cancel preedit must preserve committed prefix")
        try assert(cancelPrefix.matchesExpected(["日本語てすと", "日本語テスト"], rowCenter: 0.55, baselineHeight: 0.1),
                   "cancel preedit must use exact accepted body")
    }

    private static func observation(_ text: String, x: CGFloat, y: CGFloat) -> OCRObservation {
        OCRObservation(text: text, normalized: normalizeOCR(text), boundingBox: CGRect(x: x, y: y, width: 0.2, height: 0.1))
    }

    private static func assert(_ condition: Bool, _ message: String) throws {
        guard condition else { throw OCRTestFailure.assertion(message) }
    }

    private static func resultPath() -> URL {
        if let value = CommandLine.arguments.dropFirst().first(where: { $0.hasPrefix("--result=") }) {
            return URL(fileURLWithPath: String(value.dropFirst("--result=".count)))
        }
        return URL(fileURLWithPath: "/tmp/kcu-macos-ocr-test-result.json")
    }

    private static func writeResult(_ result: [String: Any], to path: URL) throws {
        let data = try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: path, options: .atomic)
    }
}
