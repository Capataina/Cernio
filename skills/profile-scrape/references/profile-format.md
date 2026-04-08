# Profile Format Reference

> The exact format and quality standard for profile entries. Read this before writing or updating any entry in `profile/projects.md` or `profile/skills.md`.

---

## projects.md entry format

Every project entry follows this exact format. Do not deviate from the field structure.

```markdown
## Project N

- **Name**: [Project name]
- **URL**: [GitHub URL]
- **Type**: [Personal / Personal / research-style / Academic / Open source contribution]
- **Tech stack**: [Comma-separated list of languages, frameworks, and key tools]
- **Status**: [See status guidelines in scraping-methodology.md]
- **Summary**: [2-3 sentences. What the project IS and what it DOES. Not how it works — that's for technical highlights.]
- **Your role**: [Solo / Lead / Contributor. One sentence on scope of involvement.]
- **Technical highlights**: [The critical field. See below.]
```

### The technical highlights field

This is the most important field in the entire profile. It's what job evaluations match against. It's what makes the difference between "I built a trading system" and "I built a lock-free order matching engine with slab allocation, price-time priority, and HDR latency histograms."

**Structure:** Write it as a single dense paragraph (or at most two). This is deliberate — the paragraph format forces specificity and avoids bullet-point padding. Each sentence should convey a concrete technical fact about the implementation.

**What to include:**
- The architecture: what the layers are, how they communicate, what boundaries exist
- The hard problems: what makes this non-trivial, what constraints make it harder
- Specific data structures, algorithms, or patterns used (name them)
- Performance characteristics if relevant (latencies, throughput, scale)
- What's built from scratch vs what uses libraries
- Testing and measurement approach if notable
- Design principles or constraints that shaped the implementation (e.g., "safe Rust only", "no ML framework dependencies", "local-first by construction")

**What NOT to include:**
- Generic descriptions ("uses modern architecture", "well-structured codebase")
- Feature lists without technical substance ("supports login, search, and export")
- Marketing language ("cutting-edge", "next-generation", "state-of-the-art")
- Planned features — only describe what's built and working
- Technology justifications ("we chose Rust because...") — just state what's used

**Length:** Technical highlights should be proportional to the project's depth. A substantial project (Nyquestro, NeuroDrive, Cernio) may have 15-20 sentences. A small completed project (Neuronika) may have 3-4 sentences. Don't pad thin projects or truncate deep ones.

**Voice:** Write in third person as a technical description, not first person. "The matching engine core maintains bid and ask sides as atomic price-level buckets" — not "I built a matching engine that has bid and ask sides."

---

## Example: what good technical highlights look like

### Deep project (Nyquestro — matching engine)

> Composed of independently testable layers connected through narrow, replaceable interfaces. The matching engine core maintains bid and ask sides as atomic price-level buckets, each an intrusive FIFO list managed without locks, supporting price-time priority matching, partial fills, and atomic cancellation. The hot path avoids heap allocation, lock contention, and cache misses by design: lock-free CAS operations replace mutexes (which are a tail-latency cliff under contention), and a slab allocator pre-allocates a fixed pool of order node slots with a lock-free free-list, eliminating allocator churn from the latency profile. [continues with risk guard, protocol, strategy agent, benchmarking...]

**Why this is good:** Every sentence names a specific technique, data structure, or design decision. A reader learns what the engineer actually built and how they think about systems.

### Shallow project (Neuronika — knowledge management)

> Built around the idea that personal notes are most useful when surfaced by meaning rather than by filename or folder. Combines vector embedding retrieval with a graph representation of the relationships between notes, enabling both semantic search and visual exploration of how ideas connect.

**Why this is good for its depth:** The project is smaller, so the entry is shorter. It still explains what makes it technically interesting (embedding retrieval + graph representation) without padding.

### What BAD technical highlights look like

> A well-architected application using modern Rust patterns. Features include user authentication, data persistence, and a responsive UI. Built with best practices and clean code principles.

**Why this is bad:** Could describe literally any application. Names no specific techniques, no specific data structures, no specific problems solved. Tells you nothing about what the engineer actually built or how they think.

---

## skills.md format

### Programming languages table

```markdown
| Language   | Proficiency | Notes |
|------------|-------------|-------|
| [Language] | [Level]     | [Which projects use it, what depth of usage] |
```

### Proficiency levels

| Level | What it means | Evidence required |
|-------|--------------|-------------------|
| **Proficient** | Deep, repeated, confident use across multiple projects. Idiomatic patterns, non-trivial applications, comfortable with advanced features | Multiple substantial projects, demonstrated mastery of language-specific patterns |
| **Comfortable** | Real usage beyond tutorials. Working code that solves real problems | At least one project with meaningful use, not just boilerplate |
| **Familiar** | Initial exploration. Single use case, potentially following guides | One project or coursework, limited depth |

**Rules for updating proficiency:**
- Only upgrade if the new project demonstrates meaningfully deeper usage than what was already recorded
- A second project using Rust at the same depth doesn't upgrade from Comfortable to Proficient — it adds evidence to Comfortable
- Proficient requires evidence of idiomatic, advanced usage — not just "used it in multiple projects"
- Never downgrade without evidence that proficiency has genuinely degraded (extremely rare)

### Domains and concepts table

```markdown
| Domain                | Depth       | Notes |
|-----------------------|-------------|-------|
| [Domain]              | [Level]     | [What projects demonstrate this, what specific knowledge] |
```

Domain depth levels follow the same Proficient/Comfortable/Familiar scale but applied to domain knowledge rather than language skill.

---

## portfolio-gaps.md format

When a scrape reveals a gap closure or new strength:

**Gap closure:** If a project addresses a known gap (e.g., "lacks containerisation experience" and the project has a Dockerfile with CI), note it in the "Closure Opportunities" section:
```
- **[Gap name]**: Partially addressed by [project] — [what it demonstrates]. [What remains to fully close the gap]
```

**New strength:** If a project reveals a strength not previously tracked:
```
- **[Strength]**: Demonstrated in [project] — [specific evidence]
```

---

## What NOT to touch

### profile/resume.md
The resume is the user's curated artefact. Never edit it directly. If a scrape reveals something the resume should reflect (e.g., a project has grown substantially), suggest the change conversationally.

### profile/preferences.toml
Search filters and career targets. Not affected by repo scraping.

### profile/visa.md, personal.md, education.md, experience.md
These are human-edited files about the user's personal circumstances. A repo scrape should never modify them.
