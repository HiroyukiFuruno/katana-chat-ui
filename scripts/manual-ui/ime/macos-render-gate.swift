import CoreGraphics
import Foundation

struct RenderGateRegion: Equatable {
    let pixelBounds: CGRect

    private static let empty = Self(pixelBounds: .zero)

    private static func isFinite(_ rect: CGRect) -> Bool {
        [rect.minX, rect.minY, rect.width, rect.height, rect.maxX, rect.maxY].allSatisfy(\.isFinite)
    }

    private static func clampedPixel(_ value: CGFloat, limit: CGFloat, rounding: FloatingPointRoundingRule) -> Int {
        let bounded = min(max(value, 0), limit)
        return Int(bounded.rounded(rounding))
    }

    static func fromContentHeader(contentRect: CGRect, screenBounds: CGRect, width: Int, height: Int,
                                  headerOffset: CGFloat = 2, headerHeight: CGFloat = 32) -> Self {
        guard width > 0, height > 0, isFinite(contentRect), contentRect.width > 0,
              contentRect.height > 0, isFinite(screenBounds), screenBounds.width > 0,
              screenBounds.height > 0,
              headerOffset.isFinite, headerHeight.isFinite, headerOffset >= 0, headerHeight > 0 else {
            return empty
        }
        let pixelWidth = CGFloat(width)
        let pixelHeight = CGFloat(height)
        let scaleX = pixelWidth / screenBounds.width
        let scaleY = pixelHeight / screenBounds.height
        let contentTop = screenBounds.maxY - contentRect.maxY
        let source = CGRect(x: (contentRect.minX - screenBounds.minX) * scaleX,
                            y: (contentTop - screenBounds.minY + headerOffset) * scaleY,
                            width: contentRect.width * scaleX, height: headerHeight * scaleY)
        guard scaleX.isFinite, scaleY.isFinite, isFinite(source) else { return empty }
        let left = clampedPixel(source.minX, limit: pixelWidth, rounding: .down)
        let top = clampedPixel(source.minY, limit: pixelHeight, rounding: .down)
        let right = clampedPixel(source.maxX, limit: pixelWidth, rounding: .up)
        let bottom = clampedPixel(source.maxY, limit: pixelHeight, rounding: .up)
        guard right > left, bottom > top else {
            return empty
        }
        return Self(pixelBounds: CGRect(x: left, y: top, width: right - left, height: bottom - top))
    }
}

struct RenderGateComparison: Equatable {
    let sampleCount: Int
    let mismatchCount: Int
    let mismatchRatio: Double

    var passes: Bool { sampleCount > 0 && mismatchRatio <= 0.02 }
}

enum RenderGate {
    static func darkPixelCount(_ rgba: [UInt8], threshold: UInt8 = 160) -> Int {
        guard !rgba.isEmpty, rgba.count.isMultiple(of: 4) else { return 0 }
        return stride(from: 0, to: rgba.count, by: 4).reduce(into: 0) { count, index in
            if rgba[index] < threshold || rgba[index + 1] < threshold || rgba[index + 2] < threshold { count += 1 }
        }
    }

    static func compare(_ baseline: [UInt8], _ current: [UInt8], tolerance: UInt8 = 1) -> RenderGateComparison {
        guard baseline.count == current.count, baseline.count > 0, baseline.count.isMultiple(of: 4) else {
            return RenderGateComparison(sampleCount: 0, mismatchCount: 0, mismatchRatio: 1)
        }
        let totalPixels = baseline.count / 4
        let sampleCount = (0..<totalPixels).reduce(into: 0) { count, pixel in
            let offset = pixel * 4
            if baseline[offset] < 160 || baseline[offset + 1] < 160 || baseline[offset + 2] < 160 { count += 1 }
        }
        guard sampleCount > 0 else { return RenderGateComparison(sampleCount: 0, mismatchCount: 0, mismatchRatio: 1) }
        var mismatchCount = 0
        for pixel in 0..<totalPixels {
            let offset = pixel * 4
            guard baseline[offset] < 160 || baseline[offset + 1] < 160 || baseline[offset + 2] < 160 else { continue }
            let mismatch = (0..<4).contains { channel in
                abs(Int(baseline[offset + channel]) - Int(current[offset + channel])) > Int(tolerance)
            }
            if mismatch { mismatchCount += 1 }
        }
        return RenderGateComparison(sampleCount: sampleCount, mismatchCount: mismatchCount,
                                    mismatchRatio: Double(mismatchCount) / Double(sampleCount))
    }
}
