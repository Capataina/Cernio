---
name: Performance Profiler
status: active
source_repo: https://github.com/Capataina/TerrariaPerformanceProfilerMod
lifeos_folder: Projects/Performance Profiler
last_synced: 2026-05-31
sources_read: 21
github_augmented_at: 2026-05-31
github_augmented_at_commit: ff20711f
github_augmentation_result: no-patch-needed-main-equals-lifeos-anchor
---

# Performance Profiler

## One-line summary

A tModLoader 1.4.4 client-side mod that attributes per-tick CPU and allocation cost to individual mods in the player's modlist via live IL injection (MonoMod / Cecil), surfaces the result through a local browser SPA at `http://127.0.0.1:27277/`, and persists everything to LiteDB with four-layer crash safety.

## What it is

Performance Profiler is the answer to "which mod in my 50-mod Calamity-class modlist is costing me frames, and how do I know?" It hooks every per-tick override in every loaded mod via two interchangeable backends (delegate-pair `MonoModHooks.Add` covering ~71.6% of overrides, and `ILHook` covering ~100% and now the default), times each hook invocation with `Stopwatch.GetTimestamp()` reads, and credits the delta to a per-mod / per-category / per-hook attribution table on a zero-allocation hot path. On top of timing, it captures the full interaction chain — biomes, weather, invasions, bosses, hardmode, time-of-day, sub-worlds, player position, HP, mana, equipped loadout, active buffs, damage taken / dealt, NPC spawns, items created — through generic vanilla / tModLoader surfaces only, so the same instrumentation works against any combination of any tModLoader mods that exist now or in the future (Invariant 5). The full state is streamed through a 17-stream `Data/` pipeline, exposed via a hand-rolled `TcpListener` HTTP server (loopback-bound), polled by a browser SPA with six tabs (Now, Mods, Timeline, Lag, Insights, Self), and persisted to a LiteDB file fronted by an NDJSON redo log, three rotating backups, and a quarantine path. The mod is invisible inside the game; its in-game footprint is one F9 keybind and one chat-line hint on world enter.

## Architecture

Single .NET 8 C# class library packed as a `.tmod` for tModLoader 1.4.4. Four top-level production folders plus a non-shipping `Tests/` xUnit project. One entry-point class (`PerformanceProfiler.cs`) hangs everything off `Mod.Load` / `Mod.Unload`; per-world lifecycle hangs off `ProfilerSystem : ModSystem`.

```
PerformanceProfiler (Mod)
        │  Load: RegisterDataPipeline, open ProfilerDatabase, bind Dashboard
        │  Unload: dispose Database + Dashboard, ILHookInterceptor.Uninstall()
        ▼
ProfilerSystem (ModSystem)
   PostSetupContent  → install backends
   OnWorldLoad       → session start, DataRegistry.Freeze()
   PostUpdateEverything → tick (drives frozen PerTickCallbacks array)
   OnWorldUnload     → flush
        │
        ├──► HookInterceptor (delegate path, ~71.6% coverage)
        └──► ILHookInterceptor (IL path, ~100%, default since b52f8b6)
                shared: HookCategoryRouter, PerModAttribution
                    │
                    ▼
            MetricCollector (per-tick engine, owns ring buffer + spike + stall detectors)
                    │
                    ▼
            Data/ pipeline — DataRegistry.Shared
              17 streams + 4 collectors + 13 aggregators + 23 stats
              + Insights engine (10 detectors) + 14 persistence writers
                    │
              ┌─────┼──────────┐
              ▼     ▼          ▼
            Web/  InsightsEngine  Persistence
            HTTP+SPA  (Shared)    LiteDB + WAL + NDJSON journal + 3 backups
            6 tabs
```

**Arrows are unidirectional.** The hot path stays inside the measurement layer (Hook → MetricCollector). Presentation layers (the Web dashboard, the `SessionRecorder`-driven session JSON) only read from `DataRegistry.Shared`; they do not derive numbers. The policy is encoded in the type system: `ProfilerSystem.Collector` was tightened to `internal` in v0.10 specifically so cross-assembly access becomes a compile error rather than a convention. Snapshots returned by `CurrentSnapshot()` are immutable values (fresh `struct` or `readonly record`), so dashboard polls cannot race the game-thread mutator of the ring buffer.

**The four top-level folders:**

| Folder | Role | LOC | Cardinal rule |
|---|---|---|---|
| `Profiling/` | Hook instrumentation, hot-path engine, persistence infrastructure | ~10k | Zero allocation on per-tick path; static-class backends only |
| `Data/` | Unified data pipeline; every named, typed stream the mod produces | ~8k | If it produces a number, it lives here; consumers `Lookup<TSnapshot>(name)` |
| `Web/` | Local HTTP server + browser SPA + sharded HTML/CSS/JS (41 partials) | ~10k | Bind to 127.0.0.1 only; format snapshots into wire shapes, never derive |
| `UI/` | Archived in-game overlay (v0.9.0) — 5 tabs + 13 components + donut-via-`DrawUserPrimitives` + two-mode sizing | ~5.5k | Compiled but not loaded; preserved for future Steam Deck / handheld revival |

Total ~39,121 production C# LOC at HEAD `ff20711` across 264 files; ~1,342 test LOC across 10 xUnit fixtures.

**Hot-path execution flow** (one hook timing, end to end): `Main.Update` advances a tick → `MetricCollector.BeginTick()` opens a frame, reads `GC.GetAllocatedBytesForCurrentThread()` for alloc baseline → tModLoader's `*Loader.HookList<T>` iterates each profiled mod's overrides → each enters a method patched by one of the two backends → delegate path: `HookProbe.Time*` reads `Stopwatch.GetTimestamp()`, calls `orig(self, args)` inside `try/finally`, credits via `PerModAttribution.Add(modId, categoryId, hookId, deltaTicks)`; IL path: `ProbeStack.Enter(hookId)` prologue runs first, body inside a finally-protected region, every original `ret` rewritten to `stloc retLocal; leave end`, `ProbeStack.Leave()` runs as finally and credits via the same `PerModAttribution.Add` → `EndTick` reads exit alloc counter, assembles `TickFrame`, pushes into 1800-frame ring buffer, runs spike detector → `DataRegistry.PerTickCallbacks` (immutable array frozen at `InitialiseAll`) iterated by `ProfilerSystem.PostUpdateEverything`, zero virtual dispatch → `SessionRecorder` enqueues into the writer thread via `Channel.Writer.TryWrite` (measured 276 ns/op enqueue) → writer thread batches up to 64 ops per LiteDB pass, runs `db.Checkpoint()` every 60s (LiteDB issue #1568 mitigation) → HTTP worker thread (`TcpListener` accept loop) handles browser polls via `DashboardRouter.BuildXxx` → `DataRegistry.Shared.Lookup<TSnapshot>(name).CurrentSnapshot()`, race-free because snapshots are immutable values. No disk in the per-tick path.

## Subsystems and components

### Hook Instrumentation (`Profiling/`)
Two parallel backends behind `HookBackend.Mode`. `HookInterceptor` (delegate-pair, `MonoModHooks.Add` with ~30 distinct signature families in `HookInterceptor.cs:501-776`) and `ILHookInterceptor` (signature-agnostic `new ILHook(target, manipulator, applyByDefault: true)`). Shared identity via `HookCategoryRouter.ResolveCategory(Type)` (7 category ids) and `PerModAttribution.Add` accumulator. A `Parallel` mode runs both and logs `[backend-compare]` divergence to `client.log`. `ProbeStack` is a `static class` because IL-emitted callers use `call` (not `callvirt`) and the IL emit shape is load-bearing for hot-path cost. `_tmlAssembly` filter at `ILHookInterceptor.cs:328-331` guards the JIT shared-body trap where reference-type generic instantiations of `ModType<Projectile, ModProjectile>::NewInstance` and `ModType<Player, ModPlayer>::NewInstance` JIT-share a compiled body — patching one patched both and crashed tModLoader's player path with `InvalidCastException`. The `_instrumentedHandles` dedup set (keyed on `RuntimeMethodHandle`) prevents stacking the same closed-generic body twice.

### Data Pipeline (`Data/`)
`DataRegistry.Shared` singleton; `IDataStream` declares `Name`, `Cadence` (`PerTick` / `OneHz` / `OnEvent` / `OnDemand`), `Stage` (`Collector` / `Aggregator` / `Stat` / `Detector` / `Stream` / `Exporter`), `Initialise(SessionContext)`, `Reset()`, `Dispose()`, `CurrentSnapshotBoxed()`. Typed `IDataStream<TSnapshot>` adds `CurrentSnapshot()` returning a fresh immutable struct or record. `IHasPerTickCallback` marker on every `PerTick` stream; `DataRegistry.Freeze()` captures the per-tick callback array at `OnWorldLoad` so `ProfilerSystem.PostUpdateEverything` iterates with a `for` loop over the frozen array (zero virtual dispatch). Layout: `Collectors/` (4 — FrameTime, HookCpu, Allocation, ContextTagger), `Aggregators/` (13 — Heatmap, Segments, PerModAttribution + v0.12 F2/F3), `Stats/` (23 — KpiStat, EventsFeedStat + 17 v0.12 tab stats), `Detectors/` (SpikeDetector, StallDetector, Insights/ with 10 detectors), `Streams/` (14 persistence-facing writers).

### Insights Engine (`Profiling/Insights/`)
Ten detectors: four live (`HotHookDominanceDetector`, `AllocationBurstDetector`, `FreeRemovalCandidateDetector`, `PeakContributorToSpikeDetector`) plus six gated (`ContextCorrelatedSpike`, `ContextConditionalCost`, `GcPauseCulprit`, `SustainedCostShift`, `NewContributor`, `HookFrequencyTail`). Honesty contract encoded structurally: every record carries `Confidence` (statistical strength, 0-3), `EvidenceScope` (this-session / lifetime / needs-persistence), `BaselineKind` (what it compares against), rendered as separate badges so the player can argue with each dimension independently. `InsightStore.PromoteConfidence` gates Medium on `pAdjusted <= 0.10` and High on `pAdjusted <= 0.05`; repetition alone cannot promote a record whose detector declared no hypothesis test ran. `RankingScorer.NormaliseMagnitude` switches by `IsSharePattern(PatternKey)` — shares pass through `[0,1]`, ratios keep a soft-knee 10× curve — fixing the pre-fix bug where a 40% contributor and a 90% contributor ranked identically. `InsightsEngine.Shared` lazy singleton ensures the InsightsTab and `SessionLogWriter` consume the same store.

### Persistence (`Profiling/Persistence/`)
LiteDB 5.0.21 (MIT, single 510 KB managed DLL packed in `.tmod` via `dllReferences = LiteDB` + `lib/LiteDB.dll`) replacing the v0.2 JSON-per-session writer. Four-layer crash safety: LiteDB WAL (built-in `CHECKPOINT=1000` pragma) → `profiler.events.log` (append-only NDJSON redo log, one sequential append per op) → 3 rotating `.bak-{1,2,3}` files (one copy per clean shutdown) → quarantine + fresh start (`broken-<utc>` rename). Single writer thread (`DbWriterThread`) owns every LiteDB write exclusively; game thread enqueues via `System.Threading.Channels.Channel.CreateUnbounded` (single-reader, multi-writer); writer batches up to 64 ops per LiteDB pass. `Interlocked`-tracked `_approxQueueDepth` because `Channel.Reader.Count` is unsupported on .NET 8 / macOS unbounded variant. Per-stream extension via `IPersistenceStream` + `StreamRegistry`: each logical collection group is one file declaring its own `Kinds[]`, `Apply`, `Reconstruct`, `EnsureIndexes`; adding a new collection is one new file + one registry line (refactored from a 14-case switch in a 600-line `ProfilerDatabase.ApplyOne` that dropped to 431 lines).

### Browser Dashboard (`Web/`)
~250-line hand-rolled HTTP server on `TcpListener` (not `HttpListener` — Windows `HttpListener` requires admin rights or a one-time `netsh http add urlacl` for ordinary users to bind a port, which would break the F9-and-it-just-works promise). Loopback-only bind (127.0.0.1:27277-27287). `DashboardRouter.cs` + 7 per-tab partials. `Web/Assets/` sharded into 41 partial-class files (`IndexHtml.*`, `Css/Css.*`, `Js/Js.*`) replacing the pre-v0.11 1000+-line `DashboardAssets.Js.cs`. Six tabs: Now (live mission control, 30s frame-time chart, current segments, mod ranking with cost bars, recent-events feed), Mods (per-mod ranking sortable by current / session average / composite score), Timeline (every closed segment — biome visit, weather, boss, invasion, death-run, bookmark — with per-segment waterfall, lifetime delta badges, context-transition overlay, activity heatstrip, attendance roll-up, 30s pre-death replay strips, session chronicle), Lag (spike windows + stalls + per-mod breakdown at worst frame, fingerprint clustering, cause × context heatmap, GC pressure narrative, attribution-confidence visualisation, allocation→GC causality chain, rhythm detection), Insights (ranked insight cards from the engine, per-mod observatory cards, dormant content surface, attendance breakdown, loadout influence trace, cross-cutting aggregation, engagement-vs-cost scatter, interaction correlation matrix), Self (profiler measuring itself: install footprint, bytes/hook, severity bucket, projection to bigger modlists).

### Stall Detection
Classifier with `MainThreadFreeze` cause; stall clustering aggregates "what the player perceived as one freeze" from multiple stall events.

### Events and Context
Generic-surface watchers: biome bits, weather flags, invasions, hardmode, time-of-day, sub-worlds, boss-slot mask. `ContextTagger.Snapshot(tickIndex)` stamps `TickFrame.Context` per tick; `EventAggregator.Accumulate(in tagger.Current, frameMs)` updates per-dimension bucket stats. Vanilla-only on weather flags / invasions / game mode because tModLoader 1.4.4 has no `ModWeather` / `ModInvasion` / `Main.GameModeInfo` extension API yet; the data layer's iteration is already shaped to absorb future ID-space additions without code change.

### Interaction Trackers
Damage taken / dealt, NPC spawn, item created, loadout snapshot, buff lifecycle (`BoolIndex` O(1) bit-membership for buff diff at `Profiling/Util/BoolIndex.cs`). All driven through generic vanilla / tModLoader surfaces (`NPCLoader.OnSpawn`, `Player.OnHit*`, `Player.OnHurt`, `ItemLoader.OnCreated`, `PlayerDeathReason` struct, buff arrays, armour / accessory slots).

### Test Harness
Non-shipping xUnit project (`PerformanceProfiler.Tests.csproj`) using `Compile Include` + `Link` to pure-logic source so test code stays out of the shipped `.tmod`. Ten fixtures: RingBuffer, BoolIndex, Baseline, Time, Pools, RankingScorer, InsightStore, StallDetector, StallClassifier, persistence (`Tests/Persistence/PersistenceBenchmarkTests.cs`).

### Overlay Archive (`UI/`)
Archived v0.9.0. ~5,500 lines: 5 tabs, 13 components, donut chart via `GraphicsDevice.DrawUserPrimitives` (after two failed attempts at `SpriteBatch` rotation that painted screen-spanning cyan diagonals across the game world), two-mode 1120 / 720 px sizing. Compiled into `.tmod` but not loaded; kept on disk for future Steam Deck / handheld variant.

## Technologies and concepts demonstrated

### Languages
- **C# / .NET 8** — every production line. Per-file `#nullable enable`; file-scoped namespaces (C# 10+); no project-level nullable setting (deliberate, to surface the directive in every new-file diff).

### Frameworks and libraries
- **tModLoader 1.4.4** — the mod host; pinned. Builds against tModLoader reference assemblies; packs as `.tmod`.
- **MonoMod / Cecil** — runtime IL injection. `MonoModHooks.Add` for the delegate-pair backend; `new ILHook(target, manipulator, applyByDefault: true)` for the IL backend; manipulator rewrites every `ret` instruction in every override of every mod's types to wrap the body in a `try/finally` with `ProbeStack.Enter / Leave` prologue and epilogue.
- **LiteDB 5.0.21** — single-file embedded NoSQL DB (MIT, 510 KB managed DLL packed in `.tmod`). Deliberately not 6.0 (year-long prerelease).
- **xUnit** — test runner for the non-shipping `Tests/` project.
- **System.Threading.Channels** — `Channel.CreateUnbounded` (single-reader, multi-writer) for the lock-free producer queue feeding the LiteDB writer thread.

### Runtimes / engines / platforms
- **Terraria + tModLoader 1.4.4 client** — host runtime. Client-side mod; v1 is single-player (`GlobalNPC.OnKill` is single-player only, `GlobalNPC.OnSpawn` is server-side in multiplayer, engagement-correlated detectors assume `Main.LocalPlayer`).
- **Cross-platform .NET 8** — Windows / macOS / Linux. The loopback HTTP server, the `Interlocked`-tracked queue depth (`Channel.Reader.Count` is unsupported on .NET 8 / macOS), and the `Pragma("USER_VERSION")` BsonValue typing trap (returns `Int32` wrapped, not `Int64` — the cast `(int)(long)` threw on every DB open until the test caught it) are all platform-portability artefacts.

### Tools
- **MonoMod ILHook** — IL backend default since commit `b52f8b6`.
- **Stopwatch.GetTimestamp()** — static `long` reads only; `new Stopwatch()` would allocate per call.
- **xUnit benchmarks** — `Tests/Persistence/PersistenceBenchmarkTests.cs` asserts the 276 ns / op enqueue floor.
- **`MonoMod.RuntimeDetour.HookGen`-style IL manipulation** — via Cecil.

### Domains and concepts
- **Live IL injection in production C#** — `ILHookInterceptor.ApplyTimingWrap` rewrites every `ret` instruction in every override of every mod's type and wraps the original body in `try/finally`. The `[backend-compare]` divergence log confirms identical credit shape across both backends.
- **Zero-allocation hot path** — pre-allocated `PerModSample[]`, `Stopwatch.GetTimestamp()` static reads, pre-allocated context-tagger scratch fields, `BoolIndex` for O(1) buff diff, immutable per-tick callback array frozen at `OnWorldLoad`. Hot-path changes gated by the "unmeasured hot-path change is an incomplete change" rule (Invariant 2).
- **Multi-agent parallel development with locked snapshot contracts** — v0.12's 21-addition tab rework decomposed into 76 atomic tasks across 6 waves, 14 background agents in parallel, downstream agents compiling against contract types whose implementations did not yet exist. ~25 wall-clock hours for ~76 hours of sequential work, ~3× speedup. Contracts in `Data/Contracts/RolloutContracts.cs`; pearl-on-string parallelism.
- **Honesty-contracted analytics** — three orthogonal axes (`Confidence × EvidenceScope × BaselineKind`) rendered as separate badges; `PValueAdjusted`-gated promotion; pattern-aware magnitude normalisation (share vs ratio); `InsightsEngine.Shared` lazy singleton so dashboard and session JSON cannot disagree about what fired.
- **Production crash-safety stack** — LiteDB WAL → NDJSON redo log → 3 rotating backups → quarantine, single writer thread with `Interlocked` queue-depth tracking, BsonValue typing trap caught by tests before shipping.
- **Universal-surface profiling** — every detector / tracker / classifier operates on generic vanilla / tModLoader enumeration points (`IEntitySource` subclass name, `PlayerDeathReason`, buff arrays, armour / accessory slots, the tML `HookList<T>` iteration surface); never on a named mod's identifier. Profiling works on any combination of any tModLoader mods that exist now or in the future.
- **Lock-free producer / single-consumer queue** — `Channel.CreateUnbounded` for game thread → writer thread; 276 ns / op enqueue.
- **Hand-rolled loopback HTTP server on `TcpListener`** — ~250 lines of C#, deliberately not `HttpListener` because Windows requires admin / `netsh http add urlacl`. Loopback bind side-effects: zero firewall prompts (loopback bypasses macOS' application firewall), no telemetry surface, no remote-attack surface.
- **Statistical insight gating** — `pAdjusted` (multiple-comparison adjusted p-value) thresholds for Confidence promotion; pattern-shape-aware magnitude normalisation.
- **Spike + stall detection** — median + MAD outlier ticks for spikes; stall classifier with `MainThreadFreeze` cause; stall clustering aggregates "what the player perceived as one freeze".
- **Per-mod cost attribution** — `PerModAttribution[modId, categoryId, hookId]` table, 64-bit `StableKey` packing with 16-bit slots.

## Key technical decisions

**D1 — Dashboard-first, in-game overlay archived (v0.9.0, 2026-05-21).** Archived the entire in-game overlay system after five iterations all hit the same wall (Terraria's blocky bitmap font hard to read at small sizes, charts rough, click-targets drifting, tabs not fitting). Shipped a `TcpListener` + browser SPA instead — real typography, smooth charts, real CSS, no game-HUD overlap, ignorable when not needed. The ~5,500-line overlay tree is preserved on disk for future Steam Deck / handheld revival; the v0.10 audit recommended deletion and Caner deferred. Code enforcement: `build.txt` no longer references `UI/` for runtime mounting; `ProfilerOverlaySystem.DashboardKeybind` is the only live UI-namespace symbol.

**D2 — IL backend default, delegate backend kept as baseline (b52f8b6, 2026-05-20).** Original 2026-05-19 decision restricted attribution to MonoMod On-hooks (a fault is wrong numbers, never a crash); coverage gap then visible (delegate backend ~71.6% of overrides via ~30 signature families, IL backend signature-agnostic at ~100%). The safety reasoning still holds for the delegate path; IL path mitigates the same risk via `try/finally` shape in emitted IL and explicit `Uninstall()` from `Mod.Unload`. Cost: the JIT shared-body trap. Commit `7da4058` ("Hook closed-generic instantiations to cover mods built on generic bases") enabled broader coverage; commit `5725572` ("Fix world-load crash from hooking tModLoader-internal closed generics") added the `_tmlAssembly` filter one day later. **Two commits, one day apart, are the cautionary tale: enabling broader coverage without filtering by assembly was a one-day regression.**

**D3 — Honesty contract encoded structurally (aa914ce, 2026-05-20).** Three orthogonal axes (Confidence × EvidenceScope × BaselineKind) rendered as separate badges; promotion gated on `PValueAdjusted` (statistical evidence) AND `ConfirmationCount` (repetition); magnitude normalisation pattern-aware. Pre-fix, three honesty failures shipped: (1) `PromoteConfidence` did not gate on `PValueAdjusted`, so a record with `pAdjusted=1` could reach Medium just by re-firing three times; (2) `NormaliseMagnitude` collapsed every magnitude through the ratio curve, mapping 42% share to 0.42× ratio and clamping to 0 — most-impactful insight-engine bug because it erased the strongest signal the in-scope detectors produce; (3) `InsightsEngine.Shared` did not exist, so the InsightsTab and SessionLogWriter had separate stores. v0.10 audit also rewrote dashboard copy ("possibly removable" → "idle most of session", "clean session" → "no spikes or stalls observed in the last 30s").

**D4 — No mod-specific code, ever (Invariant 5, CLAUDE.md, 2026-05-20).** Every detector / tracker / classifier operates on generic vanilla / tModLoader surfaces. Mod-specific code is an immediate-revert offence regardless of how convenient for one playtest. Four-question test: hooks a generic surface? records interaction shape? leaves presentation / storage decision downstream? per-tick cost measured? All four must pass.

**D5 — LiteDB 5.0.21 (not 6.0 prerelease), single writer thread, four-layer crash safety (v0.3, 2026-05-20).** Replaced ~940 lines of `SessionLogWriter.cs` JSON-per-session with LiteDB + NDJSON redo log + 3 rotating backups + quarantine. Why not 6.0: year-long prerelease, cannot accept that risk on the persistence layer of a public mod. Why not SQLite: no managed binding cross-platform without P/Invoke shim. Two real bugs caught by the test wiring that would have shipped silently: `Pragma("USER_VERSION")` BsonValue typing trap (`Int32` wrapped, not `Int64` — `(int)(long)` threw on every DB open in production), and `Channel.Reader.Count` unsupported on .NET 8 / macOS unbounded variant.

**D6 — Modular extension via `IPersistenceStream` + `StreamRegistry` (mid-v0.3, 2026-05-20).** First cut had `ProfilerDatabase.ApplyOne` as a 14-case switch in a 605-line file; adding a tracker meant editing three places. Refactored to one file per logical collection group declaring its own `Kinds[]`, `Apply`, `Reconstruct`, `EnsureIndexes`; `ProfilerDatabase` dropped to 431 lines and now owns only cross-cutting concerns. Validation: adding three new collections in v0.4 (`stallClusters`, `playerDeaths`, `worldSnapshots`) required one record + one stream + one `DbWriteOp` factory each, no edit to writer thread / journal / dispatch / any other stream. Same shape applied to `Data/` in v0.10 — every numeric artefact registers via `DataRegistry.Shared` by name.

**D7 — Locked snapshot contracts for multi-agent parallelism (Wave 0, v0.12, 2026-05-21).** Before fanning out to 14 background agents for the v0.12 tab rework, froze every snapshot type that downstream agents would compile against in `Data/Contracts/RolloutContracts.cs`. Implementations come later; contracts come first. Downstream agents (Wave 2 data layer, Wave 3 UI layer) compile against the locked contracts, not against in-progress implementations. Result: ~25 wall-clock hours for ~76 sequential hours, ~3× speedup. The pattern is a structural form of "contracts first" applied within a single repo for short-lived multi-agent runs.

**D8 — Versioning discipline encoded in CLAUDE.md (v0.2, 2026-05-20).** Bumps at session-end per rule (patch = pure bug fix, minor = new feature / new tab / new detector / new schema / significant UI, major = first public Workshop or breaking change to agent-readable session JSON / public `Mod.Call` API). When unsure between patch and minor, prefer minor. **A mod sitting at `0.1` after months of work is a sign the discipline has slipped.** Validated across 12 minor + patch bumps from v0.1 → v0.12 in three days.

## What is currently built

- **148 commits across three days** (2026-05-19 → 2026-05-21), v0.12 at HEAD `ff20711`. ~39,121 production C# LOC across 264 files; ~1,342 test LOC across 10 xUnit fixtures.
- **Both hook backends shipping.** Delegate path (`HookInterceptor`, ~30 signature families covering ~71.6% of overrides); IL path (`ILHookInterceptor`, signature-agnostic, ~100%, default since b52f8b6). `Parallel` mode runs both and logs divergence.
- **17-stream `Data/` pipeline** with 4 collectors, 13 aggregators, 23 stats, 10 insight detectors (4 live + 6 gated), 14 persistence writers. Every numeric artefact registered with `DataRegistry.Shared`.
- **Six dashboard tabs in the browser SPA**, every tab post-v0.12 visualisation patch (narrative ribbons, sunburst attendance, lag galaxies, GC tide charts, allocation Sankeys, polar rhythm plots, DNA-strand mod cards, chord diagrams, dust-shelf dormant rows).
- **LiteDB persistence with four-layer crash safety**, single writer thread (`DbWriterThread`), lock-free producer queue (276 ns / op enqueue), 64-op batches, 60s checkpoint cadence.
- **xUnit harness** with 10 fixtures including a persistence benchmark fixture asserting sanity floors; caught two production-crash bugs (BsonValue typing trap, `Channel.Reader.Count` unsupported) before they shipped.
- **Stall classifier** (MainThreadFreeze cause) with stall clustering aggregating multiple stall events into "what the player perceived as one freeze".
- **Interaction tracking arsenal**: damage taken / dealt, NPC spawn, item created, loadout snapshot, buff lifecycle.
- **Hand-rolled ~250-line loopback HTTP server** on `TcpListener` (not `HttpListener`), bound to 127.0.0.1:27277-27287.
- **Sharded web assets**: 41 partial-class files under `Web/Assets/` replacing the pre-v0.11 1000+-line `DashboardAssets.Js.cs`.
- **Archived in-game overlay** (~5,500 lines, 5 tabs, 13 components) compiled but not loaded, preserved for future Steam Deck / handheld revival.

## Current state

Status: **active**. HEAD `ff20711` on 2026-05-21 22:41 +0100 ("v0.12 wrap: waves 4-5 — visualisation patch + docs + version bump"); last meaningful commit 1 day before the most recent vault verification. 30 commits in the 48 hours preceding the v0.12 wrap (the v0.12 tab rework burst). In flight: v1.0 (first public Workshop release) — pending `description.txt` rewrite (still reads "Milestone 0 — hello-world scaffold"), first-launch UX, screenshots / GIFs, full Workshop description, the publish flow itself. Public GitHub repo at `Capataina/TerrariaPerformanceProfilerMod`, fully synced as of 2026-05-25.

## Gaps and known limitations

- **`description.txt` is stale** — still reads "Status: early development (Milestone 0 — hello-world scaffold)" while the mod is at v0.12 with 264 files. Ships in the `.tmod`; v1.0 prerequisite.
- **G1 — JIT shared-body trap is mitigated, not eliminated.** The `_tmlAssembly` filter at `ILHookInterceptor.cs:328-331` guards the known case. A new closed-generic inheritance scenario (a mod base class with non-tModLoader generic parents) could re-introduce the failure. Detectable and recoverable via Invariant 4 abort-clean, but a public Workshop user would see a one-time crash before abort-clean fires.
- **G2 — Backend divergence is logged, not surfaced.** `Parallel` mode emits `[backend-compare] delegate=… ilhook=… Δ=…` to `client.log`. A user without log access never sees it.
- **G3 — Live detector p-values default to 1.** None of the four live detectors runs a hypothesis test; they emit records with `Evidence.PValueAdjusted = 1` and rely on magnitude + repetition. **Consequence: no live detector's records reach Medium or High confidence today.** The honesty contract is intact (untested observations stay at Low/Preliminary), but the InsightsTab shows mostly Low-confidence rows.
- **G4 — Gated detector emit is fully disabled.** Six gated detectors are registered for roster / gate visibility but `Evaluate` short-circuits; they emit zero records.
- **G5 — `_topComparerNowTick` is shared scalar state.** If two callers ever invoke `InsightStore.TopInto` concurrently on the same store, the comparer reads whichever `nowTick` was written last. Single-caller today.
- **G6 — `StableKey` packs into 64 bits with 16-bit slots.** Collisions mathematically possible if a single mod had > 65,535 distinct hookIds; today the largest discovered hook counts are in the hundreds per mod.
- **G7 — `_installedHooks` / `_instrumentedHandles` are process-scoped static lists.** If `Mod.Unload` ever stopped firing, the `Installed` flag's `if (Installed) return;` short-circuit would leave the next session with no instrumentation.
- **G8 — OS focus-pause caveat is unfixable by design.** Terraria pauses simulation the moment the window stops being focused; if the player clicks into their browser, the dashboard freezes because the game stopped ticking. Documented in README; workarounds are side-by-side without clicking, second monitor, Host & Play multiplayer. Cannot fix without modifying Terraria's internals (Invariant 1 forbids).
- **Multiplayer coverage gap.** v1 is single-player. `GlobalNPC.OnKill` is single-player only; `GlobalNPC.OnSpawn` is server-side in multiplayer; engagement-correlated detectors assume `Main.LocalPlayer`. Instrumentation surface does not change; coverage interpretation does. Deferred to v2.
- **Per-stream honest limitations declared in code** — each Wave 2 stream's class doc-comment names data it cannot yet emit: lag clusters lack per-event `EventContext`; `PerModUsageAggregator` (F2) has no per-biome breakdown; `ModObservatoryStat`'s biome attendance is per-mod aggregate not per-mod-per-biome; death replay lacks biome-at-death-time.
- **C13 stale.** The repo's `context/notes/conventions.md` claims "no `[MethodImpl(MethodImplOptions.AggressiveInlining)]` is used anywhere in the codebase" — stale as of v0.6.1 commit `a325b37` which added the attribute to four `LangNameCache` lookup methods.

## Direction (in-flight, not wishlist)

- **v1.0 first public tModLoader Workshop release.** Blocks: `description.txt` rewrite (currently still says M0 hello-world), first-launch UX (the F9 hint chat line is the only discovery mechanism — a one-shot "open the dashboard once" tutorial would land here in `ProfilerPlayer.OnEnterWorld`), screenshots + GIFs (repo lacks `assets/`), full Workshop description, Steam Workshop publishing flow + appid.
- **Real p-value computations for the four live detectors.** Plan in `context/notes/insights-engine-plan.md` §6. Not started.
- **Opening the events-gated detectors** (`ContextCorrelatedSpike`, `ContextConditionalCost`, `GcPauseCulprit`, `HookFrequencyTail`) — closest to ready because `EventAggregator` already accumulates per-dimension bucket stats; only the transition stream is missing.
- **Cross-session comparison views.** Lifetime aggregates already persist (`tickAggregatesArchive`, `perSessionModAggregates`, `perSessionHookAggregates`); a dashboard tab surfacing them is the next user-visible step.
- **Post-session HTML report.** Design notes in `context/notes/future-html-report.md`; schema already friendly to a future reader.

## Demonstrated skills

- **Live IL injection in production C#** via MonoMod / Cecil — rewriting every `ret` in every override in every mod's types, wrapping in `try/finally` with prologue / epilogue, dispatching to a static `ProbeStack` (because `call` not `callvirt` keeps the IL emit shape tight). Includes diagnosing and fixing the JIT shared-body trap one day after enabling broader closed-generic coverage.
- **Zero-allocation hot path** verified by hand: pre-allocated arrays sized at `PostSetupContent`, `Stopwatch.GetTimestamp()` static reads, `BoolIndex` for O(1) buff diff, immutable per-tick callback array frozen at `OnWorldLoad`, snapshots returned as immutable values to keep the dashboard race-free.
- **Multi-agent parallel development under locked snapshot contracts** — decomposing a 21-feature surface into 76 atomic tasks across 6 waves, fanning out to 14 background agents, achieving ~3× wall-clock speedup. The contract file becomes the synchronisation point; downstream waves compile against shapes whose implementations do not yet exist.
- **Production crash-safety engineering** — four-layer stack (LiteDB WAL → NDJSON journal → 3 rotating backups → quarantine), single writer thread with `Interlocked` queue depth (because the standard channel API is unsupported on the target platform), tests catching two crash-on-launch bugs before shipping.
- **Lock-free producer / single-consumer queue design** — `Channel.CreateUnbounded` for game thread → writer thread; 276 ns / op enqueue measured.
- **Hand-rolled cross-platform HTTP server** — ~250 lines of C# on raw `TcpListener` (deliberately avoiding `HttpListener` due to Windows admin / `netsh` requirement), loopback-bound for zero firewall prompts / zero remote attack surface.
- **Honesty-contracted analytics design** — three orthogonal axes (Confidence × EvidenceScope × BaselineKind) rendered as separate badges, statistical gating on `pAdjusted`, pattern-aware magnitude normalisation, audit-driven dashboard-copy rewrites from normative to descriptive.
- **Universal-surface profiling design** — every detector / tracker / classifier operates on generic vanilla / tModLoader enumeration points; works on any combination of any tModLoader mods that exist now or in the future. Includes the four-question test for new trackers and the explicit code-level enforcement of "no mod-specific case statements anywhere".
- **C# / .NET 8 platform-portability discipline** — diagnosing and fixing the `BsonValue` `Int32`-not-`Int64` typing trap and the `Channel.Reader.Count` unsupported-on-macOS issue before they shipped.
- **Modular subsystem design via single-file extension points** — `IPersistenceStream` + `StreamRegistry` (one file + one registry line per new collection), `IDataStream` + `DataRegistry.Shared` (one class + one register call per new stream). Validated by adding 3 new collections in v0.4 without editing writer thread / journal / dispatch / any other stream, and by adding 17 new streams in v0.12 without modifying any of the 14 pre-existing persistence streams.
- **Visibility-as-policy** — tightening `ProfilerSystem.Collector` to `internal` so cross-assembly access becomes a compile error rather than a convention, encoding the "consumers route through immutable snapshots, period" rule in the language rather than relying on code review.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Performance Profiler/_Overview.md | 177 | "3. **The depth of the honesty contract.** Invariant 3 is not a single-sentence \"be descriptive\" rule. It threads through code-level enforcement: `Confidence × EvidenceScope` orthogonality (a record can be statistically tight and still weaker than lifetime data; the UI shows both); `PValueAdjusted`-gated promotion (a record with `pAdjusted=1` can never reach Medium by repetition alone — repetition is not statistical evidence); pattern-aware magnitude normalisation (share vs ratio); dashboard copy rewrites from the v0.10 audit (\"possibly removable\" → \"idle most of session\"); per-detector `BaselineKind` declaration so a reader can argue with the comparison itself. The principle is encoded structurally, not exhortatively." |
| Projects/Performance Profiler/Architecture.md | 200 | "- `context/integration/integration-map.md` — the per-component plug-in points (not duplicated here)" |
| Projects/Performance Profiler/Code Health Audits.md | 185 | "- `context/notes/decisions.md` — 2026-05-21 entry for pass 2" |
| Projects/Performance Profiler/Conventions.md | 141 | "- `context/notes/conventions.md` — in-repo source (slightly stale on C13)" |
| Projects/Performance Profiler/Decisions.md | 167 | "- `context/notes/decisions.md` — daily session log (309 lines); this note is the durable shape" |
| Projects/Performance Profiler/Gaps.md | 137 | "- `context/_staleness-report.md` — per-file staleness verdicts from the 2026-05-20 upkeep" |
| Projects/Performance Profiler/Multi-Agent Patterns.md | 148 | "- Auto-memory: `feedback_subagent_temp_file_briefs.md` — when dispatching parallel subagents from any skill, write per-subagent briefs to `/tmp/<run-id>/<subagent>.md`; inline-prompted briefs force compression and produce summary-shaped fixes" |
| Projects/Performance Profiler/Performance Pass.md | 169 | "- `context/perf-pass/verification.md` — comprehensive final state" |
| Projects/Performance Profiler/Philosophy.md | 147 | "- Original ~1,100-line design pitch — deleted in commit `cf67dd6` after promotion; recoverable via `git show cf67dd6~1:'Projects/Potential Projects/Modded Terraria Profiler.md'`" |
| Projects/Performance Profiler/Roadmap.md | 103 | "- `context/plans/code-health-audit/index.md` — full audit implementation receipts" |
| Projects/Performance Profiler/Systems/Browser Dashboard.md | 197 | "- `README.md` \"Why no in-game UI?\" + \"Why a local HTTP server?\" sections" |
| Projects/Performance Profiler/Systems/Data Pipeline.md | 208 | "- `context/systems/data-pipeline.md` — in-repo canonical" |
| Projects/Performance Profiler/Systems/Events and Context.md | 161 | "- `context/tmodloader/engagement-surfaces.md` — per-API plug-in slice" |
| Projects/Performance Profiler/Systems/Hook Instrumentation.md | 176 | "- `context/tmodloader/ilhook-migration-research.md` — the original research on the IL backend approach (793 lines)" |
| Projects/Performance Profiler/Systems/Insights Engine.md | 183 | "- `context/systems/insights-engine.md` — in-repo canonical" |
| Projects/Performance Profiler/Systems/Interaction Trackers.md | 147 | "- `context/notes/decisions.md` v0.5 entry — full tracker arsenal record" |
| Projects/Performance Profiler/Systems/Metric Collection and Spike Detection.md | 155 | "- `context/systems/spike-detection.md` — in-repo canonical" |
| Projects/Performance Profiler/Systems/Overlay Archive.md | 183 | "- Auto-memory: `spritebatch-rotation-trap.md` — the donut chart attempt-1 lesson" |
| Projects/Performance Profiler/Systems/Persistence.md | 198 | "- `context/systems/persistence.md` — in-repo canonical" |
| Projects/Performance Profiler/Systems/Stall Detection.md | 115 | "- `context/notes/decisions.md` — v0.4 / v0.5 / v0.6 stall classifier entries" |
| Projects/Performance Profiler/Systems/Test Harness.md | 128 | "- `context/plans/code-health-audit/build-and-tests.md` — audit deep-dive that recommended the harness" |
