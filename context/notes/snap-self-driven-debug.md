# `cernio snap` as self-driven visual verification

> Captures the design intent behind `cernio snap` — not as a debug-output tool, but as the loop that lets Claude do its own visual observation pass on web-UI changes rather than asking the user to verify.

## The intent

Before `cernio snap` existed, web-frontend changes required the user to run `cernio web`, open Chrome, look at the screen, and report back. Round-trip latency was minutes per iteration and the user had to be present. Claude's tight feedback loop on visual changes was missing — there was no way to "see what changed."

`cernio snap` closes that loop:

1. Claude makes a CSS or layout change.
2. Claude runs `cernio snap` (or hits `POST /debug/snap-all` from the floating button).
3. The CLI spawns an ephemeral axum server on a free port (7879+), drives headless Chrome via chromiumoxide over every web tab, captures full-page + per-section + viewport-slice PNGs into `/tmp/cernio-debug/<YYYYMMDD-HHMMSS>/`, then tears the server down.
4. Claude reads the PNGs with the image-reading tool.
5. Iterate without asking the user to verify visually.

This converts web-UI work from "ask the human to look" into "look myself, then ask the human only when the result is committable."

## When to use it

- After any non-trivial CSS, layout, or chart change.
- After any handler change that affects rendered markup (chip filter strips, drawer fragments, decisions page).
- Before declaring a visual change "done" — at minimum one snap of the affected page.
- After fixing a previously-snapped visual bug — confirm the fix with a fresh snap, not by reading the old snap and assuming.

## What it captures

Per page, three image kinds:

| Kind | What | Why |
|---|---|---|
| `<page>/full.png` | Entire page in one image | Holistic check; layout, spacing, vertical rhythm |
| `<page>/viewport-NN.png` | Page sliced into viewport-tall (1000 px) bands | Lets the image-reader read very long pages within its 2000×2000 limit |
| `<page>/pane-NN-<label>.png` | Each `<section.panel>` + KPI strip + filter bar + lane legend captured tight | Per-component scrutiny without surrounding noise |

The `--temporal` flag re-captures every artefact twice with a 3-second gap (`t0/` then `t1/`). Useful for spotting marquee animations, count-up tickers settling, or other time-dependent layout.

## The `PAGES` const (the truth of what gets snapped)

`src/web/debug_snap.rs::PAGES` is the canonical list of routes to snap. As the web app grows, this list grows with it. Current entries:

```rust
const PAGES: &[(&str, &str)] = &[
    ("dashboard",           "/"),
    ("companies",           "/companies"),
    ("jobs",                "/jobs"),
    ("jobs-filtered",       "/jobs?lane=hft"),
    ("companies-filtered",  "/companies?lane=hft"),
    ("jobs-lanes",          "/jobs?view=lanes"),
    ("companies-lanes",     "/companies?view=lanes"),
    ("decisions",           "/decisions"),
    ("activity",            "/activity"),
];
```

The list intentionally includes filtered variants (`?lane=hft`) and the lane-columned view (`?view=lanes`). A page with multiple visual states does not get full coverage from snapping just the unfiltered version.

> **When adding a new page or a new query-string visual state, extend `PAGES`.** A page that does not appear here cannot be snapped by `snap-all`. Forgetting to add it means visual regressions on that page surface only when a human looks at it — which is the failure this tool exists to prevent.

## Environment dependency

`CHROME_PATH` is hardcoded to `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome` — macOS only. The same chromiumoxide dependency that powers `src/autofill/` is reused here, so no new dependency, but the path assumption means the snap CLI breaks on Linux / non-default Chrome install paths.

If snap ever moves to Linux, replace the hardcoded constant with an env-var lookup (`CERNIO_CHROME_PATH`) defaulting to `which google-chrome` / `which chromium`.

## Hidden during capture

The capture script sets `display: none` on `#snap-all` (the floating button), `#snap-toast` (the result toast), and `#ambient-canvas` (the background-constellation effect) before each capture. This prevents the debug surface from polluting the PNGs and keeps the ambient animation from making temporal-mode diffs noisy.

## Self-driven loop discipline

> **Run `cernio snap` yourself. Do not ask the user to verify visually when you have a snap loop available.** Asking the user to "open `cernio web` and tell me how it looks" is the wrong default when the snap loop costs ~20 seconds and produces ground-truth PNGs.

The corollary is the verification gate: a web-UI change is not done until at least one snap PNG has been read and confirms the intended visual. Reading the old snap and assuming the fix worked is the failure mode this discipline guards against.

## See also

- `src/web/debug_snap.rs` — the CLI + `snap_all` HTTP handler
- `src/main.rs::cmd_snap` — the `cernio snap` subcommand wiring
- `context/notes/web-frontend-architecture.md` §"`cernio snap` CLI for visual debugging" — the technical mechanics
- `static/js/debug.js` — the floating-button trigger
