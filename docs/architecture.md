# Architecture

Vocab Collector is local-first. The Rust domain owns durable product rules, while the Svelte UI receives typed view models through a narrow desktop API. SQLite is the source of truth on every device. Supabase is an optional replication target and never blocks capture or review.

```text
Svelte UI
   │ typed commands
Desktop application service
   ├── Domain rules and repository contracts
   ├── SQLite repositories + transactional outbox
   ├── Platform provider traits
   └── Optional sync engine → Supabase
```

The prototype ships a deterministic capture simulator so the complete frontend flow can run without Accessibility or Translation permissions. Production macOS providers will implement the same `PlatformCapture` and `TranslationProvider` contracts.

## Runtime boundaries

- `vocab-domain` contains serializable entities, normalization, review scheduling, and view models.
- `vocab-storage` owns SQLite, migrations, repository implementations, and the transactional outbox. A duplicate lemma reuses one word and appends a new encounter.
- `vocab-application` composes capture, Today, vocabulary detail, review, Undo, and settings use cases.
- `vocab-desktop` initializes the per-scope database and exposes only typed Tauri commands.
- `ui/src/lib/backend.ts` selects the Tauri adapter in the desktop runtime and a deterministic in-memory adapter in an ordinary browser.

## Account and AI seams

The Supabase migration is account-scoped and protected by RLS. Guest data remains in a separate local database; sign-in, credential storage, guest merge, and delta transport are deliberately left outside this prototype foundation.

Future AI enrichment implements `EnrichmentProvider`. Its output has provenance fields and remains separate from the user translation, so adding an assistant cannot silently overwrite vocabulary data.
