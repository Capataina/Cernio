# Autofill Feature: Status and Known Issues

**Status: Working end-to-end on Greenhouse as of 2026-05-14.** Last verified Proton job 2558 (Rust Software Engineer): 16/18 fields filled cleanly on the most recent test, with the consumed-keys + cover-letter-manual-entry fixes from commit `1e074bd` expected to push the next test to 17/18+. The single open issue is the Resume/CV visual indicator (file attaches per telemetry but the form's filename label doesn't visibly update — needs a manual submission attempt to verify it really uploaded).

---

## Architecture (at a glance)

```
TUI (press `p` on a job)
   ↓
src/tui/app/actions.rs::autofill_selected_job
   ├─ SELECT CAST(answers AS TEXT) FROM application_packages WHERE job_id = ?
   ├─ Tokio::spawn the autofill task
   └─ Record decision (no auto-DELETE of the package)
                           ↓
src/autofill/mod.rs::fill_application
   ├─ ApplicantProfile::load from profile/ (legal name from ## Full Name)
   ├─ Parse package JSON → HashMap<String, String>
   └─ Dispatch to provider module
                           ↓
src/autofill/greenhouse.rs::fill
   ├─ greenhouse_api::JobSchema::fetch (CDP-free, hits public Job Board API)
   ├─ common::launch_and_navigate (Chrome with stealth flags, fixed window size)
   ├─ Iterate schema.questions with consumed-keys tracking
   ├─ For each Field, dispatch by FieldKind:
   │   ├─ InputText: trim to 250 chars at sentence boundary, type_into
   │   ├─ Textarea: type_into (full text)
   │   ├─ InputFile: fill_file_field (resume via DOM.setFileInputFiles,
   │   │   cover letter via Enter-manually flow)
   │   ├─ MultiValueSingleSelect: click_combobox_option
   │   └─ MultiValueMultiSelect: set_checkbox_in_fieldset per option
   └─ Park browser open; user reviews and submits manually
```

---

## Hard-earned lessons (each one cost a debug session)

### Persistence

| Lesson | Why it bit |
|---|---|
| `application_packages.answers` is **TEXT-declared but accepts BLOB silently** via SQLite type affinity. `readfile()` returns BLOB. `rusqlite::row.get::<_, String>` rejects BLOB and returns Err. Always insert via `CAST(readfile(...) AS TEXT)`. | Telemetry showed `has_package=true` from one code path and `has_package=false` from another, on the same row, simultaneously. Fixed in commit `36118b5` (defensive `CAST(answers AS TEXT)` in the autofill SELECT) and `1e074bd` (skill doctrine: ephemeral `/tmp/cernio-pkg-<id>.json`, never persistent files). |
| **Package lives in the DB only.** No `state/<job>-package.json` files. The skill writes straight to `application_packages.answers`; the `/tmp/` JSON file used during insert is deleted in the same step. | Drift surface: editing a JSON file without re-inserting leaves stale data in one place but not the other. Removed in commit `4d2737e`. |

### Greenhouse API schema

| Lesson | Why it bit |
|---|---|
| `compliance` and `demographic_questions` come back as **explicit null** (not omitted) on jobs with no EEOC block. `#[serde(default)]` alone handles missing keys but not present-and-null. Add `null_to_default` deserialize helper to every `Vec<…>` field. | Schema parse failed with `error decoding response body`. Fixed in commit `14d1ab6`. |
| `FieldOption.value` is sometimes a **JSON number** (e.g. Proton's currency dropdown has integer IDs), not a string. Schema declared `String` failed with `invalid type: integer X`. Use a `string_or_number` deserialize helper. | Fixed in commit `72c1d32`. |

### Chrome / CDP / chromiumoxide

| Lesson | Why it bit |
|---|---|
| **`Element::type_str` dispatches per-character Input.dispatchKeyEvent**, bails on chars not in chromiumoxide's keys table — including `\n`. Result: partial writes with no failure signal beyond a bool. Switch to **CDP `Input.insertText`** (atomic, no keys table, fires React's `input` event). | The "Anything else" textarea filled with only the first 30 chars of the cover letter (up to the first `\n`), reported as gh_question_skipped. Found by inspecting chromiumoxide's source. Fixed in `9254a17`. |
| chromiumoxide's DEFAULT_ARGS includes `--enable-automation`, which is what makes Chrome show the "Chrome is being controlled by automated test software" banner and amplifies reCAPTCHA's bot score. **Call `.disable_default_args()` and re-add the 22 useful defaults manually**, minus `--enable-automation`. Plus inject a stealth init script (navigator.webdriver, plugins, languages, window.chrome.runtime, Permissions.query). | The banner was visible. Fixed in `bbc56ef`. |
| **`--start-maximized` is unreliable on macOS** (the OS has no true maximize; green-button does "zoom"). Use explicit `--window-size=W,H` + `--window-position=X,Y`. | Chrome opened at default size, taking only half the screen. Fixed in `1e074bd`. |
| **Soft-conditional matching has to strip embedded quotes**, not just leading ones. Proton's `If "yes" can you specify ...` had a quote inside. Both `is_soft_conditional` AND the inner yes/no comparator need the same normalisation. | The "If yes" permit-type textarea was skipped even when right_to_work=Yes. Fixed in two commits: `35d3752` (is_soft_conditional) and `1e074bd` (inner comparator). |

### Form-filling semantics

| Lesson | Why it bit |
|---|---|
| **One semantic key per form**. The matcher might route `cover_letter` to both a dedicated Cover Letter field and a generic "Anything else you want to share?" textarea. Without consumed-key tracking, the cover letter ends up in both. Track which keys have been used per form and skip repeats. | Recruiter sees the same cover letter twice — a textbook AI tell. Fixed in `1e074bd` (consumed_keys HashSet threaded through fill_question). |
| **InputFile fields don't have an answer string**. They need a filesystem path (resume) or a "switch to manual entry" click (cover letter as text). The old code's answer-required guard fired before the InputFile branch, so file inputs were always skipped. Short-circuit InputFile BEFORE the guard, into a dedicated `fill_file_field`. | Resume / Cover Letter were always skipped on every form. Fixed in `9254a17`. |
| **Greenhouse's "Enter manually" button text-matcher needs substring + case-insensitive + a wide element set** (`button, [role="button"], a[role="button"], div[role="button"]`). Exact equality misses spans, icons, helper text. | Cover Letter manual-entry flow didn't fire on the first attempt. Fixed in `35d3752`. |
| **Greenhouse uses input_text (single-line) for some essay-length questions**. Browser silently caps at the field's maxlength (~255). Trim to 250 chars at the last sentence boundary at fill time. | Why-this-role essays cut mid-word. Fixed in `9254a17`. |

### ApplicantProfile parsing

| Lesson | Why it bit |
|---|---|
| **`profile/personal.md` uses Obsidian `## Heading\nvalue` sections**, not `**Field:** value` inline patterns. The old `extract_field` couldn't parse the section format and fell through to a hardcoded "Caner" fallback for first_name — so the form's First Name field always got "Caner" instead of "Ata Caner". Added `extract_section` + `split_full_name`. | The First Name field shipped with the preferred name instead of the legal first name (which Greenhouse uses for background checks). Fixed in `bbc56ef`. |

### Skill-level architecture

| Lesson | Why it bit |
|---|---|
| **`profile/applications.md` was redundant.** All factual values (visa, GPA, employer, etc.) live in existing profile files (`visa.md`, `education.md`, `experience.md`, `preferences.toml`). The prepare-applications skill now sources directly from those files, no intermediate `applications.md` needed. | Premature abstraction — the file would only restate what's already in profile/. Removed in commits `4d2737e` + `7880143`. |
| **`profile/application-voice.md` is the texture layer** — reasoning for tone, length, density, hooks, gap framing, anti-redundancy, anti-fawning. Portfolio-agnostic (names no specific projects), markdown-rich (tables, callouts), self-disciplined (no em-dashes in its own prose). prepare-applications reads it on every invocation. | Generated cover letters were salesy and repeated structured-field info. Voice file gives the agent the *why* behind each prose decision. Last expanded in `1e074bd` (§2.8 No redundancy, §2.9 No fawning). |

---

## Open / known-flaky

| Item | Reproduce | Suspected cause | Next step |
|---|---|---|---|
| Resume/CV visual indicator on Proton shows an empty thin bar after fill | Press `p` on Proton 2558 | `DOM.setFileInputFiles` succeeded per telemetry, but Greenhouse's UI shows the filename only after a user-interaction event the autofill didn't fire | Try submitting the form anyway — if Greenhouse accepts the file, the indicator is cosmetic. If rejected, dispatch a synthetic `change` event after `setFileInputFiles` |
| Cover Letter "Enter manually" flow may still misfire on some Greenhouse variants | Forms with non-standard manual-entry button text | The substring matcher is now permissive, but specific selectors after the click could still miss | Per-step telemetry events (`gh_cover_letter_button_clicked`, `gh_cover_letter_textarea_not_found`) are wired — next failure will name the step |
| reCAPTCHA v3 score for our automated session is likely lower than a human's | Look at server-side reCAPTCHA scoring (not visible to us) | Stealth flags help but we don't simulate mouse/scroll/keystroke timing | If applications go silent post-submit, consider adding minimal humanising delays + mouse movement; otherwise live with it |

## Out of scope (deliberate)

- **Auto-submission**. The user reviews and clicks Submit. Not a constraint we'd want to remove — false-positive risk is too high.
- **Greenhouse-internal anti-bot evasion** beyond the stealth flags + insertText. Not adding proxy rotation, fingerprint randomisation, or behavioural simulation; that crosses from "advanced autofill" into "adversarial bot".
- **Non-Greenhouse provider modules**. Lever / Ashby / Workable autofill are designed but not implemented. The architecture supports them via the provider dispatch in `mod.rs`.

---

## File map

| File | Purpose |
|---|---|
| `src/autofill/mod.rs` | Profile load, package parse, provider dispatch, AutofillResult |
| `src/autofill/common.rs` | Chrome launch (stealth flags), type_into (Input.insertText), set_file (DOM.setFileInputFiles), click_combobox_option, set_checkbox_in_fieldset, click_button_with_text |
| `src/autofill/greenhouse.rs` | fill loop, fill_question, fill_file_field, semantic matcher (12 keys), soft-conditional, trim_for_input_text, consumed-keys tracking |
| `src/autofill/greenhouse_api.rs` | JobSchema + types, public-API fetcher, url parser, null + string-or-number deserialize helpers |
| `state/cernio.db::application_packages` | Canonical home for prepared packages |
| `profile/application-voice.md` | Texture layer — reasoning for prose generation |
| `.claude/skills/prepare-applications/SKILL.md` | The judgment half — generates packages, writes them to the DB |
