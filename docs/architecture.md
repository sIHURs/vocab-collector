# Architecture

Vocab Collector is a local-first application with one shared product core and
replaceable operating-system adapters. SQLite is the source of truth on every
device. Supabase is an optional replication target and never blocks capture or
review.

```text
Svelte UI
    │ typed Tauri commands and events
vocab-desktop (composition and presentation boundary)
    ├── vocab-application + vocab-capture (product workflow)
    ├── vocab-storage + vocab-domain (durable rules and data)
    └── vocab-platform-api (portable capability ports)
            ▲
            ├── vocab-platform-macos → Swift native package
            ├── vocab-platform-linux (Plan B skeleton)
            ├── vocab-platform-windows (incremental Plan B adapter)
            └── vocab-translation-azure (replaceable network provider)
```

Dependencies point inward: shared crates never import Tauri, Swift FFI,
AT-SPI, XDG Portal, UI Automation, Win32, or an OS adapter. Target selection is
confined to `apps/desktop/src-tauri/src/bootstrap.rs` and target-specific Cargo
dependency tables.

## Shared boundaries

- `vocab-domain` owns entities, normalization, review scheduling, and
  repository contracts.
- `vocab-storage` owns SQLite, migrations, repository implementations, and the
  transactional outbox. Duplicate lemmas reuse a word and append encounters.
- `vocab-capture` owns request identity, stale-result rejection, explicit OCR
  confirmation, translation state, save-once behavior, shortcut validation,
  and portable window placement.
- `vocab-application` composes capture and the remaining product use cases. Its
  `PlatformCaptureWorkflow` publishes a captured candidate before translation,
  then records translation success or failure against the same request.
- `vocab-platform-api` contains portable DTOs, typed errors, capability flags,
  and the small `SelectionProvider`, `OcrProvider`, `TranslationProvider`,
  `PermissionProvider`, and `WindowProvider` traits.
- `vocab-platform-contract-tests` provides reusable adapter contract checks;
  application tests use fake providers without native permissions.
- `vocab-translation-azure` implements only the portable `TranslationProvider`
  contract. Azure HTTP, credentials, retry policy, and protocol DTOs stay private
  to that crate; the Windows adapter has no Azure dependency.
- `vocab-desktop` selects one platform service bundle, constructs the shared
  workflow, normalizes Tauri presentation data, and exposes stable commands and
  events. It does not call the Swift ABI directly.

All coordinates crossing `vocab-platform-api` use logical units in a top-left
virtual-desktop space. Native adapters perform native-coordinate conversion;
the desktop boundary normalizes Tauri monitor and pointer data before portable
placement.

## Capture lifecycle and concurrency

Each shortcut starts one coordinator-owned request ID. Selection is published
as `capture-ready` before translation begins, so the UI can show the captured
text immediately. Translation runs separately and may be retried or explicitly
bypassed. Provider unavailability is `translation_unavailable`; a native
translation operation failure is `translation_failed`. Only those typed codes
offer retry and save-without-translation actions; diagnostic text never controls
UI behavior.

Every asynchronous UI continuation revalidates both component lifetime and its
captured request ID before changing state or starting dependent work. Close and
timed-dismiss commands carry that request ID, and the Rust publication guard
serializes the hide side effect with request replacement. The same guarded
publication boundary protects capture-ready, capture-error, and OCR events.

Blocking macOS translation and OCR bridge calls run on Tokio blocking workers.
Swift pointers and bridge envelopes remain private to the macOS Rust adapter,
and native diagnostics must not include captured text, translations, URLs, or
screenshots.

## Capabilities and platform status

The UI branches on `PlatformCapabilities`, never on an OS name. A target may
install inert native styles on its hidden capture window so physical validation
can occur before the capability is advertised. Product presentation that relies
on those styles is enabled only when `nonActivatingWindow` is reported;
unavailable providers return explicit typed errors.

The macOS adapter and Swift package are the Plan A runtime implementation.
Linux remains a contract-compatible skeleton. Windows is an incremental Plan B
adapter: UIA selection and bounds are advertised after target-machine evidence.
In debug development only, the desktop composition root may inject the optional
Azure provider and then advertise translation; credentials stay in Rust below
the WebView boundary. OCR, Azure-backed translation behavior, and non-activating
window behavior remain physically unverified until their own evidence gates pass. Target-specific compilation, automated
tests, and physical runtime evidence are documented in `linux-development.md`
and `windows-development.md`.

## Account and enrichment seams

The Supabase migration is account-scoped and protected by RLS. Guest data stays
in a separate local database; sign-in, credential storage, guest merge, and
delta transport remain outside the current foundation.

Future AI enrichment implements `EnrichmentProvider`. Its output retains
provenance and remains separate from the user translation so an assistant
cannot silently overwrite vocabulary data.
