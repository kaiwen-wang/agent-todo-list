# AGT-56: drag kanban highlight header

## Research

### Current drag-and-drop implementation

The board uses **native HTML5 drag-and-drop** (no external library). Key files:

| File | Role |
|------|------|
| `src/web/src/views/BoardView.vue` | Board layout, horizontal scroll container (`.board` with `overflow-x: auto`) |
| `src/web/src/components/StatusColumn.vue` | Column with `dragenter`/`dragleave`/`dragover`/`drop` handlers, tracks `isDragOver` ref |
| `src/web/src/components/TodoCard.vue` | Cards are `draggable="true"`, sets drag data on `dragstart` |
| `src/web/src/stores/project.ts` | `moveTodo()` and `bulkUpdateTodos()` with optimistic updates |

### Current drag feedback

- **Column body** gets a light blue background (`rgba(59, 130, 246, 0.08)`) via `.drag-over` class when a card is dragged over it.
- **Column header** has no visual change during drag — it's a static bar with a colored dot indicator and count badge.
- The `isDragOver` state lives in `StatusColumn.vue` as a single ref; it doesn't distinguish header vs body.

### Board scroll structure

```
.board-view (flex column, height: 100%)
├── .board-toolbar (fixed, no scroll)
└── .board (flex: 1, overflow-x: auto, gap: 12px)  ← horizontal scroll container
    └── StatusColumn × N (min-width: 260px, max-width: 340px)
        ├── .column-header (flex-shrink: 0)
        └── .column-body (flex: 1, overflow-y: auto)
            └── TodoCard (draggable)
```

There is **no auto-scroll logic** during drag. The `.board` div scrolls horizontally via `overflow-x: auto`, but only responds to manual scroll/trackpad, not drag proximity.

### Google Tasks reference behavior

Google Tasks auto-scrolls the board horizontally when you drag a card near the left or right edge of the viewport. The scroll speed increases the closer you get to the edge (velocity-based). The target column header also highlights/pulses to indicate where the card will land.

---

## Approach Options

### Option 1: Minimal — Header highlight only via existing drag events

Add header-specific styling within `StatusColumn.vue`. The existing `isDragOver` ref already tracks when a card is over the column. Just extend the CSS to also highlight `.column-header` when `.drag-over` is active.

**Pros:** Trivial change (~5 lines of CSS), no new logic needed.
**Cons:** No auto-scroll. Header highlight is tied to body drag-over (not independent).

### Option 2: Header highlight + edge-based auto-scroll using `dragover` clientX

- **Header highlight:** Same as Option 1 — apply highlight styling to `.column-header` when `isDragOver` is true.
- **Auto-scroll:** In `BoardView.vue`, add a `dragover` handler on the `.board` container. On each event, check `e.clientX` relative to the board's left/right edges. If within a threshold (e.g. 80px), call `scrollBy()` on the board element with velocity proportional to distance from edge. Use `requestAnimationFrame` for smooth scrolling.

**Pros:** Covers both features. Uses only native drag events (no new dependencies). Velocity-based scroll matches Google Tasks feel.
**Cons:** `dragover` fires frequently — needs throttling or rAF gating. Need to handle cleanup when drag ends.

### Option 3: Full polish — highlight + auto-scroll + scroll indicator zones

Same as Option 2, plus visible "scroll zone" indicators (subtle gradient overlays or arrows) that appear at the board edges when a drag is active and content is scrollable in that direction.

**Pros:** Most polished UX. Clear affordance that scrolling is possible.
**Cons:** More CSS/markup. Possibly over-engineered for current needs.

---

## Recommendation

**Option 2** — it covers both requested features with minimal complexity:

1. **Header highlight:** Add `.drag-over .column-header` CSS rule in `StatusColumn.vue` to change the header background/border when a card is dragged over the column. Consider a colored top border or background tint matching the column's status color.

2. **Auto-scroll:** Add a `dragover` listener on the `.board` element in `BoardView.vue`:
   - Track whether a drag is active (listen for `dragstart`/`dragend` on the board).
   - On `dragover`, compute distance from `e.clientX` to the board's left/right bounds.
   - If within ~80px of an edge, start a `requestAnimationFrame` loop that calls `board.scrollBy({ left: speed })` where `speed` scales with proximity (closer = faster, max ~8px/frame).
   - Cancel the rAF loop on `dragleave`, `drop`, or `dragend`.

This approach requires changes to only two files, uses no new dependencies, and matches the Google Tasks behavior described in the task.

---

## Questions

- Should the header highlight use a specific color (e.g. the status dot color), or is a generic blue tint fine?
- Should vertical auto-scroll within a column body also be added (for columns with many cards), or just horizontal board scroll?
- Any preference on auto-scroll speed / edge threshold distance?

## Answers