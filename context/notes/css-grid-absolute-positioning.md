# CSS Grid + absolute positioning trap (and the row hover bar)

> Surfaced in commit `3baaea0` (2026-05-31, "fix(web): row hover + ambient glow — actual structural fix"). Three real structural bugs that earlier patches kept missing. Captured here because the underlying CSS-spec rules are not obvious from the symptoms and earlier sessions spent significant time on cosmetic patches that papered over the wrong layer.

## 1. `grid-area:1/1/-1/-1` on absolute children pins them to the grid track, not the row

### Symptom

The ambient lane glow + left-edge lane accent on every `.row-clickable` (jobs, companies, decisions, next-actions) sat at ~22 px tall and started at `left:-12 px` instead of filling the row's padding-box. Patches tried offsetting `left`/`top` further; the symptom never properly disappeared.

### Root cause

The absolute children had `grid-area: 1/1/-1/-1` set. Per the CSS Grid spec, **absolute children of grid containers are not grid items**. The `grid-area` declaration *was* honoured to pick a containing block — but the row's implicit grid track sized to the lane-badge column (~22 px tall) instead of the row's padding-box. The horizontal `-12 px` hack was a manual correction for the same thing: the implicit grid column started inside the row's padding, not at its border edge.

### Fix

Remove `grid-area` entirely. With `grid-area: auto`, the absolute child's containing block falls back to the row's **padding-box** as the spec defines for absolutely-positioned children of any element with `position: relative`. Now:

- `top: 0; bottom: 0` → full padding-box height including the 9 px row padding,
- `left: 0` → the row's actual border-left edge (no more `left:-12 px` hack).

### Generalised rule

> **If you give an absolutely-positioned child of a grid container an explicit `grid-area`, you are pinning it to a grid track, not to the parent's padding-box.** This is almost never what you want for full-bleed accent overlays. Use `position: absolute` + `inset: 0` (or `top/bottom/left/right` literals) with `grid-area: auto`.

The trap is sticky because the rule applies *only* when an explicit `grid-area` is set — bare absolutely-positioned children behave fine. The fix looks like deleting a line that "shouldn't matter."

### Audit pattern

```
grep -rn "grid-area" static/css/ | grep -v "grid-area: auto"
```

Any match on a `position: absolute` element is suspect.

---

## 2. `display: block` on an inline link → hover border-bottom becomes a horizontal bar

### Symptom

Hovering a job row produced a thick horizontal bar under the row title, running the full width of the title column (~500 px). It read as "the row got underlined," not as "the title is a link."

### Root cause

`.row-title-text` had `display: block`. The hover style had `border-bottom: 1px solid var(--accent-2)`. A block element's border spans the full content-box width — the entire `1fr` title column. The same `border-bottom` on an inline element would only have spanned the actual text glyphs.

### Fix

Drop `border-bottom` from the hover state entirely. Colour change (`color: var(--accent-2)`) is sufficient hover affordance for a row that already has the lane-accent strip glowing on hover.

### Generalised rule

> **Borders on block-display links span the parent's full width. Borders on inline-display links span only the text.** If you want a "text-underline-on-hover" feel, either use `text-decoration: underline` (which respects glyphs) or keep the link inline. Never reach for `border-bottom` on a `display: block` link expecting it to behave like an underline.

---

## 3. Variable row heights break percentage-based accent geometry

### Symptom

The lane-accent ellipse looked different between Jobs rows (taller — Apply/Watch/Reject button column adds height) and Companies rows (shorter — no decision buttons).

### Root cause

The previous geometry was `top: 4 px; bottom: 4 px;` — a height that stretches with the row. Two rows of different heights produced two different visual shapes for the same conceptual accent.

### Fix

Fixed dimensions: `140 px × 28 px`, centred vertically via `top: 50%; transform: translateY(-50%);`. Same visual now across jobs, companies, decisions, next-actions.

### Generalised rule

> **For decorative accents that should look identical across visually-related rows, fix the size in pixels and centre it. Stretchy `top/bottom` accents inherit row-height differences and will look like different shapes.**

---

## See also

- `context/notes/web-frontend-architecture.md` — full web-frontend architecture overview
- `static/css/rows.css` — the file these three bugs lived in
- `static/css/tables.css` — companion grid declarations
- Commit `3baaea0` — the structural fix with the full diagnostic in the body
