# Autofill Feature — Status and Known Issues

**Status: Scaffolded but non-functional.** The architecture is in place but form filling doesn't work on real Greenhouse forms. Needs DOM debugging.

---

## What's built

| Component | Status | Location |
|-----------|--------|----------|
| `src/autofill/mod.rs` | Working | Profile loader, provider dispatch, package JSON parsing |
| `src/autofill/common.rs` | **Broken** | Chrome launch works, field filling via JS injection doesn't |
| `src/autofill/greenhouse.rs` | **Broken** | Selectors untested against real Greenhouse DOM |
| DB migration 006 | Working | `application_packages` table created and functional |
| TUI `p` key | Working | Spawns autofill, reads package from DB, marks applied |
| TUI `●` indicator | Working | Yellow dot in "P" column for jobs with prepared packages |
| Package cleanup | Working | Deletes package on "applied" decision |
| `prepare-applications` skill | Designed | `skills/prepare-applications/SKILL.md` |
| `chromiumoxide` dependency | Working | Chrome launches in headed mode, navigates to URL |

## What's broken

### 1. Form field filling doesn't work

The JavaScript value injection approach (`el.value = "..."; el.dispatchEvent(...)`) doesn't trigger Greenhouse's React state management. React-controlled inputs ignore direct `.value` assignment — they need synthetic React events or `nativeInputValueSetter` tricks.

**Fix approach:** Use Chrome DevTools Protocol's `Input.insertText` or `DOM.setNodeValue` instead of JavaScript injection. Or use the React-aware approach:

```javascript
const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
    window.HTMLInputElement.prototype, 'value'
).set;
nativeInputValueSetter.call(el, value);
el.dispatchEvent(new Event('input', { bubbles: true }));
```

### 2. CSS selectors untested

The selectors in `greenhouse.rs` (`input#first_name`, `input[name='email']`, etc.) were written from documentation, not from inspecting real Greenhouse forms. Need to:
1. Open a real Greenhouse application form
2. Inspect the DOM to find actual field selectors
3. Update `greenhouse.rs` accordingly

### 3. LinkedIn URL was truncated on first test

`type_str` (character-by-character typing) collided with Chrome's autofill popup. Switched to JS injection which doesn't work for React reasons above. The `Input.insertText` CDP approach should solve both problems.

### 4. Console logs corrupted TUI

Fixed — removed `eprintln!` calls that wrote to stderr while TUI was in raw mode. Unsupported providers now fall back to regular `open` command.

## Architecture decisions (preserve these)

- **Packages in DB, not files** — `application_packages` table with JSON answers. Clean, queryable, auto-cleanup on applied.
- **Skill generates answers, binary fills forms** — intelligence in Claude Code session (free), mechanical filling in Rust (fast). No API key needed.
- **Per-provider modules** — `src/autofill/greenhouse.rs` pattern, ready for `lever.rs`, `ashby.rs` etc.
- **TUI indicator** — yellow `●` in "P" column shows package readiness at a glance.
- **Cleanup on applied** — package auto-deleted from DB when job is marked applied via any path (`o`, `p`, `a`).

## Next steps to make it work

1. Open a Greenhouse form, inspect the DOM, find real selectors
2. Replace JS `el.value =` with React-compatible value setting or CDP `Input.insertText`
3. Test on Cloudflare intern (job 371) — package already prepared in DB
4. Once Greenhouse works, add Lever and Ashby modules
5. Consider the `--disable-blink-features=AutomationControlled` flag effectiveness — the "Chrome is being controlled" banner appeared
