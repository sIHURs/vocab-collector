# Azure Translator Integration Tickets

Source plan: `docs/plans/2026-09-03-azure-translator-integration.md`

These tickets cover developer-configured Azure Translator integration only. They do not include user accounts, a backend proxy, shared quotas, release credential delivery, installer configuration, or production readiness. Tickets are numbered in dependency order. `ready-for-agent` means the ticket is fully specified; its blockers must still be complete before implementation begins.

Implementation evidence as of 2026-09-03: Tickets 01–07 are implemented and
**Verified automated** through their recorded gates. Ticket 08 is **Not run**;
neither a real Azure request nor the Windows capture/OCR translation scenarios
are physically verified. See `windows-development-log.md` for command-level evidence.

## Dependency overview

```text
01 Azure provider ───────→ 02 Debug composition ─┐
                                                  │
03 Language settings ─────────────────────────────┼→ 05 Native selection flow → 06 OCR and recovery → 07 Gates/docs → 08 Physical Azure proof
                                                  │
04 Editable translated draft ─────────────────────┘
```

Tickets 01, 03, and 04 can start independently. Ticket 08 is the only ticket that requires developer credentials and physical-machine evidence.

---

# 01: Translate one vocabulary selection through the Azure provider

**What to build:** Add a standalone, replaceable Azure Translation Provider that can translate one selected vocabulary text through the existing platform contract. A local mock service must make the result observable without Azure credentials or internet access. The module owns Azure protocol details, bounded retries, timeout handling, automatic source-language detection, and content-safe failure mapping.

**Blocked by:** None (can start immediately).

**Environment:** Current environment. Default verification must use a local mock server and must not contact Azure.

**Involved directories:**

- `crates/translation-azure/` (new)
- `crates/platform-api/`
- `crates/platform-contract-tests/`
- Workspace Cargo manifests and lockfile

**Status:** ready-for-agent

**Acceptance criteria:**

- [ ] The new crate implements the existing `TranslationProvider` contract without changing its public signature.
- [ ] Its public surface is limited to validated provider configuration and the provider itself; HTTP protocol DTOs and retry machinery remain private.
- [ ] A request uses Azure Translator v3, one explicit target language, a header-based key, and an optional region header.
- [ ] Source language `auto` omits the Azure `from` parameter; an explicit source includes it.
- [ ] The JSON payload contains only the selected vocabulary text. It has no context field and cannot receive captured context through the public translation interface.
- [ ] Unicode request and response data map correctly to `TranslationResult`, including Azure's detected source language.
- [ ] Authentication, quota/rate limit, timeout, connection, unsupported-language, server, and invalid-response failures remain distinguishable inside the module.
- [ ] Only retryable failures retry, at most twice, while respecting `Retry-After` and the overall timeout budget.
- [ ] Tests prove that errors and debug output do not expose the key, source text, translated text, Azure response body, or full request URL.
- [ ] All automated tests use a local mock server and pass with no Azure environment variables.

**Verification commands:**

```powershell
cargo fmt --all --check
cargo clippy -p vocab-translation-azure --all-targets -- -D warnings
cargo test -p vocab-translation-azure
cargo test -p vocab-platform-contract-tests
```

**Observable increment:** A deterministic provider contract test sends a selected Unicode word to the local Azure-shaped endpoint and receives a portable translation result.

---

# 02: Enable Windows translation with safe developer configuration

**What to build:** Let a developer opt into Azure translation by supplying process environment variables or a debug-only, untracked `.env.local`. A complete configuration installs the provider and advertises translation capability; no configuration preserves the existing manual workflow; partial or invalid configuration fails debug startup with a redacted diagnostic.

**Blocked by:** 01: Translate one vocabulary selection through the Azure provider.

**Environment:** Current environment. No real Azure request is required.

**Involved directories:**

- `apps/desktop/src-tauri/`
- `platform/windows/`
- `crates/translation-azure/`
- Repository root configuration templates and ignore rules

**Status:** ready-for-agent

**Acceptance criteria:**

- [ ] Debug configuration supports `VOCAB_AZURE_TRANSLATOR_KEY`, `VOCAB_AZURE_TRANSLATOR_ENDPOINT`, `VOCAB_AZURE_TRANSLATOR_REGION`, and `VOCAB_AZURE_TRANSLATOR_TIMEOUT_MS`.
- [ ] Process environment variables take precedence over the workspace-root `.env.local`.
- [ ] `.env.example` contains names and safe examples but no real credentials; `.env.local` remains ignored.
- [ ] All Azure variables absent results in a normally running app with `translation=false` and the unavailable provider.
- [ ] Once any Azure configuration is present, missing required values, a malformed endpoint, a non-HTTPS production endpoint, or an invalid timeout causes a content-safe debug startup error.
- [ ] A complete configuration injects the Azure provider and reports `translation=true` without performing a startup network probe.
- [ ] `platform/windows` accepts a generic provider and has no dependency on Azure, HTTP, environment variables, or dotenv loading.
- [ ] macOS continues to compose Apple Translation unchanged.
- [ ] The key never reaches a Tauri response, WebView state, SQLite, diagnostics, snapshots, or logs.
- [ ] Release-compiled code has no `.env.local` loading path, even though packaging itself remains out of scope.

**Verification commands:**

```powershell
cargo fmt --all --check
cargo clippy -p vocab-desktop -p vocab-platform-windows --all-targets -- -D warnings
cargo test -p vocab-desktop
cargo test -p vocab-platform-windows
cargo build -p vocab-desktop
git check-ignore .env.local
```

**Observable increment:** Starting the debug app without configuration keeps translation unavailable; starting with syntactically complete test configuration makes the platform-capabilities command report translation support without contacting Azure.

---

# 03: Select automatic source detection and an explicit target language

**What to build:** Make automatic source-language detection a durable user setting while requiring a concrete target language. The choice must round-trip through the shared settings workflow and appear consistently in the Windows Settings presentation.

**Blocked by:** None (can start immediately).

**Environment:** Current environment.

**Involved directories:**

- `crates/domain/`
- `crates/application/`
- `crates/storage/`
- `apps/desktop/src-tauri/`
- `ui/src/windows/`

**Status:** ready-for-agent

**Acceptance criteria:**

- [ ] `sourceLanguage = "auto"` round-trips through domain, storage, application commands, and Windows Settings.
- [ ] Target language rejects blank and `auto` values at a shared settings boundary rather than only in the UI.
- [ ] Initial language choices include `auto` for source only, plus `en`, `de`, `fr`, `es`, `zh-Hans`, and `zh-Hant` where applicable.
- [ ] Existing persisted `zh` values normalize to `zh-Hans` without losing unrelated preferences.
- [ ] Simplified and Traditional Chinese remain distinct after save and reload.
- [ ] A completed automatically detected translation records the actual provider-returned source language rather than `auto`.
- [ ] Existing settings rollback and persistence behavior remains green.

**Verification commands:**

```powershell
cargo fmt --all --check
cargo test -p vocab-domain
cargo test -p vocab-storage
cargo test -p vocab-application
cargo test -p vocab-desktop command_contract
pnpm --dir ui test -- WindowsApp.test.ts
pnpm --dir ui check
```

**Observable increment:** A developer can choose “Auto detect” as the source and a concrete target in Windows Settings, restart or reload settings, and observe the same valid choices.

---

# 04: Edit a translated Capture Candidate and save the confirmed values

**What to build:** Extend the capture workflow so an Automatic Translation produces an editable preview rather than forcing persistence. The user can independently edit selected text, context, and translation, apply those edits without saving, and then persist exactly the previewed values through one `Save capture` action, including when translation is blank.

**Blocked by:** None (can start immediately).

**Environment:** Current environment.

**Involved directories:**

- `crates/capture/`
- `crates/application/`
- `apps/desktop/src-tauri/`
- Related contract-test fixtures

**Status:** ready-for-agent

**Acceptance criteria:**

- [ ] Translation success transitions to an editable, ready-to-save Capture Candidate.
- [ ] Applying edits can independently change selected text, context, and translation after translation succeeds.
- [ ] Editing selected text does not automatically invalidate, clear, or replace the current translation.
- [ ] Applying edits never writes a Vocabulary Item or Encounter.
- [ ] `Save capture` persists exactly the final confirmed selected text, context, and optional translation once.
- [ ] A blank translation can be saved through the same command and authorization path; UI wording is not used to infer backend permission.
- [ ] Explicit retry translates the current selected text and replaces the draft translation only after success.
- [ ] A failed retry preserves the editable draft and does not discard the last confirmed field values.
- [ ] Stale request IDs and duplicate saves remain rejected.
- [ ] Existing OCR Confirmation prevents translation before confirmation.

**Verification commands:**

```powershell
cargo fmt --all --check
cargo clippy -p vocab-capture -p vocab-application -p vocab-desktop --all-targets -- -D warnings
cargo test -p vocab-capture
cargo test -p vocab-application platform_fakes
cargo test -p vocab-desktop command_contract
```

**Observable increment:** An application-level test receives an automatic translation, independently edits all three draft fields, proves no early persistence occurred, and saves the exact preview once.

---

# 05: Translate and preview a Windows Native Capture before saving

**What to build:** Connect the configured Translation Provider to the Windows Native Capture presentation. A confirmed native selection automatically translates, displays a preview, allows edit/apply, and persists only after the user clicks `Save capture`. An unconfigured app keeps the existing manual behavior.

**Blocked by:**

- 02: Enable Windows translation with safe developer configuration.
- 03: Select automatic source detection and an explicit target language.
- 04: Edit a translated Capture Candidate and save the confirmed values.

**Environment:** Current environment. Frontend and command tests use fake or mock providers, not Azure.

**Involved directories:**

- `ui/src/windows/`
- `apps/desktop/src-tauri/`
- `crates/application/`
- `crates/capture/`

**Status:** ready-for-agent

**Acceptance criteria:**

- [ ] The Windows capture backend exposes settings lookup and translation through existing Tauri commands without provider-specific types.
- [ ] When `translation=true`, a confirmed non-OCR Native Capture automatically starts one translation request using the current selected text and language settings.
- [ ] A busy state prevents duplicate translation and save actions.
- [ ] Translation success displays the detected/selected language result and translated preview but does not persist it.
- [ ] `Edit capture` exposes selected text, context, and translation; `Apply changes` returns to preview without persistence.
- [ ] `Save capture` uses the same visible label and saves translated, manually edited, or blank translation.
- [ ] When `translation=false`, the current manual edit/save workflow remains available and no translation command is issued.
- [ ] A late response from a replaced request cannot update or save the active Capture Candidate.
- [ ] Azure endpoint, region, key, raw response, and provider-internal error bodies never enter frontend state.

**Verification commands:**

```powershell
cargo test -p vocab-desktop command_contract
pnpm --dir ui test -- WindowsFloatingCapture.test.ts
pnpm --dir ui check
pnpm --dir ui build
```

**Observable increment:** A Windows capture component test receives a Native Capture, automatically obtains a fake translation, displays it, applies independent edits, and saves only after the explicit click.

---

# 06: Protect OCR translation and recover from provider failures

**What to build:** Complete the Windows recovery path. OCR-derived text must remain local until OCR Confirmation. Translation failures display a friendly message and allow retry, independent editing, or saving the current Capture Candidate through the consistently labelled save action.

**Blocked by:** 05: Translate and preview a Windows Native Capture before saving.

**Environment:** Current environment. Simulate provider and timing failures; do not use real Azure credentials.

**Involved directories:**

- `ui/src/windows/`
- `crates/capture/`
- `crates/application/`
- `apps/desktop/src-tauri/`
- `crates/platform-contract-tests/`

**Status:** ready-for-agent

**Acceptance criteria:**

- [ ] An unconfirmed OCR candidate never invokes a Translation Provider.
- [ ] Confirming the chosen OCR candidate starts translation exactly once for that candidate.
- [ ] Provider authentication, quota/rate limit, timeout, network, unsupported-language, and invalid-response failures map to stable internal categories while the initial UI uses friendly provider-neutral copy.
- [ ] Failure UI offers `Retry translation`, `Edit capture`, and `Save capture`.
- [ ] Retry uses the current selected text, cannot run concurrently, and overwrites translation only after success.
- [ ] Edit/apply remains available after failure, including manual or blank translation.
- [ ] `Save capture` after failure saves the currently previewed values once without requiring a differently labelled button.
- [ ] A stale success or failure from an older request cannot alter the current OCR or selection flow.
- [ ] Privacy tests prove that production-visible diagnostics contain no OCR text, selected text, context, translation, credential, response body, or full URL.

**Verification commands:**

```powershell
cargo fmt --all --check
cargo test -p vocab-capture
cargo test -p vocab-application platform_fakes
cargo test -p vocab-platform-contract-tests
cargo test -p vocab-desktop command_contract
pnpm --dir ui test -- WindowsFloatingCapture.test.ts
pnpm --dir ui check
```

**Observable increment:** A Windows component test confirms an OCR candidate, observes a simulated translation failure, edits the draft, retries or saves, and proves no unconfirmed or stale content reached the provider.

---

# 07: Make Azure integration reproducible in CI and truthful in documentation

**What to build:** Establish a green, network-independent verification gate for the completed development integration and reconcile architecture and Windows planning documents with the accepted development-only Azure exception. A fresh developer must be able to understand configuration and run all mock-based tests without a secret.

**Blocked by:**

- 01: Translate one vocabulary selection through the Azure provider.
- 02: Enable Windows translation with safe developer configuration.
- 03: Select automatic source detection and an explicit target language.
- 04: Edit a translated Capture Candidate and save the confirmed values.
- 05: Translate and preview a Windows Native Capture before saving.
- 06: Protect OCR translation and recover from provider failures.

**Environment:** Current environment. CI and default local commands must not contact Azure.

**Involved directories:**

- `.github/workflows/`
- `docs/`
- Repository root configuration templates
- Workspace manifests and lockfile

**Status:** ready-for-agent

**Acceptance criteria:**

- [ ] Windows, macOS, and Linux CI compile and test the Azure provider with an explicit cross-platform TLS configuration.
- [ ] Default CI contains no Azure credential and all provider tests use local mocks.
- [ ] Architecture documentation describes the replaceable network provider and keeps credentials below the WebView boundary.
- [ ] Windows plans replace the blanket network-translation prohibition with the accepted development-only, developer-configured exception.
- [ ] Development instructions document `.env.local`, environment precedence, safe failure behavior, and the ignored live-test command without including a real key.
- [ ] Capability and ticket documentation distinguish implemented, automated, and physically verified states.
- [ ] Existing Windows development-log content is preserved; only narrow or appended evidence updates are made.
- [ ] Documentation explicitly keeps accounts, proxying, quota ownership, release credentials, installers, and production readiness out of scope.
- [ ] A secret scan or targeted repository search finds no committed Azure key or populated credential value.

**Verification commands:**

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
pnpm check
pnpm test
pnpm build
git diff --check
git grep -n "VOCAB_AZURE_TRANSLATOR_KEY=" -- ':!docs/azure-translator-integration-tickets.md'
```

The final grep may find only an intentionally empty value in `.env.example` or documentation; any populated value fails the ticket.

**Observable increment:** A clean checkout with no Azure secret passes the complete automated gate and contains accurate setup and capability documentation.

---

# 08: Prove real Azure translation on the Windows physical machine

**What to build:** Use a developer-owned Azure Translator resource to prove that the completed development path works against the real service on the target Windows machine. Capture minimal, content-safe evidence and update capability status only after success.

**Blocked by:** 07: Make Azure integration reproducible in CI and truthful in documentation.

**Environment:** Must run on the current Windows physical machine with developer-provided Azure credentials and network access. This ticket cannot be completed solely through mock tests or hosted CI.

**Involved directories:**

- `crates/translation-azure/` ignored live test
- `apps/desktop/src-tauri/`
- `ui/src/windows/`
- `docs/windows-development-log.md`
- Relevant Windows capability/ticket documentation

**Status:** ready-for-agent

**Acceptance criteria:**

- [ ] The developer supplies credentials through process environment variables or the ignored `.env.local`; no credential is committed, copied to frontend state, or printed.
- [ ] The ignored provider smoke test translates fixed, non-sensitive text with `source=auto`, returns a non-empty result, reports an actual detected source language, and uses the requested target language.
- [ ] A real Windows Native Capture automatically translates selected vocabulary, displays it without saving, supports independent edit/apply, and persists only after `Save capture`.
- [ ] A real OCR candidate is not translated until explicit OCR Confirmation.
- [ ] Invalid credentials or a disconnected network produce the friendly recovery flow without exposing Azure response bodies or configured values.
- [ ] Application logs and terminal output contain no key, selected user text, context, translated content, full request URL, or raw service response.
- [ ] Evidence records only date, Windows target, language pair, pass/fail, and the commands/scenarios exercised.
- [ ] Translation capability is marked physically verified only after all required scenarios pass.

**Verification commands:**

```powershell
cargo test -p vocab-translation-azure --test live_translation -- --ignored --nocapture
pnpm tauri dev
git status --short
```

The `pnpm tauri dev` verification is an explicit human-observed physical-machine scenario: Native Capture, OCR Confirmation, edit/apply, save, invalid credential, and offline recovery must be checked and recorded without captured content.

**Observable increment:** The developer can demonstrate a real Windows selection flowing through Azure into an editable preview and an explicitly saved Vocabulary Item, with privacy-safe evidence of success.

---

## Frontier order

1. Start Tickets 01, 03, and 04 independently.
2. Start Ticket 02 after Ticket 01.
3. Start Ticket 05 after Tickets 02, 03, and 04.
4. Start Ticket 06 after Ticket 05.
5. Start Ticket 07 after Tickets 01–06.
6. Complete Ticket 08 last on the Windows physical machine with developer credentials.

No ticket authorizes creation of GitHub or Linear issues, use of shared production credentials, or implementation of the later account-backed translation architecture.
