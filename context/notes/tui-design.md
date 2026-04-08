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

MacBook touchpad two-finger scroll generates the same events as mouse scroll wheel. Crossterm captures these natively. Implementation:
- Scroll wheel → move selection in lists, scroll detail panel
- Left click on row → select it
- Left click on tab → switch view

This works across all three pages with no platform-specific code.
