# TUI v3 Plan

> Complete overhaul of the terminal UI — from functional dashboard to best-in-class interactive job hunting platform. Every change uses dynamic sizing, fills empty space, and makes the TUI genuinely enjoyable to use.

**Status:** **Complete** — implemented across TUI v3 (commit ed30573) and v4 (commit 1a737ce), 2026-04-08.

---

## Design principles

These apply to every change in this plan:

1. **Dynamic over hardcoded.** Never specify pixel counts, character widths, or row counts as constants. Bars should fill a percentage of their container. Blocks should size to content. Layouts should adapt to terminal dimensions. Think `Percentage(80)` not `Length(25)`.

2. **Density over whitespace.** Empty space is wasted screen real estate. If a block has space left, fill it with useful content. If content is shorter than the block, shrink the block.

3. **Mouse-first, keyboard-enhanced.** Mouse and touchpad should feel completely natural — scroll any pane, click any row, click any tab. Keyboard shortcuts (j/k, Tab, Enter) are power-user accelerators, not the only way to interact.

4. **Grade is the primary metric.** Remove `evaluation_status` from display everywhere — it's a coarser bucketing of `grade` and adds no information. Grade tells the story.

5. **Refactor as needed.** Break large view files into smaller components. A 500-line rendering file is unmaintainable. Split by widget or section.

---

## Phase 1: Fix what's broken

These are bugs and usability issues that make the current TUI frustrating.

### 1.1 Fix mouse scroll — viewport scrolling, not selection jumping
- [x] Change scroll events to adjust `TableState` offset, not call `next_in_list()`
- [x] The selection stays in place — the viewport moves around it
- [x] Batch trackpad momentum events (if multiple scrolls arrive within 50ms, treat as one larger scroll)
- [x] Mouse scroll should work on whichever pane the cursor is over, regardless of keyboard focus

### 1.2 Implement click-to-select
- [x] Left click on a row in any table selects that row
- [x] Calculate the clicked row index from `mouse.row` relative to the table's rendered area (account for header row and border)
- [x] Clicking in the list pane sets focus to List, clicking in detail pane sets focus to Detail

### 1.3 Implement click-on-tab
- [x] Left click in the tab bar area (rows 0-2) switches to the clicked tab
- [x] Calculate which tab was clicked from `mouse.column` and the tab label positions

### 1.4 Fix posted date display
- [x] Parse ISO timestamps and display as relative time: "3 days ago", "2 weeks ago", "1 month ago"
- [x] Fall back to short date format ("Apr 7") if parsing fails

### 1.5 Fix grade bar colours in company detail
- [x] The `░` empty portion of bars should be removed — just use `█` for filled and empty space for the rest
- [x] The filled portion should use the grade colour from the theme
- [x] Bars should be dynamically sized to fill available width (percentage of container, not fixed character count)

### 1.6 Remove redundant "Evaluation" line from job detail
- [x] Remove the "Evaluation: strong fit" line — grade already conveys this
- [x] Reclaim the space for description content

---

## Phase 2: Fill empty space with useful content

### 2.1 Full job description in job detail pane
- [x] Show the complete `raw_description` after the fit assessment, not just a truncated preview
- [x] Clean up formatting: strip leftover HTML entities (`&amp;`, `&nbsp;`, etc.), normalise whitespace, remove empty lines
- [x] Reorder the detail pane sections: Title → Company → Details (location, posted, grade) → Fit Assessment → Description → Link
- [x] The link moves to the bottom since `o` already opens it — it's reference, not primary content
- [x] The entire detail pane is scrollable so long descriptions are fully accessible

### 2.2 Dashboard — dynamic block sizing
- [x] Replace all `Constraint::Min(N)` with dynamic sizing based on content
- [x] Grade Distribution bars: width should be `(available_width - label_width - count_width - padding) * (count / max_count)` — not a fixed 6 characters
- [x] Pipeline Health bars: same dynamic sizing — bars fill available width proportionally
- [x] Action Items: if there's remaining space after the stats, list the bespoke company names that need manual search
- [x] All blocks expand their content to use available space rather than leaving whitespace

### 2.3 Dashboard Top Roles — full scrollable list
- [x] Show ALL SS, S, and A graded jobs (not just 10 SS)
- [x] Single-line format: `SS  SWE Workers Observability — Cloudflare`
- [x] Make the pane scrollable — j/k works when dashboard has focus, mouse scroll also works
- [x] Add a scroll offset to the dashboard state in App
- [x] This transforms the dashboard from a static stats page into a browseable action list

### 2.4 Company detail — full job list
- [x] Below the grade distribution chart, show ALL jobs for the selected company (not just top 5)
- [x] Single-line format: `SS  SWE Workers Observability`
- [x] Include all grades so the user sees what was filtered out and why
- [x] Scrollable within the detail panel
- [x] Replaces the current "Top Roles" section which only shows 5

### 2.5 Job description formatting
- [x] Strip HTML entities: `&amp;` → `&`, `&nbsp;` → ` `, `&lt;` → `<`, `&#39;` → `'`
- [x] Collapse multiple blank lines into one
- [x] Trim leading/trailing whitespace per line
- [x] Remove any remaining HTML tags that slipped through the strip_html function
- [x] Handle Unicode bullet points and list markers cleanly

---

## Phase 3: New features — QoL

### 3.1 Search/filter (`/`)
- [x] Pressing `/` opens a text input at the bottom of the screen (like vim's search)
- [x] Type to filter the current list by keyword — matches against title, company name, location
- [x] Filter applies instantly as you type
- [x] `Esc` clears the filter and shows all entries
- [x] The filter indicator shows in the status bar: `"engineer" — 142 matches`
- [x] Works in both Companies and Jobs views

### 3.2 Jump-to-grade shortcuts
- [x] In Jobs view, pressing a grade key (shift+S for SS, s for S, a for A, etc.) jumps selection to the first job of that grade
- [x] Visual indicator in the status bar showing which grade section you're in

### 3.3 Sort toggle
- [x] `s` cycles through sort modes: by grade (default) → by company → by date posted → by location
- [x] Current sort mode shown in the table header or status bar
- [x] Persists within the session (resets on restart)

### 3.4 Copy URL to clipboard
- [x] `y` copies the selected job's URL to the system clipboard
- [x] Toast notification: "URL copied to clipboard"
- [x] Uses `pbcopy` on macOS (the platform we're on)

### 3.5 Description indicator in job list
- [x] Add a small indicator in the job table showing whether `raw_description` exists
- [x] `·` (dim) = no description, `✓` (green) = has description
- [x] Helps identify which jobs were graded blind

### 3.6 Focused mode count
- [x] When focused mode is active, the Jobs tab shows: `Jobs (289/712) [FOCUSED]`
- [x] So you always know how many are hidden

### 3.7 Inline grade override
- [x] `g` opens a grade picker popup: `SS S A B C F`
- [x] Select a grade and it updates the job's grade in the DB immediately
- [x] Toast notification: "Grade changed to S"
- [x] Useful for correcting AI grading errors without a full re-grading session

---

## Phase 4: New features — major

### 4.1 Kanban view (4th tab)
- [x] New tab: `[4] Pipeline`
- [x] Three columns: **Watching** → **Applied** → **Interview**
- [x] Each column shows jobs as cards: `SS SWE New Grad Infra — Palantir`
- [x] Cards coloured by grade
- [x] `h`/`l` moves between columns, `j`/`k` moves within a column
- [x] `w`/`a` moves a card between columns (same keys as current decision marking)
- [x] Mouse: click to select, drag would be ideal but complex — start with click + keyboard move
- [x] The pipeline view makes the entire application process visible at a glance

### 4.2 Session summary (pre-generated)
- [x] A natural-language summary stored in a file (`state/tui-summary.md`) that the TUI reads at startup
- [x] Displayed at the top of the dashboard in a highlighted block
- [x] Content like: "17 roles to apply to. Strongest: Palantir New Grad Infra, Jane Street Linux. 6 bespoke companies unsearched. Last search: 2 hours ago."
- [x] Updated by Claude at the end of each session (before TUI launch) — no API key needed since Claude writes the file during the conversation
- [x] The skill/CLAUDE.md should instruct: "Before the user launches the TUI, write a fresh summary to `state/tui-summary.md`"

### 4.3 Application tracker enhancement
- [ ] When marking a job as "applied", prompt for application date (default: today) and optional notes
- [ ] Dashboard shows application pipeline stats: `3 applied · 1 interview · 0 offers`
- [ ] Future: add "interview" and "offer" as decision types in user_decisions
- [ ] This requires a schema consideration — currently decisions are watching/applied/rejected. Adding interview/offer stages would need either new decision types or a separate tracking table.

### 4.4 Export from TUI
- [x] `e` exports the current view to markdown
- [x] In Jobs view (especially focused mode): exports all visible jobs grouped by grade with fit assessments
- [x] In Companies view: exports company list with grades and job counts
- [x] Saves to `exports/YYYY-MM-DD-jobs.md` or similar
- [x] Toast notification: "Exported 289 jobs to exports/2026-04-08-jobs.md"

---

## Phase 5: Code quality and architecture

### 5.1 Refactor TUI into smaller files
- [x] Current structure: `views/dashboard.rs` (200+ lines), `views/companies.rs` (300+ lines), `views/jobs.rs` (250+ lines)
- [x] Split into widget components:
  ```
  src/tui/
  ├── widgets/
  │   ├── mod.rs
  │   ├── grade_bar.rs        # Proportional grade bar rendering (reused in dashboard + company detail)
  │   ├── job_table.rs         # Job list table (reused in jobs view + company detail)
  │   ├── detail_panel.rs      # Scrollable detail panel with sections
  │   ├── search_input.rs      # Filter/search text input widget
  │   ├── toast.rs             # Toast notification rendering
  │   ├── grade_picker.rs      # Grade override popup
  │   └── kanban.rs            # Kanban column layout
  ```
- [x] Each widget is a standalone rendering function that takes data + theme + area
- [x] Views become composition of widgets, not monolithic rendering functions

### 5.2 Dynamic layout system
- [x] Create a layout helper that calculates block sizes based on content
- [x] Input: list of (content_lines, min_height, grow_priority) tuples
- [x] Output: `Vec<Constraint>` that distributes space proportionally
- [x] This replaces all the hardcoded `Constraint::Min(12)` and `Constraint::Length(3)` values
- [x] Blocks with more content get more space; blocks with less shrink to fit

### 5.3 Responsive terminal size handling
- [x] Detect terminal dimensions on each frame
- [x] Below 100 columns: switch to single-column layout (list above detail, not side-by-side)
- [x] Below 80 columns: compact mode with abbreviated labels and no detail panel
- [x] Above 160 columns: wider bars, more columns in the kanban view

---

## Implementation order

Ordered by dependency and impact. Each phase can be shipped independently.

```
Phase 1: Fix what's broken (prerequisites for everything else)
  1.1 Mouse scroll → viewport (highest urgency — currently unusable)
  1.2 Click-to-select
  1.3 Click-on-tab
  1.4 Posted date formatting
  1.5 Bar colour fix
  1.6 Remove redundant evaluation line

Phase 2: Fill empty space (biggest visual impact)
  2.1 Full job description in detail pane
  2.2 Dynamic block sizing for dashboard
  2.3 Scrollable Top Roles with all SS/S/A
  2.4 Full job list in company detail
  2.5 Description text cleanup

Phase 3: QoL features (makes daily use enjoyable)
  3.1 Search/filter (/)
  3.2 Jump-to-grade
  3.3 Sort toggle
  3.4 Copy URL
  3.5 Description indicator
  3.6 Focused mode count
  3.7 Grade override

Phase 4: Major features (transforms the product)
  4.1 Kanban view
  4.2 Session summary
  4.3 Application tracker
  4.4 Export

Phase 5: Code quality (enables maintainability)
  5.1 Widget refactor
  5.2 Dynamic layout system
  5.3 Responsive sizing
```

---

## Completion criteria

- [x] Zero empty space on any page at any terminal size — every pixel shows useful information
- [x] Mouse scroll feels like scrolling a webpage, not teleporting through a list
- [x] Click works on everything clickable — rows, tabs, buttons
- [x] All bars and charts are dynamically sized to fill their container
- [x] Job descriptions are fully readable in the detail pane with clean formatting
- [x] The dashboard is an actionable command centre, not a static stats page
- [x] Search/filter lets you find any job or company in 2 keystrokes
- [x] The kanban view shows your entire application pipeline at a glance
- [x] The codebase is refactored into reusable widgets, not monolithic view files
- [x] The TUI feels as polished as lazygit or btop
