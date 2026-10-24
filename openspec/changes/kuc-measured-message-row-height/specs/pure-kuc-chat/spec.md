## ADDED Requirements

### Requirement: Message rows fit measured KUC content and preserve scroll extent

KCU SHALL size each message row from its in-flow descendant bounds measured by the visible frame's public KUC renderer, adding existing bottom card padding once and retaining the existing minimum height. Thread scroll content height SHALL use the final row heights. Nested scroll contents and positioned overlays SHALL NOT expand the parent row.

#### Scenario: One-line message keeps its existing row height

- **WHEN** a completed user or assistant message renders as one line within its existing estimate
- **THEN** its row retains that minimum height and card padding is counted once

#### Scenario: Wrapped message content fits its row and scroll document

- **WHEN** public KUC bounds show that a message's content wraps beyond its estimated row
- **THEN** KCU grows the row to contain the measured content and padding, and the thread scroll content includes that final row height

### Requirement: Empty message cards and missing geometry are distinct

KCU SHALL retain the minimum estimated height for a message Card with no child nodes. A nonempty Card without required measured descendant bounds SHALL return an explicit runtime error.

#### Scenario: Empty completed assistant message has no rendered descendants

- **WHEN** a completed assistant message has an empty or whitespace-only body and its message Card has no child nodes
- **THEN** KCU retains the existing minimum row height without treating the intentionally empty card as missing renderer geometry

#### Scenario: Nonempty message has no renderer bounds

- **WHEN** a message Card has child nodes but KUC returns no measured in-flow descendant bounds for them
- **THEN** KCU returns an explicit runtime error instead of silently using the minimum estimate
