# 02: Select Azure or DeepL through development configuration

**What to build:** Let a developer explicitly select Azure or DeepL through
development configuration. The desktop composition root validates and installs
only the chosen provider; no selection keeps automatic translation unavailable
and preserves the manual workflow.

**Blocked by:** 01: Translate one selection through the DeepL provider.

**Status:** completed — Verified automated on 2026-09-03

All acceptance criteria below are covered by desktop configuration/composition,
Windows capability, build, and ignore-rule gates recorded in the Windows
development log.

## Acceptance criteria

- [ ] `VOCAB_TRANSLATION_PROVIDER` accepts only `azure` or `deepl`; omission selects the unavailable provider and invalid values fail with content-safe diagnostics.
- [ ] `.env.example` documents empty Azure and DeepL settings plus the explicit selector, and `.env.local` remains ignored.
- [ ] Process environment variables take precedence over `.env.local`.
- [ ] Only the selected provider's endpoint, key, region or timeout settings are parsed; unused provider configuration cannot enable or break startup.
- [ ] Azure and DeepL keys may coexist, but the selector is the sole authority and there is no automatic fallback or guessing.
- [ ] Selecting either provider enables only the portable translation capability; the Windows adapter has no dependency on either network-provider crate.
- [ ] Missing selected-provider credentials and invalid endpoints fail safely without exposing configured values.
- [ ] Release behavior is documented accurately: `.env.local` loading is debug-only, while credential delivery and production readiness remain out of scope.

## Verification commands

```powershell
cargo fmt --all --check
cargo clippy -p vocab-desktop -p vocab-platform-windows --all-targets -- -D warnings
cargo test -p vocab-desktop
cargo test -p vocab-platform-windows
cargo build -p vocab-desktop
git check-ignore .env.local
```

**Observable increment:** Composition tests select Azure, DeepL, and unavailable
in turn and observe the correct capability without contacting either service.
