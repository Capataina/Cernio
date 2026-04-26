---
name: Nyquestro
status: active
source_repo: https://github.com/Capataina/Nyquestro
lifeos_folder: Projects/Nyquestro
last_synced: 2026-04-26
sources_read: 13
---

# Nyquestro

## One-line summary

A from-scratch order matching engine in safe Rust (edition 2024) whose current implemented surface is a strongly-typed domain-primitive layer, an immutable Copy-event frame system, a classified error taxonomy, and a sketched order/price-level model — with the matching engine itself documented in detail but not yet implemented.

## What it is

Nyquestro is a single-author Rust library crate (`nyquestro`) plus a demo binary and integration tests, designed as the foundation for an exchange-style limit order book. The stated ambition (in the README and roadmap) is a production-grade system that would cover lock-free order books, binary wire protocols, real-time risk controls, a market-making strategy agent, and rigorous benchmarking. The currently-built reality is significantly earlier: core domain primitives, an event frame system, a basic order/price-level model, and an error taxonomy — but no matching engine, no protocol layer, no risk system, and no strategy agent. The project's stated discipline is "Safe Rust Only" (no `unsafe`) and "correctness before performance" — get deterministic semantics right with standard-library containers first, optimise later. LifeOS treats the project as an "exchange-infrastructure" domain effort that is intentionally documentation-heavy in its foundational phase, with a detailed `IMPLEMENT_NOW_CORE_HARDEN_BOOK_MVP.md` plan acting as the next-step specification.

## Architecture

Nyquestro is a Rust library crate with a demo binary and integration tests. The Cargo.toml is minimal — Rust edition 2024, dependencies on `chrono 0.4.41` and `thiserror`. There are 13 Rust source files (~49 KB), 2 integration test files (`event_tests.rs` 732 lines, `types_test.rs` 122 lines), and unit test modules inside `errors.rs` and the three event modules.

The module dependency graph (verified against `lib.rs`):

```
                    ┌──────────┐
                    │  types   │  ← OrderID, Side, Px, Qty, Ts, Status
                    └────┬─────┘
                         │
              ┌──────────┼──────────┐
              │          │          │
         ┌────▼───┐  ┌──▼───┐  ┌──▼─────────┐
         │ errors │  │order │  │price_level │
         └────┬───┘  └──┬───┘  └──┬─────────┘
              │         │         │
              ▼         ▼         ▼
         ┌─────────────────┐
         │     events/     │  FillEvent, QuoteEvent, OrderEvent
         └─────────────────┘

         ┌─────────────────┐
         │matching_engine/ │  ← EMPTY (order_book.rs is 0 bytes,
         │  (not compiled) │     not declared in lib.rs)
         └─────────────────┘
```

`src/lib.rs` declares five modules (`errors`, `events`, `order`, `price_level`, `types`) and re-exports `errors::*` at crate root. The `matching_engine/` directory exists physically but is deliberately not declared in `lib.rs` and is not compiled — the placeholder is acknowledged in the LifeOS Architecture and Matching Engine notes.

Dependency direction is strict and explicitly enforced (per LifeOS Architecture):

| Module | Depends on | Must NOT depend on |
|---|---|---|
| `types` | `std::time`, `chrono` | `order`, `events`, `matching_engine`, `errors` |
| `errors` | `thiserror` | `order`, `events`, `matching_engine`, `types` |
| `order` | `types`, `errors` | `events`, `matching_engine` |
| `price_level` | `types`, `errors`, `order` | `events`, `matching_engine` |
| `events` | `types`, `errors` | `order`, `price_level`, `matching_engine` |

The only real execution path today is the demo binary (`src/main.rs`) — it constructs an `Order` via the typed primitives, calls `Order::fill()` twice (a partial then a full fill) and prints the result. The planned data flow (per `IMPLEMENT_NOW_CORE_HARDEN_BOOK_MVP.md`) is `OrderBook::submit_limit(order) → evaluate marketability → select next resting (best price, FIFO within level) → apply fill via Order::fill() → remove fully filled → rest remainder → return SubmitResult { fills: Vec<FillEvent>, quotes: Vec<QuoteEvent> }`.

The repository is documentation-heavy: `context/` (32.5 KB) plus README (26.3 KB) total ~59 KB of documentation against ~23 KB of source code — a deliberate planning-first posture in the foundational phase.

## Subsystems and components

### Core primitives (`src/types.rs`)

Six strongly-typed domain primitives, all `Copy`-friendly and allocation-free, designed for use on the hot path:

| Type | Inner | Validation | Constructor error type |
|---|---|---|---|
| `OrderID` | `u64` | Rejects zero | `&'static str` |
| `Side` | enum (`Buy`, `Sell`) with `opposite()` helper | N/A | N/A |
| `Px` | `u64` cents | Rejects zero/negative | `&'static str` |
| `Qty` | `u32` | None — allows zero (validated by consumers) | N/A |
| `Ts` | `u64` nanoseconds since UNIX epoch | N/A | N/A |
| `Status` | enum (`Open`, `PartiallyFilled`, `FullyFilled`, `Cancelled`) | N/A | N/A |

`Px` has two constructors — `new_from_dollars(f64)` (truncates `(dollar_price * 100.0) as u64`, so `$10.999` → 1099 cents not 1100) and `new_from_cents(u64)` (clean integer path). `Ts::now()` reads `SystemTime::now()` and panics via `.unwrap()` on clock error; `Ts::from_nanos(u64)` is the deterministic constructor used in tests. `Qty::saturating_sub(other)` clamps to `Qty(0)` instead of panicking — this is what enables the silent over-fill behaviour in `Order::fill()`.

### Error model (`src/errors.rs`)

Built on `thiserror`. The `NyquestroError` enum has 11 variants split between recoverable (input-validation failures) and fatal (state errors). A two-variant `ErrorSeverity` enum (`Recoverable`, `Fatal`) and a free `severity(&NyquestroError) -> ErrorSeverity` function provide classification. The type alias `NyquestroResult<T> = Result<T, NyquestroError>` is used by `Order`, the events, and `PriceLevel` — but **not** by `OrderID::new()`, `Px::new_from_dollars()`, or `Px::new_from_cents()`, which still return `Result<_, &'static str>`, leaving two error models coexisting in the crate.

The error enum has acknowledged structural redundancy: `RecoverableError`, `FatalError`, `ErrorSeverity { severity: &'static str }`, and `ErrorSeverityCannotBeDetermined` are generic catch-alls overlapping with the `ErrorSeverity` classification. The hardening plan flags these for retirement once consumers are migrated. The expansion from 4 to 11 variants happened in the December 2025 burst (commit `43178dc`) and is described in LifeOS as "designing for anticipated rather than actual needs".

### Event frames (`src/events/`)

Three immutable `Copy` event types that define the future matching engine's output contract:

- **FillEvent** — `buyer_order_id, seller_order_id, price, quantity, timestamp`. Constructor rejects zero quantity. Self-match check (`buyer_order_id == seller_order_id`) is **commented out** in the source — disabled pending policy decision.
- **QuoteEvent** — `price, quantity, side, timestamp`. Constructor rejects zero quantity. Per the hardening plan, future emission rule is "only when best bid or best ask changes in either price or displayed quantity at the top level" (avoid quote spam).
- **OrderEvent** — enum with `New`, `Cancelled`, `Rejected` variants (Rejected carries an `OrderRejectionReason`). `OrderEvent::new()` validates **nothing** and unconditionally returns `Ok` — inconsistent with `FillEvent::new()` and `QuoteEvent::new()`, which validate quantity.

`OrderRejectionReason` is a 7-variant enum (`InvalidQuantity`, `InvalidPrice`, `InvalidSide`, `InvalidTimestamp`, `InvalidOrderID`, `InvalidOrderStatus`, `InvalidOrderType`) — kept as an enum rather than a `String` reason field because event types must remain `Copy`.

All event fields are `pub` alongside getter methods (redundant accessor pattern). Constructors return `NyquestroResult` (except `OrderEvent::new()` which never errors).

### Order model (`src/order.rs`)

A mutable `Order` struct with `order_id, side, price, quantity, remaining_quantity, timestamp, status` — all private, accessed through getters. `Order::new()` validates non-zero quantity and reads the clock internally via `Ts::now()`. `Order::fill(&mut self, fill_amount: Qty) -> NyquestroResult<()>` calls `self.remaining_quantity = self.remaining_quantity.saturating_sub(fill_amount)` then `self.update_status()?`.

Two correctness issues are documented as critical:

- **Silent over-fill**: there is no validation that `fill_amount <= remaining_quantity`. An order with 3 remaining can be `fill(Qty::new(100))`-ed and silently becomes `FullyFilled` with no error. This is named in LifeOS as "the single most important correctness issue in the current codebase".
- **`get_status(self)` consumes the order**: the accessor takes `self` by value, not `&self`. The demo binary clones the order six times to read the status. The hardening plan adds a non-moving `status(&self) -> Status` accessor.

`update_status()` recomputes status from quantity comparison without guarding against backward transitions; the hardening plan requires `Open → PartiallyFilled → FullyFilled` to be one-way.

### PriceLevel (`src/price_level.rs`)

A container grouping orders at the same price: `price: Px`, `orders: Vec<Order>`, `total_quantity: Qty`. `add_order` validates that the order's price matches the level's price, then `clones` the order onto the Vec and increments `total_quantity`. There is **no remove operation** — fully filled orders cannot be evicted, `total_quantity` cannot decrement, and `get_orders()` clones the entire `Vec<Order>`. There is no explicit FIFO contract (insertion-order Vec gives implicit FIFO but it is not invariant-enforced).

### Matching engine (`src/matching_engine/`)

`order_book.rs` exists as a 0-byte file; the `matching_engine` module is not declared in `lib.rs` and is not compiled. The 18 KB `IMPLEMENT_NOW_CORE_HARDEN_BOOK_MVP.md` document is a detailed specification for the first implementation, scoped in two phases: Phase A core hardening (fix the over-fill, add `status(&self)`, standardise constructors on `NyquestroResult`, enable the self-match check, validate `OrderEvent::new()`, retire redundant error variants, add tests) and Phase B minimal deterministic OrderBook (sorted-map bid/ask, `submit_limit` returning `SubmitResult { fills, quotes }`, `best_bid()`/`best_ask()` inspection, integration tests for simple cross / sweep / partial-then-rest / determinism).

## Technologies and concepts demonstrated

### Languages

- **Rust (edition 2024)** — entire codebase. Used for newtype-pattern domain primitives, `Copy`-friendly value types, enum-based state and error modelling, `Result`-based error propagation, integration and unit testing. Edition 2024 is bleeding-edge — flagged in the LifeOS Architecture note as a potential compatibility constraint with older toolchains.

### Frameworks and libraries

- **`thiserror`** — derives the `NyquestroError` enum via `#[derive(thiserror::Error)]` (single-purpose; only error-formatting macro support).
- **`chrono 0.4.41`** — used solely by `Ts::to_utc_datetime()` for human-readable conversion. The README's earlier "no external crates" claim is acknowledged in LifeOS as a divergence from reality.

### Runtimes / engines / platforms

- No source evidence in LifeOS for any runtime/engine/platform beyond the standard Rust toolchain — Nyquestro is a plain library crate with a demo binary, no async runtime, no embedded platform, no GPU/VM target.

### Tools

- **Cargo** — build/test (implicit; only build tooling named in LifeOS).
- **`scan_repo.py`** — referenced as evidence-basis tool in the LifeOS Architecture frontmatter; specifics not detailed in source.
- No source evidence in LifeOS for benchmarking tools, profilers, fuzzers, property-testing crates, or CI configuration. The README mentions property-based tests, fuzz harness, and CI but LifeOS Gaps explicitly records all three as "not implemented" / "not configured".

### Domains and concepts

- **Order matching engine domain** — limit order book, FIFO price-time priority, marketability evaluation, resting orders, fills, quotes (top-of-book updates), order lifecycle (New / Cancelled / Rejected / state transitions), self-match policy. Currently expressed as types and a planned design — not yet as executing matching code.
- **Strongly-typed domain primitives (newtype pattern)** — `OrderID`, `Px`, `Qty`, `Ts`, `Side`, `Status` as wrappers over primitives to prevent accidental mixing and enforce invariants at construction.
- **Cents-based integer price representation** — explicit avoidance of floating-point in price comparison; `Px` stores `u64` cents, with deliberate scoping of float input to `new_from_dollars()` only.
- **Nanosecond `u64` timestamps** — high-frequency-trading-style time precision; `Copy`-friendly, comparison-friendly.
- **Immutable `Copy` event frames** — events allocation-free and duplicable for fan-out and replay testing without cloning costs.
- **Classified error model** — flat error enum plus a `Recoverable` / `Fatal` severity classification function; explicit decision to use classification rather than a type-hierarchy (separate `RecoverableError`/`FatalError` enums).
- **Deterministic-execution discipline** — explicit plan that matching must not call `Ts::now()` during execution (timestamps come from orders), no logging/IO on the core path, integration tests must assert exact event sequences.
- **Strict module dependency direction** — `types` and `errors` foundational; `order` and `price_level` build on them; `events` depends only on the foundation; `matching_engine` (when implemented) sits on top. The Dec 2025 FillEvent-coupling experiment was reverted within 24 hours specifically because it violated this direction.
- **Safe-Rust-only discipline** — no `unsafe` anywhere; constraint that performance optimisations (intrusive structures, manual alignment, manual memory management) must be expressed via safe abstractions. Verified upheld in current code.

The Work file `V2 Distributed Extension.md` (proposed, not built) names additional concepts — Viewstamped Replication consensus, deterministic-simulation testing via `madsim`, Kani formal verification, p50/p99/p99.9 latency benchmarking against Liquibook — but these are forward proposals, not demonstrated capabilities.

## Key technical decisions

Documented in LifeOS Decisions.md, split between explicit (called out in code/docs) and implicit (reconstructed from code patterns).

**D1 — Safe Rust only (upheld).** No `unsafe` in the codebase. Rationale: compiler eliminates data-race bugs in a concurrent system where correctness bugs are expensive. Implication: future performance work must find safe abstractions for what would normally be unsafe (intrusive structures, cache-line alignment, manual memory).

**D2 — Correctness before performance (active).** Lock-free structures, allocation optimisation, and concurrency are deferred. The MVP uses standard-library containers (e.g. `BTreeMap`) and `Vec<Order>` with cloning is acceptable. Constraint: the API surface must be stable enough that internal replacement does not cascade. Source: `MATCHING_ENGINE.md` — "the immediate next step is correctness and determinism, and lock-free structures can be introduced later if the API boundary stays stable".

**D3 — Events as immutable `Copy` frames (upheld).** All events are `Copy`, allocation-free, immutable after construction. Compatible with broadcast fan-out and replay testing without clone costs. Implication: events cannot carry heap-allocated data — `OrderRejectionReason` exists as an enum precisely because a `String` reason field would break `Copy`.

**D4 — Cents-based integer price (upheld).** `Px` stores `u64` cents, not floating-point. Rationale: integer comparison is exact, deterministic, fast, and avoids precision issues. Trade-off: dollar-to-cents conversion at the boundary involves float truncation — a known precision risk in `new_from_dollars()`.

**D5 — Nanosecond `u64` timestamps (upheld).** `Ts` is `u64` nanos since UNIX epoch. Single `u64` is `Copy` and comparison-friendly; nanosecond precision matches HFT expectations. Wraparound at ~584 years is not a practical concern.

**D6 — Severity as classification, not hierarchy (implicit).** `NyquestroError` is a flat enum; severity is computed via the `severity()` function rather than encoded as separate `RecoverableError` / `FatalError` enum hierarchies. Tension: the redundant generic variants in `NyquestroError` (`RecoverableError`, `FatalError`, `ErrorSeverity { .. }`) duplicate the classification concept inside the data model — flagged for cleanup.

**D7 — Order owns its timestamp (implicit).** `Order::new()` reads the clock internally via `Ts::now()`; the caller cannot supply a timestamp. Forces the matching engine to emit events using the order's stored timestamp rather than calling `Ts::now()` mid-match — an explicit determinism requirement in the hardening plan.

**D8 — `FillEvent` decoupled from `Order` (history).** For one day in December 2025 (commits `3556a70` → `864b488`), `Order::fill()` returned `NyquestroResult<FillEvent>`. Reverted within 24 hours because it coupled `order` to `events` (violating the dependency direction) and it (incorrectly) used `self.order_id` for both buyer and seller. Decision recorded as a healthy correction: event emission is the matching engine's responsibility, not the order's.

**Pending decisions** (documented in the hardening plan, unresolved):

| Decision | Options |
|---|---|
| Constructor error unification | `NyquestroResult<T>` everywhere vs keep `&'static str` in `types` |
| `Qty::new(0)` validation | Validate non-zero in `Qty::new()` vs enforce only at boundaries |
| `Ts::now()` panic removal | Return `NyquestroResult` vs keep panicking |
| Self-match policy | Reject at event level + engine level vs engine only |
| Severity entrypoint | Method on `NyquestroError` vs free function |
| Match price | Resting order price vs incoming order price |
| Quote emission rule | Top-of-book changes only vs any depth change |
| OrderBook data structure | Standard-library sorted map vs reuse `PriceLevel` |

LifeOS characterises the decision pattern as "conservative incrementalism" — defer until forced by implementation needs — and notes the risk that deferred decisions accumulate technical debt that must be resolved simultaneously once the matching engine depends on all of them.

## What is currently built

Implemented and tested today:

- **Core primitives** (`src/types.rs`) — six types, allocation-free, `Copy`-friendly. 16 integration tests in `tests/types_test.rs` (122 lines) covering constructors (valid/invalid), arithmetic, time operations, monotonicity. Inconsistent error returns (`&'static str` for `OrderID`/`Px`, infallible for `Qty`).
- **Error model** (`src/errors.rs`) — 11 error variants, `ErrorSeverity` enum, `severity()` classification function. 5 unit tests covering severity classification for every variant. Some redundant generic variants noted for cleanup.
- **Event frames** (`src/events/`) — three event types (`FillEvent`, `QuoteEvent`, `OrderEvent`), all `Copy`, immutable. The most comprehensive test coverage in the project: `tests/event_tests.rs` is 732 lines (~28 integration tests) covering construction, validation, semantics (Copy, Eq), boundary values, lifecycle scenarios, and error integration. In-module unit tests exist in each event file. Validation gaps: `FillEvent` self-match check commented out; `OrderEvent::new()` validates nothing.
- **Order model** (`src/order.rs`) — `Order` struct with fill mechanics and status transitions. **No dedicated tests** — only exercised indirectly through the demo binary. Known correctness bugs: silent over-fill, `get_status(self)` consumes the order, no one-way status transition guard.
- **PriceLevel** (`src/price_level.rs`) — basic `Vec<Order>` container. **No dedicated tests**. Clones on storage and on `get_orders()`. No removal API. No FIFO invariant.
- **Demo binary** (`src/main.rs`) — constructs an order, applies a partial then full fill, prints. Uses the cloning-heavy pattern around `get_status(self)` six times.

Not started:

- **Matching engine** — `src/matching_engine/order_book.rs` is 0 bytes; module is not declared in `lib.rs` and not compiled.
- Wire protocol, risk layer, strategy/market-making agent, benchmarking harness, property-based tests, fuzz harness, CI pipeline, historical market data replay, slab allocator.

Quantitative profile (LifeOS Overview.md, verified at commit `db2264d`):

| Metric | Value |
|---|---|
| Total Rust source files | 13 |
| Total source bytes | ~49 KB (Rust only) |
| External dependencies | `chrono 0.4.41`, `thiserror *` |
| Rust edition | 2024 |
| Integration test files | 2 (`event_tests.rs` 732 lines, `types_test.rs` 122 lines) |
| Unit test modules | 4 (in `errors.rs`, `fill_event.rs`, `order_event.rs`, `quote_event.rs`) |
| Total tests | ~58 across 6 locations, ~1050 lines of test code |
| Total commits | 22 (Jun 2025 – Mar 2026) |
| Contributors | 1 (Caner) |

Test-to-source ratio is ~1050 lines of tests against ~700 lines of non-test source — healthy for the areas covered, but coverage is concentrated on types and events while `Order` and `PriceLevel` have none.

## Current state

Status: active in LifeOS frontmatter, but with strong caveats. HEAD is unchanged at commit `db2264d` since 2026-04-15 (no source-code commits since December 2025 — the last 3 months have been documentation only). The Phase 4 documentation-consolidation burst ended at the Mar 25 README revision, and the LifeOS Evolution note flags the current ~1-month idle window as extending the project's pattern of "intense bursts followed by extended pauses" (two prior gaps of 5+ months). In flight: the Work folder contains one proposed-status file, `V2 Distributed Extension.md` (created 2026-04-25), scoping a forward "v2" that would add VSR consensus, a `madsim` deterministic-simulation-testing harness, Kani formal-verification proofs of 3-5 matching invariants, and reproducible p50/p99/p99.9 latency benchmarks — explicitly additive on top of v1 and explicitly gated on v1's matching engine + STP + journal shipping first.

## Gaps and known limitations

Drawn from LifeOS Gaps.md (verified all 15 gaps still present at commit `db2264d`).

**Critical (blocks the matching engine):**

- **G1 — `Order::fill()` silently accepts over-fills.** `saturating_sub` clamps to zero without error; an order with 3 remaining can be "filled" for 100 and become `FullyFilled` with no indication. The matching engine cannot trust `fill()` to validate its own invariants.
- **G2 — `get_status(self)` consumes the order.** Reading status requires cloning or surrendering ownership.
- **G3 — `PriceLevel` has no order removal.** Fully filled orders stay in the `Vec` forever; `total_quantity` is never decremented; matching cannot clean up after fills.
- **G4 — Matching engine does not exist.** 0-byte placeholder; not compiled; not part of `lib.rs`.

**Important (correctness/consistency):**

- **G5 — Inconsistent error return types.** `types.rs` constructors return `Result<_, &'static str>`; everything else returns `NyquestroResult<T>`. Demo binary mixes `.unwrap()` and `?`.
- **G6 — `FillEvent` self-match check commented out.** Disabled pending policy decision.
- **G7 — `OrderEvent::new()` validates nothing.** Always returns `Ok`. Inconsistent with `FillEvent` and `QuoteEvent`.
- **G8 — Status transitions are not guarded.** No protection against backward transitions.
- **G9 — `Qty::new(0)` is valid.** Zero-quantity validation deferred to consumers.
- **G10 — `Ts::now()` panics on clock error.** Violates the project's classified-error philosophy.
- **G11 — `Px::new_from_dollars` truncates floats.** `$10.999` → 1099 cents.

**Structural (missing capabilities):**

- **G12 — No `Order` or `PriceLevel` tests.** The two most stateful, mutation-heavy types have zero dedicated tests; the over-fill bug (G1) has never been tested.
- **G13 — `PriceLevel` clones on storage and retrieval.** Two live copies exist after `add_order`; mutations don't propagate.
- **G14 — No cancellation API.** `Status::Cancelled` exists but no `Order::cancel()` and no removal from a `PriceLevel`.
- **G15 — Error model has redundant generic variants.** `RecoverableError`, `FatalError`, `ErrorSeverity { .. }`, `ErrorSeverityCannotBeDetermined` overlap with the severity classification system; no current consumers.

**README-vs-reality divergences explicitly recorded:** lock-free order book, binary UDP protocol, FIX TCP acceptor, risk guard layer, strategy agent, benchmarking harness, property-based tests, fuzz harness, CI pipeline, historical market data replay, slab allocator — none implemented. Most are expected (the README describes the end state, not the current state); the concerning gaps are property-based tests, CI, and the named correctness issues (G1-G3) in already-implemented code.

## Direction (in-flight, not wishlist)

The single in-flight specification is `IMPLEMENT_NOW_CORE_HARDEN_BOOK_MVP.md` — an 18 KB / 296-line plan that is ready for execution (Phase A hardening, then Phase B minimal deterministic `OrderBook`). As of 2026-04-24 verification, no Phase A hardening tasks have been completed and the OrderBook MVP has not been started. Phase A includes: standardising constructors on `NyquestroResult<T>`, fixing `Order::fill()` over-fill rejection, one-way status transitions, adding `status(&self) -> Status`, enabling the `FillEvent` self-match check, validating `OrderEvent::new()` inputs, resolving error-model redundancy, and adding `Order` / `PriceLevel` tests before modifying the code. Phase B then ships `OrderBook` with `submit_limit(order) → SubmitResult { fills, quotes }`, `best_bid()` / `best_ask()` inspection, and integration tests for simple cross / sweep / partial-then-rest / determinism.

The Work folder also contains one proposed file — `V2 Distributed Extension.md` (status: proposed, created 2026-04-25) — scoping a forward additive layer on top of v1: VSR consensus in safe Rust as a `consensus/` module, a `madsim` deterministic-simulation-testing harness, Kani proofs of 3-5 matching invariants (no negative inventory, debit total = credit total, saturating fill bounds, FIFO within price level, no use-after-free), and a reproducible p50/p99/p99.9 latency benchmark vs Liquibook. This is explicitly gated on v1 matching engine + STP + journal shipping first; it is a forward proposal, not in flight.

## Demonstrated skills

What this specific project, as built today, proves:

- **Designs strongly-typed Rust domain models that enforce invariants at construction.** Newtype wrappers (`OrderID`, `Px`, `Qty`, `Ts`) over `u32`/`u64` with explicit constructors that reject invalid inputs (zero `OrderID`, non-positive price, etc.) — separating "any `u64`" from "a valid order id" at the type level.
- **Builds allocation-free `Copy`-friendly value types for hot-path use.** Every primitive and every event is `Copy`, designed for zero-allocation duplication and broadcast — and consciously protects this property (e.g. enum-based `OrderRejectionReason` instead of `String` to keep `OrderEvent: Copy`).
- **Uses safe Rust under self-imposed `unsafe`-free discipline.** Constraint upheld in current code, with documented awareness of the trade-offs it imposes on later optimisation work.
- **Designs and enforces strict module dependency direction.** Foundational `types` and `errors` layers; `order` and `price_level` build on them; `events` depends only on the foundation; cycles forbidden. Demonstrated discipline: a coupling experiment that violated the direction was identified and reverted within 24 hours.
- **Implements a classified error model with `thiserror`.** Single error enum + severity classification function + `Result` type alias, with explicit awareness of the design tension between "severity as classification" and "severity as type hierarchy".
- **Designs an immutable event-frame contract intended for fan-out and replay.** `FillEvent`, `QuoteEvent`, `OrderEvent` as the matching engine's planned output surface, structurally compatible with broadcast to multiple consumers and deterministic replay testing.
- **Integration- and unit-tests Rust crates with disproportionate coverage on the most uncertainty-laden surfaces.** ~1050 lines of test code against ~700 lines of source; 732-line `event_tests.rs` covering construction, validation, semantic properties, boundaries, lifecycle scenarios, and error integration; uses deterministic timestamps (`Ts::from_nanos`) for reproducibility.
- **Practises documentation-led design.** ~59 KB of docs (README + `context/`) against ~23 KB of source; the `IMPLEMENT_NOW_CORE_HARDEN_BOOK_MVP.md` reads as a detailed specification for the next implementation phase. The shift from `plans/` (step-by-step checklists) to `context/` (system descriptions of what exists, what is missing, why) reflects an evolving documentation methodology.
- **Reads and reasons about exchange-infrastructure domain concepts.** Limit order books, price-time FIFO priority, marketability evaluation, top-of-book quote emission rules, self-match policy, deterministic event sequences. Currently expressed in types, plans, and decisions rather than in matching code, but visibly internalised.
- **Practises honest self-assessment of build state.** LifeOS notes (Overview, Gaps, Roadmap, Evolution) carefully separate "what the README claims" from "what the code does" — explicitly naming critical correctness bugs (silent over-fill, `get_status(self)`) in the project's own foundation rather than glossing them. This is an additive signal to grading agents reading the file: the user maintains a documented separation between aspirational and implemented scope.

What the project does **not** demonstrate today (despite README pitch language): no implemented matching engine, no protocol implementation, no networking, no concurrency primitives, no benchmarking results, no production deployment, no CI. Roadmap items (lock-free structures, FIX/UDP gateways, risk layer, strategy agent, kernel bypass, SIMD, on-chain settlement) are aspirational scope, not skill evidence.
