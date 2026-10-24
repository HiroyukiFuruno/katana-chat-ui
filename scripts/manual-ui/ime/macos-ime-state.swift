import Foundation

struct NativeCommitObserver {
    private var lastText: String
    private var lastMarked: Bool
    private var lastPublishedText: String
    private var stableUnmarkedPolls = 0

    init(committedText: String = "") {
        lastText = committedText
        lastMarked = false
        lastPublishedText = committedText
    }

    mutating func observe(text: String, isMarked: Bool) -> String? {
        if text != lastText || isMarked != lastMarked {
            stableUnmarkedPolls = 0
        }
        defer {
            lastText = text
            lastMarked = isMarked
        }
        guard !isMarked, text != lastPublishedText else {
            stableUnmarkedPolls = 0
            return nil
        }
        stableUnmarkedPolls += 1
        guard stableUnmarkedPolls >= 2 else { return nil }
        stableUnmarkedPolls = 0
        lastPublishedText = text
        return text
    }
}
