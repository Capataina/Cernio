---
name: Zyphos
status: dormant
source_repo: https://github.com/Capataina/Zyphos
lifeos_folder: Projects/Zyphos
last_synced: 2026-05-31
sources_read: 13
---

# Zyphos

## One-line summary

Network-programming learning laboratory in Rust: a `std`-only, thread-per-connection HTTP/1.1 server built bottom-up from raw TCP sockets, scaffolded against a 30-milestone ladder from sockets through QUIC.

## What it is

Zyphos is Caner's structured Rust networking learning project. The stated mission is to learn sockets, HTTP, and modern network protocols end-to-end by implementing an HTTP server from raw TCP upward, progressively layering in production techniques (thread pools, zero-copy buffers, SIMD parsing, HTTP/2, QUIC). The README codifies this as a 30-milestone ladder across 7 phases — Network Foundations, Concurrency & Performance, Advanced Parsing & Optimisation, Kernel Bypass & Advanced I/O, Security & Robustness, Modern Protocols, and UDP & Alternative Protocols. The project is designed as a long-duration learning ladder; what it currently demonstrates is the first three rungs (M1, M3, partial M5) with M2 and M4 partially skipped. Repo identity vs Cargo identity diverge — GitHub repo is `Capataina/Zyphos` but the Cargo package name is still `multithreaded_http_server` v0.2.0, an artefact from before the 2025-11 "Zyphos" README rebrand.

## Architecture

Zyphos is a single Rust binary with one runtime dependency (`chrono` for date formatting). The codebase is ~500 LOC across 9 files in `src/`, organised as a strict layered pipeline with clean module boundaries:

```
+-----------+       +---------+       +--------+       +--------------+
|  main.rs  | ----> | handler | ----> | router | ----> | routes/{hello,
|  (net I/O)|       |  .rs    |       |  .rs   |       |  time, echo} |
+-----------+       +---------+       +--------+       +--------------+
                         |                |                    |
                         v                v                    v
                   +---------------+   +---------------+ +----------------+
                   | response.rs   |<--| create_       |<| create_text_   |
                   | format_       |   | responses.rs  | | response()     |
                   | response()    |   | (factories)   | +----------------+
                   +---------------+   +---------------+
```

**Dependency-direction rules that hold today:**

1. `main.rs` is the only file that touches `std::net` and `std::thread`.
2. `handler.rs` is the only file that sees the raw string request.
3. `router.rs` is the only file that maps `(method, path) → HttpResponse` builder.
4. `routes/*.rs` produce typed `HttpResponse` values; they do not serialise.
5. `response.rs` owns the `HttpResponse` struct and the wire-format serialiser.
6. `create_responses.rs` is the factory layer — the only file that injects `Content-Type`, `Content-Length`, `Connection`, `Date`.

A request crosses exactly four module boundaries to become a response, each boundary with a well-defined type. The seams align with the README's milestone ladder: M4's full header parser slots naturally into handler.rs; M6's thread pool slots into main.rs; M8's keep-alive forces `handle_connection` to become a read-loop.

**Core data shape:**

```rust
pub struct HttpResponse {
    pub status_code: i32,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}
```

Headers are stored unordered in a `HashMap`; deterministic wire order is reconstructed by `format_response` via a hardcoded "important headers" list (`Content-Type`, `Content-Length`, `Connection`, `Date`, `Server`), then iterating the remainder. `body: String` constrains the project to text payloads; any binary milestone (sendfile, WebSocket frames) will force a refactor to `Vec<u8>` or a `Body` enum.

## Subsystems and components

### Connection Handling (`main.rs`)

Owns the TCP listener lifecycle, per-connection thread spawning, panic recovery, and connection logging. Binds to `localhost:3000` (no `SO_REUSEADDR`, no `TCP_NODELAY`). Each accepted connection gets an atomic `connection_id` and a dedicated OS thread via `thread::spawn`. The thread body is wrapped in `panic::catch_unwind(AssertUnwindSafe(...))` so a panic in parsing or routing cannot kill the server; the panic payload is downcast through `&str` then `String` for logging, with a fallback for non-string payloads. `handle_connection` is trivially blocking — one `read` into a fixed `[0; 1024]` buffer, `String::from_utf8_lossy` for decoding, one `write_all`, one `flush`, no loop. One request per connection is the invariant.

### Request Parsing (`handler.rs`)

The only file that sees raw request strings. Splits on `\r\n\r\n` to separate headers from body, then takes the request line and applies `split_whitespace()` to extract method, path, and version. Validates token count == 3 and that the version starts with `HTTP/`. The body slice is captured as a local but its assignment is commented out (`// let body_section = &raw_request[pos + ..];`) — request bodies are currently discarded. Headers beyond the request line are split into lines but never parsed into a key-value structure: `header_lines[0]` is used and `header_lines[1..]` is thrown away. Carries the largest inline test suite in the repo (19 `#[cfg(test)]` tests added in commit `694ff01`, December 2025).

### Routing (`router.rs`)

A 15-line if/else dispatch chain. Returns 404 for any method that is not `GET` — POST/PUT/DELETE/HEAD all 404 by construction. Exact match for `/hello` and `/time`; prefix-strip for `/echo/`. No path parameters, no query-string parsing, no URL decoding, no `HEAD` short-circuit.

### Response Pipeline (`response.rs` + `create_responses.rs`)

`response.rs` owns the `HttpResponse` struct and `format_response`, which emits the status line as a hardcoded `HTTP/1.1 {code} {text}` regardless of the request's HTTP version, then writes "important headers" in fixed order, then remaining headers in HashMap iteration order, then the body. Trailing `\r\n\r\n` after the body is emitted (off-spec but benign against real clients). `create_responses.rs` is the factory layer: `create_text_response` and the error-response builder both hardcode `Connection: close` and set the `Date` header via `chrono::Utc::now()` in RFC 1123 format (`%a, %d %b %Y %H:%M:%S GMT`).

### Routes (`routes/{hello,time,echo}.rs`)

Three typed response producers. `/hello` returns a static string; `/time` returns `chrono::Local::now()` formatted (note: `Local` here, `Utc` in the Date header — minor timezone inconsistency); `/echo/X` returns whatever follows the prefix.

### Testing (`#[cfg(test)] mod tests` in handler.rs)

19 unit tests inline in handler.rs covering request-line parsing variants, header/body splits, whitespace tolerance, malformed input, and edge cases. No `tests/` directory; no integration tests; `main.rs` has 0% coverage (the accept loop, thread spawning, and panic recovery are untested). No CI workflow runs the tests.

## Technologies and concepts demonstrated

### Languages

- **Rust (package version 0.2.0)** — sole implementation language. Used across all 9 source files for socket I/O, thread management, parsing, and response serialisation. Stack-allocated fixed buffers, `String::from_utf8_lossy` for decoding, `HashMap` for header storage, `AtomicUsize` with `SeqCst` ordering for the connection counter, `panic::catch_unwind` with `AssertUnwindSafe` for panic recovery, ownership transfer via `move` closures for cross-thread stream handoff.

### Frameworks and libraries

- **`chrono = "0.4"`** — sole runtime dependency. Used for `Utc::now()` in the `Date` response header (`create_responses.rs`) and `Local::now()` in the `/time` route body (`routes/time.rs`). Zero test dependencies. The `Cargo.lock` is almost entirely chrono's transitive tree (iana-time-zone, windows-core).

### Runtimes / engines / platforms

- **Rust standard library `std::net` + `std::thread`** — the project deliberately avoids `tokio`, `hyper`, `axum`, `actix`, and `mio`. Per Decision D1, every networking concept must be built from std primitives so the project teaches sockets, parsing, and framing rather than hiding them behind a framework.

### Tools

- **Cargo** — build, test, dependency management. No `rustfmt.toml`, no `clippy.toml`, no GitHub Actions workflow, no Criterion benchmark surface.

### Domains and concepts

- **Raw TCP socket programming** — `TcpListener::bind` + `listener.incoming()` accept loop, `TcpStream::read`/`write_all`/`flush`, OS-default backlog.
- **Thread-per-connection concurrency model** — naïve unbounded `thread::spawn` per accepted connection, the textbook baseline before introducing pools.
- **Panic recovery in long-running servers** — `panic::catch_unwind` + payload downcast pattern (`&str` then `String` then fallback) to keep a server loop alive across bad requests.
- **Atomic counters with explicit memory ordering** — `AtomicUsize::fetch_add(1, SeqCst)` for monotonic connection IDs.
- **HTTP/1.1 wire-format generation** — hand-written status line, header ordering, CRLF framing, RFC 1123 date formatting.
- **HTTP request-line parsing** — manual tokenisation via `split_whitespace`, prefix validation of the version string, header-body separation via `\r\n\r\n` search.
- **Module boundary discipline** — strict layering where only one file touches a given concern (net I/O, raw strings, dispatch, serialisation).
- **Inline `#[cfg(test)]` unit testing for parser logic** — 19 tests pinning request-line and header-split behaviour.

## Key technical decisions

The LifeOS folder captures 10 explicit design decisions, each with alternatives considered and the conditions that would flip the call:

**D1 — Rust + `std` only, no web framework.** Zyphos depends on `chrono` and nothing else. `hyper`, `axum`, `actix`, `tokio`, `mio` were all considered and rejected. Rationale: the project's core principle is bottom-up networking; the moment Zyphos depends on a framework it stops teaching the thing it exists for. This decision would not flip for the duration of the learning project.

**D2 — Thread-per-connection, not thread-pool.** Every accepted connection gets its own OS thread. Alternatives were a thread pool (the README's M6 target), async/await with a runtime (out of scope per D1), and an epoll/kqueue event loop (M9 target). Rationale: this is the textbook naïve baseline that M3 specifies; a pool is premature optimisation at this rung. Acknowledged costs: ~2MB stack per thread, no backpressure, thread startup latency per connection.

**D3 — `panic::catch_unwind` around request handling.** Wraps `handle_connection` so a bug in parsing or routing cannot kill the server. Alternatives were letting panics propagate, full `Result`-based error propagation, or log-and-exit with a supervisor. Rationale: for learning, server liveness during iteration matters more than panic surfacing. Will flip when production-shaped operation arrives at M21.

**D4 — `HashMap` headers with deterministic serialisation order.** Headers stored unordered; `format_response` reconstructs wire order via a hardcoded "important headers" list. Alternatives were `Vec<(String, String)>`, `IndexMap` (violates D1), or `BTreeMap`. Rationale: `HashMap` is the natural std primitive; explicit ordering in serialisation recovers determinism. Will flip if M8 or M23 makes the important-list unwieldy.

**D5 — Hardcoded if/else router over a trie.** Three routes do not justify a trie or radix tree. Will flip at ~10 routes or when multi-segment path parameters (M13) arrive.

**D6 — `body: String` in HttpResponse.** Forecloses binary content. Chosen because UTF-8 text is the only payload today and `String::len()` makes Content-Length automatically correct. Will flip at M14 (caching), M16 (sendfile), or M25 (WebSocket frames).

**D7 — `Connection: close` on every response.** Hardcoded in both factories. Without a connection-reuse loop, claiming keep-alive would be a protocol violation. Will flip at M8.

**D8 — `split_whitespace()` tolerance in request-line parsing.** Collapses runs and strips edges, accepting `"GET  /hello HTTP/1.1"` (double space) where a strict parser would reject. Will flip at M20 (parser security / differential testing).

**D9 — `String::from_utf8_lossy` for request bytes.** Non-UTF-8 bytes become `U+FFFD`. Will flip when binary request bodies need exact byte handling — likely at M4 body-reading completion or M25.

**D10 — Inline `#[cfg(test)]` tests, no integration tests.** Maximises locality for `&str → String` functions; leaves `main.rs` (accept loop, panic recovery, thread behaviour) entirely uncovered.

## What is currently built

The codebase is 9 Rust files, ~14.7KB (roughly 500 lines including tests). The README is 48KB (1788 lines) — bigger than all code combined by ~3.3x. Built and working at HEAD `694ff01` (2025-12-13):

- TCP listener on `localhost:3000` with unbounded `thread::spawn` per connection.
- Atomic connection counter (`AtomicUsize` with `SeqCst`).
- Panic recovery via `panic::catch_unwind` with `&str`/`String` payload downcast.
- HTTP request-line parsing (method, path, version extraction; token-count and `HTTP/` prefix validation).
- Header/body separator split on `\r\n\r\n`.
- Response serialisation with deterministic header ordering.
- Response factories for text and error variants, with `Content-Type`, `Content-Length`, `Connection: close`, and RFC 1123 `Date` headers.
- Three routes: `/hello` (static text), `/time` (`Local::now()` formatted), `/echo/X` (prefix echo).
- 404 handler.
- GET-only dispatch (non-GET methods 404 by construction).
- 19 inline unit tests in `handler.rs` covering request-line parsing edge cases.

**Explicitly not built**, despite README ambition: actual header key-value parsing (`header_lines[1..]` is thrown away), `Content-Length` handling, request body reading (the assignment is commented out), thread pool, keep-alive / persistent connections, epoll/kqueue, TLS, HTTP/2, WebSockets, SSE, UDP, QUIC, integration tests, CI, benchmarks, rate limiting, timeouts, graceful shutdown, `SO_REUSEADDR`, `TCP_NODELAY`, EINTR/EAGAIN handling, URL decoding, query-string parsing, vhosting (`Host` header is never read), `Server` header population (the slot is reserved in the ordering list but no factory inserts a value).

Honest mapping to the README's 30-milestone ladder: M1 (Raw Sockets) started — 2/5 exit criteria plausibly met; M2 (TCP State Machine) not started; M3 (Thread-per-Connection) partial — 4/12 implementation items done, the most honestly-complete milestone; M4 (HTTP/1.0 Parser/Generator) started but with critical gaps (no header parsing, no body reading); M5 (Basic Routing) started — 3/12 items done; M6–M30 all not started. Approximately 3 of 30 milestones (10%) have meaningful code.

## Current state

Status: **dormant**. Last meaningful commit `694ff01` ("fixed handler", actually +197 LOC of unit tests) on 2025-12-13; no commits since (as of 2026-04-24 vault verification, ~4+ months of silence). The project's commit cadence is two-mode: concentrated 1–3 day bursts producing 5–10 commits (June 2025, July 2025, November 2025, December 2025), separated by 1–4 month dormant periods. 25 total commits across the repo's lifetime (2025-06-14 → 2025-12-13). No items currently in flight; the LifeOS folder has no `Work/` subdirectory. Cargo package name is still `multithreaded_http_server` v0.2.0 — `cargo run` boots a binary by that name despite the "Zyphos" README rebrand.

## Gaps and known limitations

LifeOS captures 26 explicit gaps organised by severity. Career-relevant highlights:

**Critical latent bugs in shipped code:** trailing `\r\n\r\n` after response body (off-spec but benign against real clients); fixed 1024-byte read buffer silently truncates requests larger than that (a realistic browser request with Host + User-Agent + Cookie exceeds this); request body slice is captured but the assignment is commented out, so POST/PUT/PATCH bodies are discarded (currently harmless because the router 404s non-GET); `stream.read().expect(...)` panics on any read error including a client closing mid-request, with `catch_unwind` masking the detail.

**Structural gaps blocking milestone progress:** no header parsing (`header_lines[1..]` thrown away); no `Content-Length` handling; only GET is supported; no URL decoding (`%20` passes through raw); no query-string parsing (`/echo/test?param=value` is treated as the path `test?param=value`); unbounded `thread::spawn` is DoS-trivial; `Connection: close` hardcoded forecloses keep-alive without rewriting `handle_connection`; no shutdown signalling (`for stream_result in listener.incoming()` loops forever, no SIGINT handling).

**Correctness and consistency:** `Utc::now()` vs `Local::now()` timezone inconsistency between Date header and `/time` route; `status_code: i32` allows negative values (should be `u16`); response status line is hardcoded `HTTP/1.1` regardless of request version; `Server` header slot is reserved in the ordering list but no factory inserts a value; multi-threaded `println!` interleaves under load (no `tracing` crate, no log levels); peer address never logged despite `stream.peer_addr()` being available; `Host` header never read (vhosting impossible).

**Tooling and process:** Cargo package name (`multithreaded_http_server`) does not match repo name; no GitHub Actions / CI; no `rustfmt.toml` or `clippy.toml`; commit messages of varying quality including `"latest changes, dont know what"`, `"test"`, `"nvim test"`, `"fixed handler"`.

**Testing coverage:** `main.rs` is 0% covered (accept loop, thread spawning, panic recovery untested); `response.rs` has no byte-format tests; `create_responses.rs` has no Content-Length assertion; no concurrency tests; no fuzz tests; no `tests/` integration directory.

## Direction (in-flight, not wishlist)

There is no actively-in-progress work; the project is dormant. The LifeOS Roadmap names a concrete next-session plan ordered by leverage, but none of these is currently being executed:

1. Close M4 — add a `Headers` type, parse `header_lines[1..]`, extract and validate `Content-Length`, re-enable the body slice, read exactly `Content-Length` bytes, pass the body into the router, add 5–10 new tests.
2. Close the M1 socket-option gaps — `SO_REUSEADDR`, `TCP_NODELAY`, replace `expect()` on `read`/`write`/`flush` with `Result` handling, add a TCP-level integration test.
3. Add POST routing — extend the router match, add `/echo-body`.
4. Thread pool (M6) — `std::sync::mpsc`-coordinated fixed pool, graceful shutdown, queue-depth metrics.
5. HTTP/1.1 keep-alive (M8) — read the `Connection` header, conditionally emit `keep-alive`, refactor `handle_connection` into a read-loop bounded by `Connection: close` or a max-requests counter, add idle timeouts via `TcpStream::set_read_timeout`.

The next active session is expected on the quarterly-burst cadence the commit history establishes — weeks to months out, not days.

## Demonstrated skills

- **Raw TCP socket programming in Rust without a networking framework** — implements the accept loop, per-connection lifecycle, panic recovery, and synchronous request/response handling using only `std::net`, `std::thread`, and `std::sync::atomic`.
- **Thread-per-connection concurrency with panic isolation** — uses `panic::catch_unwind(AssertUnwindSafe(...))` with `&str`/`String` payload downcast to keep a server loop alive across handler panics; uses `AtomicUsize::fetch_add(SeqCst)` for monotonic connection IDs and `move` closures for cross-thread stream ownership transfer.
- **Hand-written HTTP/1.1 parser and serialiser** — request-line tokenisation via `split_whitespace`, header/body framing via `\r\n\r\n` search, deterministic response header ordering reconstructed from an unordered `HashMap` via a hardcoded "important headers" list, RFC 1123 date formatting.
- **Strict module-boundary design in a layered pipeline** — enforces a one-direction dependency graph (`main → handler → router → routes` and `routes → response ← create_responses`) where each file owns exactly one concern (net I/O, raw strings, dispatch, struct, factories).
- **Inline unit testing for parser logic at the `&str → String` boundary** — 19 `#[cfg(test)]` tests in `handler.rs` pinning request-line parsing, whitespace tolerance, header/body splits, and malformed-input behaviour.
- **Explicit, written design-decision discipline** — captures 10 numbered design decisions in LifeOS with alternatives considered, rationale, and flip conditions; demonstrates awareness of when each chosen tradeoff stops being correct (e.g. `body: String` flips at M14/M16/M25; if/else router flips at ~10 routes).
- **Honest gap and limitation inventory** — maintains a 26-gap severity-ordered list distinguishing latent bugs in shipped code, structural blockers, correctness misses, and tooling gaps; resists the README's pitch language to keep the documented state aligned with the code state.
- **Bottom-up systems learning discipline** — refuses framework dependencies (`tokio`, `hyper`, `axum`, `mio`) to preserve the project's teaching value, even at the cost of slower progress against the milestone ladder.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Zyphos/_Overview.md | 106 | "#project #zyphos #rust #networking #http #learning-project #solo" |
| Projects/Zyphos/Architecture.md | 165 | "- [[Decisions]] — why the seams are where they are" |
| Projects/Zyphos/Decisions.md | 181 | "- [[Roadmap]] — which decisions will need revisiting" |
| Projects/Zyphos/Gaps.md | 180 | "- [[Suggestions]] — opportunistic improvements beyond these specific gaps" |
| Projects/Zyphos/Milestones.md | 174 | "- [[Systems/Routing#What This Domain Still Needs to Hit M5\|Systems/Routing: M5 exit criteria]]" |
| Projects/Zyphos/Roadmap.md | 150 | "- [[Suggestions]] — ideas outside the milestone ladder" |
| Projects/Zyphos/Suggestions.md | 135 | "- [[Systems/Testing]] — R3 (timeouts) and O6 (integration tests) live here" |
| Projects/Zyphos/Systems/_Overview.md | 40 | "- [[Projects/Zyphos/Roadmap]] — direction-of-travel" |
| Projects/Zyphos/Systems/Connection Handling.md | 152 | "- [[Milestones#Milestone 3 Thread-per-Connection Model\|Milestones: M3 detail]]" |
| Projects/Zyphos/Systems/Request Parsing.md | 119 | "- [[Gaps#Request parsing gaps\|Gaps: missing headers, body, Content-Length]]" |
| Projects/Zyphos/Systems/Response Pipeline.md | 142 | "- [[Gaps#Response pipeline gaps\|Gaps: trailing CRLF, Server header, binary bodies]]" |
| Projects/Zyphos/Systems/Routing.md | 168 | "- [[Gaps#Routing gaps\|Gaps: POST/PUT/DELETE, query strings, URL decoding]]" |
| Projects/Zyphos/Systems/Testing.md | 113 | "- [[Roadmap]] — test priorities in the next session" |
