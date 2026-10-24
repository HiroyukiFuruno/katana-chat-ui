import CoreGraphics
import Foundation

struct OCRImageRegion {
    let pixelBounds: CGRect
    let originalWidth: CGFloat
    let originalHeight: CGFloat

    static func fullFrame(width: Int, height: Int) -> OCRImageRegion {
        OCRImageRegion(pixelBounds: CGRect(x: 0, y: 0, width: width, height: height),
                       originalWidth: CGFloat(width), originalHeight: CGFloat(height))
    }

    static func rowRegion(width: Int, height: Int, rowCenter: CGFloat?, baselineHeight: CGFloat?,
                          minimumRadiusPixels: CGFloat = 32) -> OCRImageRegion {
        guard let rowCenter, let baselineHeight,
              rowCenter.isFinite, baselineHeight.isFinite,
              baselineHeight > 0, rowCenter >= 0, rowCenter <= 1 else {
            return fullFrame(width: width, height: height)
        }
        let imageHeight = CGFloat(height)
        let centerFromTop = (1 - rowCenter) * imageHeight
        let radius = max(minimumRadiusPixels, baselineHeight * imageHeight * 0.6)
        let top = max(0, Int(floor(centerFromTop - radius)))
        let bottom = min(height, Int(ceil(centerFromTop + radius)))
        guard bottom > top else { return fullFrame(width: width, height: height) }
        return OCRImageRegion(pixelBounds: CGRect(x: 0, y: top, width: width, height: bottom - top),
                              originalWidth: CGFloat(width), originalHeight: imageHeight)
    }

    /// 行の切出し内のVision座標を、原画全体の下原点の正規化座標へ戻す。
    func originalNormalizedBounds(_ cropNormalized: CGRect) -> CGRect {
        let cropWidth = pixelBounds.width
        let cropHeight = pixelBounds.height
        let sourceX = pixelBounds.minX + cropNormalized.minX * cropWidth
        let sourceWidth = cropNormalized.width * cropWidth
        let sourceTop = pixelBounds.minY + (1 - cropNormalized.maxY) * cropHeight
        let sourceHeight = cropNormalized.height * cropHeight
        return CGRect(x: sourceX / originalWidth,
                      y: 1 - (sourceTop + sourceHeight) / originalHeight,
                      width: sourceWidth / originalWidth,
                      height: sourceHeight / originalHeight)
    }
}

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
