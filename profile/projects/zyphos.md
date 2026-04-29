---
name: Zyphos
status: dormant
source_repo: https://github.com/Capataina/Zyphos
lifeos_folder: Projects/Zyphos
last_synced: 2026-04-29
sources_read: 13
---

# Zyphos

## One-line summary

Rust network-protocol learning laboratory — working thread-per-connection HTTP/1.1 echo server with three routes (panic recovery, atomic connection counter, RFC 1123 date headers, request-line parser, response serialiser, exact-match + prefix-strip routing) on a 30-milestone learning ladder targeting raw TCP through QUIC, with everything beyond Milestone 5 being README plan, not code.

## What it is

Zyphos is Caner's network-programming learning laboratory built in Rust. The stated mission is to learn sockets, HTTP, and modern network protocols end-to-end by implementing an HTTP server from raw TCP up, progressively layering in production techniques (thread pools, zero-copy, SIMD parsing, HTTP/2, QUIC). The README promises a 30-milestone structured progression from raw sockets to QUIC. The code has advanced through the first handful only. Every checkbox in the README's Learning Roadmap is unchecked. In concrete terms: Zyphos is a working thread-per-connection HTTP/1.1 echo server with three routes and no persistence. Everything after basic routing — thread pools, epoll, SIMD, TLS, HTTP/2, WebSockets, UDP, QUIC — is README ambition, not code reality. GitHub repo is `Capataina/Zyphos`; Cargo package is `multithreaded_http_server` v0.2.0 (the "Zyphos" brand lives in README only — `cargo run` still boots `multithreaded_http_server`). 9 Rust files, 14.7KB of Rust (~500 lines including tests), 48KB README (~3.3× the size of all code combined). 25 commits 2025-06-14 → 2025-12-13.

## Architecture

```
zyphos/                          # Cargo package: multithreaded_http_server v0.2.0
├── Cargo.toml                  # 1 runtime dep: chrono = "0.4"
└── src/
    ├── main.rs                 # TCP listener + thread::spawn per connection + panic recovery
    ├── handler.rs              # Request parsing, header/body separator split, 19 inline tests
    ├── response.rs             # HttpResponse + wire serialisation
    ├── create_responses.rs     # Response factory: text + error + RFC 1123 date
    ├── router.rs               # Exact match + /echo/ prefix strip; GET only
    └── (small helpers)
```

Listener at `localhost:3000` (no SO_REUSEADDR, no TCP_NODELAY). Thread-per-connection via `thread::spawn`. Panic recovery via `panic::catch_unwind`. Atomic connection counter. Routes: `/hello`, `/time`, `/echo/{message}`. 404 fallback. All responses set `Connection: close`.

## Subsystems and components

| Subsystem | Responsibility | State |
|---|---|---|
| **Connection Handling** | TCP listener, thread-per-connection model with panic recovery, atomic connection counter | Working — naïve, unbounded thread::spawn |
| **Request Parsing** | Parse HTTP request line (method/path/version), split headers/body on `\r\n\r\n` | Working for request line; **header parsing not implemented** (header lines extracted but only `header_lines[0]` used); **body reading not implemented** (`body_section` commented out) |
| **Response Pipeline** | `HttpResponse` → wire format; text + error variants; RFC 1123-ish date header (`%a, %d %b %Y %H:%M:%S GMT`) | Working |
| **Routing** | Exact match for `/hello`, `/time`; prefix strip for `/echo/`; 404 fallback; GET only by construction | Working but GET-only |
| **Testing** | 19 `#[cfg(test)]` tests inline in `handler.rs`; no integration tests; no `tests/` directory | Working — `repo_stats.py` missed these because it looks for `tests/` directories |

## Technologies and concepts demonstrated

### Languages
- **Rust** — entire codebase, 9 files (~14.7KB).

### Libraries
- **chrono 0.4** — RFC 1123-ish date header.

### Domains and concepts
- **HTTP/1.1 parsing from raw TCP** — request-line split, header/body separator on `\r\n\r\n`, response serialisation.
- **Thread-per-connection model** — naïve form with `thread::spawn`, panic recovery via `panic::catch_unwind`, atomic connection counter.
- **TCP socket programming** — `localhost:3000` listener, accept loop, blocking `read`.
- **Routing patterns** — exact match + prefix strip; GET-only by construction.
- **Inline `#[cfg(test)]` test discipline** — 19 unit tests live with source rather than in a separate `tests/` directory.
- **30-milestone structured learning ladder** — README documents progressively-layered production techniques (thread pools, epoll, SIMD, TLS, HTTP/2, WebSockets, UDP, QUIC) as a learning curriculum.

## Key technical decisions

- **Thread-per-connection (naïve form) before thread pools** — pedagogical choice; the upgrade to a thread pool is Milestone 3.
- **Panic recovery via `panic::catch_unwind`** — prevents one connection's panic from killing the listener.
- **`Connection: close` on all responses** — keeps the connection lifecycle simple at this stage; keep-alive is a future milestone.
- **`split_whitespace` for request-line parsing** — same lazy parsing decision; will be revisited when full HTTP/1.0 compliance is needed.
- **Inline tests in `handler.rs` rather than `tests/`** — 19 tests live with source.
- **README volume vs code volume inverted in November 2025** — README grew 20× while the code barely moved (`5d61ff1`, `090d860` added ~2200 lines of milestone documentation in two days). Deliberate scaffolding move for a learning project.

## What is currently built

- 25 commits across 2025-06-14 → 2025-12-13.
- 9 Rust files, ~14.7KB.
- Working `localhost:3000` listener with thread-per-connection.
- Working request-line parsing.
- Working response serialisation with RFC 1123 date header.
- Working exact-match + `/echo/` prefix routing.
- 19 inline tests.
- `Connection: close` on all responses.

## Current state

Dormant. Last commit `694ff01` on 2025-12-13 ("latest changes, dont know what" + "fixed handler"); five and a half months without code at LifeOS verification.

## Gaps and known limitations

- **Header parsing not implemented** — header lines split but only `header_lines[0]` (request line) used.
- **Body reading not implemented** — `body_section` commented out; any POST request body is discarded.
- **GET-only by construction** — router returns 404 for any other method.
- **Hardcoded `HTTP/1.1` status line** — README frames the milestone as HTTP/1.0; actual generated responses are HTTP/1.1.
- **No SO_REUSEADDR, no TCP_NODELAY, no EINTR/EAGAIN handling, no graceful shutdown.**
- **No keep-alive, no linger, no timeout detection, no TIME_WAIT tracking.**
- **No thread pool** — unbounded `thread::spawn`.
- **No epoll/kqueue/io_uring** — blocking `read` in a thread.
- **No TLS, no HTTP/2, no WebSockets, no SSE, no UDP, no QUIC.**
- **No path parameters, no query string parsing, no URL decoding, no HEAD support.**
- **No integration tests** — `tests/` directory absent.

## Direction (in-flight, not wishlist)

Dormant. If revived, the sensible next step is finishing Milestone 4 (full header parsing into key-value map, Content-Length validation, body reading) before touching the shinier items like thread pools and epoll. Current handler is one semicolon away from being production-unusable: any POST with a body is 404'd because the router only accepts GET, but more fundamentally the body bytes are literally discarded.

## Demonstrated skills

- **Raw-socket TCP server in Rust** — `localhost:3000` listener with accept loop, thread-per-connection, panic recovery.
- **HTTP/1.1 parsing from scratch** — request-line split, header/body separator, response serialisation, RFC 1123-ish date headers.
- **Thread-per-connection concurrency model** — `thread::spawn` + `panic::catch_unwind` + atomic connection counter.
- **Routing pattern design** — exact match + prefix strip + 404 fallback + GET-only constraint.
- **Inline `#[cfg(test)]` test discipline** — 19 tests live with source.
- **Network-protocol learning curriculum design** — 30-milestone progressively-layered learning ladder from raw sockets through QUIC; documents the production techniques along the path.
- **Anti-puffing discipline** — vault notes explicitly enumerate which README claims are implemented vs roadmap; refuses to overstate the implementation past Milestone 5.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Zyphos/_Overview.md | 106 | "#project #zyphos #rust #networking #http #learning-project #solo" |
| Projects/Zyphos/Architecture.md | 165 | "- [[Decisions]] — why the seams are where they are" |
| Projects/Zyphos/Decisions.md | 181 | "- [[Roadmap]] — which decisions will need revisiting" |
| Projects/Zyphos/Gaps.md | 180 | "- [[Suggestions]] — opportunistic improvements beyond these specific gaps" |
| Projects/Zyphos/Milestones.md | 174 | "- [[Systems/Routing#What This Domain Still Needs to Hit M5|Systems/Routing: M5 exit criteria]]" |
| Projects/Zyphos/Roadmap.md | 150 | "- [[Suggestions]] — ideas outside the milestone ladder" |
| Projects/Zyphos/Suggestions.md | 135 | "- [[Systems/Testing]] — R3 (timeouts) and O6 (integration tests) live here" |
| Projects/Zyphos/Systems/_Overview.md | 40 | "- [[Projects/Zyphos/Roadmap]] — direction-of-travel" |
| Projects/Zyphos/Systems/Connection Handling.md | 152 | "- [[Milestones#Milestone 3 Thread-per-Connection Model|Milestones: M3 detail]]" |
| Projects/Zyphos/Systems/Request Parsing.md | 119 | "- [[Gaps#Request parsing gaps|Gaps: missing headers, body, Content-Length]]" |
| Projects/Zyphos/Systems/Response Pipeline.md | 142 | "- [[Gaps#Response pipeline gaps|Gaps: trailing CRLF, Server header, binary bodies]]" |
| Projects/Zyphos/Systems/Routing.md | 168 | "- [[Gaps#Routing gaps|Gaps: POST/PUT/DELETE, query strings, URL decoding]]" |
| Projects/Zyphos/Systems/Testing.md | 113 | "- [[Roadmap]] — test priorities in the next session" |
