# Maud attribute gotchas

## `selected=(bool)` vs `selected[bool]`

**`selected=(false)` does not do what you want.** Maud's `attr=(expr)` ALWAYS emits the attribute with the stringified expression as the value. So:

```rust
option value=(key) selected=(false) { "fintech" }
```

renders as:

```html
<option value="fintech" selected="false">fintech</option>
```

Browsers treat **any presence** of the `selected` attribute as truthy. So `selected="false"` selects the option. When you render multiple `<option>` elements each with `selected="false"` or `selected="true"`, the browser picks the LAST `<option>` with the attribute present — which is "fintech" (the last `LANE_KEYS` entry).

This is the bug behind: *"When I click on fintech and swap to 'All', it just goes back to fintech."*

### Fix: conditional attribute brackets

Maud's `attr[bool_expr]` syntax conditionally emits the empty attribute IF the bool is true, otherwise omits it:

```rust
option value=(key) selected[q.lane.as_deref() == Some(*key)] { "fintech" }
```

Renders as `<option value="fintech" selected>fintech</option>` when the condition is true, `<option value="fintech">fintech</option>` when false. That's the correct HTML form for browser selection semantics.

### Applies to other HTML boolean attributes

Same gotcha for any attribute the browser treats as "present means true":

- `selected` (on `<option>`)
- `checked` (on `<input type="checkbox|radio">`)
- `disabled` (on form controls)
- `readonly` (on form controls)
- `required` (on form controls)
- `multiple` (on `<select>`)
- `open` (on `<details>`, `<dialog>`)
- `hidden` (on any element)
- `autofocus` (on form controls)

Use `attr[bool_expr]` for all of these.

### When `attr=(value)` IS correct

For attributes where the VALUE matters (not just presence): href, class, id, src, type, name, value, data-*, aria-*, style, title, etc. There `selected=("foo")` renders `selected="foo"` which is intended.

The gotcha is specifically: boolean attributes where HTML treats presence-as-truthy, used with maud's `=(...)` syntax. Those four conditions together produce the bug.

## Empirical evidence (2026-05-30)

Filter dropdowns on `/jobs` reverting to last value after switching to "All" was traced to:

```rust
// BUG
option value="" selected=(q.lane.as_deref().map(|s| s.is_empty()).unwrap_or(true)) { "All" }
@for key in LANE_KEYS.iter() {
    option value=(key) selected=(q.lane.as_deref() == Some(*key)) { (key) }
}
```

Every option emitted `selected="true"` or `selected="false"`. Browser picked last-in-source-order "fintech".

```rust
// FIX
option value="" selected[q.lane.as_deref().map(|s| s.is_empty()).unwrap_or(true)] { "All" }
@for key in LANE_KEYS.iter() {
    option value=(key) selected[q.lane.as_deref() == Some(*key)] { (key) }
}
```

Verified by `curl /jobs?lane=fintech&grade=B` → only `<option ... selected>` for the active value; all others bare.

## Audit pattern

Searchable: `grep -rn 'selected=(\|checked=(\|disabled=(\|open=(' src/web/`. Any match is potentially buggy. Convert each to `attr[...]` form.

## See also

- `context/notes/web-frontend-architecture.md` §"Verified bugs fixed this session" — table of all bugs landed
- maud docs: [Empty attributes](https://maud.lambda.xyz/elements-attributes.html) — official documentation of the bracket syntax
