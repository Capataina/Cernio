# TUI Design Decisions

Design decisions and future plans for the terminal UI.

---

## Responsive layouts for different terminal sizes

The TUI should eventually support multiple rendering modes based on terminal dimensions:

- **Full mode** (wide terminal, 120+ columns): The detailed layout with side-by-side panels, full stats blocks, scrollable top roles list, job distribution charts in company detail
- **Compact mode** (narrow terminal, <120 columns): Proportional stacked bars instead of full stat blocks, condensed labels, single-column layout where needed

This is a future enhancement — for now, the full layout is the only mode. The compact rendering (proportional stacked bars like `SS━━S━━━━A━━━━━━B━━━━━━━━━C━F━━`) is a good fallback for small windows.

---

## Why bar charts over pie charts

Terminal pie charts (via braille-character circles on ratatui's Canvas) look like blobs — rectangular terminal cells distort circles into ovals, small slices (SS = 17/712) become unreadable dots, and you can't label segments clearly. Bar charts are the stronger terminal visualisation: each bar is a labelled, coloured row with exact values, proportional width shows distribution, and they work at any size.

---

## Fit data simplification

The `evaluation_status` field (strong_fit / weak_fit / no_fit) is a coarser bucketing of `grade` (SS/S → strong_fit, A/B → weak_fit, C/F → no_fit). It adds no information beyond the grade. The TUI should lean on grade as the primary metric and de-emphasise evaluation_status in display.

Future possibility: split into two dimensions — **role quality** (how good is the job) and **accessibility** (can you get it). A job could be quality S but accessibility "stretch". This would require a schema change.

---

## Mouse and touchpad support

MacBook touchpad two-finger scroll generates the same events as mouse scroll wheel. Crossterm captures these natively.

### Current state (broken)

- Scroll wheel moves selection one entry at a time — feels like teleporting through the list rather than scrolling a window. The expected behaviour is viewport scrolling: the visible window moves through the list smoothly, like scrolling a webpage. Currently each scroll tick calls `next_in_list()` which jumps selection by 1 — on a trackpad with momentum scrolling this flies through hundreds of entries instantly.
- Click-to-select is NOT implemented yet. Only scroll events are handled.
- Click-on-tab is NOT implemented yet.

### Correct implementation

**Scroll should move the viewport, not the selection.** Ratatui's `TableState` has an `offset()` method that controls which row is at the top of the visible area. Scroll events should adjust the offset, not the selected row. The selection should stay where it is until the user explicitly moves it with j/k or clicks.

**For trackpad momentum scrolling:** batch scroll events — if multiple scroll events arrive within a short window (e.g. 50ms), treat them as a single larger scroll. This prevents the "flies through the list" problem with MacBook trackpads.

**Click-to-select:** On left click, calculate which row was clicked based on `mouse.row` relative to the table's rendered area. Set selection to that row.

**Click-on-tab:** On left click in the tab bar area (row < 3), determine which tab was clicked based on `mouse.column` and switch view.

---

## Dashboard empty space problem

The current dashboard has massive empty space in:
- Grade Distribution block (left side, bottom half empty)
- Action Items block (right side, bottom half empty)
- Pipeline Health block (left bottom, two thirds empty)

### Root cause

The layout uses fixed `Constraint::Min(12)` for the top blocks which forces them to take at least 12 rows even when they have 6-8 rows of content. Then `Constraint::Fill(1)` gives the bottom blocks all remaining space, most of which is empty.

### Fix

1. Use `Constraint::Length(N)` based on actual content height instead of `Constraint::Min(12)`
2. OR: fill the empty space with useful content — the blocks should expand their content to use available space, not leave it blank
3. The Grade Distribution block should have wider bars (use more of the horizontal space)
4. Pipeline Health should have proportional bars showing ATS distribution visually, not just numbers
5. Action Items should include more detail — list the bespoke companies by name, show which jobs need descriptions
6. Top Roles should take more vertical space and show more entries

### Pipeline Health bars

Pipeline Health currently shows just text: `greenhouse 34 (54%)`. It should show proportional bars:

```
greenhouse  ████████████████████████████ 34 (54%)
ashby       ██████████████░░░░░░░░░░░░░ 14 (22%)
workable    ███████░░░░░░░░░░░░░░░░░░░░  7 (11%)
lever       ██████░░░░░░░░░░░░░░░░░░░░░  6 (10%)
workday     ██░░░░░░░░░░░░░░░░░░░░░░░░░  2  (3%)
```

This fills the horizontal space and makes the distribution instantly visible.

### Grade Distribution improvements

The bars are currently only 6 characters wide — they should expand to fill available width. With a 45% column width on a 120-column terminal, that's ~50 usable characters. The bars should be ~20-25 characters wide with the grade label and count flanking them.
