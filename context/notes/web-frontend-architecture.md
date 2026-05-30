# Web Frontend Architecture

> Project-internal scope. Sister doc to `Projects/Cernio/Systems/Web.md` in LifeOS (project-meta scope). Both maintained independently per the dual-write principle — no cross-link sync.

The web frontend ships alongside the TUI as a second UI surface. Same SQLite DB, different presentation layer. Boot via `cernio web` (localhost:7878).

## Stack

- **axum** — HTTP router
- **maud** — server-rendered HTML via macros; type-checked attribute syntax
- **HTMX 1.9** — declarative inline writes via `hx-post` / `hx-swap`
- **ECharts 5.4** — all chart rendering (canvas)
- **chromiumoxide** — headless Chrome for `cernio snap` (already in deps for autofill)
- **tower-http** — `ServeDir` for `/static`

No build step. Hand-written CSS + vanilla JS. All loaded from `/static`.

## Asset bundle architecture

The single biggest structural decision in the 2026-05-30 redesign was splitting monolithic `style.css` + `app.js` into shared + per-page bundles loaded via `PageAssets`:

```rust
// src/web/templates.rs
pub struct PageAssets<'a> {
    pub page_css: Option<&'a str>,
    pub page_js: Option<&'a str>,
}

pub fn page_with(title: &str, active: &str, assets: PageAssets, body: Markup) -> Markup {
    html! {
        // ... shared bundle always loaded ...
        link rel="stylesheet" href="/static/css/base.css";
        // ... 9 more shared CSS ...

        // Per-page bundle when provided
        @if let Some(css) = assets.page_css {
            link rel="stylesheet" href=(css);
        }
        // Same for JS
    }
}
```

Each handler chooses its assets:

```rust
// src/web/handlers/jobs.rs
PageAssets::css_js("/static/css/jobs.css", "/static/js/jobs.js")
```

### Shared bundle inventory

| File | Role |
|---|---|
| `base.css` | Design tokens, reset, typography, ambient canvas styling, grid primitives, page entry transition |
| `motion.css` | 4 motion archetypes (lift / flash / pulse / drift), marquee, ripple, `prefers-reduced-motion` cascade |
| `components.css` | Panel, KPI strip, lane chip, grade pill, button, table, role-row, activity-row, day-header, status pill |
| `chrome.css` | Topbar, tabs, brand, status strip, lane-legend topbar popover |
| `filters.css` | `.filter-strip` + `.chip` variants (lane / grade / plain / segmented) + filter summary row |
| `debug.css` | Snap button + toast |
| `ops.css` | Ops menu button + popover + chip-style preview detail |
| `drawer.css` | Side drawer slide-in + backdrop |
| `cmdk.css` | Command palette modal |
| `presets.css` | Saved-searches dropdown |

### Shared JS inventory

| File | Role |
|---|---|
| `core.js` | Ambient constellation, count-up tickers, marquee on overflow, click ripple, Apply-button capture-phase `window.open`, mountPanes |
| `charts.js` | Shared ECharts theme + `window.cernio.bootEchart(kind, builder)` helper |
| `debug.js` | Snap-all button trigger + toast |
| `ops.js` | Ops panel open/close + preview load + run + structured detail rendering |
| `drawer.js` | URL-persistent drawer (`?detail=kind-N`), click delegation, HTMX re-process |
| `cmdk.js` | Cmd-K palette + g-leader keyboard shortcuts |
| `presets.js` | Saved searches via localStorage |

### Per-page bundles

Each page has one CSS + one JS file with page-specific layout and chart bootstraps. Naming: `static/css/<page>.css` + `static/js/<page>.js`. Pages: dashboard, jobs, companies, decisions, activity.

## Routes

```
GET  /                          dashboard
GET  /jobs                      jobs (filter axes: lane, grade, decision, archive, ATS, posted, sponsor, company)
GET  /companies                 companies (filter axes: lane, grade, status, ATS, sponsor, location, has_jobs)
GET  /decisions                 decisions (filter: kind = all|watching|applied|interview|rejected)
GET  /activity                  activity (filter: window=7d|30d|90d, raw=1)
POST /jobs/:id/decision         record user decision; returns updated decision-buttons fragment
POST /activity/group            (placeholder, no-op)
GET  /detail/job/:id            HTML fragment for the drawer
GET  /detail/company/:id        HTML fragment for the drawer
GET  /api/stats.json            small KPI snapshot
GET  /api/search-index.json     Cmd-K palette index (~200KB)
GET  /ops/clean/preview         dry-run preview JSON
POST /ops/clean/run             executes pipeline::clean
GET  /ops/format/preview        dry-run preview JSON
POST /ops/format/run            executes capped format pass
POST /debug/snap-all            invokes cernio snap from the floating button
GET  /static/*                  served via tower_http::ServeDir
```

## Lane and grade theming

Single source of truth for lane colours: `src/data/lane.rs::lane_hex(key)`. The same hex values are used by:

- TUI (via `tui/theme` mapping)
- Web CSS chips (via `style="--lane-color: <hex>"` inline style)
- ECharts series colours (via JSON island data + per-page chart builder)
- Ops menu chip rendering in `ops.js` (via embedded `LANE_HEX` constant — duplicated by necessity since ops menu loads on every page)

Grade colours live in `:root` CSS custom properties: `--grade-ss` through `--grade-f`.

## JSON islands

Server-side handler builds a JSON blob, emits via `json_island(kind, &value)`:

```rust
// In handler
let lane_grade_json = serde_json::json!({
    "lanes": LANE_KEYS.iter().map(|k| lane_label(k)).collect::<Vec<_>>(),
    "active": ...,
    "archived": ...,
});

html! {
    div #chart-jobs-lane-grade .chart.chart-md {}
    (json_island("jobs-lane-grade", &lane_grade_json))
}
```

Per-page JS reads + bootstraps:

```javascript
window.cernio.bootEchart('jobs-lane-grade', (data, theme) => ({
    backgroundColor: theme.bg,
    series: data.lanes.map(...),
}));
```

Convention: chart element `id="chart-<kind>"`, data element `id="data-<kind>"`. `bootEchart` looks both up and feeds the data through the builder.

## Filter chip system

Filter strips on `/jobs` and `/companies`. Each axis is rendered server-side as a row of hyperlink chips:

```rust
struct AxisDef {
    key: &'static str,
    label: &'static str,
    chips: &'static [(&'static str, &'static str)],  // (value, display)
    kind: ChipKind,
}

enum ChipKind {
    Lane,       // coloured pill with dot
    Grade,      // outline + grade colour class
    Plain,      // monochrome enum
    Segmented,  // mutually-exclusive segmented control
}
```

`render_axis` switches on `kind`:
- `Lane` / `Grade` / `Plain` → `<div class="chips">` with `<a class="chip chip-lane|chip-grade|chip-plain ...">`
- `Segmented` → `<div class="seg-group">` with `<a class="seg">`

Toggle URL is built server-side per chip — clicking flips that value in the URL's CSV. No client-side JS for filtering.

URL shape: `?lane=hft,ai-ml&grade=SS,S&decision=none&archive=archived`. Multi-value via CSV. Archive defaults to `active` when missing (and is stripped from URLs when only `active` is selected to keep URLs clean).

All analytics on the page recompute from the filtered job/company list — heatmap, freshness histogram, decision funnel, top companies, top titles — not from global tables. Filter changes one URL param; whole page reacts.

## Detail drawer

Drawer shell lives in `templates.rs` chrome (always present, hidden by default):

```html
<div id="detail-drawer-backdrop" class="hidden"></div>
<aside id="detail-drawer" class="detail-drawer hidden">
  <header class="drawer-head">
    <span class="drawer-kind"></span>
    <button class="drawer-close">×</button>
  </header>
  <div class="drawer-body">loading…</div>
</aside>
```

`drawer.js` handles:
- On `DOMContentLoaded`: parse `?detail=kind-N` from URL; if present, fetch + open
- Delegation: any click on `.row-clickable` or `[data-detail]` opens drawer (excludes button / link / HTMX targets)
- `pushState` on open; `replaceState` on close — URL persistence
- Fetch `/detail/<kind>/:id` → HTML fragment → swap into `.drawer-body` → `htmx.process()` to re-bind HTMX
- Close on Esc / backdrop click / X
- In-drawer clicks on `.drawer-job-item` swap drawer to job detail without reload

Detail handlers return ONLY the inner body markup (no chrome, no `<html>`).

## Cmd-K palette

`/api/search-index.json` returns `{companies: [...], jobs: [...]}` — id, name, kind, lane, grade, url. ~200KB total over localhost. Cached client-side for 5min.

Ranking: exact name match → prefix match → substring match → grade tie-break (SS > S > A > B > C > F). Cap 30.

`>` prefix surfaces a static command list (dashboard / jobs / companies / activity / decisions / untouched SS+S / watching / applied / clean db hint).

Global keyboard shortcuts (in same `cmdk.js`):
- Cmd/Ctrl-K → toggle palette
- `/` → open empty
- `?` → open with `>` prefilled
- `g` leader + 1.5s timeout: `d/c/j/a/x` jumps to tab
- Esc cascades: palette → drawer → ops panel → leader

Suppressed inside inputs / textareas / contenteditable.

## `cernio snap` CLI for visual debugging

`src/web/debug_snap.rs` + `src/main.rs::cmd_snap`. The CLI:
1. Spawns ephemeral axum server on a free port (7879+)
2. Polls until server is responding
3. Drives headless Chrome (chromiumoxide) over all 5 tabs
4. Sets viewport 1600×1000 at 1× DPR (image-reader 2000×2000 limit)
5. Hides `#snap-all`, `#snap-toast`, `#ambient-canvas` to declutter screenshots
6. For each tab: full-page PNG + per-section panel PNGs + viewport-height slices
7. Optional `--temporal`: re-captures into `t0/` + `t1/` with 3s gap
8. Writes to `/tmp/cernio-debug/<YYYYMMDD-HHMMSS>/`
9. Tears down the server

Used both as a self-service CLI and as the floating `snap all` button (POST `/debug/snap-all` from `debug.js`).

## Verified bugs fixed this session

| Bug | Fix |
|---|---|
| Maud `selected=(false)` browser-truthy | Use `selected[bool]` brackets — see `maud-attribute-gotchas.md` |
| Marquee pre-layout misfire | Defer detection to `window.load` + skip if `containerW < 50` |
| Component CSS dropped during modular split | Restored `.role-row`, `.activity-row`, `.day-header` etc. to `components.css` |
| Activity charts dominated by raw.* migration triggers | Add `event_type NOT LIKE 'raw.%'` AND `source NOT LIKE '%backfill%'` AND `source NOT LIKE '%migration%'` to all activity queries + KPI |
| Apply button didn't open URL | Capture-phase `window.open` in `core.js` before HTMX bubble-phase handler |
| Pipeline funnel diamond (mixed company + job counts) | Reframed to monotone-decreasing company-only stages |
| Date-axis interpolated ramp across empty days | `date_axis_n(days)` padding so missing days are zero, not interpolated |

## See also

- `context/notes/maud-attribute-gotchas.md` — `selected[bool]` syntax + browser truthy semantics
- `context/architecture.md` §"Web frontend layer" — module + bundle structure (project-level architecture)
- `src/web/` — implementation
- `src/data/analytics.rs` — query functions feeding charts
- LifeOS `Projects/Cernio/Systems/Web.md` — project-meta sister doc (Caner's cross-project view)
