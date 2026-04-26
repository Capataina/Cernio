---
name: Image Browser
status: active
source_repo: https://github.com/Capataina/PinterestStyleImageBrowser
lifeos_folder: Projects/Image Browser
last_synced: 2026-04-26
sources_read: 28
---

# Image Browser

## One-line summary

Local-first Tauri 2 desktop app for browsing, tagging, and semantically searching personal image libraries via three ONNX-Runtime encoders (CLIP + DINOv2 + SigLIP-2) fused at retrieval time with Reciprocal Rank Fusion.

## What it is

Image Browser (Cargo name `image-browser`, Tauri identifier `com.ataca.image-browser`, default branch `master`) is a single-binary Tauri 2 desktop application backing a Pinterest-style React 19 masonry frontend. The Rust backend handles filesystem scanning, SQLite (WAL) persistence, thumbnail generation, ONNX-Runtime inference across three encoder families, multi-folder lifecycle, a filesystem watcher with orphan detection, an opt-in profiling and domain-diagnostic layer, and first-launch model downloads from HuggingFace. Everything runs offline after the initial model download (~2.5 GB). The React frontend renders the masonry grid, a modal inspector with annotations, a multi-section settings drawer, an indexing-status pill, and an opt-in perf overlay. Designed as the Capataina single-user image-curation tool — privacy by construction, no cloud, no accounts, no telemetry — and currently in heavy iteration (31 commits on 2026-04-26 alone, more than the entire pre-2026 history combined).

## Architecture

The system is a layered Tauri 2 app with a strictly directional dependency graph: `main.rs` parses flags and opens the database, `lib.rs::run` registers managed state and 26 Tauri command handlers, and the frontend communicates over Tauri IPC + Tauri events with an `ApiError` discriminated union as the wire format.

```
┌──────────────────────── React 19 Frontend (WebView) ────────────────────────┐
│ pages/[...slug].tsx ── single catch-all route                                │
│   ├─ Masonry / MasonryItem / MasonryAnchor (shortest-column packing)         │
│   ├─ PinterestModal (annotations textarea, prev/next, 3D framer-motion tilt) │
│   ├─ SearchBar (#-autocomplete + create-on-no-match)                         │
│   ├─ TagDropdown                                                             │
│   ├─ IndexingStatusPill (Tauri-event subscription)                           │
│   ├─ PerfOverlay (cmd+shift+P, profiling-only mount)                         │
│   └─ settings/ (Theme · Display · Search · Sort · Folders · Reset · Encoder) │
│ queries/ ── TanStack Query hooks (staleTime: Infinity)                       │
│ services/ ── invoke() wrappers + ApiError mirror                             │
│ hooks/ ── useDebouncedValue, useUserPreferences, useIndexingProgress         │
└─────────────────────────────── Tauri IPC + Tauri events ─────────────────────┘
┌──────────────────────────── Rust Backend ────────────────────────────────────┐
│ main.rs ── parse --profiling, init tracing + opt-in PerfLayer,               │
│            spawn JSONL flush + 1 Hz RSS/CPU sampler, open DB, hand off       │
│ lib.rs ── Tauri::Builder.manage(state...).setup(...).invoke_handler![26]     │
│   Managed state:                                                             │
│     • ImageDatabase (writer Mutex<Connection> + R2 read-only secondary)      │
│     • CosineIndexState { Arc<Mutex<CosineIndex>> }                           │
│     • TextEncoderState { Mutex<Option<ClipText>>, Mutex<Option<Siglip2Text>> }│
│     • FusionIndexState { per_encoder: Arc<Mutex<HashMap<String,CosineIndex>>>}│
│     • IndexingState (AtomicBool single-flight)                               │
│     • WatcherHandle slot                                                     │
│ commands/ ── per-concern handlers (images, tags, notes, roots, similarity,   │
│               semantic, semantic_fused, profiling, encoders, error)          │
│ db/ ── post-split SQLite layer (was 1.6k-line db.rs)                         │
│ similarity_and_semantic_search/ ── encoders, ort_session, cosine/, encoder_text│
│ indexing.rs ── background single-flight pipeline (parallel-by-encoder)       │
│ watcher.rs ── notify-debouncer-mini, 5 s debounce                            │
│ model_download.rs ── first-launch HuggingFace fetch                          │
│ paths.rs ── single disk-path source + Cow-strip Windows extended prefix      │
│ settings.rs / perf.rs / perf_report.rs                                       │
└──────────────────────────────────────────────────────────────────────────────┘
                                    │ disk:
                                    ▼ <app_data_dir>/{images.db (WAL), settings.json,
                                      cosine_cache.bin, models/*.onnx (~2.5 GB),
                                      thumbnails/root_<id>/, exports/perf-<unix_ts>/}
```

**Key directional rules observed in code:**

- `db` is the only sink every backend module reads from or writes to. The post-split `db/` directory uses Rust's "multiple `impl` blocks merge" property so callers see a flat `ImageDatabase` API regardless of which submodule defines a method.
- `Arc<Mutex<CosineIndex>>` is intentionally cloned across the indexing thread and the Tauri-managed `CosineIndexState` so a finished pipeline-encode immediately makes new embeddings available to the next semantic search.
- `indexing.rs` and `watcher.rs` are coupled through `IndexingState` — a single-flight `AtomicBool` with an RAII guard that ensures clear on success, error, AND panic. Rapid filesystem events that try to spawn a second pipeline get `Err(AlreadyRunning)` and silently coalesce.
- `profiling` is not in the normal data path. When `--profiling` is absent, `PerfLayer` never registers, the env filter passes the `#[tracing::instrument]` spans but no aggregator builds them, the frontend `<PerfOverlay>` never mounts, and `record_diagnostic` returns early before constructing any JSON.
- `commands/*` returns `Result<T, ApiError>` for every handler; the frontend deserialises `{ kind, details }` and branches on `kind`.
- Frontend services never call `invoke()` directly — they wrap it in functions that translate Tauri JSON into UI types. Hooks call services; components call hooks. The `perfInvoke` wrapper preserves this layering while adding profiling-side start/end events.

**Mutex topology** (five long-lived sync primitives):

| Primitive | Holder | Acquired by | Poison surface |
|---|---|---|---|
| `Mutex<Connection>` (writer) | `ImageDatabase.connection` | every DB write + foreground writes | `unwrap()` — restart required |
| `Mutex<Connection>` (R2 read-only secondary) | `ImageDatabase.reader: OnceLock<Mutex<Connection>>` | foreground SELECTs via `read_lock()` | falls back to writer if `reader.get()` is None |
| `Arc<Mutex<CosineIndex>>` | `CosineIndexState.index` | every similarity/semantic command | `?` via `From<PoisonError>` → `ApiError::Cosine` |
| `Mutex<Option<TextEncoder>>` ×2 | `TextEncoderState.{clip, siglip2}` | semantic_search + indexing pre-warm | same `From<PoisonError>` impl |
| `AtomicBool` (single-flight) | `IndexingState.is_running` | every command that triggers an index + watcher debounce | RAII guard clears on success, error, AND panic |
| `Arc<Mutex<Option<WatcherHandle>>>` | `watcher_state` | lib.rs setup callback | silent skip if poisoned |

The project treats poisoning as unrecoverable for a single-user desktop tool (restart on panic). `parking_lot::Mutex` (which doesn't poison) is documented as the strict-upgrade if it ever bites in practice.

## Subsystems and components

### CLIP image encoder (`encoder.rs`)
Separate-graph OpenAI English `vision_model.onnx` (~352 MB). Bicubic-shortest-edge → 224 + center-crop, CLIP-native mean/std `[0.48145466, 0.4578275, 0.40821073]` / `[0.26862954, 0.26130258, 0.27577711]`, 512-d output, L2-normalised. Replaced an earlier ImageNet-stats `resize_exact(224,224)` Lanczos3 path that biased the embedding distribution.

### CLIP text encoder (`encoder_text/encoder.rs`)
HF `tokenizers` BPE, vocab 49 152, max 77 tokens, pad id 49407. Defensive output-name cascade `text_embeds → pooler_output → sentence_embedding`. Real-input `encode("warmup")` pre-warm in `new()` (Phase 12c) eliminates first-call latency spike. Replaced a pure-Rust hand-rolled WordPiece tokenizer with multilingual CLIP at max 128 tokens.

### DINOv2-Base encoder (`encoder_dinov2.rs`)
Meta self-supervised, image-only, 768-d. Bicubic-shortest-edge → 256 + center-crop 224, ImageNet stats `[0.485, 0.456, 0.406]` / `[0.229, 0.224, 0.225]`, CLS-token from `last_hidden_state[:,0,:]` (not pooled output), L2-normalised.

### SigLIP-2 Base 256 encoder (`encoder_siglip2.rs`)
Google sigmoid loss, image+text in shared 768-d space. Image branch: 256 × 256 exact-square bilinear, `[-1, 1]` range (× 2 − 1, no per-channel mean/std). Text branch: HF `tokenizers` SentencePiece (Gemma 256k vocab), max 64 tokens, **no attention_mask** (SigLIP-2 unconditional sequence). Both branches output `pooler_output` (MAP head).

### Multi-Encoder Fusion (`cosine/rrf.rs`, `commands/similarity.rs`, `commands/semantic_fused.rs`)
The most novel system in the project. Reciprocal Rank Fusion (Cormack 2009, k=60) over 3 image encoders for image-image search and 2 text encoders (CLIP + SigLIP-2; DINOv2 has no text branch) for text-image. `FusionIndexState.per_encoder: Arc<Mutex<HashMap<String, CosineIndex>>>` is lazy-populated per encoder. 6 unit tests pin the algorithm contract. Per-encoder cache footprint ~6 MiB per 2000 images × 768 floats × 4 bytes; total fusion footprint ~18 MiB across CLIP + SigLIP-2 + DINOv2 for 2000 images.

### Cosine similarity (`cosine/`)
Brute-force scoring with `select_nth_unstable_by` partial-sort (2.53× speedup at n=10 000), reusable scratch buffer, persistent `cosine_cache.bin` with mtime freshness check. Submodules: `index.rs` (with 4 stateless diagnostic emissions), `rrf.rs` (6 tests), `diagnostics.rs` (4 stateless stat helpers), `cache.rs`, `math.rs`.

### Database (`db/`)
SQLite 3 with 5 tables (`roots`, `images`, `tags`, `images_tags`, `embeddings`) plus a `meta` migration tracker. Two connections per real DB: writer `Mutex<Connection>` and read-only secondary `OnceLock<Mutex<Connection>>` (R2). PRAGMAs: `journal_mode=WAL`, `synchronous=NORMAL`, `foreign_keys=ON` (required for `ON DELETE CASCADE`), `busy_timeout=5000`, `wal_autocheckpoint=0` (manual `checkpoint_passive` between encoder batches), `journal_size_limit=64 MiB`. 4 idempotent migrations; `embedding_pipeline_version` currently at **4** (bumped 2026-04-26).

### Indexing pipeline (`indexing.rs`)
Background single-flight pipeline with phases: cache load → model download → text-encoder pre-warm → scan → orphan-mark → thumbnail (rayon-parallel) → encode (one thread per enabled encoder, parallel since Phase 11e, dynamic `intra_threads(N/encoders)` since Phase 12) → cosine populate + save → `Phase::Ready` event. Each phase emits an `indexing-progress` Tauri event so the frontend `IndexingStatusPill` renders live status.

### Multi-folder roots (`db/roots.rs`, `commands/roots.rs`)
`roots` table with `images.root_id INTEGER REFERENCES roots(id) ON DELETE CASCADE`. Per-root thumbnail subfolder (`thumbnails/root_<id>/thumb_<image_id>.jpg`) so `remove_root` `rm -rf`s the subfolder cleanly. `set_root_enabled` toggles the SQL filter without re-encoding (non-destructive disable). Legacy single-folder migration (`settings.json::scan_root` → `roots` row) runs once.

### Filesystem watcher (`watcher.rs`)
`notify-debouncer-mini` with 5 s debounce, recursive on every enabled root. Single-flight coalescing via `IndexingState`.

### Model download (`model_download.rs`)
First-launch HuggingFace fetch (~2.5 GB across 7 files: 5 ONNX + 2 tokenizer.json). HEAD preflight + chunked GET + per-byte progress callback, per-file fail-soft.

### Profiling (`perf.rs`, `perf_report.rs`)
Opt-in via `--profiling` CLI flag (NOT `--profile` — Tauri 2 owns that name) or `PROFILING=1`. `PerfLayer` + 12 named diagnostics + 1 Hz RSS/CPU sampler (sysinfo 0.32). On-exit `report.md` includes Stall Analysis + Resource Trends. 17 diagnostic call sites (4 commands + 3 indexing + 2 lib + 4 cosine + 4 frontend), all no-op when profiling absent. Frontend overlay shortcut `cmd+shift+P`.

### Tauri commands (`commands/`)
26 commands grouped by concern under `src-tauri/src/commands/` (images, tags, notes, roots, similarity, semantic, semantic_fused, profiling, encoders). `ApiError` discriminated union (`#[serde(tag="kind", content="details")]`) mirrored on the frontend in `services/apiError.ts`. Lazy text-encoder init; path-prefix normalisation via `paths::strip_windows_extended_prefix` returning `Cow::Borrowed` for zero-alloc on common paths.

### Thumbnail pipeline (`thumbnail/generator.rs`)
JPEG path uses `jpeg-decoder::Decoder::scale()` for native scaled IDCT (1/8, 1/4, 1/2 factor) then `fast_image_resize 6.x` (NEON-optimised Lanczos3) for the final downsample. Falls back to `image-rs` for non-JPEG and any decode error. Output: 400 × 400 JPEG, aspect-preserving, no upscale.

### Masonry layout (frontend)
Shortest-column packing with hero promotion across up to 3 columns. 3D framer-motion tilt. sortMode-aware (stable-sort default, opt-in shuffle). Tile dimensions sourced from backend (no DOM image-load round-trip).

### Frontend state
TanStack Query (`staleTime: Infinity`); settings drawer split into 7 sections (Theme · Display · Search · Sort · Folders · Reset · Encoder); `useUserPreferences` localStorage hook; `useIndexingProgress` event hook.

## Technologies and concepts demonstrated

### Languages
- **Rust** (edition 2021) — entire backend across 28 backend files: filesystem scanning, SQLite persistence with `rusqlite`, ONNX inference glue, three encoder implementations, RRF fusion, profiling layer, Tauri command surface. Heavy use of `Arc<Mutex<...>>`, `OnceLock`, `AtomicBool` with RAII guards, `Cow::Borrowed` for zero-alloc, `bytemuck::cast_slice` for safe-zero-copy embedding BLOB encoding.
- **TypeScript** (React 19) — 33 frontend files: TanStack Query hooks, Tauri-IPC service wrappers, `ApiError` discriminated-union mirror, debounce hooks, settings persistence to localStorage, masonry layout component.

### Frameworks and libraries
- **Tauri 2** — desktop app shell with `tauri::Builder.manage(...).setup(...).invoke_handler!`, `tauri-plugin-dialog` for the native folder picker, `assetProtocol` for image loading at gallery scale (avoids base64-over-IPC).
- **React 19** — frontend rendering layer, modal inspector, settings drawer.
- **TanStack Query** — frontend data layer with `staleTime: Infinity`.
- **`ort = 2.0.0-rc.10`** — ONNX Runtime Rust binding; shared M2-tuned `Session` builder factory (`build_tuned_session`, `Level3 + intra_threads(4) + inter_threads(1)`).
- **HuggingFace `tokenizers = "0.22.2"`** — uniform `tokenizer.json` interface for both BPE (CLIP) and SentencePiece (SigLIP-2 Gemma vocab).
- **`rusqlite`** — SQLite binding, post-split `db/` layer.
- **`notify-debouncer-mini`** — filesystem watcher.
- **`image-rs`** + **`jpeg-decoder`** + **`fast_image_resize 6.x`** — image decode and resize stack (NEON-optimised Lanczos3).
- **`bytemuck`** — safe zero-copy `&[f32] ↔ &[u8]` for embedding BLOB.
- **`tracing`** + custom `PerfLayer` — instrumentation; opt-in span aggregator.
- **`sysinfo` 0.32** — 1 Hz RSS/CPU sampler thread.
- **`rayon`** — thumbnail pipeline parallelism.
- **`vite-plugin-pages`** — frontend route generation; **Vite 7** as bundler.
- **`framer-motion`** — 3D tile tilt animation.

### Runtimes / engines / platforms
- **Tauri 2** desktop app shell with platform-default app data dir via `dirs::data_dir()` (override via `IMAGE_BROWSER_DATA_DIR`). Supports macOS, Linux, Windows.
- **ONNX Runtime via `ort 2.0.0-rc.10`** — CPU on macOS (CoreML target-disabled across all encoders due to runtime errors on transformer ops), CUDA target-gated on non-macOS with CPU fallback.
- **SQLite** in WAL mode with two connections per DB (writer + R2 read-only secondary), manual `checkpoint_passive` between encoder batches.

### Tools
- **`cargo`** + **`bun`** — backend and frontend tooling.
- **`vitest`** — frontend test runner (62/62 passing).
- **`clippy`** — restored as a build gate post-audit (clean).
- **vault-lint** — repo-side documentation hygiene tooling.
- **`cargo tauri dev -- --profiling`** — dev profile mode.

### Domains and concepts
- **Multi-encoder retrieval fusion** — three image encoders (CLIP, DINOv2, SigLIP-2) and two text encoders (CLIP, SigLIP-2) producing rankings combined via Reciprocal Rank Fusion (Cormack, Clarke & Büttcher, SIGIR 2009; canonical k=60). Pure rank-based — discards cosine score because cross-encoder distributions differ. Diversity emerges for free from consensus across encoders. Fused score (~0–0.05) is unbounded and explicitly NOT a cosine similarity.
- **Local-first ML inference** — three encoders × consumer CPU × no Python at runtime, via ONNX Runtime + HF `tokenizers` + a shared M2-tuned Session builder. First and only network call is the first-launch HuggingFace model download.
- **WAL + dual-connection SQLite** — writer mutex + read-only secondary (R2) keeps foreground SELECTs non-blocking against in-flight encoder write batches. Replaces a single-connection design that produced multi-second freezes (perf-1777212369 22 s freeze).
- **Batched encoder writes** — `BEGIN IMMEDIATE` per ~32-row chunk + `checkpoint_passive` between batches, replacing per-row autocommit which produced N implicit transactions + N fsyncs.
- **Single-flight background pipelines** — `IndexingState.is_running: AtomicBool` with RAII guard ensures clear on success, error, AND panic. Rapid filesystem events coalesce.
- **Per-encoder enable/disable with non-destructive toggle** — toggling does not delete embeddings (cheap to keep, expensive to regenerate). Re-enabling instantly restores fusion participation.
- **Pipeline-version migration** — `meta(key, value)` table tracks `embedding_pipeline_version`; bumping the const wipes legacy embeddings on first launch under the new code (currently v4).
- **Typed Tauri-IPC error wire** — `#[serde(tag="kind", content="details")]` `ApiError` enum mirrored on the frontend, with `From<rusqlite::Error>`, `From<std::io::Error>`, `From<PoisonError<T>>` impls so command bodies use `?` directly.
- **Profiling-first development culture** — opt-in `--profiling` flag mounts `PerfLayer`, frontend overlay (cmd+shift+P), 12 named diagnostics, on-exit `report.md` with Stall Analysis and Resource Trends. Replaces a planned "build a Python comparison harness" deferred work — quality issues surface in the report without a separate validation tool.
- **Code-health audit posture** — 28 findings shipped end-to-end (not "logged for later"), with `R<n>` annotations across the perf-bundle commit set giving forward + reverse traceability.
- **Reciprocal Rank Fusion** as a retrieval primitive (canonical paper: Cormack, Clarke & Büttcher, SIGIR 2009).
- **Separate-graph CLIP exports** — `vision_model.onnx` + `text_model.onnx` instead of a unified graph, eliminating the dummy-text-input hack at image-encode time.
- **Encoder-specific preprocessing** — CLIP-native mean/std + bicubic-shortest-edge + center-crop for CLIP; DINOv2-canonical 256-then-224 for DINOv2; SigLIP-2 exact-square + `[-1,1]` for SigLIP-2.
- **Future direction (planned, not built)** — Fully Homomorphic Encryption (TFHE-rs / BFV) for encrypted vector search; documented as additive to the existing plaintext path with honest 4-5 orders-of-magnitude slowdown framing.

## Key technical decisions

**D1 — Tauri 2 over Electron.** Smaller bundle (8–15 MB vs 100+), lower memory (~30–40 MB vs 200–300 MB), Rust safety for filesystem and ML paths, `ort` crate ergonomics, `assetProtocol` for image loading at gallery scale. Same choice in Aurix.

**D2 — Local-first, no cloud, no accounts.** Every byte of compute, persistence, and ML inference runs on the user's machine. Only network call is first-launch HuggingFace model download (~2.5 GB). Privacy by construction.

**D3 — SQLite over an embedded vector DB (LanceDB / Qdrant).** SQLite is single-file, zero-dependency, well-understood; project already needs SQL for tags/notes/roots so a second store would be more complex. Brute-force cosine over a few-thousand 512–768-d vectors fits in RAM and runs in milliseconds with partial-sort. Revisit at ~50k images via HNSW behind a trait (Rec-2, additive).

**D4 — Three encoders (CLIP + DINOv2 + SigLIP-2), not one.** No single encoder is best at every retrieval task. CLIP handles concept overlap, DINOv2 visual/structural similarity (image-only), SigLIP-2 modern descriptive English alignment. RRF makes their disagreements visible and rewards consensus.

**D5 — Reciprocal Rank Fusion (RRF), k=60, not score-fusion.** Different encoders produce cosines on different distributions; CLIP's "0.85" is not comparable to DINOv2's "0.85", and L2 normalisation alone doesn't fix this. RRF discards the score and uses only rank, sidestepping cross-encoder distribution differences. k=60 is canonical (Cormack 2009); shipping anything else without a labelled validation set would be unprincipled. Diversity emerges for free.

**D6 — Per-encoder enable/disable, not always-all.** Each encoder costs ~350–500 MB of weights, ~1/3 indexing wall-clock, ~6 MiB resident RAM per 2000 images for the fusion cache. Toggling does not delete embeddings (cheap to keep, expensive to regenerate). A guard prevents disabling all 3.

**D7 — Separate-graph CLIP exports.** OpenAI's separate `vision_model.onnx` + `text_model.onnx` instead of unified — eliminates the dummy-text-input hack at image-encode time; smaller files individually.

**D8 — HuggingFace `tokenizers` crate over pure-Rust WordPiece.** Single crate handles BPE (CLIP) + SentencePiece (SigLIP-2) uniformly via `tokenizer.json`; battle-tested against HF's own model exports. Traded the previous "pure-Rust no-C-deps" position for ergonomics + correctness.

**D9 — `ort 2.0.0-rc.10` with shared M2-tuned Session builder.** Single `build_tuned_session` factory in `ort_session.rs` (`Level3`, `intra_threads(4)`, `inter_threads(1)`); every encoder routes through it so they cannot drift on Session config across rewrites. Phase 12 introduced dynamic `intra_threads(N/encoders)` for parallel-by-encoder phase.

**D10 — CoreML disabled for ALL encoders on macOS.** CoreML produces runtime inference errors for transformer ops in CLIP text and SigLIP-2 text; rather than maintain a per-encoder allow-list that would silently break on CoreML version bumps, disabled uniformly. Revisit pending in R16 (`MLProgram + RequireStaticInputShapes`).

**D11 — `--profiling` CLI flag (NOT `--profile`).** Tauri 2 reserves `--profile` for cargo profile selection. Naming collision documented in `main.rs` so the next person doesn't try to "fix" it.

**D12 — WAL + read-only secondary connection (R2).** Pre-R2, foreground `get_images` queued behind in-flight encoder write batches (the perf-1777212369 22 s freeze). With R2, foreground SELECTs are non-blocking against active writes. Connection pool (`deadpool-sqlite`) considered and rejected as over-engineered for the simple writer + read-only secondary pattern.

**D13 — `BEGIN IMMEDIATE` for batched encoder writes.** Per-row `upsert_embedding` autocommit produced N implicit transactions + N fsyncs (~10–100× slower for bulk inserts). `IMMEDIATE` takes the write lock up-front so the batch can't deadlock against a concurrent reader. Manual `checkpoint_passive` between batches keeps WAL bounded under `wal_autocheckpoint=0`.

**D14 — `paths::*_dir()` as the single disk-path source; no dev/release split.** Removed `cfg(debug_assertions)` branching that pointed at `<repo>/Library/` — switching between dev and release builds was forcing re-downloads of 2.5 GB of models. `IMAGE_BROWSER_DATA_DIR` env var overrides for testing/multi-instance.

**D15 — Multi-folder roots with `ON DELETE CASCADE` (gated by `PRAGMA foreign_keys=ON`).** SQLite defaults `foreign_keys` OFF for backwards-compat; without the pragma, CASCADE silently no-ops. Per-root thumbnail subfolders (Phase 9) make `remove_root` `rm -rf` clean rather than per-row deletion.

**D16 — `bytemuck::cast_slice` for embedding BLOB encoding.** Replaces 3 sites of `unsafe { slice::from_raw_parts(...) }` with safe, zero-copy, alignment-checked-at-compile-time canonical pattern. Audit commit `0bdb5f4`.

**D17 — Typed errors via `ApiError` enum, not strings.** `#[serde(tag="kind", content="details")]` discriminated union mirrored on the frontend; `From<rusqlite::Error>`, `From<std::io::Error>`, `From<PoisonError<T>>` impls let command bodies use `?` directly. Frontend can branch on specific failures (e.g. `ApiError::TextModelMissing` triggers a re-download flow). 3 profiling commands still use `Result<_, String>` for legacy reasons.

**D-MUTEX — `std::sync::Mutex` over `parking_lot::Mutex`.** For a single-user desktop app, restart is fast; pragmatic posture is poison-then-restart. `parking_lot` is the strict-upgrade if poisoning ever proves load-bearing in practice.

**D-RTAG — R-tag perf annotation comments.** 43 `R<n>` annotations across the perf-bundle commit set giving forward + reverse traceability + commit-review aid. Worth their visual cost only when the recommendation set is structured and plan-driven; ad-hoc fixes don't get R-tags.

**Rejected paths (for the record):** unified CLIP graph with dummy text inputs (replaced by D7); pure-Rust WordPiece tokenizer (replaced by D8); single "active" encoder per direction with dropdown picker (replaced by D6); per-row autocommit encoder writes (replaced by D13); 7-tier random-sampling diversity (`get_tiered_similar_images` still exists but is no longer called from the frontend).

## What is currently built

As of HEAD `ecb4386` on 2026-04-26:

- **28 backend Rust files** split across `commands/`, `db/`, `cosine/`, `encoder_text/`, plus single-file modules; **33 frontend TypeScript files**. **125/125 cargo lib tests passing**, **62/62 vitest passing**, **clippy clean**.
- **26 Tauri commands** organised by concern (was 8 in the prior November 2025 snapshot).
- **5 SQLite tables** + `meta` migration tracker; pipeline version 4 (was 1, then 2 after CLIP rewrite, then 3 after Tier 1+2 R6+R7+R8, then 4 with Phase 11/12).
- **Three image encoders** (CLIP + DINOv2 + SigLIP-2) all encoding in parallel during indexing (Phase 11e), each with its own DB connection, with dynamic `intra_threads(N/encoders)` (Phase 12).
- **Two text encoders** (CLIP + SigLIP-2) both wired through picker dispatch.
- **Reciprocal Rank Fusion** for image-image (3 encoders) AND text-image (2 encoders) — 6 unit tests pin the algorithm.
- **Multi-folder roots** with `ON DELETE CASCADE`, per-root thumbnail subfolders, non-destructive `set_root_enabled` toggle.
- **Filesystem watcher** with 5 s debounce and single-flight coalescing via `IndexingState`.
- **First-launch model download** (~2.5 GB across 7 files) with HEAD preflight + chunked GET + per-byte progress callback + per-file fail-soft.
- **Profiling layer** behind `--profiling` flag with `PerfLayer`, 12 named diagnostics, 1 Hz RSS/CPU sampler, on-exit `report.md` with Stall Analysis + Resource Trends, `cmd+shift+P` overlay.
- **Tag CRUD** (with delete affordance, `INSERT OR IGNORE` against duplicates), AND/OR filter mode, `#tag` autocomplete + create-on-no-match, per-image notes textarea persisted to `notes` column.
- **Pinterest-style masonry grid** with shortest-column packing, hero promotion across up to 3 columns, 3D framer-motion tilt, sortMode-aware ordering (stable-sort default; shuffle is opt-in).
- **Settings drawer** split into 7 sections (Theme · Display · Search · Sort · Folders · Reset · Encoder); `useUserPreferences` localStorage persistence.
- **Typed `ApiError` wire format** on every command + frontend mirror (3 profiling commands still on legacy `Result<_, String>`).
- **28-finding code-health audit complete** with every finding shipped (audit residuals: 3 small open items).

The repo's README still describes the November 2025 single-CLIP single-folder app — it is stale and is the highest-priority documentation gap.

## Current state

Status: **active** (heavy iteration; multiple multi-phase landings/day). HEAD `ecb4386`, 80 commits tracked from 2025-11-24 through 2026-04-26. Days since last meaningful commit: 0. The previous "shipped-and-shelved" framing was wrong — silence between March and April broke into a sustained sprint that has not yet stopped (31 commits on 2026-04-26 alone). In flight: pickup of code-health audit residuals (D-SIM-1 / D-SEM-1 / D-FE-1 dead legacy single-encoder commands; `Settings::priority_image_encoder` removal; `db::get_embedding` swap to R2 secondary), README rewrite, watcher rebuild on root mutations.

## Gaps and known limitations

**HIGH — README drift.** The repo README (last updated `1b5ac35` 2026-03-04) describes a single CLIP-ViT-B/32 encoder with a hardcoded `test_images/` folder and 8 Tauri commands. The code is now 3-encoder + RRF + multi-folder + 26 commands. Every prospective reader gets the wrong picture.

**MEDIUM — Code-health audit residuals (3 items):**
- Legacy single-encoder commands (D-SIM-1 / D-SEM-1 / D-FE-1): `semantic_search`, `get_similar_images`, `get_tiered_similar_images` remain registered in `lib.rs::run`'s `invoke_handler!` despite no longer being called from the frontend. ~600 Rust + ~80 TS lines.
- `Settings::priority_image_encoder` is doc-deprecated but still read in `indexing.rs`.
- `db::get_embedding` skips the R2 read-only secondary and uses the writer mutex (last R2 gap; two-line fix).

**MEDIUM — Structural risks (not currently broken, but documented):**
- Filesystem watcher does not rebuild on `add_root` / `remove_root` — filesystem events for the new root don't fire until next launch.
- `roots.path` stored verbatim with no normalisation — `/Users/me/Photos/` and `/Users/me/Photos` appear as two rows.
- Encoder set hardcoded as a `&[&str]` constant in `commands/similarity.rs::get_fused_similar_images` — adding a fourth encoder requires editing this constant in addition to `commands::encoders::ENCODERS` and `indexing.rs::run_encoder_phase`.
- Fused score (~0–0.05 for 3 encoders + k=60) is not a cosine similarity and is unbounded; latent risk if any future tooltip surfaces it as a percentage without normalisation.

**LOW — Edge cases / belt-and-braces:**
- `dirs::data_dir()` returning `None` in release falls back to `./app-data` (mostly theoretical on supported platforms).
- Atomic `Settings::save` uses `rename` not `fsync` (acceptable on macOS / Linux ext4 / Windows NTFS in practice).
- `add_root` propagates SQL UNIQUE-constraint error as `ApiError::Db` rather than `ApiError::BadInput("already added")`.
- Empty-state UI's "Choose folder" pill goes through `set_scan_root` (replace-all) rather than `add_root` — UX path inconsistent though zero-difference at empty state.
- `wipe_images_for_new_root` only fires inside `set_scan_root`, not `add_root` — legacy NULL-`root_id` rows persist (the grid query keeps them so they still display).
- Per-root thumbnail directory removal is best-effort — `remove_root` `rm -rf`s the subfolder; logs warn on permission/busy failure.
- Session-mid lock poisoning via `std::sync::Mutex` requires restart; `parking_lot::Mutex` is the strict-upgrade path documented for if it ever bites.

**Tier 4 deferred research items** all share one blocker: without a labelled retrieval-quality test set, swapping encoders or quantising risks silent regression. Items: R5 (FP16 weights), R14 (INT8 quantisation), R15 (MobileCLIP-S2), R3 (CLIP encoder upgrade audit), R10 (foreground/background encoder split, mostly obviated by Phase 5 RRF + Phase 12c parallel encoders), R11 (decode-once fan-out; bigger refactor), R12 (`with_disable_per_session_threads` — needs binding verification), R13 (`deadpool-sqlite` connection pool — probably unneeded post-R2), R16 (one last CoreML attempt with `MLProgram + RequireStaticInputShapes`).

## Direction (in-flight, not wishlist)

- **Closing audit residuals** — D-SIM-1 / D-SEM-1 / D-FE-1 deletion sweep + `Settings::priority_image_encoder` removal + `db::get_embedding` → R2. All low-risk pure-deletion or two-line fixes. Documented in `plans/code-health-audit/`.
- **README rewrite** — half a session of work, single highest-leverage closure of public-facing-vs-reality gap.
- **Watcher rebuild on root mutations** — close + recreate the `notify-debouncer-mini` handle whenever roots change. Watcher slot is already an `Arc<Mutex<Option<WatcherHandle>>>` so the surface for replacement exists.
- **Path normalisation at insert time** — closes the second half of `notes/path-and-state-coupling.md`.
- **Encrypted Vector Search (Rec-7, active Work file)** — additive FHE-on-vectors path using TFHE-rs / BFV ciphertexts of CLIP embeddings, with a parallel `EncryptedCosineIndex` operating over ciphertexts and a debug-overlay reference application. Sequencing-gated behind v1 polish completion. Honest 4-5 orders-of-magnitude slowdown framing. Apple Wally + Pacmann + Panther papers as reference points.

Profiling diagnostics expansion (causal trace substrate with `span_id`/`parent_id`, deeper DB decomposition, frontend invalidation tracing, `perfdump` + `perfdiff` CLIs) is **proposed**, not committed timing.

## Demonstrated skills

- **Designed and shipped multi-encoder retrieval fusion** — built three independent ONNX encoders (CLIP, DINOv2, SigLIP-2), correctly handling each encoder's native preprocessing (CLIP-native mean/std + bicubic-shortest-edge + center-crop; DINOv2's canonical 256-then-224; SigLIP-2's exact-square + `[-1, 1]` no-mean-no-std) and combined them via Reciprocal Rank Fusion (Cormack, Clarke & Büttcher SIGIR 2009, k=60) with 6 unit tests pinning the algorithm contract. Reasoned out of score-fusion (cross-encoder distribution differences) and into rank-fusion explicitly; documented why uniform k=60 is principled and per-encoder weighting would need a labelled validation set first.
- **Diagnosed and fixed a 22-second indexing freeze (perf-1777212369) by introducing a read-only secondary SQLite connection (R2)** alongside the writer mutex, and by replacing per-row `upsert_embedding` autocommit with batched `upsert_embeddings_batch` (`BEGIN IMMEDIATE` per ~32-row chunk + manual `checkpoint_passive` under `wal_autocheckpoint=0`). Considered a connection pool (`deadpool-sqlite`) and rejected it as over-engineered for the writer + read-only secondary pattern.
- **Built a profiling-first development culture from scratch** — opt-in `--profiling` CLI flag (specifically NOT `--profile` because Tauri 2 owns that name), `PerfLayer` over `tracing`, 12 named diagnostics across 17 call sites, 1 Hz RSS/CPU sampler thread (sysinfo), on-exit `report.md` with Stall Analysis + Resource Trends, frontend overlay (cmd+shift+P), `perfInvoke` wrapper preserving the service layer. Designed so the entire system is dormant when off (zero overhead).
- **Ran a 28-finding code-health audit and shipped every finding** — including replacing 3 sites of `unsafe { slice::from_raw_parts(...) }` with `bytemuck::cast_slice`, extracting a triplicated `normalize_path` closure into `paths::strip_windows_extended_prefix` returning `Cow::Borrowed` for zero-alloc, extracting triplicated 3-strategy DB-id lookup blocks into `commands::resolve_image_id_for_cosine_path`, extracting duplicated `aggregate_image_rows` into `db/images_query.rs`, replacing `println!` with `tracing` spans wholesale. Every audit-touched line carries an `R<n>` annotation for forward + reverse traceability.
- **Implemented multi-folder lifecycle correctly** — `roots` table with `images.root_id INTEGER REFERENCES roots(id) ON DELETE CASCADE`, `PRAGMA foreign_keys=ON` as the explicit fix (SQLite defaults OFF), per-root thumbnail subfolders (`thumbnails/root_<id>/`) so `remove_root` is `rm -rf` clean, non-destructive `set_root_enabled` toggle (filter-only, no re-encoding), legacy single-folder migration path.
- **Implemented a typed Tauri-IPC error wire format** — `ApiError` discriminated union with `#[serde(tag="kind", content="details")]`, `From<rusqlite::Error>`, `From<std::io::Error>`, `From<PoisonError<T>>` impls so command bodies use `?` directly, mirrored on the frontend in `services/apiError.ts` so it can branch on specific failure kinds (e.g. `TextModelMissing` triggering a re-download flow).
- **Designed a single-flight background indexing pipeline** with `IndexingState.is_running: AtomicBool` and an RAII guard that ensures clear on success, error, AND panic; rapid filesystem events from the `notify-debouncer-mini` watcher coalesce into a single rescan rather than spawning competing pipelines.
- **Migrated a non-trivial encoding pipeline through 4 versions** with a `meta(key, value)` SQLite migration tracker — bumping `embedding_pipeline_version` const wipes legacy embeddings on first launch under the new code (currently v4).
- **Native ML inference in Rust with no Python at runtime** — three encoders × consumer CPU × ONNX Runtime + HF `tokenizers` + a shared M2-tuned Session builder factory (`build_tuned_session`, `Level3 + intra_threads(4) + inter_threads(1)`) with dynamic `intra_threads(N/encoders)` for the parallel-by-encoder phase.
- **Built a fast thumbnail pipeline** — `jpeg-decoder::Decoder::scale()` for native scaled IDCT (1/8, 1/4, 1/2 factor) then `fast_image_resize 6.x` (NEON-optimised Lanczos3) for the final downsample, with `image-rs` fallback for non-JPEG and any decode error. Rayon-parallel.
- **Drove a Tauri 2 + React 19 desktop app from scaffold to a 28-finding-audit-passed multi-encoder fusion-search application** — including TanStack Query (`staleTime: Infinity`), `tauri-plugin-dialog` native folder picker, Pinterest-style shortest-column-packing masonry with hero promotion, settings drawer split into 7 sections, frontend `useUserPreferences` localStorage hook, `useIndexingProgress` Tauri-event hook, `cmd+shift+P` perf overlay.
- **Researched and explicitly framed FHE-on-vectors as a future additive direction** (Apple Wally, Pacmann, Panther papers; TFHE-rs as the leading pure-Rust FHE library). Honest engineering posture — names the 4-5 orders-of-magnitude slowdown as a design constraint, scopes the encrypted path as additive on top of the existing plaintext path, and identifies which retrieval modes are FHE-tractable (single-pair similarity, top-K linear scan) vs FHE-intractable (diversity-sampled across 1000s in real time).

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Image Browser/Architecture.md | 333 | "The Coverage section in the repo's own `architecture.md` § Coverage (line 553+) enumerates what its authors inspected during their 2026-04-26 upkeep — that is the deeper source of truth for what was directly read into the repo's own context layer." |
| Projects/Image Browser/Baselines.md | 212 | "| Default top_n semantic | 50 | 50 (unchanged) | |" |
| Projects/Image Browser/Decisions.md | 277 | "- `notes/encoder-additions-considered.md` (in repo) — D4 candidate inventory + decision rule for adding a 4th" |
| Projects/Image Browser/Gaps.md | 172 | "The previous vault Suggestions note recommended \"Delete the memory-bank/ folder\" + \"Fix the README/code truth gap\" + \"Resist new features until folder picker + runtime rescan ship.\" The first is done; the second is still open (now the highest-priority HIGH item above); the third was overtaken by events — the project did add features but the audit + Tier 1+2 + Phase 11/12 hardened them in parallel." |
| Projects/Image Browser/Overview.md | 153 | "`#image-browser` `#tauri` `#rust` `#react` `#clip` `#dinov2` `#siglip2` `#rrf` `#multi-encoder-fusion` `#onnx-runtime` `#sqlite-wal` `#local-first` `#ml-inference` `#profiling` `#masonry`" |
| Projects/Image Browser/Roadmap.md | 159 | "- `Capataina/PinterestStyleImageBrowser/context/plans/code-health-audit/` — the 28-finding audit + residual list" |
| Projects/Image Browser/Suggestions.md | 159 | "- [[Profile/Professional/Resume]] + [[Profile/Professional/Interests]] — portfolio-signal targets" |
| Projects/Image Browser/Systems/CLIP Image Encoder.md | 127 | "- `Capataina/PinterestStyleImageBrowser/context/notes/clip-preprocessing-decisions.md` — full history of the rewrite" |
| Projects/Image Browser/Systems/CLIP Text Encoder.md | 156 | "- `Capataina/PinterestStyleImageBrowser/context/systems/clip-text-encoder.md` — full implementation reference" |
| Projects/Image Browser/Systems/Cosine Similarity.md | 131 | "- `Capataina/PinterestStyleImageBrowser/context/systems/cosine-similarity.md` — full implementation reference (18 KB)" |
| Projects/Image Browser/Systems/DINOv2 Encoder.md | 118 | "- `Capataina/PinterestStyleImageBrowser/context/notes/encoder-additions-considered.md` — research-grade candidate inventory" |
| Projects/Image Browser/Systems/Database.md | 234 | "- `Capataina/PinterestStyleImageBrowser/context/notes/conventions.md` § BEGIN IMMEDIATE + read_lock() patterns" |
| Projects/Image Browser/Systems/Filesystem Scanner.md | 115 | "- `Capataina/PinterestStyleImageBrowser/context/systems/filesystem-scanner.md` — full implementation reference" |
| Projects/Image Browser/Systems/Frontend State.md | 187 | "- `Capataina/PinterestStyleImageBrowser/context/notes/conventions.md` § Optimistic mutation pattern" |
| Projects/Image Browser/Systems/Indexing Pipeline.md | 165 | "- `Capataina/PinterestStyleImageBrowser/context/systems/indexing.md` — full implementation reference" |
| Projects/Image Browser/Systems/Masonry Layout.md | 141 | "- `Capataina/PinterestStyleImageBrowser/context/systems/masonry-layout.md` — full implementation reference" |
| Projects/Image Browser/Systems/Model Download.md | 118 | "- `Capataina/PinterestStyleImageBrowser/context/systems/model-download.md` — full implementation reference" |
| Projects/Image Browser/Systems/Multi-Encoder Fusion.md | 179 | "- Cormack, Clarke & Büttcher (2009), *Reciprocal Rank Fusion outperforms Condorcet and individual rank learning methods*, SIGIR '09. [PDF](https://plg.uwaterloo.ca/~gvcormac/cormacksigir09-rrf.pdf)." |
| Projects/Image Browser/Systems/Multi-Folder Roots.md | 177 | "- `Capataina/PinterestStyleImageBrowser/context/systems/multi-folder-roots.md` — full implementation reference" |
| Projects/Image Browser/Systems/Paths and State.md | 139 | "- `Capataina/PinterestStyleImageBrowser/context/systems/paths-and-state.md` — full implementation reference" |
| Projects/Image Browser/Systems/Profiling.md | 162 | "- `Capataina/PinterestStyleImageBrowser/context/notes/conventions.md` § Domain diagnostics — pattern for adding new diagnostics" |
| Projects/Image Browser/Systems/Search Routing.md | 141 | "- `Capataina/PinterestStyleImageBrowser/context/notes/random-shuffle-as-feature.md` — sortMode design" |
| Projects/Image Browser/Systems/SigLIP-2 Encoder.md | 138 | "- `Capataina/PinterestStyleImageBrowser/context/systems/siglip2-encoder.md` — full implementation reference" |
| Projects/Image Browser/Systems/Tag System.md | 175 | "- `Capataina/PinterestStyleImageBrowser/context/notes/dead-code-inventory.md` § Resolved — `db::delete_tag` wiring history" |
| Projects/Image Browser/Systems/Tauri Commands.md | 209 | "- `Capataina/PinterestStyleImageBrowser/context/notes/conventions.md` § Mutex acquire-then-execute, Typed errors via `?` and `From`-impls" |
| Projects/Image Browser/Systems/Thumbnail Pipeline.md | 163 | "- `Capataina/PinterestStyleImageBrowser/context/systems/thumbnail-pipeline.md` — full implementation reference" |
| Projects/Image Browser/Systems/Watcher.md | 83 | "- `Capataina/PinterestStyleImageBrowser/context/systems/watcher.md` — full implementation reference" |
| Projects/Image Browser/Work/Encrypted Vector Search.md | 59 | "#image-browser #work #fhe #encrypted-vector #privacy-preserving" |
