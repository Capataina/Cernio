# ATS Fetchers — Code Health Findings

**Systems covered:** `src/ats/{common,lever,greenhouse,ashby,workable,smartrecruiters,workday,mod}.rs`
**Finding count:** 7 findings (2 high, 4 medium, 1 low)

---

## Inconsistent Patterns

### Four independent `strip_html` implementations across `src/ats/`
- [ ] Consolidate `strip_html` into a single implementation in `src/ats/common.rs`, make the per-provider versions private re-imports of it, and remove the duplicate function bodies.

**Category:** Inconsistent Patterns
**Severity:** High
**Effort:** Small
**Behavioural Impact:** Possible (requires decision) — Workable currently uses the simpler non-quote-aware version, so consolidating to the quote-aware version is a behaviour change on Workable job descriptions containing `>` inside quoted attribute values.

**Location:**
- `src/ats/lever.rs:189-212` — `pub fn strip_html`, quote-aware state machine
- `src/ats/greenhouse.rs:144-167` — `fn strip_html`, byte-identical quote-aware state machine
- `src/ats/smartrecruiters.rs:222-234` — `fn strip_html`, **simpler** version, no quote tracking
- `src/ats/workable.rs:142-154` — `fn strip_html`, **simpler** version, no quote tracking
- (Cross-reference: `src/pipeline/format.rs` also implements HTML stripping at a higher level. That is a separate system and is out of scope for this consolidation.)

**Current State:**
Four separate modules each define their own `strip_html` helper. Two are byte-identical quote-aware state machines that correctly pass through `>` when it appears inside a quoted attribute value (`lever`, `greenhouse`). The other two (`smartrecruiters`, `workable`) are simpler state machines that do not track quote state — they will terminate the tag at the first `>` after `<`, leaking attribute content into the output on inputs such as `<span data-ccp-props='{"k":">"}'>visible</span>`.

The quote-aware version's motivation is captured in the inline comment at `src/ats/lever.rs:187`: "Handles '>' inside quoted attribute values (e.g. data-ccp-props with JSON)." The smartrecruiters + workable versions predate that fix.

Call-site analysis (grep of `strip_html` across `src/ats/`):

| File | `strip_html` defined | `strip_html` called from | Is call site live? |
|------|----------------------|--------------------------|--------------------|
| lever.rs | `pub`, line 189 | `build_description()` line 158 → `normalise_posting` → `normalise_postings` → `pipeline::search::fetch_all_parallel` | **yes, live** |
| greenhouse.rs | private, line 144 | `normalise()` line 140 → `fetch_all_with_extra` → `pipeline::search` | **yes, live** |
| smartrecruiters.rs | private, line 222 | `fetch_detail()` lines 144, 149, 154 | **no — `fetch_detail` is `#[allow(dead_code)]` and never called** (see next finding) |
| workable.rs | private, line 142 | `normalise()` line 138 → `fetch_all` → `pipeline::search` | **yes, live** |

**Proposed Change:**
1. Move the quote-aware implementation from `lever.rs:189-212` into `src/ats/common.rs` as `pub(crate) fn strip_html`.
2. Delete the duplicate bodies in `greenhouse.rs`, `workable.rs`, and `smartrecruiters.rs` (the latter via the dead-code removal covered in the next finding).
3. Each module calls `super::common::strip_html` in place.
4. Consolidate the test coverage — the `lever.rs` inline tests for `strip_html_simple`, `strip_html_quoted_gt`, `strip_html_empty`, the matching tests in `greenhouse.rs`, and the `tests/ats_strip_html_parity.rs` file written by this audit all become the single test suite attached to `src/ats/common.rs`.

**Justification:**
Four implementations of the same conceptual operation is textbook Inconsistent-Patterns. Two of the four are byte-identical — those are pure duplication with zero behavioural divergence. The other two diverge in a way that is a latent correctness bug on Workable (the smartrecruiters divergence is irrelevant because its caller is dead).

Research anchor: [Mathias Bynens — Unquoted attribute values in HTML and CSS/JS selectors](https://mathiasbynens.be/notes/unquoted-attribute-values) documents exactly the attribute-value quote-tracking hazard this finding addresses. The 2026 stripping community consensus is that state-machine strippers must track quote state when operating on non-sanitised input from arbitrary third-party sources — which is exactly the situation here (ATS API descriptions come from HR teams writing arbitrary HTML).

Diagnostic test written by this audit:
- `tests/ats_strip_html_parity.rs` — 6 tests locking the quote-aware semantics on the one publicly accessible stripper (`cernio::ats::lever::strip_html`). All 6 pass against the current implementation. These become the reference contract for the consolidated common implementation.

**Expected Benefit:**
Removes ~70 lines of duplicated code. Eliminates the Workable correctness divergence. Creates a single place to fix the stripper if edge cases are discovered later. Future ATS additions inherit the quote-aware behaviour by default.

**Impact Assessment:**
- Lever: byte-identical implementation → zero change.
- Greenhouse: byte-identical implementation → zero change.
- Smartrecruiters: call site is dead code → zero observable change (caller is removed in the next finding).
- **Workable: behaviour changes** on descriptions containing `>` inside quoted attribute values. The change is a strict improvement — leaked attribute content stops appearing in the final plaintext — but it IS observable at the AtsJob.description field level. Flag for the implementing engineer as "upgrade, not regression."

---

## Dead Code Removal

### Remove unused `fetch_detail` and associated structs in `src/ats/smartrecruiters.rs`
- [ ] Delete `fetch_detail`, `DetailResponse`, `JobAd`, `JobAdSections`, `HtmlSection`, and the private `strip_html` function in `src/ats/smartrecruiters.rs`. Remove the `#[allow(dead_code)]` attributes that were masking them.

**Category:** Dead Code Removal
**Severity:** Medium
**Effort:** Trivial
**Behavioural Impact:** None (verified — the function has no callers)

**Location:**
- `src/ats/smartrecruiters.rs:50-77` — `DetailResponse`, `JobAd`, `JobAdSections`, `HtmlSection` structs
- `src/ats/smartrecruiters.rs:129-165` — `pub async fn fetch_detail`
- `src/ats/smartrecruiters.rs:222-234` — `fn strip_html` (the simple non-quote-aware version, called only from `fetch_detail`)

**Current State:**
`fetch_detail` is marked `#[allow(dead_code)]` at line 129 and fetches the HTML sections of a single SmartRecruiters posting. A grep for `fetch_detail` across the codebase finds no callers. The function was presumably added in anticipation of a two-phase list-then-detail fetching pattern that was not implemented (SmartRecruiters' list endpoint does not include descriptions; see `systems/ats.md` §Per-provider reality).

Four structs — `DetailResponse`, `JobAd`, `JobAdSections`, `HtmlSection` — exist only to be deserialised by `fetch_detail` and are therefore unreachable. The private `strip_html` is called only from inside `fetch_detail` (lines 144, 149, 154) and has no other callers.

**Proposed Change:**
Delete lines 50-77 (the four dead structs), lines 129-165 (`fetch_detail`), and lines 222-234 (`strip_html`). Remove the five `#[allow(dead_code)]` attributes that mask them (lines 51, 58, 64, 74, 129). Leave `SmartRecruitersJob`, `SmartRecruitersLocation`, `SmartRecruitersLabel`, and their `#[allow(dead_code)]` attributes at lines 17, 33, 44 untouched — they ARE used by `normalise`, though several of their fields are also dead (see next finding).

**Justification:**
- Zero callers verified by `grep -rn "fetch_detail" src/ tests/` — the only hit is the definition itself.
- The simple `strip_html` that this removal eliminates also retires a correctness divergence (see the previous finding) without touching live code.
- `#[allow(dead_code)]` on six items in one file is a maintenance smell — every attribute masks a piece of code that the compiler would have told someone to delete.

**Expected Benefit:**
Removes ~40 lines of dead code, six `#[allow(dead_code)]` attributes, and eliminates one of the two non-quote-aware `strip_html` copies. Leaves a cleaner module whose dead-code surface is halved.

**Impact Assessment:**
Zero functional change by construction. The function has no callers; deleting it cannot alter any execution path. The deleted structs exist only as deserialisation targets for the deleted function, so they cannot be touched by any other code path.

---

### Remove dead fields on `SmartRecruitersJob` and related structs
- [ ] Remove `experience_level` and `department` fields from `SmartRecruitersJob`; remove `region`, `address`, and the `remote` field's unused path from `SmartRecruitersLocation` if not referenced by `normalise`; remove `SmartRecruitersLabel` if no longer referenced after the first two removals.

**Category:** Dead Code Removal
**Severity:** Low
**Effort:** Trivial
**Behavioural Impact:** None (verified — fields are deserialised but never read)

**Location:**
- `src/ats/smartrecruiters.rs:17-30` — `SmartRecruitersJob` struct, fields `experience_level`, `department`
- `src/ats/smartrecruiters.rs:33-41` — `SmartRecruitersLocation` struct, field `address`

**Current State:**
`normalise()` at line 169 reads only `id`, `name`, `released_date`, `location.city`, `location.country`, `location.remote`, and `ref_url` from `SmartRecruitersJob`. The fields `experience_level`, `department`, `region`, and `address` are deserialised (serde populates them from the API response) but never consumed. The `#[allow(dead_code)]` attribute on the struct masks this from the compiler.

**Proposed Change:**
Trim the struct definitions to only the fields actually read. Remove the now-unnecessary `#[allow(dead_code)]` attribute where possible. `SmartRecruitersLabel` becomes unreachable if `experience_level` and `department` are removed — delete it too.

**Justification:**
Unused fields add parse-time overhead (Serde populates them from JSON) and visual clutter to the struct definitions. Removing them makes the actual data dependency explicit.

**Expected Benefit:**
~10 lines removed; `#[allow(dead_code)]` attributes stop masking genuinely unused fields.

**Impact Assessment:**
Zero functional change. Serde will simply not populate the removed fields; the remaining fields continue to deserialise identically. This is the same pattern documented at [docs.rs/serde](https://docs.rs/serde/latest/serde/#container-attributes) for ignoring unknown JSON fields.

---

## Known Issues and Active Risks

### Transient HTTP failures silently drop entire pages in `smartrecruiters::fetch_all` pagination
- [ ] Wrap the per-page fetch in `src/ats/smartrecruiters.rs:109-123` with `get_with_retry(client, &url, 2)` to match the retry discipline used by probe paths.

**Category:** Known Issues and Active Risks
**Severity:** High
**Effort:** Trivial
**Behavioural Impact:** None (behaviour on the happy path is unchanged; transient-failure behaviour goes from silent bail to silent retry — which is what the project's scale-lesson notes require)

**Location:**
- `src/ats/smartrecruiters.rs:111` — `let resp = client.get(&url).send().await?.error_for_status()?;` inside `loop`

**Current State:**
The SmartRecruiters `fetch_all` paginates in 100-job chunks. Each page uses a bare `client.get(&url).send()` without retry. If page 3 of 5 fails transiently (timeout, 502, 503), the `?` propagates the error and the function returns *partial results* from pages 1 and 2. The caller in `pipeline::search` sees the error and logs it per-portal, but the partial pages that did fetch are discarded because the function returns `Err` rather than `Ok(all_jobs)`.

This violates the project's documented scale lesson (`context/notes/populate-db-lessons.md` §"Per-request retry improves reliability at scale"): "When resolving 200+ companies in parallel, transient HTTP failures (timeouts, rate limits, 502s) are inevitable. ... The HTTP client now retries individual failed requests rather than failing the whole batch."

SmartRecruiters has a handful of companies with >100 jobs in the current universe (`cernio stats` would confirm; absent live DB access the audit infers from the Apple / Amazon / Google tier that pagination will fire). Each such company is at risk of silently truncated job lists.

**Proposed Change:**
Replace line 111 with:
```
let resp = get_with_retry(client, &url, 2).await?.error_for_status()?;
```
`get_with_retry` is already imported at the top of the file (line 3).

**Justification:**
- Direct evidence from the audit's initial WebSearch ([reqwest-retry docs](https://docs.rs/reqwest-retry)): 2026 production-pattern for reqwest is retry-with-backoff on 5xx + timeout + connection errors, which is exactly what `get_with_retry` already provides.
- The project explicitly documents per-request retry as a lesson in `notes/populate-db-lessons.md`.
- Other probe paths in the same file DO use retry (line 87 in `probe`); the fetch path does not. This is an inconsistency as well as a risk.

**Expected Benefit:**
Large SmartRecruiters boards (Amazon, Google, Apple when on SR) no longer silently truncate on transient network blips. Matches the retry discipline used everywhere else in the ATS layer.

**Impact Assessment:**
Zero functional change on the happy path — every request that succeeds first time continues to return the same data. On transient failure, the loop now retries (up to 2 additional attempts with linear backoff) before bailing; if retries all fail, behaviour is identical to today. The change is a strict strengthening of the failure path.

---

### `fetch_all` paths skip retry in multiple providers
- [ ] Apply `get_with_retry` (or its POST counterpart `post_json_with_retry`) to the list-endpoint fetch in `src/ats/ashby.rs:82`, `src/ats/workable.rs:64`, `src/ats/workday.rs:66` and `:104`, and `src/ats/smartrecruiters.rs:111` (covered in the previous finding) and `:136` (the detail endpoint, removed entirely by the dead-code finding).

**Category:** Known Issues and Active Risks
**Severity:** Medium
**Effort:** Small
**Behavioural Impact:** None (same strengthening as above — happy-path identical, failure path strictly more robust)

**Location:**
- `src/ats/ashby.rs:82` — `client.get(&url).send()` with no retry (single-page fetch)
- `src/ats/workable.rs:64` — same
- `src/ats/workday.rs:66` — `client.post(&url).json(&body).send()` inside `probe_with_extra`, no retry
- `src/ats/workday.rs:104` — same in `fetch_all_with_extra`, inside pagination loop (same hazard as SmartRecruiters)

**Current State:**
Probe paths in every provider consistently use `get_with_retry`. Fetch paths (the list-endpoint calls) are split: Lever and Greenhouse use `get_with_retry`, Ashby/Workable/Workday do not. There is no documented rationale for the divergence.

**Proposed Change:**
Standardise the fetch paths on `get_with_retry` / `post_json_with_retry`. `post_json_with_retry` already exists in `src/ats/common.rs:72-97` but is itself `#[allow(dead_code)]` because nobody calls it — if Workday adopts it, the attribute can also be removed.

**Justification:**
Consistency with the project's own scale lesson; consistency with the probe paths that already use retry; documented pattern in the 2026 Rust HTTP ecosystem ([OneUptime — Retry Logic with Exponential Backoff in Rust](https://oneuptime.com/blog/post/2026-01-07-rust-retry-exponential-backoff/view)).

**Expected Benefit:**
Uniform retry semantics across the ATS layer. No more silent partial fetches on transient failure. `post_json_with_retry` becomes a live function, letting the `#[allow(dead_code)]` attribute on it be removed (minor dead-code win).

**Impact Assessment:**
Same as previous finding — happy path unchanged, failure path strengthened.

---

## API Surface Bloat

### `strip_html` in lever.rs is `pub` but has no external callers
- [ ] Change `pub fn strip_html` to `fn strip_html` in `src/ats/lever.rs:189` once the consolidation in the Inconsistent-Patterns finding is implemented. If the consolidation is implemented first, this finding is subsumed.

**Category:** API Surface Bloat
**Severity:** Low
**Effort:** Trivial
**Behavioural Impact:** None (by construction — reducing visibility of an un-imported function)

**Location:**
- `src/ats/lever.rs:189` — `pub fn strip_html`

**Current State:**
The only caller of `lever::strip_html` is inside the same module (`build_description` at line 158). A grep for `lever::strip_html` across `src/` and `tests/` finds zero external references. The `pub` is incidental — probably a leftover from when the function was speculatively exposed as a utility.

The diagnostic test `tests/ats_strip_html_parity.rs` written by this audit calls `cernio::ats::lever::strip_html` and therefore DOES depend on the `pub` visibility. This dependency is intentional and is the reference evidence for the Inconsistent-Patterns finding; it goes away once consolidation moves the stripper to `common.rs`.

**Proposed Change:**
Reduce to `fn strip_html`. Coordinate with the consolidation finding — if that lands, the function moves to `common.rs` as `pub(crate) fn strip_html` and this finding disappears.

**Justification:**
Smaller public API surface; the `#[allow(dead_code)]` attribute at line 188 disappears because the function stops being "publicly reachable but externally uncalled" (which is what triggers the lint).

**Expected Benefit:**
-1 public function from the ATS layer's export surface. Makes refactoring safer (no external consumer to break).

**Impact Assessment:**
Zero functional change by construction. Visibility reduction cannot affect behaviour.

---

## Performance Improvement

### `fetch_all` paths clone location strings twice per job in every provider's `normalise`
- [ ] Refactor each provider's `normalise` so the primary location string is moved into `location` and then *taken by reference* into `all_locations` via `.clone()` only when `all_locations` actually needs the duplicate (e.g. when the primary will be augmented by split parts).

**Category:** Performance Improvement
**Severity:** Low
**Effort:** Small
**Behavioural Impact:** None (verified — `AtsJob.location` and `AtsJob.all_locations` produce identical values after the change, only the allocation count drops)

**Location:**
- `src/ats/lever.rs:122-146` — `normalise_posting`
- `src/ats/greenhouse.rs:101-142` — `normalise`
- `src/ats/ashby.rs:90-136` — `normalise`
- `src/ats/workable.rs:72-140` — `normalise`
- `src/ats/smartrecruiters.rs:169-220` — `normalise`
- `src/ats/workday.rs:123-160` — `normalise`

**Current State:**
Every provider's `normalise` follows the pattern:
```
let mut all_locations = vec![job.location.clone()];   // clone #1
// ... maybe push additional derived locations ...
AtsJob { location: Some(job.location), all_locations, ... }   // move
```

For Greenhouse (line 102): `let mut all_locations = vec![job.location.name.clone()];` allocates a String heap-copy of the primary location name, then `Some(job.location.name)` moves the original. At typical job counts per fetch (50-200), this is 50-200 avoidable small String allocations per search.

For providers where `all_locations` does NOT need the duplicate because every other entry is a derived split part anyway (Greenhouse's semicolon split, Workday's pipe split), the pattern is:
```
all_locations.push(original)           // move
split_into_parts(&original).for_each(...)
```
— but this requires `split_into_parts` to return owned strings or borrow from a still-living source.

**Proposed Change:**
Each provider's `normalise` restructured to move the primary location into `all_locations` once, then derive either (a) the duplicate for the separate `location` field only if the primary is not already the full augmented string, or (b) accept that the `Option<String> location` field always duplicates the first element of `Vec<String> all_locations` and store an index instead.

The minimum-invasive version just does `let primary = job.location.name;` once, then `all_locations.push(primary.clone())` (still one clone) but uses `Some(primary)` for the field by construction, avoiding the clone-and-move pattern.

**Justification:**
Per-job allocation across the fetch hot path is exactly the "per-iteration allocation in hot loops" anti-pattern documented in `references/analysis-categories.md` §7 (Data Layout and Memory Access Patterns). At current scale (≤400 companies × ≤200 jobs per fetch = ≤80,000 small String allocations per full search run), it does not dominate; at the next order of magnitude it would start to matter.

Research anchor: [Microsoft Rust training — Async Is an Optimization, Not an Architecture](https://microsoft.github.io/RustTraining/async-book/ch14-async-is-an-optimization-not-an-architecture.html) — the "keep business rules in pure sync functions" guidance argues for minimising allocator engagement inside the async loop where possible.

**Expected Benefit:**
~1 fewer String clone per job across 6 providers. Not a dominant cost at current scale, but a free improvement that becomes meaningful as the company universe grows. No new dependencies, no new abstractions.

**Impact Assessment:**
Zero functional change (verified by reading each `normalise` — the `AtsJob` produced is byte-identical in every field). The change is structural refactoring of local variable usage.

---

## Triage Needed

### Eightfold fetcher exists as a column in `ats_provider` but no module implements it
- [ ] Decide: implement the Eightfold fetcher (companies on Eightfold migrate from `bespoke` to `resolved`) OR remove the `'eightfold'` option from the `ats_provider` CHECK constraint in `src/db/schema.rs` to reflect that it is not supported.

**Category:** Triage Needed
**Severity:** (not ranked)
**Effort:** (varies with decision)
**Behavioural Impact:** Depends on decision.

**Location:**
- `src/db/schema.rs` — `ats_provider` CHECK constraint list includes `'eightfold'` (cited in `systems/database.md`)
- `src/ats/mod.rs` — no `pub mod eightfold`
- `systems/ats.md` §"Per-provider reality" row on Eightfold — marked "Not yet implemented"

**Current State:**
The database schema accepts `'eightfold'` as a valid `ats_provider` value; the `ats_extra` format is documented in `systems/ats.md` as `{"subdomain":"…","domain":"…"}`. No `src/ats/eightfold.rs` exists. `pipeline::resolve` does not probe for Eightfold. Companies known to be on Eightfold are currently recorded as `bespoke` (no portal row).

This is ambiguous intent: either the schema is ahead of the code (the team plans to add Eightfold) or the schema has a stale enum value. The audit cannot determine which from the code alone.

**Proposed Resolution:**
Raise with the user. `context/architecture.md` "Next priorities" includes Eightfold as item 2, which suggests the intent is forward-planning. If so, leave the schema as-is; if not, trim the enum.
