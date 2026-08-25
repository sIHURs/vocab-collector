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

