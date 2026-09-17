# W-08 physical compatibility blockers

Updated: 2026-09-02

Target environment: the user's Windows 10 Pro x64 physical machine. Windows
reports edition `Windows 10 Pro`, display version `24H2`, and kernel build
`10.0.26100.7171`.

## Current conclusion

W-08 automated behavior and the approved current-release physical matrix are
complete. Chrome static reading content, Chrome textarea selection, and Notepad
pass the portable UIA contract. Word is a known limitation deferred to later
application testing, and elevated targets are outside the current release scope.

## Resolved blocker: Notepad selection

The physical Notepad contract initially failed because the automation used
`Ctrl+A`, which included Notepad's terminal paragraph marker. UIA returned the
visible fixture text followed by `\r`, while the expected value omitted that
marker.

The test setup was corrected by moving the selection endpoint left once after
`Ctrl+A`. No production normalization was added because trimming provider output
would corrupt intentional leading, trailing, or multiline selection whitespace.

Result: **Verified Windows physical** on the target machine with Notepad
10.0.26100.8875. TextPattern2 returned
the exact Unicode fixture, enclosing context, `notepad.exe`, a non-empty window
title, and available selection bounds.

Command:

```powershell
$env:VOCAB_UIA_EXPECTED='naïve—CAFÉ 👩🏽‍💻 Straße 中文'
$env:VOCAB_UIA_EXPECTED_APP='notepad.exe'
cargo test -p vocab-platform-windows physical_uia_selection_matches_the_portable_contract -- --ignored --nocapture
```

Result: 1 passed, 0 failed.

## Recorded limitation: VS Code exposes no usable UIA text pattern

A real selection was established in VS Code, but the Windows selection provider
returned `PlatformError::UnsupportedElement`. Diagnostics reported no
TextPattern2 or TextPattern after inspecting five relevant nodes.

The same result occurred in an isolated VS Code instance launched with
`--force-renderer-accessibility`. This makes a disabled Electron accessibility
tree an unlikely explanation.

Status: **Unsupported** through UIA in the tested VS Code configuration. The
product owner declared VS Code non-material for the current release, so this does
not block W-08 or justify clipboard fallback.

Next evidence needed:

- Record the exact VS Code version and selected editor scenario.
- Exercise explicit OCR over the same selection and judge whether its result is
  acceptable.
- Verify whether user-triggered `Ctrl+C` returns the exact selection without a
  stale clipboard value. If OCR is unacceptable and Copy is reliable, this is a
  material W-14 clipboard-fallback use case.

Do not expand traversal limits or add application-name-specific behavior without
evidence that the desired text pattern exists elsewhere in the UIA tree.

## Recorded limitation: Windows Terminal needs a real mouse selection

The attempted keyboard setup did not establish a terminal text selection. The
provider returned `PlatformError::EmptySelection` after inspecting ten nodes.
This result is not evidence that Windows Terminal is incompatible; it only shows
that the automated keyboard setup did not reproduce the required user scenario.

Status: **Not run**. The product owner declared Terminal optional for the current
release, so this does not block W-08 or W-14.

Next action: manually drag-select the known fixture text in Windows Terminal,
leave Terminal in the foreground, then run the generic ignored physical test
with `VOCAB_UIA_EXPECTED_APP=WindowsTerminal.exe` during its two-second delay.

## Recorded scope: applications absent from the target machine

The target machine currently exposes these applications:

- Notepad: installed and physically verified.
- Windows Terminal: installed; reliable selection setup pending.
- VS Code: installed; UIA compatibility failure observed.
- Google Chrome 152.0.7977.75: installed; static and textarea selection physically verified.
- Microsoft Word 16.0.20326.20112: installed; deterministic selection returned `EmptySelection` at the 64-node traversal cap.

These required matrix applications were not found:

- Microsoft Edge
- Mozilla Firefox
- A standalone PDF reader

Status: **N/A** for the current compatibility surface. The product owner decided
that applications absent from the target machine do not block the release.
Application absence is not recorded as a UIA failure.

## Product-owner decision

On 2026-09-02 the product owner decided:

- Use the current-machine application matrix.
- Chrome is the required browser baseline; other browsers may be added later.
- Installed editor/article-reading applications may be evaluated, while absent
  applications do not create blockers.
- Windows Terminal is optional and may remain unverified.
- VS Code is not an important current use case.
- OCR is acceptable when ordinary English words are accurate.
- Clipboard fallback, if ever introduced, must default off and require explicit
  enablement or per-use confirmation.
- Microsoft Word is not a required current-release application; its UIA failure
  is a known limitation awaiting later application testing.
- Elevated target processes are not part of the current-release test matrix.

Chrome static and editable selections were then verified physically with exact
text, context, source metadata, and bounds. These results satisfy the required
browser baseline and close the W-08 physical gate under the approved scope.

Reproducible Chrome setup:

1. Launch Chrome 152.0.7977.75 with a fresh temporary `--user-data-dir`,
   `--force-renderer-accessibility`, `--enable-features=UiaProvider`, and
   `--app=<fixture-file-uri>`.
2. Use `uia-browser-static.html` or `uia-browser-editable.html`; each fixture
   creates its selection through standard DOM APIs during load.
3. Keep the fixture window foreground and set
   `VOCAB_UIA_EXPECTED_APP=chrome.exe` plus the exact fixture text.
4. Run the generic ignored physical contract test shown above. The static and
   editable runs each passed with exact text, context, source metadata, and
   available bounds.

## Automated evidence

The following commands passed on 2026-09-02:

```powershell
cargo test -p vocab-platform-windows
cargo test -p vocab-platform-contract-tests
cargo test -p vocab-application --test platform_fakes
cargo fmt --all --check
git diff --check
```

Observed counts were 16 non-physical Windows adapter tests with the physical test
ignored, 8 platform-contract tests, and 13 application fake-provider tests.

## W-08 acceptance status

| Acceptance requirement | Status |
|---|---|
| Focused fast path and bounded traversal terminate predictably | Verified automated |
| UTF-16, rectangles, empty/unsupported selection, and missing metadata fixtures | Verified automated |
| Default diagnostics contain metadata and lengths rather than captured content | Verified automated |
| Required Windows 10 Pro physical UIA application matrix is recorded | Verified Windows physical |

W-14 decision: clipboard fallback is **Unsupported** for the current release.
No material target application needs it: Chrome and Notepad pass UIA; VS Code,
Word, and Terminal are non-material, deferred, or optional; elevated targets are
outside the current scope. Users can choose Manual Capture when UIA is
unavailable. OCR is a deferred designed path and is not advertised while
`screenshot_ocr` is false. No clipboard contents are read, modified, or restored.
