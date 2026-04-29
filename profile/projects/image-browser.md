---
name: Image Browser
status: active
source_repo: https://github.com/Capataina/PinterestStyleImageBrowser
lifeos_folder: Projects/Image Browser
last_synced: 2026-04-29
sources_read: 29
---

# Image Browser

## One-line summary

Local-first Tauri 2 desktop image manager with multi-encoder Reciprocal-Rank-Fusion semantic search (CLIP + DINOv2 + SigLIP-2), filesystem watcher, async indexing pipeline, and an opt-in profiling layer with 12 named diagnostics — all offline, ONNX-Runtime on CPU/CUDA.

## What it is

Image Browser is a local-first Tauri 2 desktop application for browsing, tagging, semantically searching, and annotating large personal image libraries. The Rust backend handles filesystem scanning, SQLite (WAL) persistence, thumbnail generation, ONNX-Runtime inference across **three encoder families** (CLIP ViT-B/32, DINOv2-Base, SigLIP-2 Base 256), multi-folder lifecycle, a filesystem watcher with orphan detection, an opt-in profiling + domain-diagnostic layer, and first-launch model downloads from HuggingFace. The React 19 frontend renders a Pinterest-style masonry grid, a modal inspector with annotations, a multi-section settings drawer, an indexing-status pill, and an opt-in perf overlay. Everything runs offline; CPU on macOS for ONNX (CoreML produces runtime errors for these graphs), CUDA on non-macOS with CPU fallback. The README still describes the original single-CLIP single-folder app — the code on master is now multi-encoder, multi-folder, fusion-search, profiled, audit-passed with 26 Tauri commands. Treat the README as historical; trust the code, the in-repo `context/` folder, and these vault notes.

## Architecture

The system is a Tauri 2 app with a strict IPC boundary between the Rust backend (single binary) and the React 19 frontend.

```
┌────────────────────────────────────────────────────────────────────┐
│  React 19 + Vite 7 + vite-plugin-pages frontend                    │
│  • Masonry grid (shortest-column packing, hero promotion)          │
│  • PinterestModal inspector + tag CRUD + notes + autocomplete      │
│  • TanStack Query staleTime:Infinity; useUserPreferences localStor │
│  • Cmd+Shift+P perf overlay (when --profiling)                     │
└──────────────────────────┬─────────────────────────────────────────┘
                           │ Tauri IPC (26 commands, ApiError typed)
                           ▼
┌────────────────────────────────────────────────────────────────────┐
│  Rust backend                                                       │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  commands/  (26: images, tags, notes, roots, similarity,    │   │
│  │             semantic, semantic_fused, profiling, encoders)  │   │
│  └────────────────────────────┬────────────────────────────────┘   │
│  ┌─────────────────────────┐  │  ┌─────────────────────────────┐  │
│  │  indexing.rs           │◄─┴─►│  watcher.rs (5s debounce)    │  │
│  │  single-flight pipeline│     │  notify-debouncer-mini       │  │
│  │  • cache load          │     └─────────────────────────────┘  │
│  │  • model download      │                                       │
│  │  • text encoder warm   │     ┌─────────────────────────────┐  │
│  │  • scan + orphan-mark  │────►│  encoder*.rs (3 image encs)  │  │
│  │  • rayon thumbnails    │     │  encoder_text/ (CLIP+SigLIP) │  │
│  │  • parallel encoders   │     │  ort 2.0.0-rc.10 sessions   │  │
│  │  • cosine populate+save│     └─────────────────────────────┘  │
│  └────────┬────────────────┘                                       │
│           ▼                                                          │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  SQLite (WAL+NORMAL+foreign_keys=ON, dual connection R1/R2) │   │
│  │  roots, images, tags, images_tags, embeddings (+ meta)      │   │
│  │  embedding_pipeline_version meta migration                  │   │
│  └─────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────┘
```

Dependency direction is strictly downward through SQLite. Two SQLite connections per real DB: writer (R1) for the indexing pipeline, read-only secondary (R2) for command handlers. Embedding-pipeline migrations bump a meta version that wipes legacy embeddings on first launch under new code.

## Subsystems and components

| Subsystem | Responsibility | Key files |
|---|---|---|
| **CLIP Image Encoder** | OpenAI English ViT-B/32 vision_model.onnx; bicubic-shortest-edge-224 + center-crop; CLIP-native mean/std; 512-d L2-normalised | `encoder.rs` |
| **CLIP Text Encoder** | HF tokenizers BPE (max 77, pad 49407); separate `text_model.onnx`; output cascade text_embeds → pooler_output → sentence_embedding | `encoder_text/encoder.rs` |
| **DINOv2 Encoder** | Meta self-supervised image-only; bicubic-shortest-edge-256 + center-crop-224; ImageNet stats; CLS-token from `last_hidden_state[:,0,:]`; 768-d | `encoder_dinov2.rs` |
| **SigLIP-2 Encoder** | Google sigmoid loss, image+text in shared 768-d; image: 256×256 exact-square + [-1,1]; text: Gemma SP 64 tokens *no* attention_mask; both branches use pooler_output (MAP head) | `encoder_siglip2.rs` |
| **Multi-Encoder Fusion** | RRF (Cormack 2009, k=60) over 3 image encoders for image-image and 2 text encoders for text-image; FusionIndexState lazy populate; per-encoder evidence in diagnostics | `cosine/rrf.rs`, `commands/similarity.rs::get_fused_similar_images`, `commands/semantic_fused.rs` |
| **Cosine Similarity** | partial-sort (`select_nth_unstable_by`, 2.53× speedup), reusable scratch buffer, persistent `cosine_cache.bin` with mtime freshness check; 4-helper diagnostics module | `cosine/` |
| **Database** | 5 tables; WAL+NORMAL; dual connection (writer + read-only secondary R2); embedding BLOB via `bytemuck::cast_slice` (replaces 3 unsafe blocks); 4 idempotent migrations | `db/mod.rs`, `db/schema_migrations.rs` |
| **Tauri Commands** | 26 commands grouped by concern; `ApiError` typed-error wire format (`#[serde(tag="kind", content="details")]`) mirrored in `services/apiError.ts`; lazy text-encoder init; path-prefix normalisation | `commands/` |
| **Indexing Pipeline** | background single-flight; phases: cache load → model download → text encoder pre-warm → scan → orphan-mark → thumbnail (rayon) → encode-parallel-by-encoder → cosine populate + save; 12 diagnostic emissions | `indexing.rs` |
| **Multi-Folder Roots** | `roots` table; `images.root_id` FK + ON DELETE CASCADE (gated by PRAGMA foreign_keys=ON); per-root thumbnail subfolder; `set_root_enabled` toggle without re-encode | `db/roots.rs`, `commands/roots.rs` |
| **Watcher** | `notify-debouncer-mini`, 5s debounce, single-flight coalescing via `IndexingState.is_running` AtomicBool with RAII guard | `watcher.rs` |
| **Model Download** | first-launch HuggingFace fetch (~2.5GB across 7 files), HEAD preflight + chunked GET + per-byte progress callback, per-file fail-soft | `model_download.rs` |
| **Profiling** | `--profiling` flag (NOT `--profile` — Tauri 2's CLI owns that name); PerfLayer + 12 named diagnostics + 1 Hz RSS/CPU sampler; on-exit `report.md` with Stall Analysis + Resource Trends; off by default, zero overhead | `perf.rs`, `perf_report.rs`, `cosine/diagnostics.rs` |
| **Thumbnail Pipeline** | JPEG path uses `jpeg-decoder::Decoder::scale()` for native scaled IDCT (1/8, 1/4, 1/2 factor) then `fast_image_resize 6.x` (NEON-optimised Lanczos3); falls back to `image-rs` for non-JPEG | `thumbnail/generator.rs` |
| **Masonry Layout** | shortest-column packing; hero promotion across up to 3 columns; 3D framer-motion tilt; sortMode-aware; tile dimensions sourced from backend (no DOM image-load round-trip) | `src/components/Masonry.tsx` |

## Technologies and concepts demonstrated

### Languages
- **Rust 2021** — backend, 28 source files, full ML inference + IPC + DB layer.
- **TypeScript 5.x** — frontend, 33 files, React 19 with hooks + TanStack Query.

### Frameworks
- **Tauri 2** — desktop shell with native folder picker, manage() state injection, CLI plugin, dialog plugin.
- **React 19** — frontend; functional components + hooks; TanStack Query for server state; framer-motion for masonry tilt.
- **Vite 7 + vite-plugin-pages** — file-based routing.

### Libraries
- **ort 2.0.0-rc.10** — ONNX Runtime Rust binding; shared M2-tuned `Session` builder (Level3 graph optimisation, intra=4, inter=1).
- **tokenizers 0.22.2** (HF) — BPE for CLIP, SentencePiece for SigLIP-2; uniform `tokenizer.json` interface across both.
- **rusqlite** (bundled, WAL) — embedding storage as BLOB via `bytemuck::cast_slice`.
- **fast_image_resize 6** — NEON-optimised Lanczos3 downsample.
- **jpeg-decoder** — native scaled IDCT via `Decoder::scale()`.
- **notify-debouncer-mini** — filesystem watcher with native debouncing.
- **rayon** — data-parallel thumbnail generation.
- **bytemuck** — zero-copy float-array → byte-slice conversion (replaced 3 `unsafe` blocks).

### Engines and runtimes
- **ONNX Runtime** — three encoder graphs (CLIP, DINOv2, SigLIP-2), shared session builder, CPU on macOS (CoreML produces runtime errors for these graphs), CUDA on non-macOS with CPU fallback.

### Tools
- **Vitest** — frontend testing (62 tests).
- **Cargo test** — backend testing (125 lib + integration).

### Domains and concepts
- **Multi-encoder retrieval / late fusion** — three independent embedding spaces combined via Reciprocal Rank Fusion (Cormack et al. 2009, k=60); RRF discards cosine values and uses only rank, sidestepping the trap where CLIP's 0.85 is not comparable to DINOv2's 0.85.
- **Vision-language semantic search** — text-image RRF over CLIP + SigLIP-2 (DINOv2 has no text branch).
- **Local ML inference in Rust** — three encoder families running on CPU, no Python in the runtime path, ~2.5GB of models managed via fail-soft per-file HuggingFace download.
- **Tokeniser portability** — single HF tokenizers crate handling both BPE (CLIP) and SentencePiece (SigLIP-2 Gemma) via uniform `tokenizer.json` interface.
- **SQLite WAL with dual connections** — writer + read-only secondary, foreign keys + busy_timeout + manual checkpoint between batches; replaces per-row autocommit that triggered multi-second WAL stalls.
- **Profiling-first development culture** — `--profiling` flag introduces zero overhead when off; PerfLayer + 1 Hz RSS/CPU sampler + on-exit markdown report with Stall Analysis.
- **Audit-passed posture** — 28 code-health-audit findings, all shipped.
- **CDP/IPC patterns** — Tauri 2 invoke_handler with typed ApiError + serde discriminated union; mirrored in TypeScript apiError.ts.

## Key technical decisions

- **Three encoders, late fusion via RRF.** CLIP for general semantic, DINOv2 for visual feature similarity (no text), SigLIP-2 for sigmoid-loss text-image alignment. Late fusion via rank-based RRF avoids cosine-distribution mismatch.
- **Separate-graph CLIP** (vision_model.onnx + text_model.onnx) instead of one shared model — simpler ort lifecycle, smaller download per encoder type.
- **Persistent `cosine_cache.bin` with mtime freshness check** — first launch warmth without re-running encoders.
- **No CoreML on macOS** — CoreML produces runtime errors for these graphs, so CPU is the macOS path.
- **App-data dir same in dev and release** (no `dev/release` split) — rationale: dev/release diverging on every code change forced re-downloads of 2.5GB of models. `IMAGE_BROWSER_DATA_DIR` env override is the test escape hatch.
- **Embedding-pipeline version in `meta` table** — bumping the const wipes legacy embeddings on first launch. Currently version 4.
- **Single-flight indexing via `IndexingState.is_running` + RAII guard** — bursts coalesce instead of spawning concurrent pipelines.
- **Replace 3 `unsafe` blocks with `bytemuck::cast_slice`** — embedding BLOB conversion.
- **`ApiError` typed wire format** — discriminated union mirrored in TypeScript so frontend errors are exhaustive.
- **`--profiling` flag, not `--profile`** — Tauri 2's CLI owns the `--profile` name.

## What is currently built

- 28 backend Rust files, 33 frontend TypeScript files; 125 cargo tests + 62 vitest tests passing.
- 5 SQLite tables, 4 idempotent migrations, embedding-pipeline meta version bumped to 4.
- 26 Tauri commands grouped by concern.
- 3 image encoders (CLIP, DINOv2, SigLIP-2) running in parallel on indexing.
- 2 text encoders (CLIP, SigLIP-2) wired through picker dispatch.
- Reciprocal Rank Fusion for image-image (3 encoders) and text-image (2 encoders).
- Multi-folder root system with per-root thumbnail subfolder + `set_root_enabled` toggle.
- Filesystem watcher (5s debounce, single-flight coalescing).
- Model download on first launch (~2.5GB, 7 files, fail-soft per-file).
- Tag CRUD + autocomplete + AND/OR filter mode + per-image notes.
- Profiling layer with PerfLayer + 12 named diagnostics + 1 Hz RSS/CPU sampler + on-exit markdown report.
- Code-health audit: 28 findings, all shipped.
- Tier 1+2 perf bundles + Phase 12 perf bundle.

## Current state

Active. HEAD `ecb4386` (2026-04-26). 31 commits in last 24h at time of LifeOS verification — heavy iteration with multiple multi-phase landings/day. Build green: 125/125 cargo lib · 62/62 vitest · clippy clean. The project went from quiet (Dec 2025 → late April 2026) to intense sustained development between 2026-04-25 and 2026-04-26.

## Gaps and known limitations

- **README drift** — README still describes single-CLIP single-folder app; code is multi-encoder + multi-folder + fusion-search.
- **CoreML disabled** — produces runtime errors for these graphs.
- **2.5GB model footprint** — first-launch HuggingFace download per platform.
- **Encrypted Vector Search deferred** — FHE-on-vectors plan in `Work/Encrypted Vector Search.md` not yet implemented.

## Direction (in-flight, not wishlist)

- Continued performance bundles (Phase 13+).
- Encrypted vector search (FHE) — design exists in `Work/Encrypted Vector Search.md`.

## Demonstrated skills

- **Multi-encoder retrieval system design** — three independent embedding spaces, late-fusion via Reciprocal Rank Fusion, FusionIndexState lazy populate.
- **Local ML inference in Rust** — three ONNX-Runtime encoder graphs across CLIP / DINOv2 / SigLIP-2 with shared M2-tuned Session builder.
- **Tauri 2 + React 19 desktop app** — 26 invoke handlers, typed ApiError wire format, manage() state injection, native folder picker, CLI plugin.
- **SQLite WAL with dual connections** — writer + read-only secondary, foreign keys + busy_timeout + manual checkpoint, embedding BLOB via bytemuck::cast_slice replacing 3 unsafe blocks.
- **Filesystem watcher with single-flight coalescing** — notify-debouncer-mini with RAII-guarded AtomicBool to coalesce bursts.
- **Profiling-first development discipline** — opt-in `--profiling` flag, zero overhead when off, PerfLayer + 12 named diagnostics + 1 Hz RSS/CPU sampler + on-exit markdown report with Stall Analysis.
- **Audit-passed posture** — 28 code-health findings shipped.
- **Performance engineering** — partial-sort cosine (2.53× speedup), persistent `cosine_cache.bin` with mtime freshness, parallel encoders, JPEG scaled IDCT + NEON Lanczos3 thumbnails, batch-checkpoint embedding writes replacing per-row autocommit (multi-second WAL stalls eliminated).

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Image Browser/_Overview.md | 153 | "`#image-browser` `#tauri` `#rust` `#react` `#clip` `#dinov2` `#siglip2` `#rrf` `#multi-encoder-fusion` `#onnx-runtime` `#sqlite-wal` `#local-first` `#ml-inference` `#profiling` `#masonry`" |
| Projects/Image Browser/Architecture.md | 333 | "The Coverage section in the repo's own `architecture.md` § Coverage (line 553+) is the canonical place for these counts." |
| Projects/Image Browser/Baselines.md | 212 | "\| Default top_n semantic \| 50 \| 50 (unchanged) \| \|" |
| Projects/Image Browser/Decisions.md | 277 | "- `notes/encoder-additions-considered.md` (in repo) — D4 candidate inventory + decisions per candidate" |
| Projects/Image Browser/Gaps.md | 172 | "The previous vault Suggestions note recommended \"Delete the memory-bank/ folder\"; that has been done." |
| Projects/Image Browser/Roadmap.md | 159 | "- `Capataina/PinterestStyleImageBrowser/context/plans/code-health-audit/` — the 28-finding plan that drove the audit's residuals into this roadmap" |
| Projects/Image Browser/Suggestions.md | 159 | "- [[Profile/Professional/Resume - Ata Caner Cetinkaya]] + [[Profile/Professional/Cover Letter - Ata Caner Cetinkaya]] — feed strongest current evidence forward" |
| Projects/Image Browser/Work/Encrypted Vector Search.md | 59 | "#image-browser #work #fhe #encrypted-vector #privacy-preserving" |
| Projects/Image Browser/Systems/_Overview.md | 50 | "- [[Projects/Image Browser/Roadmap]] — direction of travel" |
| Projects/Image Browser/Systems/CLIP Image Encoder.md | 127 | "- `Capataina/PinterestStyleImageBrowser/context/notes/clip-preprocessing-decisions.md` — full implementation notes" |
| Projects/Image Browser/Systems/CLIP Text Encoder.md | 156 | "- `Capataina/PinterestStyleImageBrowser/context/systems/clip-text-encoder.md` — full implementation notes" |
| Projects/Image Browser/Systems/Cosine Similarity.md | 131 | "- `Capataina/PinterestStyleImageBrowser/context/systems/cosine-similarity.md` — full implementation notes" |
| Projects/Image Browser/Systems/DINOv2 Encoder.md | 118 | "- `Capataina/PinterestStyleImageBrowser/context/notes/encoder-additions-considered.md` — full implementation notes" |
| Projects/Image Browser/Systems/Database.md | 234 | "- `Capataina/PinterestStyleImageBrowser/context/notes/conventions.md` § BEGIN IMMEDIATE — full implementation notes" |
| Projects/Image Browser/Systems/Filesystem Scanner.md | 115 | "- `Capataina/PinterestStyleImageBrowser/context/systems/filesystem-scanner.md` — full implementation notes" |
| Projects/Image Browser/Systems/Frontend State.md | 187 | "- `Capataina/PinterestStyleImageBrowser/context/notes/conventions.md` § Optimistic mutations — full implementation notes" |
| Projects/Image Browser/Systems/Indexing Pipeline.md | 165 | "- `Capataina/PinterestStyleImageBrowser/context/systems/indexing.md` — full implementation notes" |
| Projects/Image Browser/Systems/Masonry Layout.md | 141 | "- `Capataina/PinterestStyleImageBrowser/context/systems/masonry-layout.md` — full implementation notes" |
| Projects/Image Browser/Systems/Model Download.md | 118 | "- `Capataina/PinterestStyleImageBrowser/context/systems/model-download.md` — full implementation notes" |
| Projects/Image Browser/Systems/Multi-Encoder Fusion.md | 179 | "- Cormack, Clarke & Büttcher (2009), *Reciprocal Rank Fusion outperforms Condorcet and Individual Rank Learning Methods*" |
| Projects/Image Browser/Systems/Multi-Folder Roots.md | 177 | "- `Capataina/PinterestStyleImageBrowser/context/systems/multi-folder-roots.md` — full implementation notes" |
| Projects/Image Browser/Systems/Paths and State.md | 139 | "- `Capataina/PinterestStyleImageBrowser/context/systems/paths-and-state.md` — full implementation notes" |
| Projects/Image Browser/Systems/Profiling.md | 162 | "- `Capataina/PinterestStyleImageBrowser/context/notes/conventions.md` § Domain diagnostics — full implementation notes" |
| Projects/Image Browser/Systems/Search Routing.md | 141 | "- `Capataina/PinterestStyleImageBrowser/context/notes/random-shuffle-as-feature.md` — full implementation notes" |
| Projects/Image Browser/Systems/SigLIP-2 Encoder.md | 138 | "- `Capataina/PinterestStyleImageBrowser/context/systems/siglip2-encoder.md` — full implementation notes" |
| Projects/Image Browser/Systems/Tag System.md | 175 | "- `Capataina/PinterestStyleImageBrowser/context/notes/dead-code-inventory.md` § Tags — full implementation notes" |
| Projects/Image Browser/Systems/Tauri Commands.md | 209 | "- `Capataina/PinterestStyleImageBrowser/context/notes/conventions.md` § Mutex acquisition — full implementation notes" |
| Projects/Image Browser/Systems/Thumbnail Pipeline.md | 163 | "- `Capataina/PinterestStyleImageBrowser/context/systems/thumbnail-pipeline.md` — full implementation notes" |
| Projects/Image Browser/Systems/Watcher.md | 83 | "- `Capataina/PinterestStyleImageBrowser/context/systems/watcher.md` — full implementation notes" |
