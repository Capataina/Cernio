---
name: Image Browser
status: active
source_repo: https://github.com/Capataina/PinterestStyleImageBrowser
lifeos_folder: Projects/Image Browser
last_synced: 2026-05-13
sources_read: 30
---

# Image Browser

## One-line summary

Local-first Tauri 2 + React 19 desktop image library that fuses three ONNX-Runtime image encoders (CLIP ViT-B/32 + DINOv2-Base + SigLIP-2 Base 256) via Reciprocal Rank Fusion (Cormack 2009, k=60) for both image-image and text-image retrieval, with a single-flight indexing pipeline, dual-connection SQLite WAL persistence, a filesystem watcher, opt-in profiling infrastructure, and zero post-launch network calls.

## What it is

Image Browser is a local-first Tauri 2 desktop application for browsing, tagging, semantically searching, and annotating large personal image libraries. The Rust backend handles filesystem scanning, SQLite (WAL) persistence, thumbnail generation, ONNX-Runtime inference across three encoder families, multi-folder lifecycle, a filesystem watcher with orphan detection, an opt-in profiling and domain-diagnostic layer, and first-launch model downloads from HuggingFace. The React 19 frontend renders a Pinterest-style masonry grid, a modal inspector with annotations, a multi-section settings drawer, an indexing-status pill, and an opt-in perf overlay. Everything runs offline; the only network call in the entire application is the first-launch model download (~2.5 GB across 7 files). CPU on macOS for ONNX (CoreML produces runtime errors for the transformer ops); CUDA target-gated on non-macOS with CPU fallback. The GitHub repo is `Capataina/PinterestStyleImageBrowser`; the Cargo package is `image-browser`; the Tauri identifier is `com.ataca.image-browser`.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     React 19 Frontend (WebView)                  │
│                                                                  │
│   pages/[...slug].tsx  ← single catch-all route, owns search-    │
│        │                  routing, hotkeys, selectedItem state    │
│        ├─► Masonry / MasonryItem / MasonryAnchor                  │
│        ├─► PinterestModal (annotations textarea, prev/next)       │
│        ├─► SearchBar (# autocomplete + create-on-no-match)        │
│        ├─► TagDropdown                                            │
│        ├─► IndexingStatusPill (Tauri event subscription)          │
│        ├─► PerfOverlay (cmd+shift+P, profiling-only mount)        │
│        └─► settings/  (Theme · Display · Search · Sort ·          │
│                        Folders · Reset · Encoder)                 │
│                                                                   │
│   queries/  ← TanStack Query 5 hooks (staleTime: Infinity)        │
│   services/  ← invoke() wrappers + Tauri JSON ↔ UI types          │
│   hooks/  ← useDebouncedValue, useUserPreferences (localStorage), │
│             useIndexingProgress                                   │
└──────────────────────────────┬──────────────────────────────────-┘
                               │  Tauri IPC + Tauri events
                               │  (ApiError on the wire)
┌──────────────────────────────▼──────────────────────────────────┐
│                      Rust Backend (tauri::Builder)               │
│                                                                  │
│   main.rs  — parses --profiling flag (NOT --profile),            │
│              inits tracing subscriber + opt-in PerfLayer,        │
│              spawns JSONL flush thread + 1 Hz RSS/CPU sampler,   │
│              opens ImageDatabase + initialize (WAL + R2),        │
│              hands to image_browser_lib::run                     │
│                                                                  │
│   lib.rs   — Managed state via tauri::Builder::manage(...):      │
│                ImageDatabase (writer Mutex<Connection> +         │
│                  read-only secondary OnceLock<Mutex<Conn>>)      │
│                CosineIndexState { index: Arc<Mutex<CosineIndex>>}│
│                TextEncoderState { clip, siglip2 (lazy Options) } │
│                FusionIndexState { per_encoder Arc<Mutex<HashMap  │
│                  <String, CosineIndex>>> }                       │
│                IndexingState (AtomicBool single-flight)          │
│                WatcherHandle slot (Arc<Mutex<Option<...>>>)      │
│              .setup(legacy migrate · spawn pipeline · watcher)   │
│              .invoke_handler![26 commands]                       │
│              .run(|_, e| if Exit && profiling { render report }) │
│                                                                  │
│   commands/ — per-concern Tauri command handlers                 │
│       error.rs (ApiError enum + From impls), images, tags,       │
│       notes, roots (4 cmds + legacy set_scan_root), similarity   │
│       (get_similar / tiered / fused), semantic, semantic_fused,  │
│       profiling, encoders                                        │
│                                                                  │
│   db/       — post-split SQLite layer (was 1.6k-line db.rs)      │
│       mod.rs (Mutex<Connection> + R2 + PRAGMAs)                  │
│       schema_migrations.rs (4 idempotent ALTERs +                │
│         embedding_pipeline_version migrate)                      │
│       images_query.rs (aggregate_image_rows + AND/OR tag SQL)    │
│       embeddings.rs (bytemuck cast + upsert_embeddings_batch     │
│         with BEGIN IMMEDIATE per chunk)                          │
│       tags, thumbnails, roots, notes_orphans, test_helpers       │
│                                                                  │
│   similarity_and_semantic_search/                                │
│       encoders.rs (ImageEncoder + TextEncoder traits)            │
│       encoder.rs (CLIP image, separate vision_model.onnx)        │
│       encoder_dinov2.rs (DINOv2-Base image-only)                 │
│       encoder_siglip2.rs (SigLIP-2 image + text)                 │
│       ort_session.rs (shared M2-tuned Session builder)           │
│       cosine/                                                    │
│           index.rs  (partial-sort + scratch buffer + 4 diags)    │
│           rrf.rs    (RRF k=60, 6 tests)                          │
│           diagnostics.rs (4 stateless stat helpers)              │
│           cache.rs  (cosine_cache.bin + mtime freshness)         │
│           math.rs   (cosine_similarity + score_cmp_desc)         │
│       encoder_text/                                              │
│           encoder.rs (ClipTextEncoder, HF tokenizers BPE)        │
│           pooling.rs (normalize, mean_pool — ort-free)           │
│                                                                  │
│   indexing.rs        — background single-flight pipeline         │
│   watcher.rs         — notify-debouncer-mini, 5s debounce        │
│   model_download.rs  — first-launch HuggingFace fetch            │
│   paths.rs           — single disk-path source                   │
│   settings.rs        — Settings struct                           │
│   perf.rs            — PerfLayer + RawEvent log + JSONL flush    │
│   perf_report.rs     — on-exit markdown renderer                 │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼  disk: <app_data_dir>/
   images.db (WAL + .db-shm + .db-wal),
   settings.json (atomic .tmp + rename),
   cosine_cache.bin (bincode),
   models/{clip_vision,clip_text,dinov2_base_image,siglip2_vision,
           siglip2_text}.onnx + 2 tokenizer.json files (~2.5 GB),
   thumbnails/root_<id>/thumb_<image_id>.jpg,
   exports/perf-<unix_ts>/{timeline.jsonl, report.md, raw.json}
```

**Dependency direction (key directional rules observed in code):**

- `db` is the only sink every backend module reads from or writes to; it has no inverse dependencies. The post-split `db/` directory uses Rust's multiple-`impl`-block merge so callers see a flat `ImageDatabase` API.
- `Arc<Mutex<CosineIndex>>` is intentionally cloned across the indexing thread and the Tauri-managed `CosineIndexState`; both hold clones pointing at the same in-memory cache, so a finished pipeline-encode immediately makes new embeddings available to the next semantic search.
- `indexing.rs` and `watcher.rs` are coupled through `IndexingState` (single-flight `AtomicBool` with RAII guard); rapid filesystem events that try to spawn a second pipeline get `Err(AlreadyRunning)` and silently coalesce.
- Profiling is not in the normal data path; when `--profiling` is absent, `PerfLayer` never registers, the frontend `<PerfOverlay>` never mounts, `record_user_action` is a no-op, and `record_diagnostic` returns early before constructing any JSON.
- `commands/*` returns `Result<T, ApiError>` for every handler; the frontend deserialises `{ kind, details }` and branches on `kind` in `formatApiError`.
- Frontend services never call `invoke()` directly; they wrap it in functions that translate Tauri JSON into UI types. Hooks call services; components call hooks. The `perfInvoke` wrapper preserves this layering while adding profiling-side start/end events.

**Mutex topology — five long-lived sync primitives serialise backend operations:**

| Primitive | Holder | Acquired by | Poison surface |
|---|---|---|---|
| `Mutex<Connection>` (writer) | `ImageDatabase.connection` | every DB write + foreground writes | `unwrap()` — project treats poisoning as unrecoverable, restart required |
| `Mutex<Connection>` (read-only secondary R2) | `ImageDatabase.reader: OnceLock<Mutex<Connection>>` | foreground SELECTs via `read_lock()` | falls back to writer if `reader.get()` is None (in `:memory:` tests) |
| `Arc<Mutex<CosineIndex>>` | `CosineIndexState.index` (also cloned to indexing thread) | every similarity / semantic command + populate + save | `?` via `From<PoisonError>` → `ApiError::Cosine("mutex poisoned: ...")` |
| `Mutex<Option<TextEncoder>>` ×2 | `TextEncoderState.{clip, siglip2}` | semantic_search + indexing pre-warm | same `From<PoisonError>` impl |
| `AtomicBool` (single-flight) | `IndexingState.is_running` | every command that triggers an index + watcher debounce closure | RAII guard ensures clear on success, error, AND panic |
| `Arc<Mutex<Option<WatcherHandle>>>` | `watcher_state` | lib.rs setup callback (defensive `.lock().is_ok()`) | silent skip if poisoned (setup runs early before IPC channel) |

## Subsystems and components

### CLIP Image Encoder (separate-graph, OpenAI English)
Uses OpenAI's separate `vision_model.onnx` (no dummy text inputs at image-encode time, eliminating the previous unified-graph hack). Preprocessing: bicubic-shortest-edge to 224 (CatmullRom filter, closest to PIL's BICUBIC) + center-crop 224×224 + CLIP-native mean `[0.48145466, 0.4578275, 0.40821073]` and std `[0.26862954, 0.26130258, 0.27577711]` (replaced ImageNet stats which biased the embedding distribution) + L2 normalisation. Output: 512-d unit vector via `image_embeds`. Embedding store: 2048 bytes per image. Disk: ~352 MB FP32. Single-image encode ~25–40 ms on M2 CPU; batch-32 ~500–700 ms. Emits `preprocessing_sample` and `encoder_run_summary` diagnostics when profiling is enabled. Center-crop drops content outside the central 224×224 — partially mitigated by SigLIP-2's exact-square preprocessing seeing the full image in fusion.

### CLIP Text Encoder (HF tokenizers BPE, English)
Uses separate `text_model.onnx`. Tokenizer is HuggingFace `tokenizers = "0.22.2"` BPE (vocab 49 152, max 77 tokens, pad id 49407) — replaced a previous pure-Rust hand-rolled WordPiece multilingual encoder. ONNX output cascade `text_embeds → pooler_output → sentence_embedding` (defensive — different exports use different names); falls back to mean-pooled `last_hidden_state` via ort-free `pooling::mean_pool`. Real-input pre-warm `encode("warmup")` inside `new()` eliminates ~1–2 s first-call latency spike (Phase 12c). Lazy init within `commands::semantic::semantic_search`; missing files surface as typed `ApiError::TextModelMissing` so the frontend can trigger a re-download. Disk: ~254 MB FP32. Emits `tokenizer_output` and `query_embedding` diagnostics before inference.

### DINOv2-Base Encoder (Meta, image-only, self-supervised)
86M-parameter ViT trained via self-supervised contrastive learning on images alone — emphasises visual structure (pose, lighting, art style, composition) rather than text-aligned concepts. Preprocessing: bicubic-shortest-edge to 256 + center-crop 224×224 + ImageNet `[0.485, 0.456, 0.406]` / `[0.229, 0.224, 0.225]` (DINOv2's training-time stats, deliberately different from CLIP-native). Embedding extraction: CLS-token from `last_hidden_state[:, 0, :]` (not `pooler_output`); L2-normalised. 768-d output. ~30–40 ms single-image encode; ~600–800 ms batch-32; ~347 MB FP32 on disk. Outperforms CLIP on image-only similarity tasks (5× on fine-grained species recognition, 64% vs 28% on a challenging dataset per multiple benchmarks). Image-only: participates only in image-image fusion, not text-image.

### SigLIP-2 Base 256 Encoder (Google, image + text, shared 768-d space)
Sigmoid-loss vision-language model — one-vs-all binary cross-entropy rather than CLIP's softmax contrastive; stronger alignment for descriptive content. Image preprocessing: exact-square bilinear resize to 256×256 (deliberately not aspect-preserving — SigLIP-2's training pipeline) + `[-1, 1]` range (`pixel * 2 - 1`, no per-channel mean/std). Text tokenizer: HF tokenizers SentencePiece (Gemma vocab 256k, max 64 tokens, no attention_mask — single-input ONNX signature). Both branches output `pooler_output` from a Multi-Attention-Pooling head, L2-normalised. Disk: ~372 MB image + ~1.13 GB text (Gemma 256k vocab embedding table dominates). ~50–80 ms single-image encode (slowest of the three); ~1.5–2.5 s batch-32. Sees the full image (no edge content dropped) where CLIP and DINOv2 lose content outside the 224×224 center-crop — the explicit complementarity that motivates fusion.

### Multi-Encoder Fusion (Reciprocal Rank Fusion, k=60)
Image-image fusion uses 3 encoders (CLIP + SigLIP-2 + DINOv2); text-image fusion uses 2 (CLIP + SigLIP-2 — DINOv2 has no text branch). The formula `score(p) = Σ over enabled encoders e of 1 / (k_rrf + rank_e(p))` with `k_rrf = DEFAULT_K_RRF = 60` is the canonical Cormack-Clarke-Büttcher SIGIR 2009 balance between top-of-list dominance and consensus contribution. RRF discards the cosine value and only uses *rank*, sidestepping cross-encoder distribution differences (CLIP's "0.85" is not comparable to DINOv2's "0.85"). 6 unit tests in `cosine/rrf.rs` pin the contract. Per-encoder top-K is `5 × top_n`. Per-encoder caches live in `FusionIndexState.per_encoder: Arc<Mutex<HashMap<String, CosineIndex>>>` with lazy populate (~150 ms first call, warm thereafter); `invalidate_all()` wired into the 3 root-mutation IPCs (`set_scan_root`, `remove_root`, `set_root_enabled`). ~6 MiB per encoder for 2000 images × 768 floats × 4 bytes; ~18 MiB total resident across all three. Per-encoder enable/disable persisted in `settings.json::enabled_encoders` — toggling is non-destructive (existing rows resurrect instantly when toggled back on; the backend `decide_enabled_write` validator rejects empty mutations).

### Cosine Similarity Engine (partial-sort, scratch buffer, persistent cache)
Single-encoder cosine retrieval that fusion stacks on top of. Uses `select_nth_unstable_by` partial sort (O(N) average vs O(N log N) full sort — 2.53× measured speedup at N=10000, audit `c6551e2`) with a reusable scratch buffer eliminating per-call `Vec` allocation. Three retrieval modes: sampled (legacy), sorted (used by fusion), tiered (legacy 7-tier random sampler, no longer called from frontend but preserved with unit tests). `score_cmp_desc` is NaN-aware. Persistent `cosine_cache.bin` (bincode) with mtime freshness check against the DB; corrupted cache silently falls through to populate_from_db. Submodule split into `mod.rs + index.rs + math.rs + rrf.rs + diagnostics.rs + cache.rs` (was a single 860-line `cosine_similarity.rs`). Emits `cosine_cache_populated`, `embedding_stats`, `pairwise_distance_distribution`, and `self_similarity_check` diagnostics on every populate_from_db_for_encoder call.

### Database (SQLite WAL + R2 secondary + bytemuck + BEGIN IMMEDIATE batched writes)
SQLite (rusqlite) with 5 tables (`roots`, `images`, `tags`, `images_tags`, `embeddings`) plus a `meta` table for migration versions. PRAGMAs set at every initialize: `journal_mode=WAL`, `synchronous=NORMAL`, `foreign_keys=ON` (the explicit fix that made `ON DELETE CASCADE` actually fire — defaults OFF in SQLite), `busy_timeout=5000`, `wal_autocheckpoint=0` (manual via `checkpoint_passive`), `journal_size_limit=64 MiB`. Two connections per real DB: writer `Mutex<Connection>` + read-only secondary `OnceLock<Mutex<Connection>>` accessed via `read_lock()`. Foreground SELECTs use the secondary; encoder/foreground writes use the writer; both share the file's WAL (writes serialise at the WAL layer, reads non-blocking against active writes). R2 closes the prior perf-1777212369 22-second `ipc.get_images` freeze. Embedding BLOB encoding via `bytemuck::cast_slice` (safe, zero-copy, alignment-checked at compile time — replaces 3 previous `unsafe { slice::from_raw_parts(...) }` sites, audit `0bdb5f4`). Encoder writes batched via `BEGIN IMMEDIATE` per ~32-row chunk (one transaction + one fsync per batch + `checkpoint_passive` between batches — replaced per-row autocommit that produced multi-second WAL stalls). 4 idempotent ALTER TABLE migrations + an `embedding_pipeline_version` migration (currently version 4) that wipes legacy rows when bumped. Submodule split into `mod / schema_migrations / images_query / embeddings / tags / thumbnails / roots / notes_orphans / test_helpers`; multiple `impl ImageDatabase` blocks merge transparently.

### Indexing Pipeline (background single-flight, parallel-by-encoder)
Orchestration glue between filesystem-scanner, thumbnail-pipeline, every encoder, cosine-similarity, model-download, and watcher. Phases: cache load → `model_download::download_models_if_missing` → pre-warm enabled text encoders (Phase 12c) → open second `ImageDatabase` → Phase::Scan (per enabled root, `INSERT OR IGNORE`, `mark_orphaned`) → Phase::Thumbnail (rayon `par_iter`, single-SELECT `get_paths_to_root_ids`) → Phase::Encode (parallel — one OS thread per enabled image encoder, each with its own DB connection, `BEGIN IMMEDIATE` per chunk + `checkpoint_passive` between batches) → `cosine::populate_from_db` + `save_to_disk` → Phase::Ready event. Single-flight via `IndexingState.is_running: AtomicBool` with RAII `RunningGuard(Drop)` ensuring clear on success, error, AND panic. Each phase emits an `indexing-progress` Tauri event consumed by the frontend `IndexingStatusPill`. Steps 5–7 are idempotent. Parallel encoder phase (Phase 11e) reduces wall-clock to roughly `max(CLIP, SigLIP-2, DINOv2)` ≈ SigLIP-2 alone (slowest), vs the prior serial sum; Phase 12 dynamic `intra_threads(N / enabled_encoders)` shares the M2 P-cluster instead of oversubscribing. The file is currently 47 KB; audit recommends a 4-file phase-module split (deferred).

### Filesystem Watcher (notify-debouncer-mini, 5 s debounce, single-flight coalesce)
Recursive watch on every enabled root via `notify-debouncer-mini` with a 5-second debounce. Event handler calls `indexing::try_spawn_pipeline`; single-flight rejects rapid bursts silently. WatcherHandle parked in `Arc<Mutex<Option<WatcherHandle>>>` so a future "rebuild on root mutation" path can swap the handle atomically. The 5 s debounce is empirically tuned — smaller fires too often during normal file-system noise (download manager temp files, IDE auto-save); larger feels sluggish for the drop-and-see UX. Does not rebuild on `add_root` / `remove_root` post-launch — documented gap.

### Filesystem Scanner (stateless recursive walk, 7-extension whitelist)
Recursive `read_dir` walk with a `["jpg", "png", "gif", "jpeg", "bmp", "tiff", "webp"]` case-insensitive whitelist. Stateless utility — no DB or cache concerns. Per-entry permission errors log warn + continue; whole-directory errors log warn but don't abort the parent walk. The pipeline tolerates partial scans because `INSERT OR IGNORE` is idempotent on retry. 2 tests cover recursive walk + case-insensitive extension matching.

### Thumbnail Pipeline (JPEG scaled IDCT + fast_image_resize, per-root subfolders)
400×400-max aspect-preserving JPEG. JPEG fast path uses `jpeg-decoder::Decoder::scale()` for native scaled IDCT (1/8, 1/4, 1/2 factor — Tier 2 R7), then `fast_image_resize 6.x` NEON-optimised Lanczos3 for the final downsample (R6). Together ~5–10× faster than the prior `image-rs` Lanczos3 default for typical JPEG inputs; on a 2000-image library, thumbnail-phase time dropped from ~150–250 s to ~30–50 s. Non-JPEG (PNG, GIF, BMP, TIFF, WebP) and any decode error fall back to `image-rs`. Per-root subfolder layout `thumbnails/root_<id>/thumb_<image_id>.jpg` (Phase 9 reorg — `remove_root` can `rm -rf` the subfolder cleanly); legacy `root_id = NULL` rows still write to the flat `thumbnails/thumb_<id>.jpg` layout. Rayon `par_iter` parallelism across all available cores.

### Multi-Folder Roots (CASCADE, per-root toggle, legacy migration)
`roots` table with `path UNIQUE`, `enabled` boolean, `added_at` epoch seconds. `images.root_id INTEGER REFERENCES roots(id) ON DELETE CASCADE`, gated by the non-default `PRAGMA foreign_keys=ON`. Four CRUD Tauri commands: `add_root` (granular append + reindex), `remove_root` (CASCADE wipes + `rm -rf` thumbnail subfolder), `set_root_enabled` (SQL-filter toggle, no re-encode), `list_roots`. Legacy `set_scan_root` (replace-all) preserved for the empty-state UI's "Choose folder" pill. Grid filter: `WHERE images.orphaned = 0 AND (images.root_id IS NULL OR images.root_id IN (SELECT id FROM roots WHERE enabled = 1))` (NULL handling keeps legacy un-migrated rows visible). Legacy migration at `lib.rs::run::setup` translates `settings.json::scan_root` into a row in `roots` once and clears the field. 11 unit tests in `db/roots.rs::tests`. Every root-mutating command clears `cosine_state.index.cached_images` and calls `fusion_state.invalidate_all()`.

### Model Download (first-launch HuggingFace fetch, per-file fail-soft)
The only network operation in the entire application. Fetches ~2.5 GB across 7 files from HuggingFace on first launch: `clip_vision.onnx` (~352 MB), `clip_text.onnx` (~254 MB), `clip_tokenizer.json`, `dinov2_base_image.onnx` (~347 MB), `siglip2_vision.onnx` (~372 MB), `siglip2_text.onnx` (~1.13 GB), `siglip2_tokenizer.json`. Mechanism: HEAD preflight (skip if exists with matching size) → chunked GET with progress callback (`ureq`) → atomic `.part` + rename to final name. Per-file fail-soft: a single 401/404 doesn't abort the batch (the encoder phases gate on `path.exists()` and gracefully degrade with `warn` + IndexingStatusPill signalling). No resume support (`.part` files are deleted and re-downloaded from scratch); no mirror selection; no SHA256 / signature validation beyond content-length match (trust placed in HuggingFace + HTTPS). Tracing spans: `model_download.all`, `model_download.head`, `model_download.file`.

### Paths and State (single disk-path source, no dev/release split)
All disk paths flow through `paths::*_dir()` helpers — single source of truth. `paths::app_data_dir()` resolution order: `$IMAGE_BROWSER_DATA_DIR` env override → `dirs::data_dir()/com.ataca.image-browser/` (macOS `~/Library/Application Support/com.ataca.image-browser/`, Linux `$XDG_DATA_HOME/com.ataca.image-browser/`, Windows `%APPDATA%/com.ataca.image-browser/`) → `./app-data/com.ataca.image-browser/` fallback. The `cfg(debug_assertions)` dev/release split that pointed at `<repo>/Library/` for debug builds was removed in 2026-04-26 because dev and release diverged on every code change, forcing re-downloads of 2.5 GB of models on every switch. `IMAGE_BROWSER_DATA_DIR` is the supported successor for sandboxed testing. `paths::strip_windows_extended_prefix(&str) -> Cow<'_, str>` was extracted from 3 inline closures (audit `02b12b9`) and returns `Cow::Borrowed` for zero allocation on the common path. `Settings::save` uses atomic `.tmp` + rename. 6 unit tests cover layout + per-root subfolder + filename stability.

### Tauri Commands (26 commands, ApiError typed wire)
Grouped by concern under `src-tauri/src/commands/`: images (1), tags (5), notes (2 — Phase 11 annotations), roots (6), similarity (3 — `get_similar_images` + `get_tiered_similar_images` legacy + `get_fused_similar_images` active), semantic (1 — legacy), semantic_fused (1 — active), profiling (5), encoders (3). Every command returns `Result<T, ApiError>` where `ApiError` is `#[serde(tag = "kind", content = "details")]` discriminated union covering `Db`, `Io`, `Cosine`, `Encoder`, `BadInput`, `NotFound`, `TextModelMissing`, `ImageModelMissing`, `TokenizerMissing`, and a few more. `From` impls for `rusqlite::Error`, `std::io::Error`, `std::sync::PoisonError<T>` let command bodies use `?` directly. Frontend `services/apiError.ts` mirrors the union; `formatApiError` covers `ApiError` + legacy strings + `Error` instances uniformly; `isMissingModelError` enables a re-download flow. Three legacy single-encoder commands remain registered (`get_similar_images`, `get_tiered_similar_images`, `semantic_search` — audit residuals D-SIM-1, D-FE-1, D-SEM-1, ~600 Rust + ~80 TS dead lines). Three profiling commands still use `Result<_, String>` for legacy reasons.

### Profiling (opt-in --profiling, PerfLayer + 12 diagnostics + 1 Hz RSS/CPU sampler)
Opt-in via `--profiling` CLI flag (the long form is deliberate — Tauri 2 owns `--profile <NAME>` for cargo profile selection) or `PROFILING=1` env var. When absent, `PerfLayer` never registers; every `record_diagnostic` returns early before constructing any JSON; the `<PerfOverlay>` never mounts; overhead is one tracing dispatch per instrumented call. When on: PerfLayer aggregates per-span timing (`SpanStats { count, sum, min, max, recent_ringbuffer }` for p50/p95/p99 percentile estimation); `record_diagnostic` writes `RawEvent::Diagnostic` to a JSONL flush thread; a 1 Hz RSS/CPU sampler (sysinfo 0.32) writes `system_sample` diagnostics; on `tauri::RunEvent::Exit`, `perf_report::render_session_report` writes `<app_data_dir>/exports/perf-<unix_ts>/report.md` with Stall Analysis (spans where p99 / p50 > threshold) and Resource Trends sections. Span naming convention uses prefixes `ipc.`, `pipeline.`, `cosine.`, `model_download.`, `watcher.` so the report groups cross-phase. 12 named diagnostics: `embedding_stats`, `pairwise_distance_distribution`, `self_similarity_check`, `score_distribution_stats`, `tokenizer_output`, `query_embedding`, `preprocessing_sample`, `encoder_run_summary`, `cosine_math_sanity`, `path_resolution_outcomes`, `cross_encoder_comparison`, `cosine_cache_populated`. Frontend `perfInvoke` wraps every IPC call with start/end breadcrumbs; `<React.Profiler id="Masonry" onRender={onRenderProfiler}>` emits per-render events; cmd+shift+P toggles the overlay. The repo's framing: "these diagnostics replace the deferred 'build a comparison harness vs Python reference' validation tool — quality issues surface in the report without a separate test harness."

### Search Routing (frontend priority chain)
`pages/[...slug].tsx` priority chain: selected image (similar) > non-tag query (semantic-fused) > `#tag` query (filtered grid) > all images (stable-sorted, optionally shuffled). Similar uses `useTieredSimilarImages` (hook name preserved post-fusion for caller stability — D-FE-1 audit residual). Semantic uses `useSemanticSearch` routed through `fetchFusedSemanticSearch` + `get_fused_semantic_search`. 300 ms debounce on the search input. `selectedItem` lookup resolves against `displayImages` not `images.data` (audit `9d04f69` — fixed silent miss on semantic-search results). Hotkeys: Esc (clear selection), ←/→ (prev/next in modal), cmd/ctrl+K (focus search), cmd+shift+P (PerfOverlay).

### Tag System (CRUD + AND/OR mode + `#tag` autocomplete + Phase 6 delete affordance)
5 Tauri commands: `get_tags`, `create_tag`, `delete_tag` (newly wired in Phase 6 — was a known gap pre-2026-04-25), `add_tag_to_image` (`INSERT OR IGNORE` Phase 6 hardening), `remove_tag_from_image`. Schema: `tags(id, name UNIQUE, color)` + `images_tags(image_id, tag_id)` with `ON DELETE CASCADE` on both columns and `PRIMARY KEY (image_id, tag_id)`. AND mode: `GROUP BY images.id HAVING COUNT(DISTINCT it.tag_id) = ?`. OR mode: `WHERE EXISTS (... tag_id IN (?, ?, ...))`. UI toggle in Settings drawer's SearchSection. `#tag` autocomplete + create-on-no-match in SearchBar. Optimistic mutations across `useImages.ts` / `useTags.ts` / `useRoots.ts` with snapshot rollback `onError`. Color is a CSS hex string stored in the `color TEXT` column. Tests: round-trip + UNIQUE + CASCADE in `db/tags.rs`, service-layer wrappers in `services.test.ts`, hook + optimistic mutation behaviour in `useTags.test.ts`.

### Frontend State (TanStack Query 5 staleTime:Infinity + localStorage prefs + 7-section settings drawer)
TanStack Query 5 with `defaultOptions: { staleTime: Infinity, gcTime: 10min, refetchOnWindowFocus: false, retry: 0 }`. Caches never auto-stale — mutations explicitly invalidate (tag mutations → `["images"]` + `["tags"]`; root mutations → `["images"]` + `["roots"]`; `Phase::Ready` indexing event → `["images"]`). `useUserPreferences` mirrors prefs in `localStorage["imageBrowserPrefs"]` (theme, columns, sortMode, animationLevel, similarResultCount, semanticResultCount, tagFilterMode, legacy imageEncoder/textEncoder ids). Theme is pre-applied via `main.tsx` reading `localStorage["theme"]` and adding the `dark` class to `<html>` before React mounts (FOUC fix). Settings drawer split into 7 sections (audit modularisation finding): `index.tsx` shell + `controls.tsx` primitives + Theme / Display / Search / Sort / Folders / Reset / Encoder sections. `useIndexingProgress` subscribes to backend `indexing-progress` Tauri events. Service layer discipline: services translate Tauri JSON ↔ UI types, hooks call services, components never call `invoke()` directly. `perfInvoke` wrapper preserves layering while emitting profiling breadcrumbs.

### Masonry Layout (shortest-column packing, framer-motion tilt, backend dimensions)
Pinterest-style shortest-column packing with selected image promoted to top spanning up to 3 columns. Default 4 columns, user-tunable via Settings. Tile dimensions sourced from the backend (`images.width` + `images.height` populated at thumbnail generation time) — audit `fb23bdb` "Unified ImageSearchResult: dimensions to backend, drop DOM image-loads" eliminated the DOM `Image.onload` round-trip per tile and dropped initial-paint of a 500-tile grid from ~3–5 s to ~200–300 ms. Hover 3D tilt via framer-motion `useTransform` on `rotateX` / `rotateY`, gated by `prefs.animationLevel` ('minimal' | 'subtle' | 'full'). SortMode-aware: `shuffleSeed` increments on modal close so closing a modal returns a freshly-shuffled grid (random shuffle is opt-in, not the default). `<React.Profiler id="Masonry" onRender={onRenderProfiler}>` wraps the grid when profiling is enabled and short-circuits otherwise.

## Technologies and concepts demonstrated

### Languages

- **Rust (edition 2021)** — entire backend including `src-tauri/src/` (28 files): commands handlers, SQLite layer, ONNX encoders, cosine/RRF retrieval, indexing pipeline, watcher, model download, profiling infrastructure, paths, settings.
- **TypeScript** — entire frontend (33 files) including React 19 components, TanStack Query 5 hooks, service-layer IPC wrappers, ApiError mirror, profiling-overlay integration.
- **SQL (SQLite dialect)** — schema (5 tables + meta), 4 idempotent ALTER TABLE migrations, embedding-pipeline-version migration, AND/OR tag filter SQL, `INSERT OR IGNORE` + `INSERT OR REPLACE` patterns, `BEGIN IMMEDIATE` transactions, `ON DELETE CASCADE` gated by `PRAGMA foreign_keys=ON`.

### Frameworks and libraries

- **Tauri 2** — desktop application shell; identifier `com.ataca.image-browser`. Owns the IPC layer, `tauri::Builder::manage(...)` state injection, `invoke_handler![26 commands]`, the `tauri-plugin-dialog` native folder picker, the `assetProtocol` solving image loading at gallery scale without base64-over-IPC, and `tauri::RunEvent::Exit` for the on-exit profiling report.
- **React 19** — frontend rendering layer; functional components, hooks, `React.Profiler` integration for the masonry grid under `--profiling`.
- **Vite 7 + vite-plugin-pages** — frontend bundler + file-system routing.
- **TanStack Query 5** — server-state caching; `staleTime: Infinity` + manual invalidation pattern.
- **framer-motion** — hover-tilt animation respecting `prefs.animationLevel`.
- **rusqlite** — SQLite bindings; dual-connection writer + R2 secondary; `bytemuck::cast_slice` for safe BLOB casts; `BEGIN IMMEDIATE` for batched encoder writes; manual `checkpoint_passive` between batches.
- **ort = 2.0.0-rc.10** — ONNX Runtime Rust bindings. Single shared `Session` builder factory (`build_tuned_session`) tuned for Apple M2: `Level3 + intra_threads(4) + inter_threads(1)`. Phase 12 dynamic `intra_threads(N / enabled_encoders)` for parallel encoder phase. CoreML disabled on macOS (transformer ops produce runtime errors); CUDA target-gated on non-macOS with CPU fallback.
- **HF tokenizers = "0.22.2"** — uniform `tokenizer.json` interface for CLIP BPE (vocab 49 152, max 77, pad 49407) and SigLIP-2 SentencePiece (Gemma vocab 256k, max 64, no attention_mask).
- **bytemuck** — `cast_slice` for safe, zero-copy, alignment-checked `&[f32] ↔ &[u8]` views (replaces 3 prior `unsafe { slice::from_raw_parts(...) }` sites).
- **fast_image_resize 6.x** — NEON-optimised Lanczos3 thumbnail downsample.
- **jpeg-decoder 0.3** — `Decoder::scale()` for native scaled IDCT at 1/8, 1/4, 1/2 factor.
- **rayon** — `par_iter` parallelism for the thumbnail phase.
- **notify-debouncer-mini** — filesystem watcher with 5-second debounce window.
- **tracing + tracing-subscriber** — structured spans + log levels; PerfLayer is added only when profiling is enabled.
- **sysinfo 0.32** — 1 Hz RSS / CPU% sampler for the profiling Resource Trends section.
- **bincode** — persistent `cosine_cache.bin` serialization.
- **serde / serde_json** — `#[serde(tag = "kind", content = "details")]` discriminated union for the `ApiError` typed wire format.
- **dirs** — platform-default app-data dir resolution.
- **ureq** — HTTP client for first-launch HuggingFace model downloads (HEAD preflight + chunked GET).
- **image-rs** — fallback decode path for non-JPEG formats; `DynamicImage` boundary.

### Runtimes / engines / platforms

- **ONNX Runtime** via `ort 2.0.0-rc.10` — three encoder graphs (`vision_model.onnx`, `text_model.onnx`, `dinov2_base_image.onnx`, `siglip2_vision.onnx`, `siglip2_text.onnx`) plus 2 `tokenizer.json` files, ~2.5 GB total FP32 on disk. CPU on macOS (CoreML disabled — runtime errors for the transformer ops); CUDA target-gated on non-macOS with CPU fallback.
- **WebView (Tauri 2)** — frontend host; cmd+shift+P PerfOverlay only mounts when `is_profiling_enabled()` returns true.
- **SQLite (WAL journal mode)** — 5 tables + meta; dual-connection writer + read-only secondary; manual checkpoint between batches.

### Tools

- **cargo** — `cargo test --lib` (125 lib + integration tests), `cargo test` integration, `cargo clippy` (clean post-Phase-6 + post-Phase-12).
- **bun + vitest** — frontend test runner (62 tests).
- **cargo tauri dev** + `cargo tauri dev -- --profiling` — development with optional profiling.
- **tracing** — span instrumentation; PerfLayer aggregates only under `--profiling`.

### Domains and concepts

- **Local-first software** — every byte of compute, persistence, and ML inference on the user's machine; the only network call is first-launch model download. Privacy by construction, not configuration.
- **Multi-encoder rank fusion (RRF, Cormack 2009)** — `score(p) = Σ 1 / (k + rank_e(p))` with `k = DEFAULT_K_RRF = 60`; discards cosine values and uses only rank, sidestepping cross-encoder distribution differences. Both image-image (3 encoders) and text-image (2 encoders — asymmetric because DINOv2 has no text branch). Diversity emerges for free without a separate sampling step.
- **ONNX Runtime CPU inference at scale** — three encoder families × CPU × consumer M2 hardware running through `ort` + HF tokenizers via a shared M2-tuned `Session` builder. Pure-Rust posture (no Python at runtime) is a credibility marker.
- **Self-supervised representation learning vs contrastive vs sigmoid-loss vision-language** — DINOv2 (visual structure / pose / lighting), CLIP (concept overlap), SigLIP-2 (descriptive content via sigmoid loss = one-vs-all binary cross-entropy rather than CLIP's softmax). Fusion exploits the explicit complementarity.
- **WAL + dual-connection SQLite** — writer mutex + read-only secondary `OnceLock<Mutex<Connection>>`; `read_lock()` for foreground SELECTs; both share the WAL; foreground reads non-blocking against active writes. Closes the perf-1777212369 22-second `ipc.get_images` freeze.
- **`BEGIN IMMEDIATE` batching with manual checkpoint** — one transaction + one fsync per ~32-row chunk + `checkpoint_passive` under `wal_autocheckpoint=0`; replaced per-row autocommit that produced multi-second WAL stalls.
- **R-tag perf annotation pattern** — 43 inline `R<n>` comments across the perf-bundle commit set for forward + reverse traceability between recommendation plan and source.
- **Partial sort + reusable scratch buffer** — `select_nth_unstable_by` is O(N) average vs O(N log N) full sort; 2.53× measured speedup at N=10000 for top-K retrieval. NaN-aware `score_cmp_desc`.
- **Tracing-based profiling** — opt-in `--profiling` flag with no-overhead-when-off semantics; PerfLayer adds aggregation only when enabled; `record_diagnostic` early-returns; 12 named domain diagnostics replace a planned Python-reference validation harness.
- **Single-flight orchestration via AtomicBool + RAII guard** — `IndexingState.is_running` with `RunningGuard(Drop)` ensures clear on success, error, AND panic; rapid filesystem events coalesce into one pipeline.
- **Idempotent migrations + embedding-pipeline versioning** — pipeline-version bump (currently v4) propagates preprocessing changes by wiping affected rows on next launch; next indexing pass re-encodes cleanly.
- **Typed errors on the IPC wire** — `ApiError` discriminated union mirrored frontend-side; specific kinds (`TextModelMissing`) trigger specific UI flows (re-download); generic strings would force fragile string-matching.
- **Asymmetric resource ownership** — `Arc<Mutex<CosineIndex>>` cloned across indexing thread and Tauri-managed state, intentional shared mutable cache; `Arc<Mutex<Option<WatcherHandle>>>` allows future atomic handle swap.
- **Atomic file-write pattern** — `.tmp` + rename for settings; `.part` + rename for model downloads. Survives mid-write power loss without corrupting the prior file.
- **CASCADE-gated foreign-keys** — `ON DELETE CASCADE` is a no-op without `PRAGMA foreign_keys=ON` (SQLite defaults OFF for backwards compat); the pragma is the explicit fix that made root removal actually wipe its images.
- **Diagnostic-driven validation (encode-it-in-the-runtime)** — 17 `record_diagnostic` call sites across `commands/`, `indexing.rs`, `lib.rs`, `cosine/index.rs` mean a future bug in any of those areas surfaces in the next perf report without source diving.
- **Center-crop spatial-coverage trade-off across encoders** — CLIP and DINOv2 drop content outside the 224×224 center-crop; SigLIP-2's exact-square preprocessing sees the full image. The fusion of all three partially mitigates the edge-content gap.

## Key technical decisions

| # | Decision | Rejected alternatives | Rationale |
|---|---|---|---|
| D1 | **Tauri 2 + Rust backend + React 19 frontend** | Electron + Node; pure-Rust GUI via `egui` / `iced`; native Swift/Kotlin | Smaller bundle (8–15 MB vs Electron 100+ MB); lower memory (~30–40 MB vs 200–300 MB); Rust safety for filesystem-scanning + ML-inference paths; `ort` gives ergonomic ONNX integration; Tauri's `assetProtocol` solves image loading at gallery scale without base64-over-IPC |
| D2 | **Local-first, no cloud, no accounts** | Cloud CLIP inference; hybrid (local thumbnails, cloud embeddings); personal server with optional sync | Personal image libraries are private by nature; cloud sync defeats the purpose. ONNX Runtime + tokenizers crate prove ML inference on consumer CPUs is feasible. Only network call is first-launch model download |
| D3 | **SQLite over an embedded vector DB (LanceDB / Qdrant)** | LanceDB; Qdrant embedded; in-memory only; FAISS-RS | Single-file, zero-dependency, well-understood; already need SQL for tags/notes/roots; adding a second store would be the more complex choice; brute-force cosine over a few-thousand 512–768-d vectors fits comfortably in RAM and runs in milliseconds with partial-sort. HNSW (Rec-2) deferred until library scales past ~50k images |
| D4 | **Three encoders (CLIP + DINOv2 + SigLIP-2), not one** | Single CLIP (original); single SigLIP-2 (newer, better text alignment); CLIP + DINOv2 only | No single encoder is best at every retrieval task. CLIP cares about *concept overlap*, DINOv2 about *visual / structural similarity* (image-only, no text branch), SigLIP-2 about *descriptive content*. Three is the right number until a real query class misses on all three |
| D5 | **Reciprocal Rank Fusion (RRF), k=60, not score-fusion** | Score-fusion (sum/mean of normalised cosines); per-encoder weighting; pick "best" encoder per query class | Different encoders produce cosines on different distributions — CLIP's "0.85" is not comparable to DINOv2's "0.85". L2 normalisation alone doesn't fix this. RRF discards the score and uses only the **rank**. `k=60` is the canonical Cormack 2009 balance. Diversity emerges for free; consensus dominates without a separate sampling step |
| D6 | **Per-encoder enable/disable, not always-all** | Force all encoders always; pick exactly one ("active encoder" — what shipped pre-Phase-11c); user-choose at install time only | Each encoder costs ~350–500 MB models + ~1/3 of indexing wall-clock + ~6 MiB resident RAM per 2000 images. Some users want speed/disk savings; some want all three. Toggling is non-destructive — re-enabling instantly restores fusion participation |
| D7 | **Separate-graph CLIP exports (vision_model.onnx + text_model.onnx)** | Unified `model.onnx` requiring dummy-input hack | Eliminates the dummy-text-input hack at image-encode time; model files are smaller individually (~352 MB vision, ~254 MB text); encoding paths are conceptually clean |
| D8 | **HuggingFace `tokenizers` crate over pure-Rust WordPiece** | Pure-Rust hand-rolled WordPiece; per-tokenizer crates; embed C `tokenizers-cpp` | Single crate handles BPE (CLIP) and SentencePiece (SigLIP-2) uniformly via `tokenizer.json`; battle-tested against HF's own model exports; eliminates the maintenance surface of a hand-rolled tokenizer; pure-Rust posture preserved (no C deps in the Rust path) |
| D9 | **`ort = 2.0.0-rc.10` with shared M2-tuned Session builder** | Per-encoder Session config; default ort settings | Single tuned factory means a future tuning change lands in one place; encoders cannot drift on Session config across rewrites. `intra=4 + inter=1` choice serialises ONNX op execution across the four M2 P-cores rather than over-subscribing |
| D10 | **CoreML disabled for ALL encoders on macOS** | CoreML for image encoders only; full CoreML; `MLProgram + RequireStaticInputShapes` (deferred R16) | CoreML produces runtime inference errors for transformer ops in CLIP text and SigLIP-2 text. Rather than maintain a per-encoder allow-list that would silently break on CoreML version bumps, disable uniformly on macOS. CUDA target-gated on non-macOS with CPU fallback |
| D11 | **`--profiling` CLI flag (NOT `--profile`)** | `--profile` (collides with Tauri 2 reserving it for cargo profile selection); `-p` (Cargo's package selector); `--debug-perf` | Tauri 2 reserves `--profile`. Re-using would silently fail or do unexpected things to cargo profile selection. The naming collision is documented in `main.rs` so the next person doesn't try to "fix" it |
| D12 | **WAL + read-only secondary connection (R2)** | Single connection with WAL (previous); `r2d2-sqlite` / `deppool-sqlite` connection pool; per-thread connections | Pre-R2, foreground `get_images` queued behind in-flight encoder write batches — the perf-1777212369 22-second freeze. With R2, foreground SELECTs are non-blocking against active writes. Connection pool considered and rejected as over-engineered for the simple writer + read-only secondary pattern (deferred as R13) |
| D13 | **`BEGIN IMMEDIATE` for batched encoder writes** | Per-row `upsert_embedding` autocommit (previous — produced 22 s freeze); `BEGIN DEFERRED`; `BEGIN EXCLUSIVE` | `IMMEDIATE` takes the write lock up-front so the batch can't deadlock against a reader who decided to upgrade. Per-row autocommit produces N implicit transactions + N fsyncs (~10–100× slower for bulk). Manual `checkpoint_passive` between batches keeps WAL bounded under `wal_autocheckpoint=0` |
| D14 | **`paths::*_dir()` as single disk-path source; no dev/release split** | Keep dev/release split | Pre-2026-04-26 the `cfg(debug_assertions)` split pointed dev at `<repo>/Library/`; dev and release diverged on every code change, forcing re-downloads of 2.5 GB of models. Removed; `IMAGE_BROWSER_DATA_DIR` env override is the supported successor for sandboxed testing |
| D15 | **Multi-folder roots with `ON DELETE CASCADE` (gated by `PRAGMA foreign_keys=ON`)** | Single-folder model (original); soft-delete via `enabled` only; per-image root reference without FK | SQLite defaults `foreign_keys` OFF for backwards compat; without the pragma, `ON DELETE CASCADE` silently no-ops — root removal would leave orphan image rows forever. The pragma is the explicit fix. Per-root thumbnail subfolders make `remove_root` `rm -rf` the subfolder cleanly |
| D16 | **`bytemuck::cast_slice` for embedding BLOB encoding** | `unsafe { slice::from_raw_parts(...) }` (previous, 3 sites); base64-encoded strings; serde_json arrays; separate `.bin` per image | Canonical safe way to view a `&[f32]` as a `&[u8]` and back. The audit (`0bdb5f4`) extracted this pattern. BLOB-in-SQLite avoids the JSON parse overhead and keeps embeddings co-located with image rows |
| D17 | **Typed errors via `ApiError` enum, not strings** | `Result<T, String>` (original — what 3 profiling commands still use); per-command error types; `anyhow::Error` | Typed wire format means frontend can branch on specific failures (e.g. `TextModelMissing` triggers re-download flow via `isMissingModelError`); generic strings would force string-matching and break silently on phrasing changes. `From<PoisonError>` surfaces mutex poisoning as typed signals |
| D-MUTEX | **`std::sync::Mutex` over `parking_lot::Mutex`** | `parking_lot::Mutex` (no poisoning); `RwLock`; lockfree structures | For a single-user desktop app, restart is fast. The pragmatic posture is poison-then-restart, not flailing in a partially-broken state. `From<PoisonError>` converts poison to `ApiError::Cosine("mutex poisoned: ...")` so frontend gets typed signals. `parking_lot` is the strict-upgrade if poisoning ever proves load-bearing in practice |
| D-RTAG | **R-tag perf annotation comments in code** | No annotation; commit-message-only traceability; separate "blame-style" doc | 43 `R<n>` annotations across the perf-bundle commits provide forward + reverse traceability + commit-review aid. A reader can grep `R<n>` to find every line that landed for that recommendation. Worth its visual cost only when the recommendation set is structured + plan-driven |

**Approaches tried and rejected (preserved as historical context):**

- Tiered random-sampling diversity for image-image similarity (the pre-Phase-5 approach, 7 tiers × 5 images from each cosine band) — replaced by RRF (D5). Function preserved at `cosine/index.rs::get_tiered_similar_images` for reference + unit tests but no longer called.
- Single "active" encoder per direction with a dropdown picker — two problems (no single encoder is best across queries; picking one disabled the others' embeddings); replaced by per-encoder enable/disable + always-fuse.
- Hardcoded `test_images/` path in `main.rs` — replaced by native folder picker + multi-folder `roots` table.
- Pre-Phase-9 flat `thumbnails/` layout — replaced by `thumbnails/root_<id>/` per-root subfolders.
- `println!`-shaped logging — replaced wholesale by `tracing` spans with `ipc.` / `pipeline.` / `cosine.` / `model_download.` / `watcher.` prefixes.
- Single CLIP graph with dummy text inputs at image-encode time — replaced by separate-graph CLIP exports.
- Pure-Rust hand-rolled WordPiece tokenizer — replaced by HF `tokenizers` crate.
- `unsafe { slice::from_raw_parts(...) }` for embedding BLOB casts (3 sites) — replaced by `bytemuck::cast_slice` in audit commit `0bdb5f4`.
- Triplicated `normalize_path` closure for Windows extended-prefix stripping (3 sites) — extracted into `paths::strip_windows_extended_prefix` returning `Cow::Borrowed`.
- `Library/` folder inside the repo for dev state — removed alongside the dev/release split.

## What is currently built

The project has shipped a multi-encoder fusion-search desktop app with profiling, multi-folder lifecycle, audit-passed code health, and a comprehensive internal context layer. Concretely, at HEAD `7e5cf85`:

| Dimension | Value |
|---|---|
| Cargo package | `image-browser` v0.1.0, Rust edition 2021 |
| Tauri identifier | `com.ataca.image-browser` |
| Backend Rust files | 28 (split across `commands/`, `db/`, `cosine/`, `encoder_text/`, plus single-file modules) |
| Frontend TypeScript files | 33 |
| Backend tests (`cargo test`) | 125 lib + integration, all passing |
| Frontend tests (`vitest`) | 62, all passing |
| Tauri commands | **26**, organised by concern under `src-tauri/src/commands/` |
| SQLite tables | 5 (`roots`, `images`, `tags`, `images_tags`, `embeddings`) + `meta` for migration version |
| Image encoders | **CLIP ViT-B/32** (512-d), **DINOv2-Base** (768-d, image-only), **SigLIP-2 Base 256** (768-d shared text+image) |
| Text encoders | **CLIP** (BPE 49k, 77 tokens, pad 49407), **SigLIP-2** (Gemma SP 256k, 64 tokens, no attention_mask) |
| Multi-encoder fusion | **RRF Cormack 2009, k=60** for image-image (3 encoders) and text-image (2 encoders) |
| Models on disk | ~2.5 GB across 7 files (5 ONNX + 2 tokenizer.json), all FP32 |
| Embedding-pipeline version | 4 (bumped 2026-04-26) |
| Clippy | Clean post-Phase-6 + post-Phase-12 |

**Concrete features the app delivers today:**

- Indexes one or more user-picked folders via native folder picker (`tauri-plugin-dialog`); multi-folder via `roots` table; per-root toggle; CASCADE delete on root removal.
- Watches the filesystem (`notify-debouncer-mini`, 5 s debounce); single-flight coalescing for rapid bursts.
- Generates 400 × 400 thumbnails using JPEG scaled IDCT + `fast_image_resize 6.x` NEON Lanczos3; falls back to `image-rs` for non-JPEG and decode errors; per-root subfolder layout.
- Encodes every enabled image encoder in parallel (Phase 11e) — one thread per encoder, each with its own DB connection, batched `BEGIN IMMEDIATE` writes per ~32-row chunk.
- Searches three ways: click an image → image-image RRF over CLIP + SigLIP-2 + DINOv2; type a query → text-image RRF over CLIP + SigLIP-2; `#tag` query → tag-filter SQL with AND/OR mode.
- Tags + annotations: tag CRUD with optimistic mutations; `#tag` autocomplete + create-on-no-match + delete affordance; AND/OR filter mode; per-image notes textarea persisted to a `notes` column.
- Profiles when asked: `--profiling` flag mounts PerfLayer + cmd+shift+P PerfOverlay + 1 Hz RSS/CPU sampler + on-exit markdown report with Stall Analysis and Resource Trends sections. Zero overhead when off.
- Settings drawer with 7 sections (Theme / Display / Search / Sort / Folders / Reset / Encoder); IndexingStatusPill for live progress; FOUC-free dark mode via pre-React theme application.
- 28-finding code-health audit shipped end-to-end (every finding closed, three residuals tracked as targeted near-term work).
- README rewrite (2026-04-28, `7e5cf85`, +208/-132 lines) describing 3-encoder + RRF + multi-folder + `--profiling` feature surface, design principles, tech stack table, per-platform app-data-dir table — closes the prior HIGH-severity README drift item.

## Current state

Status: `active`. HEAD commit `7e5cf85` dated 2026-04-28; last code commit `ecb4386` on 2026-04-26 (17 days before sync date 2026-05-13). The 2026-04-25/26 sprint landed full encoder-pipeline overhaul (separate-graph CLIP + DINOv2 + SigLIP-2), multi-folder roots + watcher + orphan handling, async indexing pipeline with live progress, complete profiling + 12-diagnostic stack + 1 Hz RSS/CPU sampler, code-health audit (28 findings, all shipped), Tier 1+2 perf bundles, RRF fusion (Phase 5) + per-encoder toggles + parallel encoders + text-image fusion (Phases 11a-d), Phase 12 perf bundle — 31 commits on 2026-04-26 alone (more than the entire pre-2026 history combined). The 2026-04-28 README rewrite closed the highest-priority drift item flagged in the prior LifeOS extraction. The 15-day post-sprint quiescence is short relative to the project's prior 4-month silences and reads as expected cooldown. In-flight: deferred FHE-on-vectors plan tracked at `Work/Encrypted Vector Search.md` (proposed, not started); README walkthrough video planned at `Work/README Demo.md` (pending — Caner records, mp4 at repo root under a `## Demo` section).

## Gaps and known limitations

**MEDIUM — code-health audit residuals (near-term sweep):**

- **Legacy single-encoder commands D-SIM-1, D-SEM-1, D-FE-1.** Phase 11d wired text-image fusion through `get_fused_semantic_search`; `useSemanticSearch` routes through it. The legacy `semantic_search` (single CLIP), `get_similar_images` (single-encoder by id), and `get_tiered_similar_images` (pre-fusion 7-tier random sampler) are no longer called from the frontend but remain registered in `lib.rs::run`'s `invoke_handler!`. ~600 Rust + ~80 TS dead lines across `commands/{semantic,similarity}.rs`, `cosine/index.rs::get_tiered_similar_images`, `services/images.ts`, `queries/useSimilarImages.ts`. Pure deletion sweep tracked in `plans/code-health-audit/`. The hook name `useTieredSimilarImages` is preserved (caller stability) but routes through fusion under the hood.
- **`Settings::priority_image_encoder` field is doc-deprecated but still read.** Phase 11c per-encoder enable/disable replaced the single-choice picker; the field is documented as "LEGACY (Phase 11c)" but `indexing.rs` still reads it. A user editing `settings.json` to set this field would have no effect. Fix: remove the read in `indexing.rs`; keep the field on the struct so old `settings.json` files deserialise without error.
- **`db::get_embedding` skips the R2 secondary (last R2 gap).** Every other foreground SELECT routes through `read_lock()`; `db::get_embedding` still uses `self.connection.lock()` (writer mutex). A fusion search calling it for the query vector queues behind in-flight encoder write batches. Two-line fix: swap `connection.lock()` for `read_lock()`.

**MEDIUM — known structural risks (not currently broken, documented):**

- **Filesystem watcher does not rebuild on `add_root` / `remove_root`.** `watcher_state` is filled once during `lib.rs::run::setup` and never refreshed. Adding a root after launch updates the DB + spawns a rescan, but filesystem events for the new root don't fire until the next launch. The architectural surface for fixing exists — `watcher_state` is already `Arc<Mutex<Option<WatcherHandle>>>`.
- **`paths.path` is stored verbatim (no normalisation).** The `roots.path` `UNIQUE` constraint compares strings literally. A user picking `/Users/me/Photos/` then later `/Users/me/Photos` (without trailing slash) gets two rows. Cosmetic — both work, the Folders list shows duplicates.
- **Encoder set is hardcoded in `commands/similarity.rs`.** The list `["clip_vit_b_32", "siglip2_base", "dinov2_base"]` is a `&[&str]` constant. Adding a fourth encoder requires editing this constant in addition to registering it in `commands::encoders::ENCODERS` and adding a branch in `indexing.rs::run_encoder_phase`. Additive change; no current functional issue.
- **Frontend score labelling for fused results.** The fused score (~0–0.05 for 3 encoders + k=60) is not a cosine similarity and is unbounded. The masonry grid currently doesn't display the score; any future tooltip surfacing it must label it "Fused" rather than "Cosine similarity" or normalise to [0, 1] for display.

**LOW — documented edge cases:**

- `dirs::data_dir()` returning `None` in release would fall back to `./app-data` (relative to cwd) and log a warn. Mostly theoretical on macOS / Linux / Windows.
- Atomic save uses `rename` not `fsync`. Power loss between `write` and `rename` could leave the `.tmp` file on disk with the original `settings.json` unchanged (acceptable).
- `add_root` propagates a `UNIQUE` constraint error as `ApiError::Db` rather than `ApiError::BadInput("already added")`. Cosmetic; user gets a typed-but-generic DB error.
- Empty-state UI's "Choose folder" pill uses `set_scan_root` (replace-all) where the top-bar uses `add_root` after the 2026-04-26 rename; with zero existing roots the difference is zero but UX paths are inconsistent.
- `wipe_images_for_new_root` only fires inside `set_scan_root`, not `add_root`; legacy NULL-root_id rows persist. Functionally fine (grid keeps NULL rows visible).
- Per-root thumbnail directory removal is best-effort; permissions reject logs warn (DB rows are gone via CASCADE, so orphaned files are inert).
- Session-mid lock poisoning surfaces as `ApiError::Cosine("mutex poisoned: ...")` via `From<PoisonError>`; recovery requires restart. `parking_lot::Mutex` is the strict-upgrade if it ever bites in practice.

**Closed since the prior 2026-04-24 LifeOS extraction:** README drift, hardcoded `test_images` path, missing folder picker UI, missing tag deletion in UI, missing slideshow (subsumed by modal keyboard nav + 3D tilt + annotations), memory-bank/ folder, `unsafe` BLOB casts, triplicated `normalize_path` closure, triplicated 3-strategy DB-id lookup, duplicated `aggregate_image_rows` pattern, `println!`-shaped logging, `models/` "user-supplied" assumption, `add_tag_to_image` plain INSERT, orphaned `useSimilarImages` hook, orphaned `ImageData::with_thumbnail`, missing test infrastructure (now 125 + 62 passing), hardcoded `Path::new("test_images")` in `main.rs`, ImageNet-stats CLIP preprocessing (replaced with CLIP-native), single-graph CLIP with dummy text inputs, pure-Rust WordPiece tokenizer, per-row encoder autocommit (22 s freezes), foreground SELECTs queuing behind writes (R2), thumbnail pipeline using image-rs default Lanczos3 (R6+R7), random shuffle as default sortMode (now opt-in).

## Direction (in-flight, not wishlist)

**Highest-impact next pickup (active near-term work, not aspirational):**

1. **Code-health audit residuals D-SIM-1 / D-SEM-1 / D-FE-1** — pure deletion sweep of ~600 Rust + ~80 TS lines.
2. **`Settings::priority_image_encoder` removal** — one-line state cleanup.
3. **`db::get_embedding` → R2 secondary** — two-line fix closing the last R2 gap.

All three are documented in the repo's `plans/code-health-audit/` and are low-risk surface drift closures.

**Active Work files in LifeOS:**

- **Encrypted Vector Search (`Work/Encrypted Vector Search.md`)** — status `proposed`, cross-pollinated from Strategy/2026-04-25/proposals/03. Adds an encrypted-vector storage path (TFHE-rs or BFV ciphertexts of CLIP embeddings) alongside the existing plaintext path; strictly additive — existing plaintext flow stays in production. Driven by Apple Wally (iOS 18) + Pacmann + Panther papers + TFHE-rs validating FHE-on-vectors as a real category in 2024–2025. Honest framing: FHE has a 4–5 orders of magnitude slowdown vs plaintext; the encrypted path is not for "competing with plaintext speed" but for privacy-sensitive use cases where the alternative is "not running the search at all." Sequencing gate: requires Image Browser v1 polish to be shipped first (now done). Significant new code surface (encrypted index, two-process architecture, reference application demonstrating "ciphertext only", benchmark harness comparing encrypted vs plaintext CosineIndex on a 10k-image dataset). Optional ambitious-vision additions (do not block MVP): structured tree indexes (encrypted KD-tree variant), `swift-homomorphic-encryption` cross-language interop, differential-privacy noise calibration, encrypted-text-search adapter.
- **README Demo (`Work/README Demo.md`)** — status `pending`. Pattern: Cernio-style walkthrough video (30–60 s mp4 at repo root under a `## Demo` section after the project description). Image Browser is a UI tool — value is in the workflow (browsing, filtering, search), so a guided walkthrough reads better than a frame-loop GIF. Open items: Caner records walkthrough capturing golden path (open folder, browse, filter or search, optionally the encrypted vector search if stable enough to demo), convert to mp4, add `## Demo` section + TOC entry.

## Demonstrated skills

The deliberate signal-amplification section. Specific, project-anchored capabilities this codebase proves.

- **Local-first multi-encoder ML inference at scale in pure Rust.** Three encoder families (CLIP ViT-B/32 + DINOv2-Base + SigLIP-2 Base 256) × CPU × consumer M2 hardware running through `ort 2.0.0-rc.10` + HuggingFace `tokenizers` via a shared M2-tuned `Session` builder factory (`Level3 + intra_threads(4) + inter_threads(1)`, with Phase 12 dynamic `intra = N / enabled_encoders` for parallel encoder phase). No Python at runtime; CoreML disabled on macOS because of transformer-op runtime errors; CUDA target-gated with CPU fallback on non-macOS.
- **Reciprocal Rank Fusion as a production retrieval algorithm.** Correct implementation of the Cormack-Clarke-Büttcher SIGIR 2009 formula `score(p) = Σ 1 / (k + rank_e(p))` with `k = DEFAULT_K_RRF = 60`, 6 unit tests pinning the contract in `cosine/rrf.rs`, asymmetric image-image (3 encoders) vs text-image (2 encoders, DINOv2 has no text branch) configurations, per-encoder evidence captured in diagnostics, lazy per-encoder cache populates in `FusionIndexState`, non-destructive toggle semantics. Includes the principled defence of why RRF over score-fusion (cross-encoder cosine distribution differences) and why uniform k over per-encoder weighting (no labelled validation set; unprincipled to weight without one).
- **Designing for the temporal asymmetry between indexing-time and search-time.** Image encoders run at indexing time (once per image, in parallel by encoder); text encoders run at search time (once per query). Both produce L2-normalised vectors in their respective spaces; search-time cosine reuses indexing-time embeddings. The shared `Arc<Mutex<CosineIndex>>` cloned across the indexing thread and Tauri-managed state means a finished encode immediately makes new embeddings searchable.
- **WAL-mode dual-connection SQLite with R2 secondary closing a 22-second foreground freeze.** Writer `Mutex<Connection>` + read-only secondary `OnceLock<Mutex<Connection>>` accessed via `read_lock()`; foreground SELECTs non-blocking against active writes; identified specifically because `perf-1777212369` showed `ipc.get_images` queueing behind in-flight encoder write batches.
- **`BEGIN IMMEDIATE` batched writes with manual `checkpoint_passive` between batches.** Replaced per-row autocommit (N implicit transactions + N fsyncs) with one transaction + one fsync per ~32-row chunk, under `wal_autocheckpoint=0`. Diagnosed via the profiling Stall Analysis.
- **Replacing `unsafe { slice::from_raw_parts(...) }` with `bytemuck::cast_slice` for zero-copy, alignment-checked safe casts.** 3 sites consolidated in audit `0bdb5f4`.
- **`PRAGMA foreign_keys=ON` as the explicit fix for `ON DELETE CASCADE` no-oping silently in SQLite.** Identified the SQLite-defaults-OFF gotcha that would have left orphan image rows forever; pragma is set at every connection open.
- **Single-flight orchestration with RAII guarantees.** `IndexingState.is_running: AtomicBool` with `RunningGuard(Drop)` ensures clear on success, error, AND panic — coalescing rapid filesystem events into one pipeline without locking out future indexing if the pipeline panics.
- **`tracing`-based opt-in profiling infrastructure with no-overhead-when-off semantics.** `--profiling` flag (deliberately not `--profile` due to Tauri 2 collision); PerfLayer registers only when enabled; `record_diagnostic` early-returns; 12 named domain diagnostics (`embedding_stats`, `pairwise_distance_distribution`, `self_similarity_check`, `score_distribution_stats`, `tokenizer_output`, `query_embedding`, `preprocessing_sample`, `encoder_run_summary`, `cosine_math_sanity`, `path_resolution_outcomes`, `cross_encoder_comparison`, `cosine_cache_populated`) spread across 17 call sites; 1 Hz RSS+CPU sampler via `sysinfo 0.32`; on-exit markdown report with Stall Analysis and Resource Trends. Built specifically because the perf-1777212369 freeze was undebuggable without it; pays off immediately on Tier 1+2 perf-bundle selection.
- **Typed errors on the IPC wire with `#[serde(tag, content)]` discriminated unions.** `ApiError` Rust enum mirrored in `services/apiError.ts`; `From<rusqlite::Error>`, `From<std::io::Error>`, `From<PoisonError<T>>` impls let command bodies use `?` directly; specific kinds (`TextModelMissing`) trigger specific frontend flows; unknown kinds handled gracefully by `formatApiError` default case.
- **Atomic file-write patterns** for settings (`.tmp` + rename) and model downloads (`.part` + rename); survives mid-write power loss without corrupting the prior file.
- **Code-health audit shipped end-to-end (28 findings).** Not "logged for later" — every finding shipped; clippy gate restored; dead-code inventory current; residuals list small and tracked. Includes structural extractions: `aggregate_image_rows` from 4 callers, 3-strategy DB-id lookup into `commands::resolve_image_id_for_cosine_path`, Windows extended-prefix stripping into `paths::strip_windows_extended_prefix(Cow<'_, str>)`, multiple `impl ImageDatabase` blocks across `db/` submodules.
- **Embedding-pipeline versioning** as the migration mechanism for propagating preprocessing changes. `meta(key, value)` table tracks `embedding_pipeline_version` (currently v4); bump-then-migrate wipes affected legacy rows on first launch; next indexing pass re-encodes cleanly under the new pipeline.
- **Partial-sort + reusable scratch buffer** for top-K retrieval. `select_nth_unstable_by` is O(N) average vs O(N log N) full sort; 2.53× measured speedup at N=10000 (audit `c6551e2`). NaN-aware `score_cmp_desc` so unstable sort doesn't panic on degenerate inputs.
- **Documented architectural complementarity across encoder preprocessing.** CLIP uses CLIP-native mean/std + shortest-edge-224 + center-crop; DINOv2 uses ImageNet stats (its training-time pipeline) + shortest-edge-256 + center-crop-224; SigLIP-2 uses exact-square stretch + `[-1, 1]` (its training-time pipeline). Different by design, not by accident. SigLIP-2's exact-square preprocessing sees the full image where CLIP and DINOv2 lose edge content — fusion exploits the complementarity.
- **Long-form structured engineering documentation as a discipline.** The repo maintains a 76 KB `context/architecture.md`, 9 `notes/*.md` files, 19 `systems/*.md` deep-dives, `plans/code-health-audit/`, `enhancements/recommendations/` (77 source notes + 11 commitment-graded recommendations + audience analysis). The LifeOS Suggestions note flags that the `notes/fusion-architecture.md` reasoning trail (why score-fusion is fragile, why RRF, why uniform k=60, why per-encoder caches not single shared, why text-image fusion is asymmetric, why per-encoder enable-disable non-destructive) is itself a portfolio-worthy artefact.
- **Designing systems against named failure modes.** The profiling framing — "encode the validation into the runtime; the report is the debugger" — replaces a planned Python-reference validation harness with 12 named diagnostics spread across 17 call sites, ensuring future bugs surface in the next perf report without source diving.
- **Closure of LifeOS-vault-to-README drift in a single docs-only commit.** Commit `7e5cf85` (2026-04-28) rewrote the README from describing the November 2025 single-CLIP single-folder app to mirroring `context/architecture.md`'s structure: project intent + features (browsing / multi-folder / tagging / notes / visual similarity / semantic search / settings / performance / privacy), how-to-use section, IPC architecture diagram, design-principles section, tech-stack table, per-platform app-data-dir table. Closes the prior HIGH-severity drift item with +208/-132 lines.

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
