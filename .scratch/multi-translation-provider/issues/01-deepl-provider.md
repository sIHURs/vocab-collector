# 01: Translate one selection through the DeepL provider

**What to build:** Add a replaceable DeepL Translation Provider that translates
one selected vocabulary value through the existing portable translation
contract. A local DeepL-shaped mock service makes the complete request/result
path observable without credentials or internet access.

**Blocked by:** None (can start immediately).

**Status:** completed — Verified automated on 2026-09-03

All acceptance criteria below are covered by the provider, privacy, timeout,
configuration, and ignored-live-test compilation gates recorded in the Windows
development log. No real DeepL request was made.

## Acceptance criteria

- [ ] The provider implements the existing portable translation contract without changing its signature or exposing DeepL protocol types.
- [ ] DeepL Free and Pro are selected solely through a validated HTTPS endpoint.
- [ ] Automatic source detection omits `source_lang`; explicit sources are sent, and every request has one explicit target language.
- [ ] Language-code adaptation is private to the provider and preserves the application's normalized language settings and returned detected language.
- [ ] The request contains only selected vocabulary text, never capture context or an OCR image.
- [ ] Authentication, quota/rate limit, timeout, network, unsupported-language, server, and invalid-response failures remain stable internal categories.
- [ ] Retryable failures retry at most twice while respecting `Retry-After` and one overall timeout budget.
- [ ] Errors and debug output contain no key, selected text, translated text, raw response, or full URL.
- [ ] Default tests use only a local mock; a fixed-text real-service smoke test is present but ignored by default.

## Verification commands

```powershell
cargo fmt --all --check
cargo clippy -p vocab-translation-deepl --all-targets -- -D warnings
cargo test -p vocab-translation-deepl
cargo test -p vocab-platform-contract-tests
```

**Observable increment:** A deterministic test translates a Unicode selection
through a local DeepL-shaped endpoint and receives a portable result.
