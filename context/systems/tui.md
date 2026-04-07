# TUI System

> Last updated: 2026-04-07. v1 implemented — dashboard, companies, jobs views with detail panels and user decisions.

---

## Scope / Purpose

Cernio's primary user interface — a real-time, interactive terminal dashboard for browsing companies, evaluating jobs, and tracking decisions. Built with ratatui 0.29 and crossterm, designed to feel like lazygit or btop rather than a spreadsheet viewer.

Reads from SQLite (auto-refreshing every 2 seconds) and writes user decisions back, staying in sync with concurrent Claude sessions and script runs.

---

## Boundaries / Ownership

### Module structure

```
src/tui/
├── mod.rs          # Terminal setup/teardown, event loop, public run()
├── app.rs          # App state, View/Focus enums, data models, navigation methods
├── handler.rs      # Key event dispatch — global keys + view/focus-specific handlers
├── theme.rs        # Semantic colour palette using terminal ANSI colours
├── queries.rs      # All DB read queries (companies, jobs, stats, top matches)
└── views/
    ├── mod.rs      # Draw dispatcher, tab bar, status bar, help overlay
    ├── dashboard.rs # Stats overview — universe, ATS coverage, job grades, top matches
    ├── companies.rs # Company table (grade, name, status, jobs, ATS) + detail panel
    └── jobs.rs      # Job table (grade, title, company, location, decision) + detail panel
```

### What the TUI owns

- Terminal lifecycle (raw mode, alternate screen, cursor)
- View rendering and navigation state
- Key event handling and dispatch
- Semantic theme (ANSI colour mapping)
- DB read queries for display
- Writing user decisions to `user_decisions` table

### What the TUI does not own

- Company or job data — read-only from SQLite
- Evaluation logic — that's Claude's domain
- ATS fetching — that's the script layer
- Schema or migrations — that's the database layer

### Dependencies

```toml
ratatui = { version = "0.29", features = ["crossterm"] }
```

Crossterm is re-exported through ratatui — no separate dependency needed. Also uses `rusqlite`, `chrono` from the shared Cargo.toml.

---

## Current Implemented Reality

### Layout

```
┌─ cernio ─── Dashboard │ Companies (9) │ Jobs (25) ──────────────┐
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Content area — varies by active view:                              │
│                                                                     │
│  Dashboard: two-column stats grid                                   │
│  Companies/Jobs: master/detail split (40-45% list / 55-60% detail)  │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ j/k navigate  Tab detail  Enter view jobs  ? help  q quit          │
└─────────────────────────────────────────────────────────────────────┘
```

Three layers: tab bar (3 rows), content area (fills remaining), status bar (1 row).

### Views

| View | Content | Layout |
|------|---------|--------|
| **Dashboard** | Universe stats with grade bars, ATS coverage breakdown, job grade/eval breakdowns, top matches (SS/S/A) | Two-column grid of bordered stat blocks |
| **Companies** | Table: grade, name, status, job count (with fit count), ATS provider. Detail: description, grade reasoning, relevance, ATS info, job summary | 40% list / 60% detail master-detail split |
| **Jobs** | Table: grade, title, company, location, decision status. Detail: location, remote policy, posted date, grade, evaluation, fit assessment, URL | 45% list / 55% detail master-detail split |

### Colour strategy

All colours use the terminal's 16-colour ANSI palette — never hardcoded hex. This means:

- The TUI inherits the user's terminal theme (Catppuccin, Dracula, Nord, etc.)
- Terminal transparency works (backgrounds use `Color::Reset`)
- Works in 16-colour, 256-colour, and truecolor terminals

Semantic colour assignments:

| Element | Colour mapping |
|---------|---------------|
| **Grade SS** | Magenta, bold |
| **Grade S** | Green, bold |
| **Grade A** | Cyan |
| **Grade B** | Yellow |
| **Grade C** | Red |
| **Grade F** | Dark gray |
| **strong_fit** | Green, bold |
| **weak_fit** | Yellow |
| **evaluating** | Yellow, bold |
| **pending / no_fit** | Dark gray |
| **watching** | Cyan |
| **applied** | Green, bold |
| **rejected** | Dark gray |
| **Focused border** | Cyan |
| **Unfocused border** | Dark gray |
| **Headers** | Cyan, bold |

### Navigation

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Move selection in list, scroll in detail |
| `g` / `G` | Jump to top / bottom |
| `Enter` / `l` / `→` | Drill into company → show its jobs |
| `Esc` / `h` / `←` | Go back (jobs → companies, detail → list) |
| `Tab` | Toggle focus between list and detail panel |
| `1` `2` `3` | Switch view: Dashboard / Companies / Jobs |
| `w` | Mark selected job as "watching" |
| `a` | Mark selected job as "applied" |
| `x` | Mark selected job as "rejected" |
| `o` | Open selected item's URL in default browser |
| `?` | Toggle help overlay |
| `q` / `Ctrl+C` | Quit |

### App state model

The `App` struct holds all runtime state:

- `view: View` — current active view (Dashboard, Companies, Jobs)
- `focus: Focus` — whether the list or detail panel has focus
- `companies: Vec<CompanyRow>` — cached company data from DB
- `jobs: Vec<JobRow>` — cached job data, optionally filtered by company
- `stats: DashboardStats` — aggregate statistics
- `company_state / job_state: TableState` — ratatui selection tracking
- `detail_scroll: u16` — scroll offset for the detail panel
- `job_filter_company: Option<i64>` — company drill-down filter

### Entry point

`cernio tui` — dispatched from `main.rs`, calls `tui::run()` which sets up the terminal, creates the App, runs the event loop, and restores the terminal on exit (even on error).

---

## Key Interfaces / Data Flow

```
SQLite ──[poll every 2s]──► App state ──[every frame]──► Ratatui widgets
                                 ▲
                                 │
                      user decisions (w/a/x keys)
                      write directly to user_decisions table
```

### DB → TUI (read path)

`src/tui/queries.rs` provides three query functions:

| Function | Returns | Query pattern |
|----------|---------|---------------|
| `fetch_companies()` | `Vec<CompanyRow>` | Joins `company_portals` for primary ATS, subqueries for job/fit counts, sorted by grade |
| `fetch_jobs(filter)` | `Vec<JobRow>` | Joins companies for name, subquery for latest decision, optional company_id filter, sorted by grade |
| `fetch_stats()` | `DashboardStats` | Multiple aggregate queries: counts by grade/status/evaluation, ATS coverage, top matches |

Each refresh opens a fresh `Connection` — acceptable for SQLite's low overhead and avoids connection lifetime complexity.

### TUI → DB (write path)

Only user decisions: `INSERT INTO user_decisions (job_id, decision, decided_at)`. Triggered by `w`/`a`/`x` keys. Multiple decisions per job are allowed (history), and the TUI queries the latest one for display.

### Event loop

100ms poll rate for responsive keyboard input. 2-second refresh interval for DB data. Ratatui's immediate-mode rendering diffs against the previous frame to minimise terminal writes.

---

## Implemented Outputs / Artifacts

- `src/tui/` — 9 Rust source files (mod, app, handler, theme, queries, views/mod, views/dashboard, views/companies, views/jobs)
- CLI command `cernio tui` — launches the TUI
- User decisions written to `state/cernio.db` `user_decisions` table

---

## Known Issues / Active Risks

- The `open` command for URLs is macOS-specific. Needs `xdg-open` on Linux.
- No graceful handling of very long company/job names in narrow terminals — truncation is basic.
- Three dead-code warnings on struct fields kept for future use (`company_id`, `raw_description` on JobRow, `pending_companies` on DashboardStats, `bold` on Theme).

---

## Partial / In Progress

Nothing — v1 is feature-complete for its scope.

---

## Planned / Missing / Likely Changes

### Functional features (v2)

- **Activity / progress view** — fourth tab showing live operation progress, modelled on Homebrew's `brew upgrade` UI with status indicators and completed items floating to the top. Depends on the pipeline CLI existing and reporting structured progress events via `mpsc::Receiver<ProgressEvent>`.
- **Filtering and search** — `/` opens a text input to filter lists by name/title. Needs `tui-input` crate.
- **Sorting** — `s` opens a sort picker. Current sort shown in column header.
- **Database cleanup** — `D` key opens confirmation popup showing what would be removed (F/C grades, stale >14d). Full design in `notes/db-maintenance.md`.
- **Export** — `e` triggers markdown export of the current view.
- **Job description preview** — full raw description in a popup. Currently only fit_assessment is shown.
- **Inline grade override** — override a job's grade directly from the TUI.
- **Focused mode** — `f` hides F/C grades, shows only SS/S/A/B.
- **Mouse support** — click to select, scroll wheel, click tabs.

### Visual enhancements (v2+)

These ideas aim to elevate the TUI from functional to genuinely delightful — all implementable with ratatui's existing widgets:

| Enhancement | What it does | Implementation approach |
|-------------|-------------|------------------------|
| **Donut/pie charts** | Grade distribution as braille-character arcs on the dashboard | `ratatui::widgets::canvas::Canvas` with `Circle` shapes |
| **Fit radar chart** | Spider chart of fit dimensions in job detail (career ceiling, stack match, sponsorship) | Canvas with lines to axis points. Needs structured fit dimension data |
| **Sparkline history** | Thin strips showing jobs discovered / strong fits / applications over 14 days | Built-in `Sparkline` widget, data from `discovered_at` bucketing |
| **Colour tag pills** | Sector/skill tags as coloured inline badges: `[infrastructure]` `[Rust]` | `Span` with `bg` colour. 256-colour palette with ANSI fallback |
| **Animated spinners** | Evaluating status cycles `◐ → ◑ → ◒ → ◓` instead of static text | `frame_count % 4` selects spinner character |
| **Kanban view** | Three columns: Unwatched / Watching / Applied with grade-coloured cards | Fourth tab, three `List` widgets, `h`/`l` between columns |
| **Tier ribbon** | Coloured left-border instead of grade column: `▎ Cloudflare` | Grade-coloured `▎` character prefix per row |
| **Toast notifications** | Ephemeral bottom-right messages: `✓ Marked applied` (3s fade) | `Vec<Toast>` with timestamps, floating widget |
| **Section folding** | Collapsible detail sections with `▸`/`▾` toggles | Section state in App, toggle on Enter/Space |
| **At-a-glance summary** | Natural-language line: `9 companies · 25 jobs · 3 strong matches` | `Paragraph` at dashboard top |
| **Search pulse** | "Last search: X ago" coloured by freshness (green/yellow/red/dim) | Timestamp comparison with colour thresholds |

---

## Durable Notes / Discarded Approaches

- **ANSI palette over truecolor:** Deliberately chose the 16-colour ANSI palette instead of truecolor or 256-colour. This makes the TUI inherit the user's terminal theme rather than imposing its own. Tools like btop and lazygit do the same — they look native in any terminal setup.
- **Fresh connection per refresh:** Opens a new `Connection` on each 2-second refresh rather than holding one open. SQLite opens are fast and this avoids connection lifetime complexity in the App struct. Could be optimised later if profiling shows it matters.
- **`x` for reject instead of `r`:** Used `x` because `r` would conflict with a future refresh key. Also maps intuitively to "cross out" / reject.
- **No mouse in v1:** Kept the initial version keyboard-only to avoid the complexity of mixed input modes. Mouse support is planned but not critical — the target users (developers) are comfortable with keyboard navigation.

---

## Obsolete / No Longer Relevant

Nothing at this stage.
