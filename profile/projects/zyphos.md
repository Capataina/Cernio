---
name: Zyphos
status: dormant
source_repo: https://github.com/Capataina/Zyphos
lifeos_folder: Projects/Zyphos
last_synced: 2026-04-26
sources_read: 12
---

# Zyphos

## One-line summary

Bottom-up, std-only HTTP/1.x learning server in Rust — currently a thread-per-connection blocking listener with a 19-test inline parser suite, structured against a 30-milestone progression from raw sockets through to QUIC.

## What it is

Zyphos is Caner's network-programming learning laboratory built in Rust. The stated mission is to learn sockets, HTTP, and modern network protocols end-to-end by implementing an HTTP server from raw TCP up, progressively layering in production techniques (thread pools, zero-copy, SIMD parsing, HTTP/2, QUIC). The README defines a 30-milestone ladder across 7 phases (Network Foundations, Concurrency & Performance, Advanced Parsing & Optimisation, Kernel Bypass & Advanced I/O, Security & Robustness, Modern Protocols, UDP & Alternative Protocols). The project is deliberately written against `std` only with one runtime dependency (`chrono`); the constraint exists because pulling in `hyper`, `axum`, `tokio`, or `mio` would skip the very concepts the project is designed to teach. LifeOS frames Zyphos honestly as a learning project: roughly three of thirty milestones have meaningful code (M1, M3, M5 partial-complete), and every checkbox in the README's Learning Roadmap is unchecked. The repo's GitHub name is `Zyphos` but the Cargo package is still `multithreaded_http_server` v0.2.0 — the "Zyphos" brand lives in the README only.

## Architecture

The implemented server is a five-module Rust crate with a clean four-boundary request flow. LifeOS captures the full module dependency graph:

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
                   +---------------+   +---------------+ +----------------+
```

Direction rules currently held by the code (per LifeOS Architecture.md):

1. `main.rs` is the only file that touches `std::net` and `std::thread`.
2. `handler.rs` is the only file that sees the raw string request.
3. `router.rs` is the only file that maps `(method, path)` → `HttpResponse` builder.
4. `routes/*.rs` produce typed `HttpResponse` values; they do not serialise.
5. `response.rs` owns the `HttpResponse` struct and the wire-format serialiser.
6. `create_responses.rs` is the factory layer — the only file that injects `Content-Type`, `Content-Length`, `Connection`, `Date`.

Request lifecycle: `TcpListener.incoming()` accepts a stream → `main.rs` spawns a thread under `panic::catch_unwind(AssertUnwindSafe(...))` → a single 1024-byte `read` decoded via `String::from_utf8_lossy` → `handle_request` finds the `\r\n\r\n` separator, splits the request line via `split_whitespace`, validates token count (=3) and version prefix (`HTTP/`) → `route(method, path)` dispatches by hardcoded if/else → typed `HttpResponse` returned → `format_response` serialises with deterministic header ordering → `stream.write_all` + `flush` → thread ends, stream dropped via RAII.

Data shape:

```rust
pub struct HttpResponse {
    pub status_code: i32,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}
```

Headers are stored unordered in a `HashMap<String, String>`; deterministic wire order is reconstructed by `format_response` writing a hardcoded "important headers" list (`Content-Type`, `Content-Length`, `Connection`, `Date`, `Server`) first, then iterating the remainder. `body: String` constrains responses to text and forecloses binary payloads without refactoring.

Invariants the code currently holds: requests fit in 1024 bytes (anything larger is silently truncated), requests are valid UTF-8 (lossy decode via `from_utf8_lossy`), every response sets `Connection: close` (one request per connection), every response wire-line is `HTTP/1.1` regardless of request version, and concurrency is unbounded (`thread::spawn` with no pool).

Dependency surface is intentionally minimal: `chrono = "0.4"` is the single runtime dependency (used for `Utc::now()` in the Date header and `Local::now()` in `/time`). LifeOS Decisions.md flags this as deliberate — the moment Zyphos depends on `tokio` or `hyper` it stops teaching the thing it is for.

## Subsystems and components

### Connection Handling (`main.rs`)

Owns the listener lifecycle, per-connection thread spawning, panic recovery, and logging. Atomic `CONNECTION_COUNTER: AtomicUsize` produces monotonic connection IDs. Each connection runs on a fresh OS thread with `panic::catch_unwind(AssertUnwindSafe(...))` wrapping `handle_connection`; the panic payload is downcast first to `&str`, then `String`, then a fallback "could not be extracted" branch. `handle_connection` itself is one `read`, one `write`, one `flush` — there is no read-loop, so one request per connection is a structural invariant. ~50 lines of actual logic. The "added true multithreading" commit (`1778e9a`) was 4 lines added and 1 removed — a single `thread::spawn(move || ...)` wrapping the existing handler.

Costs LifeOS documents: unbounded thread count (~2MB stack each, scheduler collapse past ~10K connections), blocking `read` (slowloris-trivial DoS), fixed 1024-byte buffer, `expect()` on `read`/`write`/`flush` (panics propagate to `catch_unwind`), `String::from_utf8_lossy` (silent corruption), no shutdown signalling, no backpressure.

### Request Parsing (`handler.rs`)

Owns the transition from raw `&str` to a dispatched `HttpResponse`. Validation ladder is intentionally thin: (1) presence of `\r\n\r\n` separator → 400 if missing, (2) request line splits to exactly 3 whitespace tokens → 400 if not, (3) version token starts with `"HTTP/"` → 400 if not. Headers past line 0 are collected into `Vec<&str>` but never interpreted; the body slice is computed in a commented-out line and discarded. The 19 inline tests live in this file.

What is explicitly NOT parsed (LifeOS Request Parsing.md is explicit about the negative space): headers past the request line, request body (body line is commented out at `src/handler.rs:63`), Content-Length, Transfer-Encoding chunked, header line continuations, case-insensitive header matching, URL decoding (`%20` passes through raw), query strings (`?` is treated as part of the path), Host header (no vhosting possible).

### Response Pipeline (`response.rs` + `create_responses.rs`)

`response.rs` defines the `HttpResponse` struct (32 lines) and `format_response`, which serialises by writing the status line as `HTTP/1.1 {code} {text}\r\n`, then the five "important" headers in deterministic order, then any remaining HashMap entries, then `\r\n` separator + body + trailing `\r\n\r\n`. The trailing `\r\n\r\n` after the body is off-spec — well-behaved clients ignore it but strict HTTP validators will flag it.

`create_responses.rs` is the factory layer: `create_text_response(body: String)` and `create_error_response(code, text, body)` populate identical header sets (Content-Type: text/plain, Content-Length via `body.len()` which is correctly bytes-not-chars in Rust, Connection: close, Date via RFC 1123 formatting `%a, %d %b %Y %H:%M:%S GMT`). `Server` header slot is reserved in the ordering list but never populated. No commits have touched the response layer since 2025-07-11 — it has been stable for six months.

### Routing (`router.rs` + `routes/`)

18 lines of hand-written dispatch. Three routes: GET `/hello` (exact match → "Hello World!"), GET `/time` (exact match → `DD/MM/YYYY HH:MM:SS` via `Local::now()`), GET `/echo/{text}` (prefix-strip via `strip_prefix("/echo/")`). Anything else 404s, including any non-GET method (the router's outermost arm is `if http_method == "GET"`). Closed enumeration over `(method, path)` pairs; adding a route requires editing `router.rs`, the `use` block, and re-exporting in `routes/mod.rs`. Lookup complexity is O(routes) per request via literal string comparison. Case-sensitive method (`get` 404s); case-sensitive path. Echo is a directory-traversal/reflected-XSS surface in principle — defended only by the absence of filesystem access and the `text/plain` content type.

### Testing (inline `#[cfg(test)]` in `handler.rs`)

19 unit tests pinning request-parser behaviour, all added in a single commit (`694ff01`, 2025-12-13, "fixed handler", +197 LOC). Tests are characterisation-style — they pin what the code does, not what the spec says it should do. Coverage is concentrated in the request-parsing layer; `response.rs`, `create_responses.rs`, and the `main.rs` accept/spawn loop have zero test coverage. No integration tests, no `tests/` directory, no benchmarks, no CI (`.github/workflows/` does not exist). Test infrastructure is minimal: each test constructs a raw request `&str` and calls `handle_request` directly; assertions are substring-matching via `assert!(response.contains(...))` rather than byte-exact wire-format checks.

## Technologies and concepts demonstrated

### Languages

- **Rust** — the entire codebase. ~500 lines including tests across 9 files. Uses `std` exclusively for I/O, threading, and sync primitives. Idiomatic ownership-transfer-via-`move` for cross-thread state, `AssertUnwindSafe` wrapping for panic-safety, RAII-driven socket close.

### Frameworks and libraries

- **chrono 0.4** — single runtime dependency. Used for `Utc::now()` in the Date header and `Local::now()` in the `/time` route body. The `Cargo.lock` is essentially the chrono transitive tree (iana-time-zone, windows-core).

### Runtimes / engines / platforms

- **Rust `std::net` (TcpListener / TcpStream)** — blocking BSD-sockets-style API. No `mio`, no `tokio`, no async runtime. The whole accept loop runs on the main thread; I/O on per-connection OS threads.
- **Rust `std::thread`** — `thread::spawn(move || ...)` per accepted connection. No `Builder` (default 2MB stack, no thread name), no pool.
- **Rust `std::sync::atomic`** — `AtomicUsize` with `SeqCst` ordering for the connection counter. The only lock-free construct in the project.

### Tools

- **cargo** — build and test runner. `cargo test` runs the 19-test inline suite. No `rustfmt.toml`, no `clippy.toml`, no CI.
- **git** — version control. 25 commits total (2025-06-14 → 2025-12-13), with the project's natural rhythm being concentrated 1-3 day bursts separated by 1-4 month dormant periods.

### Domains and concepts

- **TCP server fundamentals** — bind, listen (default OS backlog of 128 on Linux), accept loop. No SO_REUSEADDR, no TCP_NODELAY, no EINTR/EAGAIN handling, no graceful shutdown.
- **Thread-per-connection concurrency model** — the textbook M3 baseline. Not a thread pool. LifeOS Decisions.md frames this explicitly as the correct shape for the current milestone, with explicit costs documented (~2MB stack/thread, no backpressure, thread startup latency).
- **Panic recovery via `panic::catch_unwind`** — wrapping request handling so a parser bug cannot kill the server. LifeOS Decisions.md flags this as the right pattern for a learning loop, the wrong pattern for production where a supervisor should restart on panic.
- **HTTP/1.x request-line parsing** — separator-find on `\r\n\r\n`, `split_whitespace` tokenisation, version-prefix validation. LifeOS Decisions.md notes the deliberate `split_whitespace` tolerance over strict `split(' ')` — accepts `"GET  /foo  HTTP/1.1"` (double-space) by design, pinned by `test_request_line_with_extra_whitespace`.
- **Deterministic-order serialisation over an unordered HashMap** — the "important headers" list reconstructs wire order despite the underlying `HashMap` having none. Captures the trade-off between `Vec<(String, String)>`, `IndexMap`, `BTreeMap` and `HashMap` + explicit ordering.
- **RFC 1123 date formatting (IMF-fixdate)** — `%a, %d %b %Y %H:%M:%S GMT` matches the RFC 7231 preferred form.
- **Content-Length byte-correctness** — `String::len()` in Rust returns bytes, which is exactly what Content-Length expects. LifeOS Response Pipeline.md flags this as a Rust-specific correctness win that hand-rolled HTTP servers in Go or Python frequently get wrong.
- **Closed-enumeration if/else routing** — explicit dispatch table, exact-match plus one prefix-strip pattern. Sized correctly for M5; would be wrong shape at M13 (trie/radix routing).
- **Inline `#[cfg(test)]` characterisation testing** — substring-matching assertions pinning current behaviour. Standard Rust idiom for unit tests adjacent to the code under test; LifeOS Testing.md notes that `repo_stats.py`-style scanners that look for `tests/` directories or `*_test.rs` filenames produce false-negative `test_files: 0` for this layout.

## Key technical decisions

LifeOS captures ten explicit decisions, each with alternatives considered, why-this-won, and what would flip the call.

1. **D1 — Rust + `std` only, no web framework.** Chosen over `hyper`/`reqwest` (would hide sockets and framing), `tokio` (would skip the blocking → thread-pool → epoll progression), and `mio` (would skip M9's from-scratch event loop). The README's core principle is bottom-up implementation; pulling in any of these would defeat the entire learning project. Would flip only for a hypothetical post-learning production rewrite.

2. **D2 — Thread-per-connection, not thread-pool.** Chosen over thread pool (M6 target), async/await (out of scope per D1), and event loop (M9 target). This is the textbook naïve baseline and exactly the shape M3 asks for. Costs already paid: ~2MB stack/thread, no backpressure, thread startup overhead. Will flip when M5 lands and M6 begins.

3. **D3 — `panic::catch_unwind` around request handling.** Chosen over let-it-propagate (one bad request kills the process), pervasive `Result` propagation (correct but verbose), and a supervisor-restart pattern (no supervisor exists). Pragmatic for iterative learning. Will flip on the move to production-shaped operation, naturally aligned with M21 (timeouts/backpressure).

4. **D4 — `HashMap<String, String>` headers with deterministic serialisation order.** Chosen over `Vec<(String, String)>` (preserves insertion order naturally), `indexmap` (would violate D1), and `BTreeMap` (alphabetical wire order is fine but not idiomatic). Std-only constraint forces the explicit ordering hack. Will flip if M8 or M23 add enough headers that the important-list becomes unwieldy.

5. **D5 — Hardcoded if/else router over a trie.** Chosen over a `HashMap<(method, path), fn>` (no prefix matching), trie/radix tree (M13 target — overkill for 3 routes), and regex-based router. Minimum-correct for M5. Will flip past ~10 routes or with the introduction of multi-segment path params (`/users/{id}/posts/{postId}`).

6. **D6 — `body: String` in HttpResponse.** Chosen over `Vec<u8>` (binary-compatible from day one), an enum `Body::Text|Bytes|File`, and `&'a [u8]` (lifetime-complex). Simplest possible response type for M4-M5. UTF-8 text is the only payload in current routes; `String::len()` returning bytes makes Content-Length automatically correct. Will flip on M14 (caching), M16 (sendfile), or M25 (WebSocket frames) — flagged as the next major refactor in LifeOS Roadmap.md.

7. **D7 — `Connection: close` on every response.** Chosen over omitting the header (default behaviour depends on HTTP version) and `Connection: keep-alive` with real keep-alive logic. Without a connection-reuse loop, claiming keep-alive would be a protocol violation. Will flip with M8 — requires restructuring `handle_connection` from one-request-per-call into a read-loop with connection-alive tracking.

8. **D8 — `split_whitespace()` tolerance in request-line parsing.** Chosen over strict `split(' ')` (would reject double-space) and a byte-level state machine. More robust to slightly-malformed clients; pinned by tests. Will flip at M20 (parser security / differential testing) when Zyphos needs explicit strict and lax modes.

9. **D9 — `String::from_utf8_lossy` for request bytes.** Chosen over `std::str::from_utf8` (returns Result, rejects non-UTF-8) and raw `&[u8]` parsing throughout. UTF-8 is expected for HTTP request lines and headers; lossy decode is lazier but less fragile. Will flip when binary correctness in the request body matters — M4's Content-Length body reading will likely force a move to `&[u8]` for the body section at minimum.

10. **D10 — Inline `#[cfg(test)]` tests, no integration tests.** Chosen over a separate `tests/handler_integration.rs` (would exercise the real TCP listener) and a hybrid layout. Minimal infrastructure, maximal locality for `&str → String` functions. The missing integration tests reflect that `main.rs` is essentially untested. Will flip when accept-loop / panic-recovery / threading paths need coverage.

## What is currently built

Honest current scope (LifeOS Overview.md captures this in a per-feature table):

- **Working:** TCP listener bound to `localhost:3000` with default backlog, thread-per-connection via `thread::spawn`, panic recovery via `catch_unwind`, atomic connection counter, HTTP request-line parsing (method/path/version), `\r\n\r\n` head/body separator split, response serialisation, response factory (text + error variants), RFC 1123 Date header, exact-match routing for `/hello` and `/time`, prefix-strip routing for `/echo/`, 404 fallthrough, GET-only method filtering by construction, 19 inline unit tests on the handler.
- **Not built:** actual header parsing (header lines past `[0]` are thrown away), body reading (line is commented out), Content-Length handling, integration tests, thread pool, keep-alive (every response is `Connection: close`), epoll/kqueue/io_uring, TLS, HTTP/2, WebSockets, SSE, UDP, QUIC.

Scale: 9 Rust files, ~14.7KB of Rust (~500 lines including tests), 1 runtime dependency (`chrono`), 25 commits across the repo's lifetime. The README is 48KB / 1788 lines — bigger than all code combined by ~3.3x. Roughly 3 of 30 milestones have meaningful code (10%); every README checkbox is unchecked. The "Zyphos" brand is in the README only — the Cargo package is still `multithreaded_http_server` v0.2.0.

## Current state

Status: **dormant** (LifeOS Overview.md frames the project as side-learning with a 4-month silence cadence). Last meaningful activity at HEAD `694ff01` on 2025-12-13: a single day adding 19 unit tests + minor handler tweaks (+213 LOC), labelled "fixed handler" but actually a test-suite addition. Commit cadence has two modes: concentrated bursts (5-10 commits over 1-3 days) and 1-4 month dormant periods. The November 2025 burst added ~2200 lines to the README (milestones, conceptual exploration) without touching code, then added the `/echo` route. No active in-flight work captured in LifeOS — there is no `Work/` folder for Zyphos.

## Gaps and known limitations

LifeOS Gaps.md enumerates 26 named gaps grouped by severity. The career-relevant subset:

- **Latent bugs in shipped code:** trailing CRLFs after the response body (off-spec, benign against real clients but flags strict validators); fixed 1024-byte read buffer silently truncates large requests (a realistic Host + User-Agent + Cookie set already approaches this); request body is discarded (the body slice is commented out at `src/handler.rs:63`); `stream.read().expect(...)` panics on any read error (a flaky network or `curl --max-time 0.1` triggers it).
- **Structural gaps blocking milestone progress:** no header parsing (only `header_lines[0]` is used); no Content-Length handling (consequence of the above); only GET supported (router 404s every other method); no URL decoding (`%20` is preserved verbatim); no query string parsing (`?param=value` becomes part of the path); unbounded thread spawn (DoS-trivial); `Connection: close` hardcoded; no shutdown signalling.
- **Correctness and consistency:** inconsistent timezones (`Utc::now()` for Date header, `Local::now()` for `/time` body); `status_code: i32` is the wrong type (should be `u16` — HTTP codes are 100-599); hardcoded `HTTP/1.1` status line regardless of request version; reserved `Server` header slot never populated; interleaved `println!` logging under concurrency; no client-address logging despite `stream.peer_addr()` being available; no `Host` header usage (vhosting impossible).
- **Cosmetic / maintenance:** Cargo package name (`multithreaded_http_server`) does not match repo name (`Zyphos`); no CI; no `rustfmt.toml`/`clippy.toml`; commit message quality varies ("test", "nvim test", "fixed handler", "latest changes, dont know what" all in history).
- **Testing gaps:** `main.rs` has zero coverage (accept loop, spawn, panic recovery untested); `response.rs` has no byte-format tests; `create_responses.rs` has no Content-Length-equals-body-bytes assertion; no concurrency tests; no fuzz tests; no integration tests (no `tests/` directory).
- **Risks LifeOS Suggestions.md flags as "will bite if ignored":** unbounded `thread::spawn` (a few-thousand-connection load test exhausts the OS thread budget — default `ulimit -u` is often 1024-4096 — and `catch_unwind` does not catch thread-creation failures); 1024-byte buffer ("works on localhost, fails on public internet"); no read timeout (slowloris-trivial DoS via TCP-connect-and-never-send).

The 27 unbuilt milestones span everything from M6 (thread pool) through to M30 (QUIC + 0-RTT). LifeOS Milestones.md explicitly notes that completing the ladder represents months-to-years of work at the project's natural cadence and that overstating progress is the failure mode to avoid.

## Direction (in-flight, not wishlist)

LifeOS Roadmap.md sequences the next sessions if Caner returns to Zyphos. None of these are in flight today (the project has been dormant since 2025-12-13), so this section reads as "the realistic next steps when picked up again" rather than active work:

1. **Session 1 — Close M4 (header parsing + body reading).** The biggest single gap. Add a `Headers` type, parse `header_lines[1..]` into it, case-fold keys for lookup, extract Content-Length, re-enable the commented-out `body_section`, read exactly Content-Length bytes, pass body into the router, add 5-10 new tests. Unlocks POST and any header-aware behaviour; required for M8 keep-alive later.
2. **Session 2 — Close the M1 socket-option gaps.** ~30 lines: SO_REUSEADDR on the listener, TCP_NODELAY on accepted streams, replace `expect()` with `Result` handling, graceful Ctrl-C shutdown, a TCP-level integration test.
3. **Session 3 — First method expansion (POST).** Extend router match to handle POST, add a `/echo-body` route that echoes the request body, add tests. Tests M4's body handling end-to-end.
4. **Session 4 — Thread pool (M6).** Fixed-size pool with `std::sync::mpsc` or crossbeam channels, replaces `thread::spawn` in `main.rs`, graceful drain on shutdown, queue-depth and active-worker metrics.
5. **Session 5 — HTTP/1.1 keep-alive (M8).** Read `Connection:` header, conditionally emit `Connection: keep-alive`, refactor `handle_connection` into a read-loop bounded by Connection: close or max-requests, add idle timeout via `set_read_timeout`, new tests for multiple-requests-per-connection and timeout behaviour.

Aspirational beyond Session 5 (LifeOS Roadmap.md treats M11+ as a "year+ out" backlog, not direction): M9 epoll/kqueue event loop (significant architectural pivot — likely "Zyphos v2"), M13 trie/radix router, M19-M22 security hardening, M22 TLS (would relax D1 for `rustls`), M23-M24 HTTP/2, M25 WebSockets, M27-M30 UDP/QUIC.

## Demonstrated skills

Specific, evidence-anchored capabilities this project proves Caner has exercised:

- **Implements an HTTP/1.x server in safe Rust from raw `std::net` primitives** with no web framework, no async runtime, and no parsing crate — every concept (sockets, parsing, framing, dispatch, serialisation) is handwritten.
- **Writes the thread-per-connection concurrency model with panic recovery** — uses `panic::catch_unwind(AssertUnwindSafe(...))` plus the standard double-downcast (`&str`, then `String`) for panic-payload extraction, keeping the listener alive across handler bugs.
- **Designs clean module boundaries on a small project** — five files with strict direction rules: `main.rs` is the only file touching `std::net`/`std::thread`; `handler.rs` is the only file seeing the raw string; `router.rs` is the only file mapping (method, path) → builder; `response.rs` owns serialisation; `create_responses.rs` is the sole header-injection point. A request crosses exactly four module boundaries with a typed signature at each.
- **Reasons explicitly about design alternatives and records the why** — LifeOS Decisions.md captures ten decisions with alternatives considered, why this won, costs being paid, and what would flip the call. The std-only constraint, thread-per-connection over pool, HashMap-with-deterministic-ordering over Vec/IndexMap/BTreeMap, body-as-String over Vec<u8>, and inline tests over integration are all reasoned through.
- **Produces a deterministic wire-format serialiser over an unordered backing store** — `format_response` writes a hardcoded "important headers" list first, then iterates the remaining HashMap, recovering deterministic order without an ordered map dependency.
- **Knows the byte-correctness gotcha for HTTP Content-Length in Rust** — uses `String::len()` (which returns bytes, not chars) so Content-Length is automatically correct for any UTF-8 body. LifeOS notes this as a Rust-specific win that hand-rolled HTTP servers in Go or Python frequently get wrong.
- **Writes characterisation tests pinning current behaviour** — 19 inline `#[cfg(test)]` tests covering the request-parser surface (separator handling, request-line tokenisation, version prefix, exact and prefix routing, special characters in path, double-space tolerance, case-sensitive method, query-string passthrough, multiple-headers tolerance, POST 404). Tests are honest about being characterisation rather than spec-driven (some carry "might be error or might work depending on your implementation choice" comments).
- **Identifies and documents latent bugs without fixing them mid-extraction** — the trailing-CRLF-after-body bug (`"{}{}\r\n{}\r\n\r\n"` produces an extra CRLF pair), `status_code: i32` defensive-typing miss (should be `u16`), `Server` header slot reserved-but-never-populated, `expect()` on socket reads, hardcoded `HTTP/1.1` regardless of request version, fixed 1024-byte buffer silently truncating large requests — all are catalogued in LifeOS Gaps.md with severity, fix size, and consequences rather than papered over.
- **Maintains commit-cadence discipline appropriate for a side-learning project** — accepts the burst-then-dormant rhythm (1-3 day bursts separated by 1-4 month silences) and sizes next-session goals to fit in one burst rather than planning linear weekly progress.
- **Maps a deep learning curriculum to a concrete 30-milestone ladder** — partitions network programming into seven phases (foundations, concurrency, advanced parsing, kernel bypass, security, modern protocols, alternative protocols) and resists the temptation to overstate progress (every README checkbox is honestly unchecked; LifeOS frames the project as "M3-ish on a 30-step ladder", not "in development on QUIC").

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Zyphos/Overview.md | 106 | "#project #zyphos #rust #networking #http #learning-project #solo" |
| Projects/Zyphos/Architecture.md | 165 | "- [[Decisions]] — why the seams are where they are" |
| Projects/Zyphos/Decisions.md | 181 | "- [[Roadmap]] — which decisions will need revisiting" |
| Projects/Zyphos/Gaps.md | 180 | "- [[Suggestions]] — opportunistic improvements beyond these specific gaps" |
| Projects/Zyphos/Milestones.md | 174 | "- [[Systems/Routing#What This Domain Still Needs to Hit M5\|Systems/Routing: M5 exit criteria]]" |
| Projects/Zyphos/Roadmap.md | 150 | "- [[Suggestions]] — ideas outside the milestone ladder" |
| Projects/Zyphos/Suggestions.md | 135 | "- [[Systems/Testing]] — R3 (timeouts) and O6 (integration tests) live here" |
| Projects/Zyphos/Systems/Connection Handling.md | 152 | "- [[Milestones#Milestone 3 Thread-per-Connection Model\|Milestones: M3 detail]]" |
| Projects/Zyphos/Systems/Request Parsing.md | 119 | "- [[Gaps#Request parsing gaps\|Gaps: missing headers, body, Content-Length]]" |
| Projects/Zyphos/Systems/Response Pipeline.md | 142 | "- [[Gaps#Response pipeline gaps\|Gaps: trailing CRLF, Server header, binary bodies]]" |
| Projects/Zyphos/Systems/Routing.md | 168 | "- [[Gaps#Routing gaps\|Gaps: POST/PUT/DELETE, query strings, URL decoding]]" |
| Projects/Zyphos/Systems/Testing.md | 113 | "- [[Roadmap]] — test priorities in the next session" |
