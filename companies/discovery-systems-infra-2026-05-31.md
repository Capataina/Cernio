# Discovery — systems-infra lane — 2026-05-31

Lane: `systems-infra` (densest lane in the DB at 160 entries — focus on less-obvious sub-categories: deterministic-testing platforms, financial OLTP, streaming DBs, local-first sync engines, WASM runtimes, eBPF profiling, RISC-V / compiler-IP, Nix-as-a-service, version-controlled DBs).

Dedup probes against `/tmp/cernio-universe.txt` cleared every entry below. Excluded from this file (already in DB): Cockroach Labs, Groq, Graphcore, Rivos, Codasip, Embecosm, Ferrous Systems, Isovalent, Linaro, SiFive, Tailscale, Aiven, ClickHouse, Couchbase, CrowdStrike, Databricks, Datadog, Denodo, Elastic, Grafana Labs, HashiCorp, Neon, Northflank, PostHog, Sentry, SingleStore, Snyk, Starburst, Stripe, SurrealDB, Synthesized, Turso, Veeam, Wiz, Codeplay (sponsor licence revoked Feb–Apr 2026), Polar Signals.

---

### TigerBeetle
- **Website**: https://tigerbeetle.com
- **Location**: Distributed; no UK office listed (US-incorporated)
- **What they do**: Single-binary financial-transactions OLTP database in Zig — all memory allocated upfront, single-core deterministic, NASA power-of-ten style ("Tiger style"), every assertion enforced. Designed for accounting workloads at exchanges, banks, fintechs.
- **Why relevant**: Cernio's SQLite WAL discipline + Nyquestro's lock-free / single-core thinking map directly onto TigerBeetle's static-allocation, deterministic-replication style. Xyntra's typed-IR + assertion-heavy newtype design is the same engineering culture (zero-`unsafe`, exhaustive invariants).
- **Source**: https://github.com/tigerbeetle/tigerbeetle , https://tigerbeetle.com/blog/2025-10-25-synadia-and-tigerbeetle-pledge-512k-to-the-zig-software-foundation/
- **Sponsor**: unknown (no UK entity surfaced; verify on GOV.UK register)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Materialize
- **Website**: https://materialize.com
- **Location**: NYC HQ, distributed remote
- **What they do**: Streaming SQL data warehouse built on Differential Dataflow (Frank McSherry's incremental-computation engine), entirely in Rust. Maintains up-to-date materialised views over change-data-capture streams from Postgres / Kafka.
- **Why relevant**: Pure Rust systems work on incremental-compute algorithms — directly analogous to Xyntra's compiler-IR / dataflow-graph reasoning. Cernio's SQLite event sourcing is the same problem class one tier down.
- **Source**: https://materialize.com/careers/ , https://materialize.com/blog/rust-for-data-intensive-computation/
- **Sponsor**: unknown (US-only career page; verify via register)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### RisingWave Labs
- **Website**: https://risingwave.com
- **Location**: San Francisco HQ; distributed engineers
- **What they do**: Open-source distributed streaming SQL database (Rust), positioned for agentic-AI event-stream workloads. Continuous ingestion, transformation, serving at scale.
- **Source**: https://risingwave.com/careers/ , https://github.com/risingwavelabs/risingwave
- **Sponsor**: unknown (Workable board; verify)
- **Why relevant**: Same Rust-distributed-systems engineering envelope Caner aims at; streaming-DB internals overlap with Cernio query-engine work and Xyntra dataflow framing.
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Modular
- **Website**: https://www.modular.com
- **Location**: Los Altos, CA; remote US/Canada per current listings
- **What they do**: Builds the Mojo programming language (Pythonic syntax, MLIR-based compiler targeting heterogeneous compute) and the MAX AI inference stack. Chris Lattner (LLVM / Clang / Swift) is CEO.
- **Why relevant**: Xyntra's stated ambition is exactly this surface (kernel-fusion compiler, MLIR-style IR, GPU codegen). Even though current listings are US-only, applying as a pipeline pinnacle is correct positioning; Mojo open-sources in 2026 which expands contributor-to-hire pathways.
- **Source**: https://www.modular.com/company/careers
- **Sponsor**: unknown (US/Canada only listed — likely no UK sponsor)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Fermyon
- **Website**: https://www.fermyon.com
- **Location**: Boulder, Colorado; remote
- **What they do**: WebAssembly serverless platform (Fermyon Wasm Functions on Akamai edge, plus Spin OSS runtime). Cold starts in sub-millisecond range via Wasmtime.
- **Why relevant**: Compiler-runtime systems work at WASM-component level; the same low-level binary-format / sandboxing concepts Xyntra would target if it added a WASM backend. Spin is Rust.
- **Source**: https://www.fermyon.com/ , https://www.fermyon.dev/
- **Sponsor**: unknown (US-centric; verify)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Wasmer
- **Website**: https://wasmer.io
- **Location**: San Francisco; remote-friendly (YC S19)
- **What they do**: Universal WebAssembly runtime targeting edge / serverless / plugin embedding, plus WAPM package registry. Competing runtime to Wasmtime in the Rust WASM ecosystem.
- **Why relevant**: Rust systems-runtime work with direct compiler / VM internals exposure. Xyntra-adjacent backend-target territory.
- **Source**: https://www.workatastartup.com/companies/wasmer
- **Sponsor**: unknown
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Antithesis
- **Website**: https://antithesis.com
- **Location**: Northern Virginia HQ; launched 2024
- **What they do**: Autonomous deterministic-simulation testing platform — runs your distributed system inside a hypervisor that controls scheduling, network, time and faults, replays years of production in hours with full reproducibility. Backed by $105M Series A led by Jane Street; customers include Jane Street, MongoDB, Ethereum, WarpStream.
- **Why relevant**: Deterministic-replay / simulation-testing is *the* technique Cernio's SQLite-WAL discipline aspires to and Nyquestro's lock-free engineering needs. The engineering culture (FoundationDB lineage — founders ex-FDB) is exactly the systems-correctness mindset Xyntra's `unsafe`-free + assertion-heavy IR work signals toward.
- **Source**: https://antithesis.com/ , https://www.prnewswire.com/news-releases/jane-street-leads-antithesiss-105m-series-a-to-make-deterministic-simulation-testing-the-new-standard-302631076.html
- **Sponsor**: unknown (verify; US-headquartered, may have UK presence given Jane Street tie)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Zed Industries
- **Website**: https://zed.dev
- **Location**: Distributed remote (US-led)
- **What they do**: GPU-accelerated multiplayer code editor built ~1M lines of Rust by the creators of Atom and Tree-sitter. Custom GUI framework (GPUI), CRDT-based real-time collaboration, language-server integrations.
- **Why relevant**: Pure systems-Rust at meaningful scale (≈1M LOC) with GPU + custom rendering + CRDT internals — every concept-domain Caner has touched (Xyntra Rust IR, Nyquestro low-level, NeuroDrive rendering). Hiring is open-source-led (less than half of recent hires submitted a traditional application).
- **Source**: https://zed.dev/jobs , https://zed.dev/blog/hiring
- **Sponsor**: unknown
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### PingCAP (TiDB)
- **Website**: https://www.pingcap.com
- **Location**: London office listed (plus distributed)
- **What they do**: Distributed SQL HTAP database (TiDB) — Rust storage engine (TiKV, CNCF graduated), Go SQL layer, MySQL-compatible, Raft replication, columnar TiFlash for analytics. TiDB SCaiLE Europe event Jun 2026 indicates active EU engagement.
- **Why relevant**: TiKV is one of the canonical large-scale Rust distributed-storage codebases — Cernio's SQLite WAL + Nyquestro's lock-free engineering both punch into this exact territory. London presence + sponsor likely.
- **Source**: https://www.pingcap.com/careers/ , https://job-boards.greenhouse.io/pingcap
- **Sponsor**: unknown (London office present; verify on register)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Memgraph
- **Website**: https://memgraph.com
- **Location**: London office (plus Zagreb HQ)
- **What they do**: In-memory graph database in C++ optimised for real-time streaming workloads; Neo4j Cypher-compatible.
- **Why relevant**: Graph-engine internals + in-memory data-structures work overlap with Xyntra's `Graph<NodeID, Node>` design and DAG-validation surface. London office is the rare on-shore lane-pinnacle pure-engineering employer.
- **Source**: https://builtinlondon.uk/company/memgraph , https://uk.linkedin.com/company/memgraph
- **Sponsor**: unknown (London office present; verify)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### QuestDB
- **Website**: https://questdb.com
- **Location**: London-headquartered; remote-first
- **What they do**: Open-source high-performance time-series database (Java + zero-GC patterns + SIMD), SQL-compatible, optimised for financial-tick and IoT ingest at >1M rows/sec on commodity hardware.
- **Why relevant**: London HQ; the SIMD / zero-allocation / single-writer-thread discipline maps to Nyquestro's lock-free engineering and Cernio's WAL constraints. Time-series-DB internals are pinnacle systems work on UK soil.
- **Source**: https://questdb.com/careers/ , https://uk.indeed.com/cmp/Questdb-1
- **Sponsor**: yes (UK-headquartered — almost certainly on register; verify exact licence entry)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Dolt (DoltHub)
- **Website**: https://www.dolthub.com
- **Location**: San Francisco; distributed
- **What they do**: "Git for data" — a version-controlled SQL database with branches, merges, diffs, written in Go. DoltgreSQL extends the model to Postgres wire protocol.
- **Why relevant**: Version-control-as-storage-engine is concept-domain overlap with Cernio's event-sourced archival history and Xyntra's IR snapshots. Storage-engine + merge-algorithm work.
- **Source**: https://www.dolthub.com/blog/2020-10-21-we-are-hiring/ , https://github.com/dolthub/dolt
- **Sponsor**: unknown
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### ElectricSQL
- **Website**: https://electric-sql.com
- **Location**: Team members in London, Cambridge, Brighton (plus EU)
- **What they do**: Local-first sync layer between Postgres and SQLite for reactive offline-capable apps. Open hires for Staff Engineer (Elixir/TS) and Systems Engineer (C/WASM).
- **Why relevant**: UK team distribution including London/Cambridge; SQLite-replication + CRDT + WASM-systems engineering — direct overlap with Cernio's SQLite-as-backbone architecture and Xyntra's WASM-codegen-target future.
- **Source**: https://electric-sql.com/about/team , https://hr.linkedin.com/company/electric-sql
- **Sponsor**: unknown (UK team present; verify register)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Liveblocks
- **Website**: https://liveblocks.io
- **Location**: Paris HQ; remote-friendly
- **What they do**: Realtime collaboration sync engine (Liveblocks Storage for Figma-class tools; Liveblocks Yjs for collaborative text). Multiplayer-primitives platform.
- **Why relevant**: CRDT / sync-engine internals — same concept-domain as ElectricSQL and Cernio's planned multi-device sync. European HQ; sponsorship-friendly hiring posture.
- **Source**: https://liveblocks.io/docs/collaboration-features/multiplayer/sync-engine
- **Sponsor**: unknown
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Spacelift
- **Website**: https://spacelift.io
- **Location**: Distributed; engineering across EU
- **What they do**: Infrastructure-as-Code orchestration platform (Terraform / OpenTofu / Pulumi / Ansible / Kubernetes) — policy engine, dependency graph, stateful drift detection.
- **Why relevant**: IaC orchestration is graph-execution + dependency-resolution at scale — same algorithmic spine as Xyntra's DAG validation and Cernio's job-pipeline orchestration. Polish founders, EU engineering distribution.
- **Source**: https://spacelift.io/careers , https://careers.spacelift.io/jobs
- **Sponsor**: unknown
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Determinate Systems
- **Website**: https://determinate.systems
- **Location**: Distributed remote (US-led, with EU contributors)
- **What they do**: Enterprise Nix distribution — Determinate Nix (validated, signed, reproducible), nix-installer, Flake-Hub registry. Founded by Eelco Dolstra (Nix creator) and Graham Christensen.
- **Why relevant**: Nix is the reproducibility-as-systems-engineering pinnacle; Cernio's deterministic-build discipline + Xyntra's `cargo`-toolchain rigour both land in this concept-domain. Hires from Nix-OSS contributor pool.
- **Source**: https://docs.determinate.systems/determinate-nix/ , https://github.com/DeterminateSystems
- **Sponsor**: unknown
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Tweag (Modus Create)
- **Website**: https://www.tweag.io
- **Location**: Paris / London / distributed EU
- **What they do**: Functional-programming + Nix + Haskell + Rust + Bazel consultancy. The Nix Technical Group are top OSS contributors; client work targets reproducible-build / data-engineering / formal-methods.
- **Why relevant**: London presence; pinnacle Nix / Haskell / Rust contractor culture. The exact "Caner ships rigorous systems work" employer-fit. Engineering-blog density signals real technical depth.
- **Source**: https://www.tweag.io/group/nix/ , https://nixos.org/community/commercial-support/
- **Sponsor**: unknown (London office — high prior on sponsor licence; verify)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Numtide
- **Website**: https://numtide.com
- **Location**: UK presence per LinkedIn ("uk.linkedin.com/company/numtide")
- **What they do**: NixOS + DevOps consultancy; binary-cache hosting via Cachix relationship.
- **Why relevant**: UK-listed Nix consultancy — exact peer of Tweag at smaller scale. Nix-engineering pinnacle on a sponsor-able UK entity.
- **Source**: https://uk.linkedin.com/company/numtide , https://nixos.org/community/commercial-support/
- **Sponsor**: yes (UK LinkedIn entity — verify exact licence) | unknown
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Chronosphere
- **Website**: https://chronosphere.io
- **Location**: NYC HQ; distributed (EU + UK engineers per LinkedIn)
- **What they do**: Cloud-native observability platform built on M3 (open-source Prometheus-compatible TSDB co-founded by ex-Uber). Time-series metrics + traces + logs at hyperscale.
- **Why relevant**: M3DB internals are pinnacle Go / storage-engineering work. Observability + TSDB systems overlap with Nyquestro's HFT-style metrics and Cernio's lane-tagging analytics. Recent funding + active hiring.
- **Source**: https://chronosphere.io/careers/ , https://jobs.ashbyhq.com/chronospherejobs
- **Sponsor**: unknown (verify)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Tinybird
- **Website**: https://www.tinybird.co
- **Location**: Madrid HQ; distributed EU
- **What they do**: Managed ClickHouse + API-publication layer for real-time analytics. Engineers query their own ClickHouse via SQL and ship endpoints in minutes.
- **Why relevant**: ClickHouse-internals adjacent work, columnar-OLAP + API-gateway engineering. EU base, sponsor-friendly culture, Y Combinator-backed.
- **Source**: https://www.fastaijobs.com/companies/tinybird
- **Sponsor**: unknown
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

---

## Report

- WebSearch calls: 19 (lane is densest in the DB — net-new finds were sparser than greenfield lanes; many obvious targets already present including Cockroach, Codeplay, Embecosm, Ferrous Systems, Isovalent, SiFive, Codasip, Linaro, Tailscale, ClickHouse, Databricks, HashiCorp, Grafana, Snyk, Wiz, Sentry, Turso, Neon, SurrealDB, Polar Signals).
- 20 net-new entries written. Strict dedup against `/tmp/cernio-universe.txt` confirmed each.
- **Top 5 net-new (lane-pinnacle, UK-relevant or sponsor-plausible):** QuestDB (UK HQ, time-series internals), Memgraph (London office, graph internals), Tweag (London office, Nix/Haskell/Rust consultancy), PingCAP/TiDB (London office, TiKV Rust distributed storage), Antithesis (deterministic-simulation testing, Jane Street-backed, FoundationDB lineage).
- **Dry sources:** Vespa.ai (Trondheim-only; no London surfaced), MariaDB (no sponsor info returned), most Nix consultancies (Determinate / Cachix US-led with no clear UK entity). Codeplay was a candidate but already in DB *and* lost its UK sponsor licence between Feb-Apr 2026 (Tarve revoked-list reference) — flag for `check-integrity` to update DB `sponsors_uk` from yes to no.
- **Caveat:** Every entry except QuestDB and Numtide marks sponsor as `unknown`. `populate-db` / `resolve-portals` must verify each against the GOV.UK register (https://www.gov.uk/government/publications/register-of-licensed-sponsors-workers, updated 29 May 2026) before insertion; non-sponsors should be dropped per the sponsor-only universe rule in `profile/career-goals.md`.
