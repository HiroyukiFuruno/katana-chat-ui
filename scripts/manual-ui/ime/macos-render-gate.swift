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
        let bodyWidth = min(CGFloat(560), contentRect.width)
        let bodyOriginX = contentRect.minX + (contentRect.width - bodyWidth) / 2
        let source = CGRect(x: (bodyOriginX - screenBounds.minX) * scaleX,
                            y: (contentTop - screenBounds.minY + headerOffset) * scaleY,
                            width: bodyWidth * scaleX, height: headerHeight * scaleY)
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

struct RenderStability {
    private var priorToken: String?
    private var priorPixels: [UInt8]?
    private(set) var consecutiveMatches = 0

    mutating func observe(token: String, pixels: [UInt8]) -> Bool {
        guard !pixels.isEmpty, pixels.count.isMultiple(of: 4) else {
            priorToken = nil; priorPixels = nil; consecutiveMatches = 0
            return false
        }
        if priorToken == token, priorPixels == pixels { consecutiveMatches += 1 }
        else { consecutiveMatches = 1 }
        priorToken = token; priorPixels = pixels
        return consecutiveMatches >= 2
    }
}

struct RuntimeFrameStamp: Decodable {
    let kind: String
    let source: String
    let seq: UInt64
    let observed_at: Double
    let marked: Bool

    func followsNative(callbackUptime: Double, captureUptime: Double, captureUnixTime: Double,
                       expectedMarked: Bool) -> Bool {
        guard [observed_at, callbackUptime, captureUptime, captureUnixTime].allSatisfy(\.isFinite),
              callbackUptime <= captureUptime, seq > 0,
              kind == "ime-state", source == "floem-kuc-runtime-frame", marked == expectedMarked else { return false }
        let callbackUnixTime = captureUnixTime - (captureUptime - callbackUptime)
        return observed_at >= callbackUnixTime && observed_at <= captureUnixTime
    }
}
