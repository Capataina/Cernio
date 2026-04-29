---
name: Nyquestro
status: active
source_repo: https://github.com/Capataina/Nyquestro
lifeos_folder: Projects/Nyquestro
last_synced: 2026-04-29
sources_read: 15
---

# Nyquestro

## One-line summary

From-scratch limit order book matching engine in safe Rust — currently a typed primitive layer (allocation-free `Px`/`Qty`/`Ts`/`OrderID`/`Side`/`Status`), an immutable `Copy`-based event frame system (`FillEvent`/`QuoteEvent`/`OrderEvent`), an 11-variant error model with severity classification, and a basic order/price-level container, with the matching engine itself unbuilt — exploring lock-free order-book design and wait-free concurrent data structures.

## What it is

Nyquestro is a from-scratch order matching engine written in safe Rust. The stated ambition is a production-grade system covering lock-free order books, binary wire protocols, real-time risk controls, a market-making strategy agent, and rigorous benchmarking. The current reality is significantly earlier: the project has implemented core domain primitives, an event frame system, a basic order model, and an error taxonomy — but has no matching engine, no protocol layer, no risk system, and no strategy agent. The README presents a comprehensive, feature-complete exchange system; the code implements only the foundational type system and event model. Source code has not changed since December 2025; the last 3 months have been dominated by documentation passes (context docs, hardening plan, README revision). This is a learning project building methodically from primitives upward — the documentation-to-code ratio is ~2.6:1 (32.5KB context + 26.3KB README vs ~23KB Rust source), which is the project's current shape rather than a pathology.

## Architecture

Single Rust crate (edition 2024). Public surface flows through `lib.rs`. External dependencies: `chrono 0.4.41`, `thiserror`. ~49KB of Rust source across 13 files.

```
src/
├── lib.rs                  # Public surface
├── types.rs                # 6 primitive types (allocation-free, Copy)
├── errors.rs               # 11-variant error taxonomy + ErrorSeverity
├── order.rs                # Order struct (fill mechanics, status transitions)
├── price_level.rs          # PriceLevel container (Vec<Order>)
├── events/
│   ├── mod.rs
│   ├── fill_event.rs       # FillEvent (immutable, Copy)
│   ├── quote_event.rs      # QuoteEvent (immutable, Copy)
│   └── order_event.rs      # OrderEvent (immutable, Copy — partial validation)
└── matching_engine/        # Empty placeholder; not compiled into crate

tests/
├── event_tests.rs          # 732 lines
└── types_test.rs           # 122 lines
```

The dependency direction is acyclic. Primitives in `types.rs` are pure data with bounds-checking constructors. Events embed primitives and validate at construction time. `Order` and `PriceLevel` consume events. The matching engine would consume orders + price levels + emit events, but does not exist yet.

## Subsystems and components

| Subsystem | Responsibility | Key files |
|---|---|---|
| **Core Types** | `OrderID`, `Side`, `Px`, `Qty`, `Ts`, `Status` — primitives, invariants, validation | `types.rs` |
| **Error Model** | `NyquestroError` (11 variants), `ErrorSeverity` enum, severity classification function | `errors.rs` |
| **Event System** | `FillEvent` / `QuoteEvent` / `OrderEvent` — immutable frames, validation, Copy semantics | `events/` |
| **Order Model** | `Order` struct with fill mechanics + state transitions; over-fill silently clamps to zero (a known issue); `get_status()` takes `self` by value | `order.rs` |
| **Price Level** | Basic `Vec<Order>` container; no removal, no FIFO contract, clones on storage | `price_level.rs` |
| **Matching Engine** | Empty placeholder; not yet wired into the crate | `matching_engine/` |

## Technologies and concepts demonstrated

### Languages
- **Rust 2024** — entire codebase, ~49KB across 13 files. Allocation-free primitives, Copy semantics for events, `&'static str` and `NyquestroError` mixed in error returns (a known mixed-error-return inconsistency).

### Libraries
- **chrono 0.4.41** — `Ts` timestamp wrapper for nanosecond precision.
- **thiserror** — declarative error enum.

### Domains and concepts
- **Market microstructure** — limit order book design, price-time priority intent, fill mechanics, state transitions for order lifecycle.
- **Lock-free / wait-free concurrent data structures** — design intent (per README and Roadmap) is a lock-free order book; current implementation is single-threaded but designed with the lock-free target in mind.
- **Allocation-free / Copy-based design** — primitives are `Copy`; events are immutable `Copy` frames; zero allocations on the hot path is the design target.
- **Domain-driven type design** — `Px`, `Qty`, `Ts`, `OrderID`, `Side`, `Status` as distinct types instead of primitive obsession; bounds-checking constructors enforce invariants.
- **Hardening-first methodology** — explicit hardening plan in `IMPLEMENT_NOW_CORE_HARDEN_BOOK_MVP.md` lists every known issue (over-fill clamp, `get_status` self-by-value, mixed error returns, partial event validation) before any matching-engine implementation.

## Key technical decisions

- **Foundational primitives before matching engine.** The implementation order is primitives → events → errors → hardening → matching → protocol → risk → strategy. The README describes the end state; the code is at "events, errors, basic order/price-level, hardening pending".
- **Copy-based event frames.** All events are `Copy`. Immutable. Validation at construction.
- **Allocation-free primitive types.** No heap allocation on the hot path is the design target.
- **Defer decisions until forced by implementation.** From `Decisions.md`: "the project consistently defers decisions until they are forced by implementation".
- **Documentation-first iteration.** Context docs (32.5KB) + README (26.3KB) outweigh source (~23KB) by ~2.6×. The hardening plan exists before the matching engine.
- **`Result<T, NyquestroError>` as primary return type** for fallible operations; `&'static str` retained in some primitive constructors as an inconsistency to clean up in hardening.
- **`OrderEvent::new()` validates nothing currently** — the self-match check is commented out. Flagged as a hardening gap.

## What is currently built

- 13 Rust source files, ~49KB. 22 commits Jun 2025–Mar 2026.
- 6 primitive types in `types.rs` — allocation-free, Copy-friendly, mixed error returns.
- 11-variant error taxonomy in `errors.rs` with `ErrorSeverity` enum and severity classification function.
- 3 event types (FillEvent, QuoteEvent, OrderEvent), all `Copy`, immutable.
- `Order` struct with fill and status transitions (over-fill silently clamps to zero — known issue).
- `PriceLevel` as basic `Vec<Order>` container (no removal, no FIFO contract, clones on storage).
- 2 integration test files (`event_tests.rs` 732 lines, `types_test.rs` 122 lines).
- 4 unit test modules (errors, fill_event, order_event, quote_event).

## Current state

Active by category, but **source has not changed since December 2025**. The last 3 months have been documentation passes (context docs, hardening plan, README revision). HEAD has not moved in 9 days at LifeOS verification. The next concrete step is the OrderBook MVP described in `Context/IMPLEMENT_NOW_CORE_HARDEN_BOOK_MVP.md` — but every gap identified in the hardening plan is a prerequisite. The dependency chain is **hardening → OrderBook MVP → all further features**.

## Gaps and known limitations

- **No matching engine** — `matching_engine/` is empty and not compiled.
- **No wire protocol layer.**
- **No risk layer.**
- **No strategy agent.**
- **No benchmarking** — no criterion suite, no perf harness.
- **Over-fill silently clamps to zero in `Order::fill`** — should error.
- **`get_status()` takes `self` by value** — forces clones; should take `&self`.
- **Mixed error returns** (`&'static str` vs `NyquestroError`) across primitive constructors.
- **`OrderEvent::new()` validates nothing** — self-match check commented out.
- **`PriceLevel` clones orders on storage; no FIFO contract.**
- **README describes ~50+ features across 6 major categories**; current implementation is the foundational type+event layer.

## Direction (in-flight, not wishlist)

- **Hardening** (the active in-flight workstream): finalise the 11 hardening-plan items. Tighten error returns, fix over-fill, remove `self`-by-value, add self-match validation, define FIFO contract for PriceLevel.
- **OrderBook MVP** — after hardening lands. Single-threaded, deterministic, price-time priority.
- **Subsequent**: lock-free OrderBook, wire protocol (binary), risk layer, strategy agent, benchmarking suite.
- **Work files** capture two future planned extensions: HFT Observability Dashboard (additive) and V2 Distributed Extension (consensus-based distributed extension for v2).

## Demonstrated skills

- **Domain-driven type design in Rust** — `Px`, `Qty`, `Ts`, `OrderID`, `Side`, `Status` as distinct types with bounds-checking constructors; primitive obsession avoided.
- **Allocation-free / Copy-based event frame design** — immutable event frames, `Copy` semantics, zero-allocation hot-path design target.
- **Error model design** — 11-variant error taxonomy with `ErrorSeverity` classification; `Result<T, NyquestroError>` as primary return type.
- **Documentation-first methodology** — hardening plan written before matching engine; context docs (32.5KB) outweighs source (~23KB).
- **Lock-free / market-microstructure design awareness** — design intent (lock-free OrderBook, wait-free queues, price-time priority) demonstrated through architecture documents even before implementation lands.
- **Methodical incremental build discipline** — the project's roadmap is the implementation order: primitives → events → errors → hardening → matching → protocol → risk → strategy.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Nyquestro/_Overview.md | 80 | "The README is written as a portfolio piece describing a comprehensive exchange system. The code is a careful, incremental build starting from primitives up. This is not a disconnect — it is a roadmap expressed as prose. The implementation approach (primitives → events → errors → hardening → matching → protocol → risk → strategy) is methodical and sound. The risk is that the aspirational README creates expectations the code cannot yet satisfy." |
| Projects/Nyquestro/Architecture.md | 131 | "> Context docs (32.5KB) + README (26.3KB) = ~59KB of documentation vs ~23KB of source code." |
| Projects/Nyquestro/Decisions.md | 110 | "> The project consistently defers decisions until they are forced by implementation." |
| Projects/Nyquestro/Evolution.md | 114 | "Source code has not changed since December 2025. The last 3 months have been documentation passes." |
| Projects/Nyquestro/Gaps.md | 147 | "> Most divergences are expected — the README describes the end state, not the current state." |
| Projects/Nyquestro/Roadmap.md | 148 | "> The README describes ~50+ features across 6 major categories. Current implementation is the foundational type+event layer." |
| Projects/Nyquestro/Testing.md | 97 | "\| Determinism (OrderBook) \| `matching_engine/` \| Same inputs → identical outputs (planned) \|" |
| Projects/Nyquestro/Systems/_Overview.md | 42 | "- [[Projects/Nyquestro/Roadmap]] — direction-of-travel" |
| Projects/Nyquestro/Systems/Core Types.md | 141 | "4. Should `Ts::now()` return a recoverable error instead of panicking?" |
| Projects/Nyquestro/Systems/Error Model.md | 105 | "4. Should there be a single severity entrypoint or is the free function acceptable?" |
| Projects/Nyquestro/Systems/Event System.md | 140 | "This means the event system as currently designed is sufficient for the MVP. The matching engine will be the consumer." |
| Projects/Nyquestro/Systems/Matching Engine.md | 136 | "The dependency chain is: **hardening → OrderBook MVP → all further features**." |
| Projects/Nyquestro/Systems/Order Model.md | 149 | "These are all identified in the hardening plan. The matching engine implementation can begin once hardening completes." |
| Projects/Nyquestro/Work/HFT Observability Dashboard.md | 85 | "#nyquestro #work #observability #hft #additive" |
| Projects/Nyquestro/Work/V2 Distributed Extension.md | 67 | "#nyquestro #work #distributed-systems #consensus" |
