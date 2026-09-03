# Azure and DeepL Development Provider Plan

## Goal

Allow a developer to select Azure Translator or DeepL without changing the
shared capture workflow or the Windows-specific platform API. Automatic
translation remains optional: without an explicit provider selection, the app
uses the unavailable provider and preserves manual edit/save behavior.

## Decisions

- `VOCAB_TRANSLATION_PROVIDER` accepts `azure` or `deepl`; missing configuration
  selects `unavailable`. The app never guesses from whichever key happens to exist.
- Development settings are documented in `.env.example` and may be supplied in
  the ignored workspace-root `.env.local`. Process environment variables keep
  precedence over the file.
- DeepL Free and Pro use the same provider implementation. The configured
  endpoint selects the service tier; no separate plan/tier setting is added.
- Only the selected provider's configuration is parsed. An unused provider key
  has no effect and must never cross the Rust/WebView boundary.
- `TranslationProvider`, `TranslationResult`, capture coordination, OCR
  confirmation, edit/apply, retry, and explicit save remain provider-neutral.
- Default tests use local HTTP mocks and make no Azure or DeepL request. Each
  provider may expose a separately invoked ignored fixed-text smoke test.

## Design

Add a standalone DeepL provider beside the Azure provider. It owns DeepL request
and response DTOs, authentication, language-code adaptation, retry policy,
overall timeout budgeting, and content-safe error mapping. The desktop
composition root reads the explicit provider selector, validates only that
provider's configuration, and injects it through the existing portable trait.

DeepL automatic source detection omits `source_lang`. The target is always
explicit. Free and Pro endpoints are validated as HTTPS. Only selected vocabulary
text is sent; context, OCR images, credentials, raw responses, and full request
URLs never enter frontend state or production-visible diagnostics.

## Delivery sequence

1. Build and mock-test the DeepL provider independently.
2. Add explicit provider selection and safe configuration composition.
3. Verify both choices through the Windows workflow and complete CI/documentation.

## Out of scope

User accounts, backend proxying, shared quota ownership, automatic provider
fallback, provider quality comparison, release credential delivery, installers,
billing UI, and production readiness remain out of scope.

## Completion evidence

The work is complete for development when a clean checkout with no credentials
passes all automated tests, explicit Azure and DeepL selections install only the
chosen fake/mock provider, and the Windows capture workflow behaves identically
through the portable contract. Real-service and physical Windows evidence must
remain separately labelled and cannot be inferred from automation.
