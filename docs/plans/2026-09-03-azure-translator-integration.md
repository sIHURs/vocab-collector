# Azure Translator Development Integration Plan

## Goal

Add developer-configured Azure text translation to the Windows development build while preserving the existing platform boundary, OCR confirmation rule, manual fallback, and explicit save behavior. This plan does not implement accounts, a backend proxy, production credential delivery, packaging, or release behavior.

## Fixed product decisions

- Developers supply their own Azure credentials through process environment variables or an untracked debug-only `.env.local`.
- Only selected vocabulary text is sent to Azure; context is never sent.
- Source language may be `auto`; target language is always explicitly selected.
- An eligible native capture starts translation automatically. The result is previewed and saved only when the user chooses `Save capture`.
- Selected text, context, and translation can be edited independently. `Apply changes` updates the current draft; it does not persist. Editing selected text does not invalidate or replace an existing translation unless the user explicitly retries translation.
- OCR candidates must be confirmed before any network translation.
- Translation failures offer retry, edit, and `Save capture`; a blank translation is allowed without changing the button label.
- Automated tests never call Azure. A real Azure smoke test is ignored by default and run explicitly by a developer.

## Intended module structure

```text
crates/translation-azure/
  Cargo.toml
  src/
    lib.rs        public AzureTranslatorConfig and AzureTranslationProvider
    client.rs     request construction, timeout, retry, and HTTP execution
    protocol.rs   private Azure request/response DTOs
    error.rs      typed internal failures and content-safe platform mapping
  tests/
    provider.rs   mock-server contract and privacy tests
    live_translation.rs  ignored developer smoke test

apps/desktop/src-tauri/src/
  config.rs       debug configuration loading and validation
  bootstrap.rs    target composition only

platform/windows/
  provider injection and capability composition, with no Azure knowledge

ui/src/windows/
  capture backend seam and translation-aware Windows presentation
```

Dependency direction remains:

```text
vocab-desktop → vocab-translation-azure → vocab-platform-api
vocab-desktop → vocab-platform-windows → vocab-platform-api
vocab-application                         → vocab-platform-api
```

Neither Azure crate nor Windows adapter may depend on application, capture, storage, Tauri, or Svelte code.

## Milestone 1: Lock down configuration behavior

### Tests first

Add desktop configuration tests covering:

1. All Azure variables absent returns `None` and permits normal startup.
2. A complete configuration returns a redacted `AzureTranslatorConfig`.
3. A partial configuration fails startup without printing any configured value.
4. Invalid endpoint scheme or malformed URL fails startup.
5. Invalid or unreasonable timeout fails startup.
6. Existing process variables take precedence over `.env.local`.
7. Release-mode configuration code has no `.env.local` loading path.

### Implementation

Create `apps/desktop/src-tauri/src/config.rs`. In debug builds only, load the workspace-root `.env.local` without overriding existing environment variables. Parse:

```dotenv
VOCAB_AZURE_TRANSLATOR_KEY=
VOCAB_AZURE_TRANSLATOR_ENDPOINT=https://api.cognitive.microsofttranslator.com
VOCAB_AZURE_TRANSLATOR_REGION=
VOCAB_AZURE_TRANSLATOR_TIMEOUT_MS=10000
```

Rules:

- Key is required once any Azure configuration is present.
- Endpoint defaults to the global Azure Translator endpoint and must use HTTPS outside tests.
- Region is optional for a global single-service resource; developers using regional or multi-service resources must set it.
- Timeout defaults to 10 seconds.
- Configuration `Debug` and error output must redact the key and must not echo arbitrary environment values.

Add `.env.example` with an empty key and explanatory comments. Confirm `.env.local` remains covered by the existing `.env.*` ignore rule.

### Acceptance

- Debug startup without Azure variables retains manual translation.
- A partially configured developer environment fails early with a precise, content-safe message.
- No credential reaches frontend commands, SQLite, logs, or committed fixtures.

## Milestone 2: Build the Azure provider as a deep module

### Tests first

Create mock-server tests that exercise the public `TranslationProvider` seam and assert:

1. `POST /translate` uses `api-version=3.0` and one explicit `to` value.
2. `source == "auto"` omits `from`; an explicit source includes it.
3. Request JSON contains exactly the selected text and no context field.
4. Authentication key and optional region use headers, never query parameters.
5. Unicode text is serialized and decoded correctly.
6. The first Azure translation becomes `translated_text` and detected languages map to `TranslationResult`.
7. Empty translations and malformed success responses fail safely.
8. Authentication, quota/rate limit, timeout, connection, unsupported-language, 5xx, and invalid-response failures remain distinguishable inside the module.
9. Error display, `Debug`, and logs contain none of the key, source text, translated text, response body, or full request URL.
10. Only retryable failures retry, at most twice; `Retry-After` is respected within the overall time budget.

Reuse the repository's privacy contract helper where its seam applies. Use a local server exclusively; do not require credentials in default tests.

### Implementation

Add `crates/translation-azure` to the workspace and implement:

- `AzureTranslatorConfig`: validated endpoint, optional region, secret-bearing credential wrapper, and timeout.
- `AzureTranslationProvider`: the only public operational type.
- Private protocol DTOs matching Azure Translator v3.
- A bounded retry policy for connection failures, HTTP 429, and 5xx responses. Never retry 400/401/403 or unsupported-language responses.
- Content-safe mapping to the existing public platform error boundary. Preserve internal categories for retry policy and tests; expose the existing desktop-level `translation_failed` behavior initially.

Use an explicit cross-platform TLS configuration and verify the crate on Windows, macOS, and Linux CI. Do not introduce Azure SDK types into public contracts.

### Acceptance

- The provider satisfies `TranslationProvider` without changing its signature.
- HTTP and Azure details are hidden behind two public construction/operation types.
- All protocol, retry, Unicode, timeout, and privacy tests pass without network access.

## Milestone 3: Compose the provider on Windows

### Tests first

Update Windows and desktop composition tests to prove:

1. Default `WindowsPlatform::new()` still exposes unavailable translation.
2. Injecting a translation provider enables `capabilities.translation`.
3. Missing Azure configuration selects the unavailable provider.
4. Valid configuration injects Azure without exposing Azure types from `platform/windows`.
5. Invalid partial configuration prevents debug startup.

### Implementation

Add a provider-injection constructor such as:

```rust
WindowsPlatform::with_translation(Arc<dyn TranslationProvider>)
```

Keep capability ownership at the platform composition boundary. In the desktop bootstrap:

1. Read `DesktopConfig`.
2. Build `AzureTranslationProvider` when configured.
3. Inject it into Windows services.
4. Otherwise construct the existing unavailable Windows services.

Do not let `platform/windows` read environment variables or depend on the Azure crate.

### Acceptance

- Windows reports translation capability when a provider is installed, even if a later request discovers invalid credentials or exhausted quota.
- No startup connectivity probe consumes Azure quota.
- macOS continues to inject Apple Translation unchanged.

## Milestone 4: Add automatic source detection to settings

### Tests first

Extend domain, storage, command-contract, and Windows settings tests for:

1. `sourceLanguage = "auto"` round-trips through settings.
2. Target language rejects blank and `auto`.
3. Existing `zh` settings normalize to `zh-Hans` without losing other preferences.
4. Simplified and Traditional Chinese remain distinct.

### Implementation

Offer these initial values:

| Display | Code | Source | Target |
|---|---|---:|---:|
| Auto detect | `auto` | yes | no |
| English | `en` | yes | yes |
| German | `de` | yes | yes |
| French | `fr` | yes | yes |
| Spanish | `es` | yes | yes |
| Chinese (Simplified) | `zh-Hans` | yes | yes |
| Chinese (Traditional) | `zh-Hant` | yes | yes |

Keep language normalization at a shared settings boundary rather than inside the Windows UI. When Azure detects a source language, persist the actual returned code in `TranslationResult`; do not persist `auto` as the language of a completed translation.

### Acceptance

- The target language is always explicit before a translation begins.
- Existing users with `zh` retain Simplified Chinese behavior.
- Provider request tests cover both explicit and automatic source modes.

## Milestone 5: Extend the capture draft state safely

### Tests first

Add coordinator and application tests for:

1. Translation success reaches an editable preview rather than forcing persistence.
2. Applying edits after translation can independently change selected text, context, and translation.
3. Editing selected text does not automatically clear or replace the translation.
4. Applying edits never writes to storage.
5. Saving persists exactly the final three field values once.
6. A blank final translation can be saved through the same `Save capture` action.
7. Retrying translation uses the current selected text and replaces the translation only on success.
8. A failed retry preserves the editable draft and recovery actions.
9. Stale request results cannot modify or save a newer capture.
10. OCR cannot begin translation before explicit confirmation.

### Implementation

Introduce the smallest explicit coordinator transition needed to edit a translated draft. Avoid UI-only mutation that bypasses request identity and save-once guarantees. Preserve independent field semantics and make retry an explicit action that overwrites the translation only after a successful provider result.

Adjust the save command/workflow so a blank translation can be persisted through `Save capture` without requiring a differently labelled UI action. The backend may still carry an explicit optional-translation decision; do not infer authorization from error-message text.

### Acceptance

- The coordinator remains the authority for draft ownership, edit transitions, translation state, and save-once behavior.
- The final stored record exactly matches the user-confirmed preview.
- OCR privacy and stale-result protections remain green.

## Milestone 6: Implement the Windows translation presentation

### Tests first

Extend `WindowsCaptureBackend` and `WindowsFloatingCapture` tests for:

1. Capability unavailable preserves the current manual workflow.
2. Capability available starts translation for a confirmed native candidate.
3. OCR candidates do not translate until confirmed.
4. A translating state prevents duplicate requests and saves.
5. Success displays translated preview but does not persist.
6. `Edit capture` exposes all three independent fields.
7. `Apply changes` returns to preview without saving.
8. `Save capture` saves translated, manually edited, or blank translation consistently.
9. Failure displays friendly copy plus retry, edit, and save actions.
10. A late response from an old request is ignored.
11. Retry translates the current selected text and replaces the translation on success.

### Implementation

Add the missing Windows backend seams:

```ts
getSettings(): Promise<Settings>
translate(requestId, text, source, target): Promise<TranslationResult>
```

Drive UI behavior from `PlatformCapabilities.translation`, not an OS-name check. Replace the hard-coded unavailability notice only when a provider is installed. Suggested failure copy:

> Automatic translation is temporarily unavailable. You can retry, edit the capture, or save it as it is.

Keep the visible save label `Save capture` in every preview state.

### Acceptance

- Windows provides the agreed automatic-translate, edit/apply, preview, and explicit-save flow.
- Manual capture and unavailable-provider behavior do not regress.
- Frontend never receives Azure endpoint, region, key, raw response, or provider-specific error bodies.

## Milestone 7: Documentation and evidence reconciliation

Update these documents narrowly:

- `docs/architecture.md`: add the optional network translation provider and configuration boundary.
- `docs/windows-platform-plan-v2.md`: replace the blanket no-network rule with the accepted development-only exception.
- `docs/windows-platform-tickets.md`: add provider, UI, privacy, and physical verification acceptance items.
- `docs/windows-development-log.md`: append implementation and verification evidence; do not rewrite existing uncommitted content.
- `docs/development.md`: document safe `.env.local`, environment precedence, and smoke-test commands.
- `.env.example`: document names without secrets.

Do not claim production readiness, account integration, packaged configuration, or physical verification that has not occurred.

## Milestone 8: Verification gates

Run the repository's existing formatting, lint, test, frontend, and Windows build gates plus the new provider tests. At minimum:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check
pnpm test
pnpm build
```

Then, only when a developer has supplied their own credentials, run the ignored live test explicitly:

```powershell
cargo test -p vocab-translation-azure --test live_translation -- --ignored --nocapture
```

The live fixture must use non-sensitive text. Its output must not print the key, full request URL, selected user content, or translated content. Record only pass/fail, target platform, language pair, and date in the development log.

## Completion criteria

- A configured Windows debug build translates selected vocabulary through Azure, previews it, permits independent edits, and saves only after explicit confirmation.
- An unconfigured build retains the current manual workflow.
- Automatic detection records the detected source language; target language is always explicit.
- OCR text never leaves the device before confirmation.
- Credentials and captured content are absent from source control, frontend state, SQLite configuration, logs, diagnostics, snapshots, and test output.
- Default CI is deterministic and makes no external Azure calls.
- The ignored Windows smoke test passes before physical capability verification is claimed.
- The documentation no longer contradicts the accepted development-only network translation decision.
- Account integration, proxying, quota ownership, release credentials, and packaging remain explicitly out of scope.
