---
name: Zyphos
status: dormant
source_repo: https://github.com/Capataina/Zyphos
lifeos_folder: Projects/Zyphos
last_synced: 2026-05-13
sources_read: 14
---

# Zyphos

## One-line summary

Bottom-up HTTP/1.1 server in safe Rust built from raw `std::net` sockets — currently a thread-per-connection echo server with hand-written request-line parsing and a typed response pipeline, scaffolded as a 30-milestone networking-protocols learning ladder from TCP through QUIC.

## What it is

Zyphos is a Rust network-programming learning laboratory whose stated mission is to learn sockets, HTTP, and modern network protocols end-to-end by implementing an HTTP server from raw TCP up, progressively layering in production techniques (thread pools, zero-copy, SIMD parsing, HTTP/2, QUIC). The design ambition is a 30-milestone ladder across 7 phases — Network Foundations, Concurrency and Performance, Advanced Parsing and Optimisation, Kernel Bypass and Advanced I/O, Security and Robustness, Modern Protocols, and UDP / Alternative Protocols. The deliberate constraint throughout the entire ladder is `std`-only plus `chrono` — no `hyper`, no `axum`, no `tokio`, no `mio` — so that every concept must be built from primitives rather than pulled in as a dependency. What it currently demonstrates is the first three discrete topics on that ladder (M1, M3, M5) implemented as a working thread-per-connection HTTP/1.1 echo server with three GET routes (`/hello`, `/time`, `/echo/{text}`), an inline `#[cfg(test)] mod tests` regression suite, and `panic::catch_unwind`-isolated request handling. The repo name is `Capataina/Zyphos` but the Cargo package is still `multithreaded_http_server` at version 0.2.0 — a vestigial naming artefact from before the 2025-11 README rewrite introduced the Zyphos brand. The project's natural rhythm is concentrated 1-3 day commit bursts punctuated by 1-4 month dormant periods, so a future session should treat the next milestone as something to finish in a single focused day rather than as a weekly cadence.

## Architecture

Single-binary Rust application with four-module dependency direction from `main.rs` to the response pipeline. The boundaries are sharp enough that a request crosses exactly four module boundaries on its way to a response, and each boundary has a well-defined type.

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

**Direction rules currently held:**

1. `main.rs` is the only file that touches `std::net` and `std::thread`.
2. `handler.rs` is the only file that sees the raw `&str` request.
3. `router.rs` is the only file that maps `(method, path) → HttpResponse` builder.
4. `routes/*.rs` produce typed `HttpResponse` values; they do not serialise.
5. `response.rs` owns the `HttpResponse` struct and the wire-format serialiser.
6. `create_responses.rs` is the factory layer — the only file that injects `Content-Type`, `Content-Length`, `Connection`, `Date`.

**Request lifecycle on the wire:** `TcpListener::bind("localhost:3000")` accepts a stream → atomic `CONNECTION_COUNTER.fetch_add(1, SeqCst)` produces a monotonic connection ID → `thread::spawn(move || ...)` transfers ownership of the stream into a new OS thread → `panic::catch_unwind(AssertUnwindSafe(|| handle_connection(stream)))` wraps the work so a bad request cannot bring the server down → `stream.read(&mut [0; 1024])` performs one blocking read → `String::from_utf8_lossy` decodes the bytes (lossy: non-UTF-8 becomes `U+FFFD`) → `handle_request(&str)` finds the `\r\n\r\n` head/body separator, takes line 0 as the request line, `split_whitespace`-tokenises into `[method, path, version]`, validates three-token shape and `HTTP/` version prefix → `route(method, path)` performs exact-match dispatch (with `strip_prefix("/echo/")` for the one parametric route) → typed `HttpResponse` flows back → `format_response` writes the status line, ordered "important headers" (`Content-Type`, `Content-Length`, `Connection`, `Date`, `Server`), then any other headers from `HashMap` iteration, then the body → `stream.write_all` + `stream.flush` → thread exits, stream is RAII-closed.

**HttpResponse data shape:**

```rust
pub struct HttpResponse {
    pub status_code: i32,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}
```

Status code is signed `i32` rather than `u16` (a defensive-typing miss); headers are an unordered `HashMap` with deterministic wire order reconstructed by a hardcoded "important headers" list in `format_response`; body is `String` rather than `Vec<u8>`, which forecloses binary payloads without a future refactor.

**Dependency surface:** one runtime dependency — `chrono = "0.4"` — used for the RFC 1123 `Date` header in `create_responses.rs` (`Utc::now()`) and the human-readable timestamp in `routes/time.rs` (`Local::now()`). Zero test dependencies. The `Cargo.lock` is almost entirely `chrono`'s transitive tree.

**Architectural invariants currently enforced:** request must fit in 1024 bytes (anything larger is silently truncated); request bytes must decode as UTF-8 or become `U+FFFD`; `Connection: close` is hardcoded in every response, foreclosing keep-alive without a rewrite of `handle_connection`; one request per connection (consequence of `Connection: close` plus the handler returning a single `String`); the wire status line is always `HTTP/1.1` regardless of request version; thread spawning is unbounded with no concurrency limit.

## Subsystems and components

### Connection Handling (`main.rs`)

The entry point and the only file in the project that touches `std::net`, `std::thread`, or sync primitives. Owns the listener lifecycle, per-connection thread spawning, panic recovery, and logging. The pattern is textbook "naïve HTTP server" — `for stream_result in listener.incoming()` runs forever; each accepted stream gets its own OS thread; the thread wraps `handle_connection(stream)` in `panic::catch_unwind(AssertUnwindSafe(...))` and downcasts the panic payload to `&str` then `String` on failure, logging whichever matches. Per-connection cost: ~2MB default thread stack + OS thread. The accept loop has no shutdown signalling, no backpressure, and no concurrency limit. `handle_connection` itself is trivially blocking — one `stream.read` into a fixed `[0; 1024]` buffer, one `String::from_utf8_lossy` decode, one `handle_request` call, one `stream.write_all`, one `stream.flush`, no read-loop. Around 50 lines of actual logic.

### Request Parsing (`handler.rs`)

Owns the transition from raw `&str` to a dispatched `HttpResponse`. "Parsing" is generous — what happens is: `find("\r\n\r\n")` to locate the head/body separator, `lines().collect()` to gather header lines, take `header_lines[0]` as the request line, `split_whitespace().collect()` to tokenise it, validate token count is 3, validate version token starts with `"HTTP/"`, call `route(method, path)`. Headers past the request line are collected into `Vec<&str>` but never interpreted. The body section is computed positionally but the binding (`let body_section = &raw_request[pos + HEAD_BODY_SEPARATOR.len()..];`) is commented out — bodies are silently discarded. Three structural-validation 400 paths plus the router's 404 path are the only failure shapes the handler emits. `split_whitespace` collapses whitespace runs, so `"GET  /hello  HTTP/1.1"` (double space) is tolerated — pinned by a test. Production logic is ~106 lines; the file totals 298 lines including the inline test module.

### Routing (`router.rs`)

Eighteen lines of hand-written if/else dispatch. Three routes: exact match for `/hello` (delegates to `hello::handle()`), exact match for `/time` (delegates to `time::handle()`), prefix strip via `strip_prefix("/echo/")` for `/echo/{text}` (delegates to `echo::handle(text)`). Every non-GET method falls through to the 404 path; every unknown GET path falls through to the same 404 path. No query-string parsing — `/echo/test?x=y` echoes `"test?x=y"` (pinned by test). No URL decoding — `%20` passes through verbatim (pinned by test). Case-sensitive method check — `"get"` 404s (pinned by test). Adding a new route requires three edits: `use` import in `router.rs`, branch in the `if/else` chain, re-export in `routes/mod.rs`. The route handler files (`routes/hello.rs`, `routes/time.rs`, `routes/echo.rs`) are each effectively one line — they build a body string and delegate to `create_text_response`.

### Response Pipeline (`response.rs` + `create_responses.rs`)

Two files, the cleanest seam in the project. `response.rs` owns the `HttpResponse` struct and `format_response(HttpResponse) -> String` serialiser; `create_responses.rs` owns `create_text_response(body)` (200 OK factory), `create_error_response(code, text, body)` (400/404 factory), and `get_http_date()` (RFC 1123-formatted `chrono::Utc::now()`). Both factories populate an identical four-header set — `Content-Type: text/plain`, `Content-Length: body.len()` (Rust's `String::len()` returns bytes, which is exactly what HTTP wants), `Connection: close`, `Date: <RFC 1123>`. `format_response` walks a hardcoded `["Content-Type", "Content-Length", "Connection", "Date", "Server"]` ordering list before draining the remaining `HashMap` entries — recovers deterministic wire order from an unordered store. The `"Server"` slot is reserved in the ordering list but never populated by any factory. A latent off-spec quirk: the format string `"{}{}\r\n{}\r\n\r\n"` emits a trailing `\r\n\r\n` after the body, on top of the `\r\n` that terminates the last header — benign against real clients but will fail a strict validator. Response-side code has been stable for six months — all December 2025 activity was in `handler.rs`.

### Testing (inline `#[cfg(test)] mod tests` in `handler.rs`)

19 unit tests (20 functions if counting one weak-assertion test) all inline in `src/handler.rs`, all added in the single commit `694ff01` (2025-12-13, "fixed handler", +197 LOC). Each test constructs a raw request `&str` and calls `handle_request` directly. No mocks, no fixtures, no setup/teardown. All assertions are `assert!(response.contains(...))` — substring-matching. Coverage is concentrated in the request-parsing layer: 5 tests on request-line validation, 3 on separator handling, 5 on routing dispatch, 4 on echo parameter extraction, 2 on method filtering. Zero tests touch `response.rs`, `create_responses.rs`, or the `main.rs` accept/spawn loop. No `tests/` directory. No `.github/workflows/` CI. No benchmarks. No fuzz tests. The tests are characterisation tests pinning current behaviour, not spec-driven tests asserting RFC compliance — several carry comments like "might be error or might work / depending on your implementation choice".

## Technologies and concepts demonstrated

### Languages

- **Rust** — the sole implementation language. Used across 9 files / ~500 lines (including tests), exercising `std::net::TcpListener`/`TcpStream`, `std::thread::spawn`, `std::sync::atomic::AtomicUsize` with `SeqCst` ordering, `std::panic::{catch_unwind, AssertUnwindSafe}`, `String::from_utf8_lossy`, `split_whitespace`, `strip_prefix`, `HashMap`, ownership transfer via `move` closures, and RAII drop for socket close. The project's `std`-only discipline means every Rust primitive used here is the bare standard-library form, not a higher-level abstraction.

### Frameworks and libraries

- **None.** No web framework, no async runtime, no HTTP library. This is deliberate — D1 in `Decisions.md` formalises the constraint: "The moment Zyphos depends on tokio or hyper, it stops teaching the thing it is for."

### Runtimes / engines / platforms

- **OS-thread runtime (`std::thread::spawn`)** — thread-per-connection model with no pool, no work-stealing, no scheduling. Each connection consumes the platform default stack (~2MB on Linux).
- **Blocking I/O** — `TcpStream::read` and `write_all` block the calling thread; no epoll, no kqueue, no `io_uring`, no `mio`.

### Tools

- **chrono 0.4** — date/time formatting (`Utc::now().format("%a, %d %b %Y %H:%M:%S GMT")` for the `Date` header; `Local::now().format("%d/%m/%Y %T")` for the `/time` route body).
- **`cargo test`** — runs the inline `#[cfg(test)]` suite. No CI, no `cargo bench`, no `cargo fmt` config, no `cargo clippy` config.

### Domains and concepts

- **TCP socket programming from primitives** — `TcpListener::bind`, `listener.incoming()`, accepting streams, owning the listener lifecycle without a framework wrapper.
- **Thread-per-connection concurrency** — the textbook "naïve HTTP server" baseline; OS-thread per accepted connection with `move`-ownership of the stream.
- **Lock-free atomic counters** — `AtomicUsize::fetch_add(1, SeqCst)` for monotonic connection IDs. The only lock-free construct in the project.
- **Panic isolation via `catch_unwind`** — wrapping per-request work in `panic::catch_unwind(AssertUnwindSafe(...))` so a bad request cannot kill the server; double-downcast of panic payload to `&str` then `String` is the standard Rust idiom for extracting the message.
- **HTTP/1.1 wire-format serialisation by hand** — building status lines, header sections, and bodies as raw strings; ordering headers deterministically out of an unordered `HashMap` via a hardcoded "important headers" list.
- **RFC 1123 / IMF-fixdate date formatting** — for the HTTP `Date` header.
- **HTTP request-line parsing** — three-token validation (`method path version`), version prefix check, head/body separator on `\r\n\r\n`. Headers past line 0 are read but not interpreted; bodies are read but discarded.
- **Exact-match routing with one prefix-strip parametric route** — closed-enumeration if/else dispatch over `(method, path)` pairs; `strip_prefix` extracts the parameter for `/echo/{text}`.
- **Module-boundary discipline** — sharp seams between net I/O (`main.rs`), parsing (`handler.rs`), dispatch (`router.rs`), route logic (`routes/*.rs`), response factories (`create_responses.rs`), and wire serialisation (`response.rs`); each module has a single responsibility and the dependency direction is unidirectional.
- **Inline `#[cfg(test)] mod tests` regression testing** — characterisation tests pinning current behaviour through `assert!(response.contains(...))` substring assertions.

## Key technical decisions

**D1 — Rust + `std` only, no web framework.** Chose to depend only on `chrono`. Rejected `hyper`/`reqwest` (would hide sockets, parsing, framing), `tokio` (would bypass the blocking-then-thread-pool-then-epoll progression), and `mio` (still skips writing the event loop from scratch). The constraint exists because the project's pedagogical contract is "every concept must be built, not pulled in"; pulling in `tokio` or `hyper` would defeat the entire learning intent. Flippable only if Caner ever wants a production-ready outcome — at that point a rewrite onto `tokio` for the production version is sensible, but the learning version stays `std`-only.

**D2 — Thread-per-connection, not thread-pool.** Every accepted connection gets its own OS thread via `thread::spawn`. Rejected thread pools (the M6 target — deferred until M5 lands), async runtimes (out of scope per D1), and event loops (M9 target). This is the textbook "naïve HTTP server" baseline — correct, simple, and exactly the shape M3 asks for. Costs already being paid: ~2MB stack per thread (scales poorly past ~1000-5000 connections), no ability to apply backpressure, thread startup latency on every connection. Flippable on reaching M6.

**D3 — `panic::catch_unwind` around request handling.** Wraps `handle_connection` inside `panic::catch_unwind(AssertUnwindSafe(...))` and logs the panic message without killing the server. Rejected: letting panics propagate (one bad request kills the process), `Result`-based propagation everywhere (correct but verbose), supervisor-pattern log-and-exit (out of scope; no process supervisor). The pragmatic choice for a learning project — Caner can iterate on the parser without re-starting. Will need to flip toward let-panic-kill-process behaviour around M21 (timeouts/backpressure), where production-shaped operation expects a panic to surface for a supervisor restart.

**D4 — `HashMap` headers with deterministic serialisation order.** Headers stored in `HashMap<String, String>` (unordered); `format_response` reconstructs deterministic wire order by writing a hardcoded "important headers" list first, then iterating the remaining map. Rejected: `Vec<(String, String)>` (preserves insertion order but O(n) lookup), `IndexMap` (best of both, violates the `std`-only D1), `BTreeMap` (alphabetical sort, not spec-idiomatic). Pragmatic, not principled — adding a new "important" header requires editing two places. Flippable if M8 (keep-alive) or M23 (HTTP/2) makes the important-list unwieldy.

**D5 — Hardcoded if/else router over a trie.** Three routes dispatched via a 15-line if/else chain. Rejected: `HashMap` of `(method, path) → fn` (no prefix matching), trie / radix tree (the M13 target, overkill for 3 routes), regex routing. The minimum correct implementation for M5's exact-match exit criterion. Flippable at ~10+ routes or the introduction of multi-segment path parameters like `/users/{id}/posts/{postId}` — that is M13 territory.

**D6 — `body: String` in `HttpResponse`.** Response bodies typed as `String`; all routes produce `text/plain`. Rejected: `Vec<u8>` (binary-compatible from day one), `Body::Text | Body::Bytes | Body::File` enum (richest), `&'a [u8]` (avoids allocation, costs lifetime complexity). Simplest possible response type for M4-M5; UTF-8 text is the only payload in current routes; `String::len()` returning byte length means `Content-Length` is automatically correct. Will need to flip when M14 (caching), M16 (sendfile / static files), or M25 (WebSocket frames) arrives.

**D7 — `Connection: close` on every response.** All factories hardcode `Connection: close`. Rejected: omitting the header (HTTP/1.1 default is keep-alive — would be a protocol violation given the one-shot handler), claiming `keep-alive` (false advertising without a connection-reuse loop). The correct shape for M4-M5. Flippable on arrival of M8 (HTTP/1.1 keep-alive), which requires restructuring `handle_connection` from one-request-per-call into a read-loop with connection-alive tracking.

**D8 — `split_whitespace()` tolerance in request-line parsing.** Request line tokenised with `split_whitespace()`, which collapses runs and strips leading/trailing whitespace. Rejected: `split(' ')` (strictly spec-compliant; rejects `"GET  /hello HTTP/1.1"` with double space), byte-level state machine. Laziness with small upside — more robust to slightly malformed clients. Will need to become explicit-with-mode (strict vs lax) at M20 (parser security / differential testing).

**D9 — `String::from_utf8_lossy` for request bytes.** Bytes read from the socket decoded as UTF-8 with invalid sequences becoming `U+FFFD`. Rejected: `std::str::from_utf8` (returns `Result`; rejects non-UTF-8 with an error), `&[u8]`-throughout parsing. UTF-8 is the expected encoding for HTTP request lines and headers; lossy decode means a non-UTF-8 byte in a URL doesn't crash. Will need to flip to `&[u8]`-based parsing for the body section at minimum when M4 body reading lands.

**D10 — Inline `#[cfg(test)]` tests, no integration tests.** 19 tests live in a `mod tests` block inside `src/handler.rs`. No `tests/` directory. Rejected: separate `tests/handler_integration.rs`, hybrid layout. Minimal test infrastructure, maximal locality — natural for `&str → String` pure functions. The missing integration tests reflect the fact that `main.rs` (accept loop, panic recovery, thread behaviour) is 0% covered.

## What is currently built

- **TCP listener and accept loop** on `localhost:3000`. No `SO_REUSEADDR`, no `TCP_NODELAY`, no `EINTR`/`EAGAIN` handling, no graceful shutdown.
- **Thread-per-connection** via `thread::spawn` with ownership transfer through `move`.
- **Monotonic atomic connection counter** (`AtomicUsize::fetch_add(1, SeqCst)`).
- **Per-thread panic recovery** via `panic::catch_unwind(AssertUnwindSafe(...))` with `&str`/`String` payload downcast.
- **HTTP request-line parsing** — three-token validation, `HTTP/` prefix check, two 400 paths.
- **Head/body separator detection** on `\r\n\r\n` with a 400 fallthrough.
- **Header / body line splitting** — header lines collected into `Vec<&str>` but only `[0]` (the request line) is used; body section computed positionally but assignment is commented out.
- **`HttpResponse` struct** — `status_code: i32`, `status_text: String`, `headers: HashMap<String, String>`, `body: String`.
- **Wire-format serialisation** via `format_response` with deterministic ordering of `Content-Type` / `Content-Length` / `Connection` / `Date` / `Server` headers ahead of the remaining `HashMap` entries.
- **Two factories** — `create_text_response` (200 OK + four-header set) and `create_error_response` (400/404 + same headers).
- **RFC 1123 `Date` header** from `chrono::Utc::now()`.
- **Three GET routes** — `/hello` (returns `"Hello World!"`), `/time` (returns `Local::now().format("%d/%m/%Y %T")`), `/echo/{text}` (echoes the prefix-stripped tail).
- **Catch-all 404** for unknown paths and any non-GET method.
- **19 inline `#[cfg(test)]` regression tests** in `handler.rs` covering request-line validation, separator handling, routing dispatch, echo parameter extraction, method filtering, and HTTP version variations.

**Scale snapshot:** 9 Rust files, ~14.7KB of Rust (roughly 500 lines including tests), 48KB README (~3.3x the size of all code combined), 1 runtime dependency (`chrono`), 25 commits across the repo's lifetime spanning 2025-06-14 to 2025-12-13. The README grew ~2200 lines in November 2025 while the code barely moved — the project is currently overwhelmingly a detailed learning plan with a tiny demonstrator server attached. Of the README's 30-milestone ladder, roughly 3 milestones have meaningful code (10%); the remaining 27 exist only as plan text. None of the README's 30 checkboxes is marked complete.

## Current state

Status: dormant. Last meaningful commit `694ff01` (2025-12-13) added the 19-test handler suite plus minor logic tweaks; the previous active session was 2025-11-17 (added the `/echo/` route). No work has landed since 2025-12-13 — 4+ months of silence as of the LifeOS verification date. The project's documented natural rhythm is concentrated bursts of 1-3 days where 5-10 commits land, separated by 1-4 month dormant periods; no work is in flight at the moment LifeOS notes were last verified. The LifeOS notes themselves treat Zyphos as ranked below Cernio, Aurix, Flat Browser, and NeuroDrive in portfolio priority.

## Gaps and known limitations

**Critical — latent bugs in shipped code:**

- **Trailing CRLFs after response body.** `format_response` emits `"{}{}\r\n{}\r\n\r\n"`, producing `STATUS\r\nHDR1\r\nHDR2\r\n\r\nBODY\r\n\r\n`. The trailing `\r\n\r\n` after the body is off-spec — benign against real clients but will fail a strict HTTP validator. One-character fix.
- **Fixed 1024-byte read buffer silently truncates large requests.** `let mut buffer = [0; 1024];` in `main.rs`. A realistic browser request with Host + User-Agent + Cookie + Accept can exceed 1024 bytes; truncated reads then fail the `\r\n\r\n` separator check and 400.
- **Request body is discarded.** Line 63 of `handler.rs` has the body-section binding commented out. Any POST/PUT/PATCH body is lost. Harmless today because the router 404s all non-GET methods, but bites the moment POST is added.
- **`stream.read().expect(...)` panics on any read error.** If the client closes mid-request or a network blip occurs, `read` returns `Err`, `expect` panics, and `catch_unwind` recovers but loses the error detail. Small fix — match on `Result`.

**High — structural gaps blocking milestone progress:**

- No header parsing — `header_lines[1..]` are thrown away.
- No `Content-Length` handling (consequence of the above).
- Only GET is supported — router 404s any non-GET method.
- No URL decoding — `%20` passes through raw.
- No query-string parsing — `/echo/test?param=value` treats `test?param=value` as the path argument.
- Unbounded `thread::spawn` in the accept loop — DoS-trivial.
- `Connection: close` hardcoded — forecloses HTTP/1.1 keep-alive without a `handle_connection` rewrite.
- No shutdown signalling — `for stream_result in listener.incoming()` loops forever, no SIGINT/SIGTERM handling.

**Medium — correctness and consistency:**

- Inconsistent timezones — `Date` header uses `Utc::now()` but `/time` route body uses `Local::now()`.
- `status_code: i32` is the wrong type (should be `u16`; HTTP codes are 100-599 and `i32` allows negative).
- Hardcoded `HTTP/1.1` status line regardless of request version — violates strict M4 interpretation.
- `Server` header slot is reserved in `format_response`'s ordering list but never populated by any factory.
- Logging is interleaved `println!` from multiple threads — produces tangled lines under concurrent load.
- No client address logged on accept (`stream.peer_addr()` is available but unused).
- `String::from_utf8_lossy` can mask invalid request bytes — silent corruption.
- `Host` header is never read — vhosting is impossible; the server only works on `localhost:3000`.

**Low — naming and maintenance:**

- Cargo package name is `multithreaded_http_server` while the repo is `Zyphos` — vestigial.
- No CI (`.github/workflows/` does not exist); `cargo test` is run manually.
- No `rustfmt.toml` / `clippy.toml` configuration.
- Several commit messages are weak (`"test"`, `"nvim test"`, `"fixed handler"`, `"latest changes, dont know what"`).

**Testing gaps:** `main.rs` is 0% covered (accept loop, spawning, panic recovery untested); `response.rs` has no byte-format tests; `create_responses.rs` has no assertions that `Content-Length` matches body bytes; no concurrency tests; no fuzz tests; no integration tests (no `tests/` directory).

**Milestone-level gaps:** Everything in README Phases 2-7 (M6-M30) is unbuilt — no thread pool, no memory pools, no HTTP/1.1 keep-alive, no epoll/kqueue, no SIMD parser, no lock-free metrics beyond the single counter, no trie router, no caching, no `io_uring`, no `sendfile`, no `SO_REUSEPORT`, no rate limiting, no parser-security work, no timeouts, no TLS, no HTTP/2, no WebSockets, no SSE, no UDP, no multicast, no QUIC.

## Direction (in-flight, not wishlist)

The pragmatic next-session ordering captured in LifeOS Roadmap.md, sized for single focused sessions of half-day to a full day each:

1. **Close M4 — header parsing + body reading.** Add a `Headers` type (`Vec<(String, String)>` or `HashMap<String, String>`), parse `header_lines[1..]` into it case-fold-keyed for lookup, extract and validate `Content-Length`, uncomment the body-section binding, read exactly `Content-Length` bytes of body, pass body into the router alongside method and path, add 5-10 new tests covering header parsing and body reading. This is the biggest single gap in the codebase and required for M8 keep-alive later.
2. **Close M1 socket-option gaps.** Set `SO_REUSEADDR` on the listener and `TCP_NODELAY` on accepted streams, replace `expect()` on `read`/`write`/`flush` with `Result` handling, handle graceful shutdown via `Ctrl-C` (probably `ctrlc` crate, mildly violating D1, or raw signal handling), add a TCP-level integration test. ~30 lines plus a test file.
3. **First method expansion (POST).** Now that Session 1 made body reading real, add a POST route that echoes the request body, extend router match to handle POST. Tests M4's body handling end-to-end.
4. **Thread pool (M6).** Fixed-size pool with `std::sync::mpsc` channels replacing `thread::spawn` in `main.rs`; graceful shutdown via queue drain; metrics for queue depth and active workers. NOT in scope at this stage: work-stealing, per-thread local queues, task cancellation.
5. **HTTP/1.1 keep-alive (M8).** Read the `Connection:` header in `handler.rs`, conditionally emit `Connection: keep-alive` in `create_text_response`, refactor `handle_connection` into a read-loop bounded by `Connection: close` or a max-requests counter, add idle timeout via `TcpStream::set_read_timeout`, new tests for multiple requests on one connection and timeout behaviour. The moment Zyphos stops being a toy.

None of the above is actively being worked on at the moment LifeOS notes were last verified (2026-04-24) — this is the documented near-term plan, not in-flight scope. Anything past M10 is years out at the project's natural cadence; M11-M30 are treated by LifeOS as optional chapters.

## Demonstrated skills

- **Hand-rolled HTTP server in safe Rust with no framework dependency.** Demonstrates ability to build a working HTTP/1.1 server from `std::net` primitives, including listener lifecycle, accept loop, per-connection threading, request-line parsing, response serialisation, and three-route dispatch — all without `hyper`, `axum`, `tokio`, or `mio`. The discipline of building from primitives is a portfolio signal for systems-engineering roles where understanding the network stack matters more than wiring up frameworks.
- **OS-thread concurrency with panic isolation.** Demonstrates safe Rust concurrency patterns: ownership transfer through `move` closures, `panic::catch_unwind(AssertUnwindSafe(...))` with `&str`/`String` payload downcast, lock-free atomic counters via `AtomicUsize::fetch_add(1, SeqCst)` — all using `std` primitives, no external crate.
- **TCP socket programming from primitives.** Demonstrates direct work with `TcpListener::bind`, `listener.incoming()`, `TcpStream::read`/`write_all`/`flush`, awareness of the socket-option gaps (`SO_REUSEADDR`, `TCP_NODELAY`) that production code requires, and understanding of the trade-offs between blocking and event-driven I/O models.
- **HTTP/1.1 wire-format authorship.** Demonstrates byte-level construction of HTTP responses — status line, ordered headers, RFC 1123 `Date` formatting, `Content-Length` byte-correctness, and deterministic header serialisation from an unordered `HashMap` via a hardcoded "important headers" list. Demonstrates awareness that `String::len()` in Rust returns bytes (the right answer for `Content-Length`), which is a Rust-specific correctness win that distinguishes the code from a hand-rolled HTTP server in Go or Python.
- **Clean module-boundary discipline.** Demonstrates unidirectional dependency direction across four modules (`main.rs` → `handler.rs` → `router.rs` → `routes/*.rs`) plus a two-file response pipeline (`response.rs` + `create_responses.rs`), with each module owning exactly one responsibility and each request crossing four well-typed boundaries. Layering sharp enough that adding a future thread pool slots cleanly into `main.rs` and a future full header parser slots cleanly into `handler.rs` without ripple effects.
- **Inline characterisation testing in Rust.** Demonstrates use of the idiomatic `#[cfg(test)] mod tests` pattern with `use super::*;` to access module internals, substring-matching assertions for HTTP response shape, and characterisation-style coverage of edge cases (whitespace tolerance, case sensitivity, version-string variations, query-string-as-path artefacts) that pin current behaviour ahead of a richer parser rewrite.
- **Pragmatic decision-making with documented alternatives.** Demonstrates a habit of recording design decisions alongside the alternatives that were rejected and the conditions under which the decision would flip — the LifeOS Decisions.md captures ten such decisions (std-only stack, thread-per-connection, panic-catch, HashMap headers, if-else router, `String` body, hardcoded `Connection: close`, `split_whitespace` tolerance, `from_utf8_lossy`, inline tests) each with explicit "what would flip this" criteria.
- **Honest self-assessment of project state vs ambition.** Demonstrates separation of design ambition (30-milestone README ladder) from implemented scope (3 milestones with meaningful code) — the LifeOS notes maintain a deliberate anti-puffing stance distinguishing what the project is designed to teach from what it currently demonstrates.

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
