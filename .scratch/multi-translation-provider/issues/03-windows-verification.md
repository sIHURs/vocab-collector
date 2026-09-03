# 03: Verify provider-neutral Windows translation and reproducible gates

**What to build:** Complete the development integration by proving that Windows
Native Capture and confirmed OCR behave identically with Azure or DeepL selected,
then make that behavior reproducible in CI and development documentation.

**Blocked by:** 02: Select Azure or DeepL through development configuration.

**Status:** completed — Verified automated on 2026-09-03

All acceptance criteria below are covered by the credential-free workspace,
frontend, CI-configuration, documentation, and secret-scan gates recorded in
the Windows development log. Physical provider behavior remains Not run.

## Acceptance criteria

- [ ] Windows component and command tests exercise both selected providers through portable fake/mock results, without provider-specific frontend types or branches.
- [ ] Native selections translate once, preview without persistence, allow independent edit/apply, and persist only through `Save capture` for either provider.
- [ ] OCR text reaches neither provider before explicit OCR Confirmation; retry uses the current edited selection and remains single-flight.
- [ ] Provider failures produce the same friendly recovery flow and never expose provider name, credentials, selected text, context, translation, raw response, or full URL.
- [ ] macOS, Linux, and Windows CI compile and mock-test both network provider crates with no credentials and no external service request.
- [ ] Development documentation explains provider selection, both DeepL endpoints, environment precedence, safe failure, ignored smoke tests, and the absence of automatic fallback.
- [ ] Documentation distinguishes implemented/automated behavior from real-service and physical Windows verification.
- [ ] A targeted secret scan finds no populated Azure or DeepL credential.

## Verification commands

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --exclude vocab-platform-macos
cargo build --workspace
pnpm check
pnpm test
pnpm build
git diff --check
git grep -n -E "(VOCAB_AZURE_TRANSLATOR_KEY|VOCAB_DEEPL_API_KEY)=" -- ':!docs/plans/2026-09-03-multi-translation-provider.md'
```

The final grep may find only intentionally empty template/documentation values;
any populated value fails the ticket.

**Observable increment:** A credential-free clean checkout passes the complete
Windows-applicable gate and proves that switching the configured provider changes
only composition, not capture behavior.
