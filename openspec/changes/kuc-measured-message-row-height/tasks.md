## Implementation

- [x] 1.1 Add a real-runtime regression for the existing Storybook Japanese prompt and verify measured body bounds fit inside the user row. The exact seed is one line under macOS metrics; the test uses the Linux-specific two-line extent where available.
- [x] 1.2 Measure message descendants through the runtime's public KUC renderer and rebuild rows using the measured extent.
- [x] 1.3 Apply the same measured row heights to thread scroll content sizing.
- [x] 1.4 Add narrow Japanese, multi-heading Markdown, and expanded-thinking regressions, including final-line visibility and scroll extent.
- [x] 1.5 Run focused KUC layout/scroll tests (22 passed), Rust formatting, strict package Clippy, and AST lint (19 passed) without changing baselines or gates.
- [x] 1.6 Keep one-line rows at their existing 44px height; exclude the Card root from measured content so bottom padding is counted once, and aggregate message bounds in one tree index/hit pass.
- [x] 1.7 Preserve the existing minimum row height for an actually empty completed assistant card, with a real-runtime regression for empty and whitespace-only bodies.
- [x] 1.8 Add the pure-kuc-chat contract delta for measured rows, scroll extent, and empty rows; validate all OpenSpec items in strict mode.

## Definition of Done

- Each message row contains all measured rendered descendants plus its existing vertical padding.
- Thread scroll content height equals or exceeds the sum of the final row heights.
- Existing message widths, right alignment, spacing, rendering baselines, and quality gates remain unchanged.

## Final-source evidence before public KUC 0.4.4 adoption

- `just harness-screenshot-check` completed with exit 0 (`tmp/verification/harness-screenshot-check-background.log`). All five existing PNG hashes remain unchanged; root viewed the actual empty, narrow, standard, tall, and thinking images. Paths and hashes are recorded in `tmp/verification/measured-message-five-images.json`.
- Background Linux/private-Xvfb `just storybook-window-smoke` completed with exit 0 (`tmp/verification/storybook-background-final.log`). Root viewed the rebuilt interactive window at `target/storybook-window-smoke/actual-storybook-window.png` (SHA-256 `e711f319668aca74a7612f36b3e2f5906164c436b34cc6d06271afc27cf15206`) and confirmed the Japanese prompt's second line is fully visible. Content below the thread viewport remains scrollable rather than expanding over the composer.
- These checks use public registry KUC 0.4.3. The final KCU release gates, native matrix, and public KUC 0.4.4 integration remain tracked in `openspec/release-tasks.md`; this evidence does not certify those future steps.
