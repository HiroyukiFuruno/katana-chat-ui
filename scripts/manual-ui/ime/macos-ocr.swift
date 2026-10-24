import CoreGraphics
import Foundation

struct OCRObservation {
    let text: String
    let normalized: String
    let boundingBox: CGRect

    var boundingBoxJSON: [String: CGFloat] {
        ["x": boundingBox.origin.x, "y": boundingBox.origin.y,
         "width": boundingBox.width, "height": boundingBox.height]
    }
}

struct OCRSnapshot {
    let observations: [OCRObservation]

    var allJSON: [[String: Any]] {
        observations.map { ["text": $0.text, "normalized": $0.normalized, "bounds": $0.boundingBoxJSON] }
    }

    func filteredBody(rowCenter: CGFloat, baselineHeight: CGFloat) -> String {
        filtered(rowCenter: rowCenter, baselineHeight: baselineHeight)
            .sorted { $0.boundingBox.minX < $1.boundingBox.minX }
            .map(\.normalized).joined()
    }

    func filteredBoundsJSON(rowCenter: CGFloat, baselineHeight: CGFloat) -> [[String: CGFloat]] {
        filtered(rowCenter: rowCenter, baselineHeight: baselineHeight)
            .sorted { $0.boundingBox.minX < $1.boundingBox.minX }
            .map(\.boundingBoxJSON)
    }

    func matchesExpected(_ expected: [String], rowCenter: CGFloat, baselineHeight: CGFloat) -> Bool {
        expected.contains(filteredBody(rowCenter: rowCenter, baselineHeight: baselineHeight))
    }

    private func filtered(rowCenter: CGFloat, baselineHeight: CGFloat) -> [OCRObservation] {
        observations.filter {
            abs($0.boundingBox.midY - rowCenter) < 0.6 * max($0.boundingBox.height, baselineHeight)
        }
    }
}

func normalizeOCR(_ text: String) -> String {
    text.lowercased().filter { !$0.isWhitespace }
}
