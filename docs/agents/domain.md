# Domain Docs

How the engineering skills should consume this repository's domain
documentation when exploring the codebase.

## Before exploring, read these

- `CONTEXT.md` at the repository root.
- ADRs under `docs/adr/` that touch the area about to be changed.

If either location does not exist, proceed silently. Do not create a
multi-context structure pre-emptively; domain documentation is added lazily
when terminology or a durable decision is resolved.

## Layout

This is a single-context repository:

```text
/
├── CONTEXT.md
├── docs/
│   └── adr/
└── crates, platform, apps, and ui
```

The Rust crates, operating-system adapters, desktop shell, and presentations
share one Vocab Collector domain language. Their package boundaries do not
create separate domain contexts.

## Use the glossary's vocabulary

When output names a domain concept in a ticket title, proposal, test, or design,
use the term defined in `CONTEXT.md`. In particular, preserve canonical terms
such as Vocabulary Item, Encounter, Capture Candidate, Review, and Review
Insight, and avoid synonyms that the glossary rejects.

If a needed concept is missing, reconsider whether new terminology is actually
necessary. Use the domain-modeling workflow when a genuine vocabulary gap must
be resolved.

## Flag ADR conflicts

If proposed work contradicts an existing ADR, surface the conflict explicitly
instead of silently overriding the decision. Cite the affected ADR and explain
why reopening it may be justified.
