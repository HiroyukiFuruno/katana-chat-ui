# KUC measured message row height

## Why

KCU estimates message height from character counts. The public KUC renderer can wrap the same text into more lines, so user messages and expanded message content can extend beyond their allocated rows and be clipped. Thread scrolling also uses the estimate, which can make the document height too short.

## What changes

- Measure message content with the same public KUC renderer, viewport width, text configuration, and structured tree used for the visible frame.
- Allocate each message row enough height for its measured descendants and padding, while retaining the existing estimated height as a minimum.
- Use those same row heights for the scroll area's content height.
- Add renderer-backed regressions for Japanese wrapping, narrow layouts, Markdown blocks, and expanded thinking content.

## Non-goals

- Change message width, right alignment, padding, fonts, screenshots, or baselines.
- Change KUC APIs, public KCU APIs, or UI framework adapters.
- Relax quality gates or add platform-specific font heuristics.
