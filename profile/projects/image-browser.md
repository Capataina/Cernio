---
name: Image Browser
status: active
source_repo: https://github.com/Capataina/PinterestStyleImageBrowser
lifeos_folder: Projects/Image Browser
last_synced: 2026-05-31
sources_read: 30
---

# Image Browser

## One-line summary

Local-first Tauri 2 desktop application that indexes personal image libraries with three ONNX-Runtime image encoders (CLIP ViT-B/32, DINOv2-Base, SigLIP-2 Base 256) and serves a Pinterest-style masonry UI backed by Reciprocal-Rank-Fusion image-image and text-image retrieval over SQLite-WAL with a profiling-first development culture.

## What it is

Image Browser is a local-first Tauri 2 desktop application for browsing, tagging, semantically searching, and annotating large personal image libraries. The Rust backend handles filesystem scanning, SQLite (WAL) persistence, thumbnail generation, ONNX-Runtime inference across three encoder families, multi-folder lifecycle, a filesystem watcher with orphan detection, an opt-in profiling and domain-diagnostic layer, and first-launch model downloads from HuggingFace. The React 19 frontend renders a Pinterest-style masonry grid, a modal inspector with annotations, a multi-section settings drawer, an indexing-status pill, and an opt-in perf overlay. Everything runs offline — CPU on macOS (CoreML disabled for the relevant transformer ops), CUDA on non-macOS with CPU fallback. The only network call is the first-launch ~2.5 GB model download from HuggingFace; once models are on disk, the app never reaches out again.

The project's distinguishing ambition is not "a CLIP image search demo" but an honest multi-encoder retrieval system: three encoders run at indexing time, every enabled encoder ranks the candidate set in its own embedding space at query time, and Reciprocal Rank Fusion (Cormack 2009, k=60) combines the ranked lists. That decision and its supporting infrastructure (per-encoder enable/disable, lazy per-encoder caches, parallel-by-encoder indexing, on-exit profiling report with Stall Analysis and Resource Trends) define the project's technical character.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    React 19 Frontend (WebView)                   │
│   pages/[...slug].tsx (catch-all route, search routing, hotkeys) │
│        ├─► Masonry / MasonryItem / MasonryAnchor                 │
│        ├─► PinterestModal (annotations textarea, prev/next)      │
│        ├─► SearchBar (# autocomplete + create-on-no-match)       │
│        ├─► TagDropdown                                           │
│        ├─► IndexingStatusPill (Tauri event subscription)         │
│        ├─► PerfOverlay (cmd+shift+P, profiling-only mount)       │
│        └─► settings/ (Theme · Display · Search · Sort ·          │
│                       Folders · Reset · Encoder)                 │
│   queries/  ← TanStack Query 5 (staleTime: Infinity)             │
│   services/ ← invoke wrappers + ApiError discriminated-union TS   │
│   hooks/    ← useDebouncedValue, useUserPreferences, useIndexing │
└──────────────────────────────┬──────────────────────────────────┘
                               │ Tauri IPC + Tauri events
                               │ (typed ApiError on the wire)
┌──────────────────────────────▼──────────────────────────────────┐
│                     Rust Backend (tauri::Builder)                │
│   main.rs ── parse --profiling, init tracing + opt-in PerfLayer, │
│              spawn flush + 1 Hz RSS/CPU sampler, open DB         │
│   lib.rs::run ── manage(state...) .setup(...)                    │
│                   .invoke_handler![26 commands]                  │
│   commands/ (error, images, tags, notes, roots, similarity,      │
│              semantic, semantic_fused, profiling, encoders)      │
│   db/ (mod, schema_migrations, images_query, embeddings, tags,   │
│        thumbnails, roots, notes_orphans, test_helpers)           │
│   similarity_and_semantic_search/                                │
│     encoder.rs (CLIP image)  encoder_dinov2.rs  encoder_siglip2  │
│     ort_session.rs (shared M2-tuned Session builder)             │
│     encoder_text/ (ClipTextEncoder + pooling)                    │
│     cosine/ (index.rs, math.rs, rrf.rs, diagnostics.rs, cache.rs)│
│   indexing.rs ── single-flight background pipeline               │
│   watcher.rs  ── notify-debouncer-mini, 5 s debounce             │
│   model_download.rs ── first-launch HuggingFace fetch            │
│   paths.rs    ── single disk-path source                         │
│   settings.rs ── Settings struct (enabled_encoders, legacy field)│
│   perf.rs / perf_report.rs ── PerfLayer + on-exit markdown report│
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼ disk: <app_data_dir>/
   images.db (WAL + .db-shm + .db-wal)
   settings.json (atomic .tmp + rename)
   cosine_cache.bin (bincode)
   models/{clip_vision,clip_text,dinov2_base_image,siglip2_vision,
           siglip2_text}.onnx + 2 tokenizer.json files (~2.5 GB)
   thumbnails/root_<id>/thumb_<image_id>.jpg
   exports/perf-<unix_ts>/{timeline.jsonl, report.md, raw.json}
```

**Dependency direction.** `db` is the only sink every backend module reads from or writes to and has no inverse dependencies. The post-split `db/` directory uses Rust's "multiple `impl ImageDatabase` blocks merge" property so callers see a flat API regardless of which submodule defines a method. `indexing.rs` and `watcher.rs` are coupled through `IndexingState` (single-flight `AtomicBool` with RAII guard) — rapid filesystem events that try to spawn a second pipeline get `Err(AlreadyRunning)` and silently coalesce. `commands/*` returns `Result<T, ApiError>` for every handler; the frontend deserialises `{ kind, details }` and branches on `kind`. Frontend components never call `invoke()` directly — services wrap it, hooks call services, components call hooks.

**Mutex topology.** Five long-lived sync primitives serialise backend operations:

| Primitive | Holder | Acquired by |
|---|---|---|
| `Mutex<Connection>` (writer) | `ImageDatabase.connection` | every DB write + foreground writes |
| `Mutex<Connection>` (read-only secondary R2) | `ImageDatabase.reader: OnceLock<Mutex<Conn>>` | foreground SELECTs via `read_lock()` |
| `Arc<Mutex<CosineIndex>>` | `CosineIndexState.index` (cloned to indexing thread) | every similarity / semantic command + populate + save |
| `Mutex<Option<TextEncoder>>` ×2 | `TextEncoderState.{clip, siglip2}` | semantic_search + indexing pre-warm |
| `AtomicBool` (single-flight) | `IndexingState.is_running` | every command triggering an index + watcher debounce closure |
| `Arc<Mutex<Option<WatcherHandle>>>` | `watcher_state` | lib.rs setup callback |

Std-Mutex with a `From<PoisonError>` impl converting poisoning into `ApiError::Cosine("mutex poisoned: ...")` — typed wire instead of stringly-typed errors. A panic poisons the lock for the rest of the session; restart-to-recover is the accepted posture for a single-user desktop tool.

## Subsystems and components

### Multi-encoder fusion (`cosine/rrf.rs`, `commands/similarity.rs`, `commands/semantic_fused.rs`)

Reciprocal Rank Fusion: `fused_score(p) = Σ over enabled encoders e of 1 / (k_rrf + rank_e(p))` with `k_rrf = DEFAULT_K_RRF = 60` (Cormack, Clarke & Büttcher, SIGIR 2009). Image-image fusion runs over three encoders (CLIP + SigLIP-2 + DINOv2); text-image fusion runs over two (CLIP + SigLIP-2 — DINOv2 has no text branch). The fused score is **not** a cosine similarity and is unbounded (~0–0.05 for 3 encoders + k=60). `FusionIndexState.per_encoder: Arc<Mutex<HashMap<String, CosineIndex>>>` lazy-populates the first time a given encoder is queried; `invalidate_all()` is wired into the three root-mutation IPCs (`set_scan_root`, `remove_root`, `set_root_enabled`). Every fused result carries `per_encoder: Vec<(encoder_id, 1-based-rank, encoder_score)>` so the `search_query` diagnostic can surface *why* an image was ranked highly. 6 RRF unit tests in `cosine/rrf.rs`.

### Indexing pipeline (`indexing.rs`)

Background single-flight pipeline with the following phases: cache load → model download → text-encoder pre-warm → open second DB → Phase::Scan (every enabled root, INSERT OR IGNORE + mark_orphaned) → Phase::Thumbnail (rayon par_iter, single-SELECT root-id lookup) → Phase::Encode (Phase 11e: one OS thread per enabled image encoder, each with its own DB connection) → `cosine::populate_from_db` + `save_to_disk` → Phase::Ready. Each phase emits an `indexing-progress` Tauri event. Single-flight via `IndexingState.is_running` AtomicBool with a `RunningGuard` RAII Drop that clears the flag on success, error, AND panic. Phase 12 dynamic `intra_threads(N / enabled_encoders)` shares the M2 P-cluster across parallel encoders instead of oversubscribing. `indexing.rs` is a single 47 KB file; the code-health audit recommends a four-file split.

### Cosine retrieval engine (`cosine/`)

Post-split into `mod.rs` + `index.rs` + `math.rs` + `rrf.rs` + `diagnostics.rs` + `cache.rs` (from a single 860-line `cosine_similarity.rs`). `get_similar_images_sorted` uses `select_nth_unstable_by` partial sort (2.53× speedup at n=10000 per the diagnostic integration test) and a reusable `Vec<f32>` scratch buffer to avoid PathBuf clones per inner loop. Three retrieval modes are preserved: `get_similar_images` (sampled), `get_similar_images_sorted` (used by fusion), `get_tiered_similar_images` (legacy pre-fusion 7-tier random sampler, no longer called from frontend). Persistent `cosine_cache.bin` (bincode) loaded via `load_from_disk_if_fresh` with DB mtime freshness check. Four quality diagnostics emitted on every populate: `embedding_stats`, `pairwise_distance_distribution`, `self_similarity_check`, `cosine_cache_populated`.

### Database (`db/`)

SQLite (rusqlite) with WAL + `synchronous=NORMAL` + `foreign_keys=ON` + `busy_timeout=5000` + `wal_autocheckpoint=0` (manual `checkpoint_passive` between encoder batches) + `journal_size_limit=64 MiB`. Five tables (`roots`, `images`, `tags`, `images_tags`, `embeddings`) + `meta` (migration version). **Dual connection per real DB**: writer `Mutex<Connection>` + read-only secondary `OnceLock<Mutex<Connection>>` opened at `initialize()` time; foreground SELECTs route through `read_lock()`. Encoder writes batched via `BEGIN IMMEDIATE` per ~32-row chunk in `upsert_embeddings_batch`; embedding `&[f32]` cast to `&[u8]` BLOB via `bytemuck::cast_slice` (safe, zero-copy — replaces three previous `unsafe { slice::from_raw_parts(...) }` blocks in audit `0bdb5f4`). Four idempotent ALTER-TABLE migrations + an `embedding_pipeline_version` meta migration that wipes legacy rows when bumped (currently version 4, bumped 2026-04-26).

### Multi-folder roots (`db/roots.rs`, `commands/roots.rs`)

`roots(id, path UNIQUE, enabled, added_at)` table; `images.root_id INTEGER REFERENCES roots(id) ON DELETE CASCADE`. `PRAGMA foreign_keys = ON` is the explicit fix that makes CASCADE actually fire (SQLite defaults this OFF). Per-root thumbnail subfolder layout `thumbnails/root_<id>/thumb_<id>.jpg` so `remove_root` can `rm -rf` the subfolder in one filesystem call. `set_root_enabled` toggles a SQL filter without re-encoding; `add_root` is granular (does not remove existing roots); `set_scan_root` is replace-all (legacy semantic). Idempotent legacy single-folder migration runs once at startup. 11 unit tests in `db/roots.rs::tests`.

### Watcher (`watcher.rs`)

`notify-debouncer-mini` with 5 s debounce, recursive on every enabled root. Debounce-callback calls `try_spawn_pipeline` so rapid filesystem events get `Err(AlreadyRunning)` and silently coalesce. Known gap: `watcher_state` is filled once during `lib.rs::run::setup` and never refreshed, so filesystem events for a root added after launch do not fire until the next launch.

### Model download (`model_download.rs`)

First-launch HuggingFace fetch of ~2.5 GB across seven files (5 ONNX + 2 tokenizer.json). HEAD preflight per file (skip if existing with matching size) + chunked GET with per-byte progress callback; per-file fail-soft so a single 401/404 doesn't abort the batch.

### Encoders (`similarity_and_semantic_search/`)

Three image encoders share a single M2-tuned ONNX session builder (`ort_session.rs::build_tuned_session` with `Level3 + intra_threads(4) + inter_threads(1)`). CLIP and SigLIP-2 also have text encoders wired through a picker dispatch in `commands/semantic.rs`. CoreML EP is disabled for ALL encoders on macOS (runtime errors on transformer ops in CLIP text and SigLIP-2 text); CUDA EP is target-gated on non-macOS with CPU fallback.

### Tauri commands (`commands/`)

26 commands grouped by concern (images, tags, notes, roots, similarity, semantic, semantic_fused, profiling, encoders). Every command returns `Result<T, ApiError>` where `ApiError` is a `#[serde(tag="kind", content="details")]` discriminated union; `From` impls for `rusqlite::Error`, `std::io::Error`, and `std::sync::PoisonError<T>` let command bodies use `?` directly. Frontend mirrors the union in `services/apiError.ts` so the UI can branch on specific kinds (e.g. `isMissingModelError` triggers a re-download flow on `TextModelMissing` / `ImageModelMissing` / `TokenizerMissing`).

### Frontend state (`src/queries/`, `src/hooks/`, `src/services/`)

TanStack Query 5 with `staleTime: Infinity` + manual invalidation only (mutations explicitly invalidate affected keys; `Phase::Ready` indexing event triggers `invalidateQueries(["images"])`). Settings drawer split into seven sections (Theme, Display, Search, Sort, Folders, Reset, Encoder) — audit modularisation finding. `useUserPreferences` stores prefs in `localStorage["imageBrowserPrefs"]`; theme is applied to `<html>` in `main.tsx` *before* React mounts to avoid FOUC. `useIndexingProgress` subscribes to `indexing-progress` Tauri events. `perfInvoke<T>(cmd, args)` wraps every IPC call to emit `recordAction("ipc.start"/"ipc.end")` breadcrumbs when profiling is on, pure pass-through when off.

### Thumbnail pipeline (`thumbnail/generator.rs`)

400×400 max, aspect-preserving, JPEG. JPEG fast path uses `jpeg-decoder::Decoder::scale()` for native scaled IDCT at 1/8, 1/4, 1/2 factor (Tier 2 R7), then `fast_image_resize 6.x` (NEON-optimised Lanczos3) for the final precise downsample (R6). Falls back to `image-rs` for non-JPEG (PNG, GIF, BMP, TIFF, WebP) or decode error. Tile dimensions are recorded in the DB at thumbnail-generation time and consumed by the masonry layout directly (no DOM image-load round-trip per tile — audit `fb23bdb`).

### Profiling (`perf.rs`, `perf_report.rs`)

Opt-in via `--profiling` CLI flag (NOT `--profile` — Tauri 2 owns that for cargo-profile selection) OR `PROFILING=1` env var. Three layers of evidence: spans (`tracing::info_span!` with `ipc.` / `pipeline.` / `cosine.` / `model_download.` / `watcher.` prefixes) collected by `PerfLayer` into `SpanStats { count, sum, min, max, recent_ringbuffer }`; 12 named diagnostics emitted via `record_diagnostic(name, json_payload)`; 1 Hz RSS/CPU sampler (sysinfo 0.32). On `tauri::RunEvent::Exit`, `perf_report::render_session_report` writes `<app_data_dir>/exports/perf-<unix_ts>/report.md` including Stall Analysis (spans where p99/p50 > threshold) and Resource Trends sections plus `timeline.jsonl` and `raw.json`. Zero overhead when off (`record_diagnostic` early-returns before constructing any JSON; `<PerfOverlay>` only mounts when `isProfilingEnabled()` returns true; toggle via `cmd+shift+P`).

## Technologies and concepts demonstrated

### Languages

- **Rust** (edition 2021) — entire backend, 28 backend files across `commands/`, `db/`, `similarity_and_semantic_search/`, plus single-file modules (`indexing.rs`, `watcher.rs`, `model_download.rs`, `paths.rs`, `settings.rs`, `perf.rs`, `perf_report.rs`, `main.rs`, `lib.rs`). Heavy use of `Arc`, `Mutex`, `AtomicBool`, `OnceLock`, RAII Drop guards for single-flight, `Cow` for zero-alloc path normalisation.
- **TypeScript** — entire frontend, 33 TS files. React 19 + TanStack Query 5 + Vite 7 with `vite-plugin-pages`.

### Frameworks and libraries

- **Tauri 2** (`com.ataca.image-browser`) — desktop shell; Vite 7 frontend bundler; native folder picker via `tauri-plugin-dialog`; `manage()` state injection across six pieces of long-lived state.
- **React 19** + **TanStack Query 5** (`staleTime: Infinity`, manual invalidation) + **framer-motion** (3D hover tilt on masonry tiles) + **shadcn primitives** for settings drawer.
- **rusqlite** with WAL — five tables + meta migration version.
- **`ort = 2.0.0-rc.10`** — ONNX Runtime Rust bindings; single shared M2-tuned `Session` builder factory.
- **`tokenizers = "0.22.2"`** (HuggingFace Rust crate) — uniform BPE (CLIP, vocab 49 152, max 77) + SentencePiece (SigLIP-2 Gemma vocab 256k, max 64) via shared `tokenizer.json` interface.
- **`fast_image_resize 6.x`** — NEON-optimised Lanczos3 thumbnail resize.
- **`jpeg-decoder` 0.3** — native scaled IDCT at 1/8, 1/4, 1/2 factor (Tier 2 R7 fast path).
- **`bytemuck`** — safe, zero-copy `&[f32]` ↔ `&[u8]` casts for embedding BLOB storage.
- **`notify-debouncer-mini`** — filesystem watcher with 5 s debounce.
- **`rayon`** — parallel thumbnail generation (`par_iter`).
- **`tracing`** + **`tracing_subscriber`** — span instrumentation collected by a custom `PerfLayer`.
- **`sysinfo` 0.32** — 1 Hz RSS/CPU sampler thread.
- **`bincode`** — persistent `cosine_cache.bin` serialisation.
- **`dirs`** — platform `data_dir()` resolution for `<app_data_dir>`.

### Runtimes / engines / platforms

- **ONNX Runtime** via `ort = 2.0.0-rc.10` — CPU-only on macOS (CoreML disabled for ALL encoders due to transformer-op runtime errors), CUDA on non-macOS with CPU fallback. `Level3` graph optimisation, `intra_threads=4 + inter_threads=1` baseline tuned for the M2 P-cluster; Phase 12 dynamic `intra_threads(N / enabled_encoders)` shares the cluster across parallel encoders.
- **SQLite WAL** — writer + read-only secondary connection pattern (R2) closes the perf-1777212369 22 s `ipc.get_images` freeze. Manual `checkpoint_passive` between encoder batches under `wal_autocheckpoint=0`.
- **Tauri 2 WebView** — assetProtocol for image loading at gallery scale (avoids base64-over-IPC).

### Tools

- **`cargo tauri dev` / `cargo tauri build`** — development + bundling.
- **`vitest`** — 62/62 passing frontend tests.
- **`cargo test --lib`** — 125/125 passing backend tests including 11 roots-CRUD, 6 RRF unit tests, `cosine_topk_partial_sort_diagnostic.rs` integration.
- **`clippy`** — restored gate post-audit.
- **`--profiling` flag + on-exit `report.md`** — the project's primary debugging tool; built specifically because the perf-1777212369 22 s freeze was undebuggable without it.

### Domains and concepts

- **Multi-encoder rank fusion (RRF, Cormack 2009)** — implements `score(p) = Σ 1 / (k + rank_e(p))` with canonical k=60; per-encoder evidence in diagnostics; lazy per-encoder cache populates.
- **Local-first ML inference at scale** — three encoders running through ONNX Runtime + HF tokenizers on consumer CPU; pure-Rust posture (no Python at runtime).
- **Vision-language retrieval** — CLIP ViT-B/32 OpenAI English (512-d) for concept overlap, DINOv2-Base (768-d, image-only) for visual/structural similarity, SigLIP-2 Base 256 (768-d shared text+image) for descriptive content alignment.
- **Cross-encoder preprocessing discipline** — CLIP-native mean `[0.48145466, 0.4578275, 0.40821073]` and std `[0.26862954, 0.26130258, 0.27577711]` (replaced ImageNet stats which biased the CLIP distribution); bicubic-shortest-edge-224 + center-crop (replaced `resize_exact` which squashed); L2-normalize on output; DINOv2 uses ImageNet stats + bicubic-shortest-edge-256 + center-crop-224 + CLS-token from `last_hidden_state[:,0,:]`; SigLIP-2 uses `[-1, 1]` range with exact-square 256×256 + Gemma SentencePiece 64-token text with no attention_mask.
- **Lock-free single-flight via AtomicBool + RAII guard** — `RunningGuard(Arc<IndexingState>)` implements `Drop` to clear the flag on success, error, AND panic, so a panic inside the pipeline cannot lock out future indexing.
- **Typed wire format for IPC errors** — `#[serde(tag, content)]` ApiError discriminated union with `From<PoisonError>` mapping mutex poisoning to a typed signal.
- **Production-grade self-observability** — three-layer evidence (spans + diagnostics + resource samples) with on-exit Stall Analysis + Resource Trends report. Explicitly framed as "encode the validation into the runtime; the report is the debugger" — replaces a deferred Python-reference comparison harness.
- **Partial-sort top-K (`select_nth_unstable_by`)** — 2.53× speedup at n=10000 over full sort, verified in `tests/cosine_topk_partial_sort_diagnostic.rs`.
- **Scaled IDCT for JPEG thumbnails** — skipping high-frequency DCT coefficients before the downsample yields ~5–10× faster thumbnailing for typical JPEG libraries.
- **WAL + dual-connection pattern** — writer Mutex + read-only secondary OnceLock<Mutex> keeps foreground SELECTs non-blocking against in-flight encoder write batches; `BEGIN IMMEDIATE` per ~32-row chunk avoids deadlock with concurrent readers upgrading to writers.
- **Encrypted vector search (FHE)** — the project has a documented additive plan for TFHE-rs / BFV ciphertext storage parallel to plaintext embeddings, an `EncryptedCosineIndex` doing linear-scan top-K over ciphertexts, with honest framing of FHE's 4–5 orders-of-magnitude slowdown vs plaintext.

## Key technical decisions

- **D1 — Tauri 2 over Electron** for smaller bundle (8–15 MB vs 100+ MB), lower memory (~30–40 MB vs 200–300 MB), Rust safety for filesystem and ML paths, ergonomic `ort` integration, and `assetProtocol` for image loading at scale. Same choice in Aurix — becoming a repeatable Capataina pattern.
- **D2 — Local-first, no cloud, no accounts.** Only network call is first-launch model download. Privacy by construction, not configuration. The FHE-on-vector-search work item strengthens local-first rather than weakening it.
- **D3 — SQLite over an embedded vector DB.** Already needs SQL for tags/notes/roots/relations; adding a second store would be the more complex choice. Brute-force cosine over a few-thousand 512–768-d vectors fits comfortably in RAM. HNSW behind a trait (`enhancements/recommendations/02-hnsw-index-behind-trait.md`) is the documented future expansion past ~50k images.
- **D4 — Three encoders (CLIP + DINOv2 + SigLIP-2), not one.** No single encoder is best at every retrieval task: CLIP cares about concept overlap, DINOv2 about visual/structural similarity, SigLIP-2 about descriptive content. The candidate inventory for a fourth lives in `notes/encoder-additions-considered.md` (Tier A: OpenCLIP-LAION-2B, EVA-CLIP-B/16, SigLIP-2 Large-384; Tier B: perceptual hash for "Find Duplicates", MobileCLIP held for future CoreML attempt).
- **D5 — Reciprocal Rank Fusion (RRF), k=60, not score-fusion.** Different encoders produce cosines on different distributions — CLIP's "0.85" is not comparable to DINOv2's "0.85". L2 normalisation alone doesn't fix this. RRF discards the score and uses only the *rank*, sidestepping cross-encoder distribution differences entirely. k=60 is the canonical Cormack 2009 balance between top-of-list dominance (small k) and consensus contribution (large k); shipping anything else without a labelled validation set would be unprincipled.
- **D6 — Per-encoder enable/disable, not always-all.** Persisted in `settings.json::enabled_encoders`; toggling does NOT delete embeddings (cheap to keep, expensive to regenerate); guard against disabling all 3 lives in `commands::encoders::decide_enabled_write`.
- **D7 — Separate-graph CLIP exports (`vision_model.onnx` + `text_model.onnx`).** Replaced the unified-graph dummy-text-input hack at image-encode time; smaller files individually.
- **D8 — HuggingFace `tokenizers` crate over hand-rolled WordPiece.** Single crate handles BPE (CLIP) and SentencePiece (SigLIP-2) via uniform `tokenizer.json` interface. The previous "pure-Rust no-C-deps" position was traded for ergonomics + correctness.
- **D9 — Shared M2-tuned `Session` builder.** Single factory in `ort_session.rs::build_tuned_session` with `Level3 + intra_threads(4) + inter_threads(1)`; every encoder constructor routes through it so future tuning changes land in one place.
- **D10 — CoreML disabled for ALL encoders on macOS.** CoreML produces runtime inference errors for the transformer ops in CLIP text and SigLIP-2 text. Uniform disable avoids a per-encoder allow-list that would silently break on CoreML version bumps.
- **D11 — `--profiling` CLI flag (NOT `--profile`).** Tauri 2 reserves `--profile` for cargo-profile selection; collision is documented in `main.rs`'s comment so the next person doesn't try to "fix" it.
- **D12 — WAL + read-only secondary connection (R2).** Closes the perf-1777212369 22 s freeze. A connection pool (`deadpool-sqlite` / `r2d2-sqlite`) was considered and rejected as over-engineered for the writer + read-only secondary pattern; deferred as R13 if concurrent-read requirements grow.
- **D13 — `BEGIN IMMEDIATE` for batched encoder writes.** One transaction + one fsync per ~32-row chunk replaces the per-row autocommit pattern that triggered the 22 s freeze. Manual `checkpoint_passive` between batches keeps WAL bounded under `wal_autocheckpoint=0`.
- **D14 — `paths::*_dir()` as the single disk-path source; no dev/release split.** The previous `cfg(debug_assertions)` split pointed dev builds at `<repo>/Library/`; switching between dev and release forced 2.5 GB of model re-downloads. `IMAGE_BROWSER_DATA_DIR` env var overrides for testing/multi-instance.
- **D15 — Multi-folder roots with `ON DELETE CASCADE` gated by `PRAGMA foreign_keys=ON`.** Without the pragma, CASCADE silently no-ops — the pragma is the explicit fix that made root removal actually wipe its images.
- **D16 — `bytemuck::cast_slice` for embedding BLOB encoding.** Replaces three `unsafe { slice::from_raw_parts(...) }` blocks (audit `0bdb5f4`); safe, zero-copy, alignment-checked at compile time.
- **D17 — Typed errors via `ApiError` enum, not strings.** `#[serde(tag="kind", content="details")]` discriminated union mirrored in `services/apiError.ts`; `From<rusqlite::Error>`, `From<std::io::Error>`, `From<PoisonError<T>>` impls let command bodies use `?`. The three profiling commands still use `Result<_, String>` for legacy reasons and are tracked as residuals.
- **D-MUTEX — `std::sync::Mutex` over `parking_lot::Mutex`.** Single-user desktop posture: restart is fast; poison-then-restart beats flailing in a partially-broken state. `parking_lot` is the strict-upgrade if poisoning ever proves load-bearing.
- **D-RTAG — R-tag perf annotation comments in code.** 43 `R<n>` inline-comment annotations across the perf-bundle commit set provide forward + reverse traceability to the (since-deleted) per-recommendation perf plan. Lifecycle is undecided — `notes/conventions.md` is explicit that R-tags are worth their visual cost only when the recommendation set is plan-driven; ad-hoc fixes don't get them.

## What is currently built

- **28 Rust backend files + 33 TypeScript frontend files** (per `scan.json` against HEAD `7e5cf85` / 2026-04-28).
- **26 Tauri commands** grouped by concern under `commands/` (images: 1, tags: 5, notes: 2, roots: 6, similarity: 3, semantic: 1, semantic_fused: 1, profiling: 5, encoders: 3). Three are documented audit residuals (`get_similar_images`, `get_tiered_similar_images`, `semantic_search`) — kept registered for caller stability, not called from the frontend; ~600 Rust + ~80 TS lines preserved.
- **Three working image encoders** (CLIP ViT-B/32 separate-graph, DINOv2-Base, SigLIP-2 Base 256) and **two working text encoders** (CLIP BPE, SigLIP-2 SentencePiece) all wired through the shared M2-tuned `Session` builder.
- **Image-image RRF over 3 encoders** and **text-image RRF over 2 encoders** both wired as the active search paths (Phase 5 + Phase 11d).
- **Multi-folder roots with CASCADE** + per-root thumbnail subfolders + `set_root_enabled` toggle without re-encode + idempotent legacy single-folder migration.
- **Async indexing pipeline** with live progress events, parallel-by-encoder phase (Phase 11e), and single-flight via AtomicBool + RAII guard.
- **Filesystem watcher** with 5 s debounce and single-flight coalescing.
- **First-launch HuggingFace model download** (~2.5 GB across 7 files, per-file fail-soft).
- **Profiling system** with `--profiling` CLI flag, 12 named diagnostics, 1 Hz RSS/CPU sampler, on-exit `report.md` with Stall Analysis + Resource Trends, frontend perfInvoke wrapper + `<PerfOverlay>` mounted on `cmd+shift+P`.
- **Tag CRUD** with delete + AND/OR filter mode + `#tag` autocomplete with create-on-no-match + delete affordance in `TagDropdown`.
- **Per-image notes** (Phase 11 annotations) persisted in `images.notes`.
- **Settings drawer** split into 7 sections (Theme · Display · Search · Sort · Folders · Reset · Encoder) post-audit.
- **Typed error wire** via ApiError discriminated union, mirrored in `services/apiError.ts`.
- **125 cargo lib tests** + **62 vitest tests** passing; clippy clean.
- **28-finding code-health audit shipped end-to-end** (every finding implemented, not "logged for later"); 3 small residuals remain.
- **Pipeline-version migration system** currently at version 4 — bumping the const wipes legacy embeddings on first launch under new code.

## Current state

`active` — between 2026-04-25 and 2026-04-26 the project landed a sustained 31-commit-in-one-day sprint (full encoder-pipeline overhaul, multi-folder + watcher, profiling stack, 28-finding audit, Tier 1+2 perf bundles, RRF for image-image + text-image, Phase 11e parallel encoders, Phase 12 perf bundle). On 2026-04-28 the README rewrite (`7e5cf85`, +208/-132 lines) closed the previously HIGH-severity drift item. As of the LifeOS `last_verified: 2026-05-13` snapshot the code state at HEAD is unchanged since the 2026-04-26 extraction (the only commit between is the docs-only README rewrite); the 15-day quiescence reads as post-sprint cooldown relative to the project's prior 4-month silences. In-flight items per LifeOS Work/: the Encrypted Vector Search MVP (proposed, not started — additive on top of plaintext path) and the README walkthrough video demo (pending Caner recording).

## Gaps and known limitations

**Medium — code-health audit residuals (the explicitly-tracked next pickup):**

- Legacy single-encoder commands `get_similar_images` (D-SIM-1), `semantic_search` (D-SEM-1), and `get_tiered_similar_images` (D-FE-1) remain registered in `lib.rs::run`'s `invoke_handler!` and ~600 Rust + ~80 TS lines exist solely because they were superseded but not deleted. Pure deletion sweep; documented in `plans/code-health-audit/`.
- `Settings::priority_image_encoder` is doc-deprecated but still read in `indexing.rs` — a Phase 11c per-encoder enable/disable left it stranded. One-line removal.
- `db::get_embedding` still uses the writer mutex while every other foreground SELECT routes through `read_lock()`. A fusion search calling `get_embedding(image_id, encoder_id)` for the query vector can stall briefly on the writer mutex during indexing. Two-line fix; last R2 gap.

**Medium — structural risks (documented, not currently broken):**

- Filesystem watcher does not rebuild on `add_root` / `remove_root`. Adding a folder after launch updates the DB + spawns one rescan, but subsequent filesystem events for the new folder don't fire until the next launch.
- `roots.path` is stored verbatim (no normalisation). A user picking `/Users/me/Photos/` then `/Users/me/Photos` (no trailing slash) gets two rows. Cosmetic — the Folders list shows duplicates.
- Encoder set is hardcoded as `["clip_vit_b_32", "siglip2_base", "dinov2_base"]` in `commands/similarity.rs`; adding a fourth requires editing this constant in addition to registering in `commands::encoders::ENCODERS` and adding a branch in `indexing.rs::run_encoder_phase`.
- The fused score (~0–0.05 for 3 encoders + k=60) is **not** a cosine similarity and is unbounded. The masonry grid does not display it today, so this is invisible to users — but any future tooltip surfacing it must label it "Fused" rather than "Cosine similarity" or normalise to [0, 1].

**Low — documented edge cases:** `dirs::data_dir()` returning None falls back to `./app-data` (warn-logged); `Settings::save` uses atomic `.tmp` + rename without explicit `fsync` (modern filesystems handle this implicitly); `add_root` propagates UNIQUE-constraint errors as `ApiError::Db` rather than `ApiError::BadInput("already added")`; per-root thumbnail directory removal is best-effort (`rm -rf` logs warn on permission failure); session-mid mutex poisoning requires restart (std-Mutex; `parking_lot::Mutex` is the strict-upgrade if it ever bites).

**Tier 4 deferred research items** all share the same blocker: without a labelled (query, expected matches) test set, swapping encoders or quantising risks silent retrieval-quality regression. Items: R5 FP16 ONNX weights, R14 INT8 quantisation, R15 MobileCLIP-S2 evaluation, R3 CLIP encoder upgrades (EVA-CLIP, DFN-CLIP), R16 one last CoreML attempt with `MLProgram + RequireStaticInputShapes`.

## Direction (in-flight, not wishlist)

- **Audit residuals deletion sweep** (D-SIM-1 / D-SEM-1 / D-FE-1 + `Settings::priority_image_encoder` removal + `db::get_embedding` R2 fix). All three are low-risk and close known surface drift; a focused half-session.
- **Watcher rebuild on root mutations.** The watcher slot is already `Arc<Mutex<Option<WatcherHandle>>>` so the surface for replacement exists; ~half a session.
- **`indexing.rs` four-file split** into `pipeline.rs` / `encoder_phase.rs` / etc. Pure-movement extraction; audit recommendation.
- **`pages/[...slug].tsx` route extraction** (516-line component; audit recommendation to pull route-state hooks into per-concern files).
- **README walkthrough video** at `image-browser-demo.mp4`, 30–60s under a `## Demo` section (LifeOS Work/README Demo.md, pending Caner recording).
- **Encrypted Vector Search MVP** (LifeOS Work/Encrypted Vector Search.md): additive TFHE-rs / BFV ciphertext storage alongside plaintext path, parallel `EncryptedCosineIndex` doing linear-scan top-K over ciphertexts, reference-application overlay showing "ciphertext only" indicator, benchmark harness measuring the 4–5 orders-of-magnitude FHE slowdown honestly. Sequencing gate: requires v1 polish to ship first; strictly additive, plaintext path stays in production.

## Demonstrated skills

- **Multi-encoder rank-fusion retrieval system.** Implements Reciprocal Rank Fusion (Cormack 2009) over three image encoders and two text encoders, with `k=60` chosen as the canonical balance, per-encoder evidence in diagnostics for *why* each fused result won, lazy per-encoder caches with `invalidate_all()` wired into root-mutation IPCs, and non-destructive toggle semantics that preserve embeddings (cheap to keep, expensive to regenerate). 6 RRF unit tests pin the contract.
- **Local-first ML inference in pure Rust on consumer CPU.** Three ONNX encoders + two text encoders + HuggingFace `tokenizers` crate (BPE + SentencePiece via uniform `tokenizer.json` interface) + shared M2-tuned `Session` builder (Level3, intra=4, inter=1) with Phase 12 dynamic `intra_threads(N/encoders)` for parallel-by-encoder sharing. CoreML target-disabled uniformly due to transformer-op runtime errors; CUDA target-gated on non-macOS with CPU fallback. No Python at runtime.
- **SQLite-WAL performance engineering.** Writer + read-only secondary connection pattern (`OnceLock<Mutex<Conn>>`) closing a real 22 s freeze; `BEGIN IMMEDIATE` per ~32-row chunk for encoder write batches; manual `checkpoint_passive` under `wal_autocheckpoint=0`; `bytemuck::cast_slice` replacing 3 `unsafe { slice::from_raw_parts(...) }` blocks; partial-sort `select_nth_unstable_by` for 2.53× top-K speedup at n=10000 verified in an integration test.
- **Production-grade self-observability for a personal project.** PerfLayer + 12 named domain diagnostics + 1 Hz RSS/CPU sampler + on-exit `report.md` with Stall Analysis + Resource Trends sections + frontend `perfInvoke` wrapper correlating IPC start/end pairs + React.Profiler integration. Built as infrastructure before it hurt; chose Tier 1+2 perf bundles by reading the Stall Analysis section, not by guessing. Explicitly framed: "encode the validation into the runtime; the report is the debugger" — replaces a deferred Python-reference comparison harness.
- **Typed IPC error contract.** `ApiError` `#[serde(tag, content)]` discriminated union with `From` impls for `rusqlite::Error` / `std::io::Error` / `std::sync::PoisonError<T>` lets command bodies use `?` directly; frontend mirrors the union and branches on `kind` (e.g. `isMissingModelError` triggers re-download flow on `TextModelMissing` / `ImageModelMissing` / `TokenizerMissing`). Generic strings would force string-matching and break on phrasing changes.
- **JPEG-aware thumbnail pipeline.** `jpeg-decoder::Decoder::scale()` for native scaled IDCT at 1/8, 1/4, 1/2 factor (skipping high-frequency DCT coefficients that would be discarded anyway), followed by `fast_image_resize 6.x` (NEON-optimised Lanczos3) for the final precise downsample. ~5–10× faster for typical JPEG inputs vs the `image-rs` default Lanczos3 path. `fast_image_resize` fallback for non-JPEG (PNG, GIF, BMP, TIFF, WebP).
- **Lock-free single-flight via AtomicBool + RAII guard.** `RunningGuard(Arc<IndexingState>)` implements `Drop` to clear the flag on success, error, AND panic — without the guard a panic inside the pipeline would lock out future indexing for the rest of the session. Watcher debounce-callbacks try-spawn; rapid filesystem events get `Err(AlreadyRunning)` and silently coalesce.
- **Idempotent multi-folder lifecycle.** `roots` table + `images.root_id INTEGER REFERENCES roots(id) ON DELETE CASCADE` gated by `PRAGMA foreign_keys=ON`; per-root thumbnail subfolders so `remove_root` `rm -rf`s in one syscall; `set_root_enabled` toggles a SQL filter without re-encoding; idempotent legacy single-folder migration runs once and clears the legacy setting field.
- **Audit discipline.** 28-finding code-health audit shipped end-to-end (every finding implemented, not "logged for later"); clippy gate restored; dead-code inventory current; residuals list small and explicitly tracked. The `R-tag` annotation convention (43 `R<n>` inline comments) provides forward + reverse traceability between source lines and recommendation numbers.
- **Cross-encoder preprocessing correctness.** CLIP-native mean/std replacing biased ImageNet stats; bicubic-shortest-edge + center-crop replacing `resize_exact` (which squashed non-square images); L2-normalize on output (embeddings had previously drifted off the unit sphere); SigLIP-2 `[-1, 1]` range with exact-square 256×256 and Gemma SentencePiece tokenizer with no attention_mask; DINOv2 CLS-token from `last_hidden_state[:,0,:]` (not the pooled output). Each documented with the previous wrong behaviour and the fix.
- **Pipeline-versioned embedding migration.** `meta(key='embedding_pipeline_version')` migration that wipes affected legacy rows when bumped, paired with code-side preprocessing changes; currently at version 4 (bumped 2026-04-26). Surfaces the "preprocessing change invalidates embeddings" coupling explicitly rather than letting it become silent.
- **FHE-on-vectors literacy.** A documented, honestly-scoped additive plan for encrypted vector search using TFHE-rs / BFV ciphertexts (validated by Apple Wally's production iOS 18 deployment) with explicit acknowledgment of the 4–5 orders-of-magnitude FHE slowdown vs plaintext and a benchmark harness designed to measure that cost honestly rather than minimise it.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Image Browser/_Overview.md | 159 | "`#image-browser` `#tauri` `#rust` `#react` `#clip` `#dinov2` `#siglip2` `#rrf` `#multi-encoder-fusion` `#onnx-runtime` `#sqlite-wal` `#local-first` `#ml-inference` `#profiling` `#masonry`" |
| Projects/Image Browser/Architecture.md | 333 | "The Coverage section in the repo's own `architecture.md` § Coverage (line 553+) enumerates what its authors inspected during their 2026-04-26 upkeep — that is the deeper source of truth for what was directly read into the repo's own context layer." |
| Projects/Image Browser/Baselines.md | 212 | "\| Default top_n semantic \| 50 \| 50 (unchanged) \| \|" |
| Projects/Image Browser/Decisions.md | 277 | "- `notes/encoder-additions-considered.md` (in repo) — D4 candidate inventory + decision rule for adding a 4th" |
| Projects/Image Browser/Gaps.md | 156 | "The previous vault Suggestions note recommended \"Delete the memory-bank/ folder\" + \"Fix the README/code truth gap\" + \"Resist new features until folder picker + runtime rescan ship.\" The first is done; the second is still open (now the highest-priority HIGH item above); the third was overtaken by events — the project did add features but the audit + Tier 1+2 + Phase 11/12 hardened them in parallel." |
| Projects/Image Browser/Roadmap.md | 163 | "- `Capataina/PinterestStyleImageBrowser/context/plans/code-health-audit/` — the 28-finding audit + residual list" |
| Projects/Image Browser/Suggestions.md | 157 | "- [[Profile/Professional/Resume - Ata Caner Cetinkaya]] + [[Profile/Professional/Interests]] — portfolio-signal targets" |
| Projects/Image Browser/Systems/_Overview.md | 55 | "- [[Projects/Image Browser/Roadmap]] — direction of travel" |
| Projects/Image Browser/Systems/CLIP Image Encoder.md | 127 | "- `Capataina/PinterestStyleImageBrowser/context/notes/clip-preprocessing-decisions.md` — full history of the rewrite" |
| Projects/Image Browser/Systems/CLIP Text Encoder.md | 156 | "- `Capataina/PinterestStyleImageBrowser/context/systems/clip-text-encoder.md` — full implementation reference" |
| Projects/Image Browser/Systems/Cosine Similarity.md | 131 | "- `Capataina/PinterestStyleImageBrowser/context/systems/cosine-similarity.md` — full implementation reference (18 KB)" |
| Projects/Image Browser/Systems/DINOv2 Encoder.md | 118 | "- `Capataina/PinterestStyleImageBrowser/context/notes/encoder-additions-considered.md` — research-grade candidate inventory" |
| Projects/Image Browser/Systems/Database.md | 234 | "- `Capataina/PinterestStyleImageBrowser/context/notes/conventions.md` § BEGIN IMMEDIATE + read_lock() patterns" |
| Projects/Image Browser/Systems/Filesystem Scanner.md | 116 | "- `Capataina/PinterestStyleImageBrowser/context/systems/filesystem-scanner.md` — full implementation reference" |
| Projects/Image Browser/Systems/Frontend State.md | 187 | "- `Capataina/PinterestStyleImageBrowser/context/notes/conventions.md` § Optimistic mutation pattern" |
| Projects/Image Browser/Systems/Indexing Pipeline.md | 165 | "- `Capataina/PinterestStyleImageBrowser/context/systems/indexing.md` — full implementation reference" |
| Projects/Image Browser/Systems/Masonry Layout.md | 141 | "- `Capataina/PinterestStyleImageBrowser/context/systems/masonry-layout.md` — full implementation reference" |
| Projects/Image Browser/Systems/Model Download.md | 118 | "- `Capataina/PinterestStyleImageBrowser/context/systems/model-download.md` — full implementation reference" |
| Projects/Image Browser/Systems/Multi-Encoder Fusion.md | 179 | "- Cormack, Clarke & Büttcher (2009), *Reciprocal Rank Fusion outperforms Condorcet and individual rank learning methods*, SIGIR '09. [PDF](https://plg.uwaterloo.ca/~gvcormac/cormacksigir09-rrf.pdf)." |
| Projects/Image Browser/Systems/Multi-Folder Roots.md | 178 | "- `Capataina/PinterestStyleImageBrowser/context/systems/multi-folder-roots.md` — full implementation reference" |
| Projects/Image Browser/Systems/Paths and State.md | 139 | "- `Capataina/PinterestStyleImageBrowser/context/systems/paths-and-state.md` — full implementation reference" |
| Projects/Image Browser/Systems/Profiling.md | 162 | "- `Capataina/PinterestStyleImageBrowser/context/notes/conventions.md` § Domain diagnostics — pattern for adding new diagnostics" |
| Projects/Image Browser/Systems/Search Routing.md | 141 | "- `Capataina/PinterestStyleImageBrowser/context/notes/random-shuffle-as-feature.md` — sortMode design" |
| Projects/Image Browser/Systems/SigLIP-2 Encoder.md | 138 | "- `Capataina/PinterestStyleImageBrowser/context/systems/siglip2-encoder.md` — full implementation reference" |
| Projects/Image Browser/Systems/Tag System.md | 175 | "- `Capataina/PinterestStyleImageBrowser/context/notes/dead-code-inventory.md` § Resolved — `db::delete_tag` wiring history" |
| Projects/Image Browser/Systems/Tauri Commands.md | 209 | "- `Capataina/PinterestStyleImageBrowser/context/notes/conventions.md` § Mutex acquire-then-execute, Typed errors via `?` and `From`-impls" |
| Projects/Image Browser/Systems/Thumbnail Pipeline.md | 163 | "- `Capataina/PinterestStyleImageBrowser/context/systems/thumbnail-pipeline.md` — full implementation reference" |
| Projects/Image Browser/Systems/Watcher.md | 83 | "- `Capataina/PinterestStyleImageBrowser/context/systems/watcher.md` — full implementation reference" |
| Projects/Image Browser/Work/Encrypted Vector Search.md | 62 | "#image-browser #work #fhe #encrypted-vector #privacy-preserving" |
| Projects/Image Browser/Work/README Demo.md | 49 | "- NeuroDrive demo commits (alt pattern, considered but rejected for UI-tool nature): `Capataina/neurodrive` `11e0d45` + `141be5b`" |
