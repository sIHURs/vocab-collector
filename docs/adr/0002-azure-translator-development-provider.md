# Use Azure Translator as a replaceable development provider

## Status

Accepted on 2026-09-03 for development and debugging only.

## Context

The Windows application has a platform-neutral `TranslationProvider` contract but currently injects an unavailable provider. The macOS implementation uses Apple Translation and cannot be reused on Windows. The current Windows planning documents intentionally prohibit hidden network translation, so enabling Azure requires an explicit replacement decision with a narrow privacy and release boundary.

This stage is for developers validating automatic translation on Windows. It does not include user accounts, a service-owned proxy, shared quotas, production credential distribution, or a release configuration. A later account-backed design will replace the development credential path before release.

## Decision

Add Azure Translator as a cross-platform provider in `crates/translation-azure`, depending only on `vocab-platform-api`. The desktop composition root reads development configuration and injects the provider into Windows; `platform/windows` remains unaware of Azure, HTTP, credentials, and environment variables. macOS continues to inject Apple Translation.

Debug builds may load an untracked workspace-root `.env.local`. Existing process environment variables take precedence. Release builds must not contain `.env.local` loading behavior. `.env.example` documents names but contains no credentials. Azure credentials remain in Rust and must never enter the WebView, SQLite, frontend state, logs, diagnostics, test snapshots, or committed files.

The provider sends only the selected vocabulary text. It never sends the captured context. Source language `auto` means omitting Azure's `from` parameter and recording the detected source language returned by Azure. Target language is always explicit and cannot be `auto`. OCR-derived text may be sent only after OCR Confirmation.

Windows automatically requests a translation after an eligible capture, then presents a preview. The user explicitly saves. Selected text, context, and translation are independently editable, and applying edits does not persist them. The system does not infer semantic consistency between independently edited fields. `Save capture` persists their final values, including when translation is blank.

Provider-internal errors distinguish authentication, quota, rate limiting, timeout, network, unsupported language, invalid response, and other service failures. The initial Windows presentation exposes a friendly translation-unavailable message with retry, edit, and save recovery paths. Errors and logs must not contain credentials, selected text, context, translated text, full request URLs, or Azure response bodies.

All automated tests use a local mock HTTP server. A separately invoked ignored smoke test may call Azure with developer credentials. A configured provider may advertise translation capability, but Windows translation is not documented as physically verified until that smoke test succeeds on the target machine.

## Consequences

- Azure transport and protocol code can later be reused by Linux or a service process without moving it out of a Windows adapter.
- Missing development configuration preserves the existing manual workflow; partial or invalid configuration fails debug startup with a content-safe diagnostic.
- A desktop binary built with this development design cannot safely carry a shared production key. Account-backed proxying, authentication, quotas, and production secrets require a later ADR before release.
- The Windows plans and capability documentation must be revised narrowly because their blanket prohibition on network translation is superseded by this opt-in development decision.
