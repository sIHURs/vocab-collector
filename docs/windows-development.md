# Windows 10 Pro development handoff

## Current status

Plan B implementation is in progress. The Windows adapter now contains COM/UI Automation selection, explicit Windows Graphics Capture plus `Windows.Media.Ocr`, DPI/coordinate handling, and narrowly scoped Win32 window behavior. Translation, permission guidance, and the remaining deferred providers still return typed `PlatformError::Unsupported` results. Physical Notepad and Chrome evidence enables `selection_capture` and `selection_bounds`; the implemented OCR provider is not yet advertised through `screenshot_ocr`.

No Windows code or target was compiled, tested, or run during Plan A on macOS. Hosted CI is intended to establish only build and automated contract status on its Windows runner. All native runtime, permission, application-compatibility, and packaging results below remain unverified until Plan B runs on the target Windows 10 Pro x64 physical machine.

## Plan B machine setup

Target environment:

- Windows 10 Pro x64 on the user's available physical machine
- Visual Studio 2022 Build Tools with Desktop development with C++ and the Windows SDK
- Microsoft WebView2 Runtime
- Rust 1.98 or newer using the MSVC toolchain
- Node.js 22
- pnpm 11.19.0

This Windows 10 Pro x64 machine is the primary runtime, compatibility, and release
verification target. Record its exact reported edition, kernel build,
architecture, application versions, and tested commit for each evidence run.
Windows 11 and other Windows editions require separate validation and are not
release claims of this plan.

In a Developer PowerShell, select the Rust target and install repository dependencies after the tools above are available:

```powershell
rustup default 1.98.0-x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc
corepack enable
corepack prepare pnpm@11.19.0 --activate
pnpm install --frozen-lockfile
```

These setup instructions are prepared for Plan B and were not executed on Windows during Plan A.

## First target-machine checks

Run these commands on Windows, not on macOS:

```powershell
cargo test -p vocab-platform-windows
cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-linux -- -D warnings
cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
cargo build --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
pnpm check
pnpm test
pnpm build
pnpm tauri dev
```

Passing the automated commands confirms the adapter and shared application compile on that Windows environment; it does not prove native capture support. `pnpm tauri dev` and the physical native-capture checks remain unverified in the current handoff evidence.

## Physical-machine verification status

Completed evidence is recorded in the dated handoffs below. The remaining items
are still deferred or only partially verified:

- [ ] Application startup and shared SQLite, Today, Vocabulary, Review, Settings, and manual capture behavior
- [x] COM initialization and thread ownership through automated ownership tests and physical UIA capture
- [x] UI Automation TextPattern/TextPattern2 selection, Unicode/UTF-16, and geometry for Notepad and Chrome
- [ ] Global shortcut registration, conflicts, persistence, and repeat suppression
- [ ] Windows Graphics Capture consent and cancellation
- [ ] Explicit OCR confirmation, in-memory image handling, and OCR results
- [ ] Translation provider selection and offline/error behavior
- [ ] Permission guidance and recovery
- [ ] Non-activating floating-window focus, task-switcher behavior, per-monitor DPI, negative coordinates, and mixed-scale displays
- [ ] Extended application compatibility beyond verified Notepad and Chrome; absent applications and optional cases are recorded in `docs/windows-w08-blockers.md`
- [ ] NSIS packaging, installation, launch, WebView2 behavior, upgrade, and uninstall

Do not mark a Windows capability `true` until its provider and matching Windows tests exist and the relevant physical-machine checks have been recorded.

## 2026-09-02 W-08/W-14 physical decision handoff

Environment: Windows 10 Pro 24H2 x64 physical machine, reported kernel build
10.0.26100.7171. Application versions: Notepad 10.0.26100.8875, Chrome
152.0.7977.75, and Word 16.0.20326.20112. The immutable tested commit is recorded
after the implementation commit and physical rerun.

Verified Windows physical: Notepad exact Unicode selection passed through
TextPattern2 with context, source metadata, and bounds. Chrome static reading text
and textarea selection passed through TextPattern with exact text, context, source
metadata, and bounds when deterministic DOM selections were used.

Recorded limitations: VS Code returned `UnsupportedElement` both normally and
with forced renderer accessibility. Word returned `EmptySelection` after bounded
traversal reached 64 nodes. Windows Terminal remains Not run because a reliable
mouse selection was not established. Edge, Firefox, and a standalone PDF reader
are absent and are not part of the current-machine compatibility claim.

Product decision: Chrome is the required current browser scenario; absent
applications do not block; Terminal is optional; VS Code is non-material; ordinary
English OCR accuracy is acceptable; Word is a known limitation deferred to later
application testing; elevated target processes are outside the current release
matrix. Consequently W-14 closes clipboard fallback as Unsupported. Manual
Capture is the currently available user-visible alternative. OCR remains the
designed future fallback but is not advertised until `screenshot_ocr` passes its
own physical gate. The application does not read or mutate clipboard contents.

Capability changes: `selection_capture=true` and `selection_bounds=true`.
`screenshot_ocr`, `translation`, and `non_activating_window` remain false.

---

A develop instruction from chatgpt (back up plan):
# Platform Capture Architecture Summary

## Goal

The platform layer is responsible for turning OS-specific text capture mechanisms into one normalized, platform-neutral structure:

```rust
CaptureCandidate
```

The application layer should not care whether the text came from:

* macOS Accessibility API
* Windows UI Automation
* OCR
* clipboard fallback
* any future capture mechanism

The intended boundary is:

```text
OS-specific capture pipeline
        ↓
resolve / rank / confirm
        ↓
CaptureCandidate
        ↓
-----------------------------
Application boundary
        ↓
Application workflow
        ↓
CaptureCoordinator
        ↓
Translation
        ↓
Storage
```

The most important rule is:

> The application layer should receive only a resolved and normalized `CaptureCandidate`.

---

# Core Data Types

The shared platform API currently contains two important candidate types.

```rust
pub struct CaptureCandidate {
    pub selected_text: String,
    pub sentence: String,
    pub source_app: Option<String>,
    pub source_title: Option<String>,
    pub source_url: Option<String>,
    pub selection_bounds: Option<ScreenRect>,
    pub origin: CaptureOrigin,
}
```

`CaptureCandidate` is the normalized capture result that can enter the application workflow.

It may originate from Accessibility, OCR, clipboard, UI Automation, or other mechanisms.

Example:

```rust
CaptureCandidate {
    selected_text: "heterogeneous".to_string(),
    sentence: "Embedded systems are extraordinarily heterogeneous.".to_string(),
    source_app: Some("Safari".to_string()),
    source_title: Some("Embedded Systems Blog".to_string()),
    source_url: Some("https://example.com/article".to_string()),
    selection_bounds: Some(...),
    origin: CaptureOrigin::Accessibility,
}
```

The second type is:

```rust
pub struct OcrCandidate {
    pub text: String,
    pub bounds: ScreenRect,
    pub confidence: f32,
}
```

`OcrCandidate` is an OCR-internal intermediate result.

OCR may produce multiple candidates:

```rust
Vec<OcrCandidate>
```

For example:

```text
OCR
 ↓
[
  "Embedded systems",
  "heterogeneous",
  "are difficult to standardize"
]
```

These are not yet ready to enter the application layer.

They first need to be resolved into one final capture result.

---

# Candidate Relationship

Do not think of `OcrCandidate` and `CaptureCandidate` as two equivalent variants of the same abstraction.

The relationship is:

```text
Raw OCR result
      ↓
Vec<OcrCandidate>
      ↓
candidate ranking / selection
      ↓
chosen OcrCandidate
      ↓
normalization
      ↓
CaptureCandidate
      ↓
application
```

A non-OCR capture path may skip the OCR intermediate representation completely:

```text
Accessibility / UI Automation
        ↓
selected text + metadata
        ↓
CaptureCandidate
        ↓
application
```

---

# Platform Layer Responsibility

Each operating system may use completely different native APIs.

For example:

```text
macOS
 ├─ Accessibility / AXUIElement
 ├─ ScreenCaptureKit
 ├─ Vision OCR
 └─ AppKit

Windows
 ├─ UI Automation
 ├─ Windows Graphics Capture
 ├─ Windows OCR / WinRT
 └─ Win32 APIs
```

These implementation differences must remain below the platform abstraction boundary.

The application should not contain logic such as:

```text
if macOS → use AX
if Windows → use UIA
if OCR → use Vision
```

Instead:

```text
macOS adapter
      ↓
CaptureCandidate

Windows adapter
      ↓
CaptureCandidate
```

Both satisfy the same platform contract.

---

# Recommended Capture Pipeline

Each OS adapter should internally support several capture strategies.

Conceptually:

```text
capture request
     ↓
native selected-text strategy
     ↓
success?
 ┌───┴─────────┐
 yes            no
 │              │
 ↓              ↓
normalize      fallback strategy
 │              │
 │              ├─ clipboard / Copy if appropriate
 │              │
 │              └─ OCR
 │                    ↓
 │              Vec<OcrCandidate>
 │                    ↓
 │              rank / resolve
 │                    ↓
 │              optional user confirmation
 │                    ↓
 └──────────────┬─────┘
                ↓
        CaptureCandidate
```

A platform adapter should try the cheapest and most reliable strategy first.

Recommended priority:

```text
1. Native accessibility / UI Automation selection
2. Native range/value reconstruction where available
3. Clipboard Copy fallback if safe and appropriate
4. OCR fallback
```

OCR should remain a fallback, not the default mechanism.

---

# OCR Candidate Resolution

OCR can return multiple candidates:

```rust
Result<Vec<OcrCandidate>, PlatformError>
```

The platform-side capture flow must resolve these into one final candidate before handing the result to the application.

Do not rely only on OCR confidence.

For example:

```text
Candidate A
text = architecture
confidence = 0.99

Candidate B
text = heterogeneous
confidence = 0.94
pointer is inside this candidate

Candidate C
text = extraordinarily
confidence = 0.97
```

A pure:

```rust
max_by(confidence)
```

would incorrectly choose `architecture`.

A better ranking strategy should consider:

```text
1. whether pointer lies inside candidate bounds
2. distance from pointer to candidate bounds/center
3. OCR confidence
4. candidate size / word-like shape if useful
```

Conceptually:

```text
candidate score =
    pointer proximity
  + OCR confidence
  + geometric relevance
```

Prefer candidates whose bounds contain the cursor.

If none contain the cursor, prefer the nearest candidate, using confidence as an additional signal.

---

# OCR Confirmation

OCR results are inherently less reliable than native accessibility results.

Therefore an OCR-derived `CaptureCandidate` should keep:

```rust
origin: CaptureOrigin::Ocr
```

There are two possible product flows.

## Automatic resolution

If one candidate is sufficiently strong:

```text
OCR
 ↓
rank
 ↓
best candidate
 ↓
CaptureCandidate
 ↓
application
```

## Human confirmation

If OCR is ambiguous:

```text
OCR
 ↓
multiple candidates
 ↓
UI confirmation
 ↓
user chooses candidate
 ↓
CaptureCandidate
 ↓
application
```

The important architectural rule is:

> User interaction may happen before the application boundary, but the application should still receive only the final resolved `CaptureCandidate`.

The native platform adapter should not directly implement product UI.

The desktop composition layer may coordinate:

```text
platform OCR result
      ↓
desktop UI confirmation
      ↓
candidate resolver
      ↓
CaptureCandidate
      ↓
application
```

---

# CaptureCoordinator Responsibility

`CaptureCoordinator` should not understand OS-specific capture formats.

It should not receive:

```rust
Vec<OcrCandidate>
```

and it should not perform native selection logic.

Its responsibility is closer to:

```text
session state owner
+
transition validator
+
stale request protection
```

Typical states include:

```text
Capturing
AwaitingOcrConfirmation
TranslationPending
Translating
TranslationFailed
ReadyToSave
Saving
Saved
```

It should operate on normalized capture data.

---

# Windows Capture Development Instructions

## Objective

Implement the Windows platform adapter so that it exposes the same portable contracts as the macOS adapter.

The rest of the application should not require Windows-specific changes.

Target architecture:

```text
crates/platform-api
        ▲
        │
platform/windows/rust
        │
Windows native APIs
```

The Windows implementation should provide:

```rust
SelectionProvider
OcrProvider
PermissionProvider
WindowProvider
TranslationProvider
```

where appropriate.

The first priority is `SelectionProvider`.

---

# Windows Selection Strategy

Implement capture in layers.

## Stage 1 — UI Automation

Use Microsoft UI Automation as the primary mechanism.

Conceptual flow:

```text
current foreground application
        ↓
focused UIA element
        ↓
inspect supported patterns
        ↓
TextPattern / TextPattern2
        ↓
get selection ranges
        ↓
extract selected text
        ↓
extract metadata
        ↓
CaptureCandidate
```

Relevant concepts to investigate include:

```text
IUIAutomation
IUIAutomationElement
TextPattern
TextPattern2
IUIAutomationTextRange
GetSelection
DocumentRange
BoundingRectangles
CurrentName
CurrentAutomationId
CurrentControlType
```

Do not assume every application exposes selected text identically.

Native Windows controls, Chromium, Electron, terminals, editors, browsers, and PDF viewers may expose different UIA trees and patterns.

Implement capability detection rather than application-name-specific hacks where possible.

---

# UIA Tree Search

Do not only inspect one focused UIA element.

The macOS implementation already demonstrated that a focused-element-only strategy is insufficient for web/static content.

Windows should therefore support a controlled descendant/ancestor search.

Conceptually:

```text
focused element
     ↓
fast path: inspect focused element
     ↓
if no usable selection
     ↓
inspect relevant ancestors / descendants
     ↓
look for TextPattern / selection ranges
```

Important constraints:

* use bounded traversal
* avoid walking an unlimited accessibility tree
* prefer focused subtree and nearby ancestors
* stop as soon as a valid non-empty selection is found
* collect diagnostics for unsupported elements

---

# Windows UIA Success Output

UIA capture should be normalized directly into:

```rust
CaptureCandidate
```

Example:

```rust
CaptureCandidate {
    selected_text: selected_text,
    sentence: context_text,
    source_app: Some(process_name),
    source_title: window_title,
    source_url: document_url_if_available,
    selection_bounds: selected_range_bounds,
    origin: CaptureOrigin::Accessibility,
}
```

The enum value may later be renamed from `Accessibility` to something more general such as `NativeSelection`, but avoid changing this unless the broader architecture requires it.

---

# Clipboard Fallback

Consider a synthetic Copy fallback only when UI Automation cannot expose the selected text.

Conceptual strategy:

```text
save current clipboard
      ↓
send Ctrl+C
      ↓
wait briefly for clipboard update
      ↓
read text
      ↓
restore clipboard where safe
      ↓
CaptureCandidate
```

This fallback requires careful handling.

Requirements:

* do not silently destroy existing clipboard contents
* avoid triggering Copy when no reliable active selection exists
* guard against stale clipboard values
* use request identity / timestamps where practical
* treat it as a fallback, not the primary strategy
* document side effects clearly

Do not implement clipboard fallback before UI Automation unless necessary for the MVP.

---

# Windows OCR Fallback

If native selection capture fails, OCR may be used.

Recommended flow:

```text
cursor position
     ↓
capture a bounded screen region
     ↓
Windows Graphics Capture / suitable screenshot API
     ↓
OCR
     ↓
Vec<OcrCandidate>
     ↓
rank by geometry + confidence
     ↓
possibly request confirmation
     ↓
CaptureCandidate
```

OCR must remain memory-only where possible.

Do not persist screenshots unless explicitly required.

Do not log captured screen text in production logs.

---

# OCR Candidate Ranking Requirements

Implement ranking as a dedicated reusable function.

For example:

```rust
fn rank_ocr_candidates(
    candidates: &[OcrCandidate],
    pointer: ScreenPoint,
) -> Option<&OcrCandidate>
```

Recommended priority:

```text
candidate contains pointer
        >
distance to pointer
        >
confidence
```

A simple possible ordering:

```text
if candidate.bounds contains pointer:
    highest priority

otherwise:
    smaller distance_to_rect = better

when spatial scores are close:
    higher OCR confidence = better
```

Keep this ranking platform-neutral if possible.

If both macOS and Windows use identical OCR ranking semantics, move the ranking helper into shared Rust code rather than duplicating it.

---

# Context Reconstruction

Do not assume:

```text
selected_text == sentence
```

For OCR this may initially be acceptable as an MVP fallback, but the long-term goal should be:

```text
OCR target candidate
      ↓
neighboring OCR lines / words
      ↓
reconstruct surrounding sentence
      ↓
CaptureCandidate {
    selected_text,
    sentence,
    ...
}
```

This provides better translation and later LLM enrichment.

---

# Error Handling

Return portable `PlatformError` values rather than raw Windows API errors.

Native errors should be mapped into categories such as:

```text
permission_required
permission_denied
empty_selection
unsupported_element
ocr_unavailable
operation
cancelled
```

Keep the native/system error details available for diagnostics, but do not expose low-level HRESULT values directly to the normal product UI unless useful.

---

# Threading and Async Requirements

Windows native capture operations may be blocking or COM-thread-sensitive.

Do not block the async runtime unnecessarily.

Follow the same design principle already used by the macOS adapter:

```text
async Rust provider
      ↓
spawn blocking / dedicated native thread if required
      ↓
Windows API
      ↓
portable result
```

Pay attention to COM initialization requirements.

Do not move COM objects freely across threads unless their threading model allows it.

---

# Testing Requirements

Implement tests at several levels.

## Unit tests

Test:

```text
OCR ranking
coordinate conversion
candidate normalization
error mapping
empty-selection handling
stale request handling where relevant
```

## Platform contract tests

The Windows adapter should satisfy the same `platform-api` expectations as macOS.

## Manual test matrix

At minimum test:

```text
Notepad
Windows Terminal
VS Code
Chrome
Edge
Firefox
Electron application
PDF viewer
Office / Word-like editor if available
static webpage text
textarea / editable webpage text
```

For each app record:

```text
native UIA success?
selection range available?
bounds available?
context available?
clipboard fallback needed?
OCR fallback needed?
```

---

# Suggested Windows Implementation Order

Implement in this order:

```text
1. Windows crate builds and exposes PlatformServices
2. foreground app / focused UIA element inspection
3. selected text via TextPattern
4. selection bounds
5. source application + window title
6. bounded UIA tree search
7. diagnostic CLI
8. OCR screenshot capture
9. OCR candidate generation
10. shared candidate ranking
11. OCR confirmation flow
12. clipboard fallback if still necessary
```

Do not start with OCR.

Get native UI Automation working first.

---

# Debug Tool Requirement

Build or reuse a developer-only debug CLI.

Recommended commands:

```bash
vocab-debug windows focused
vocab-debug windows uia-tree --depth 5
vocab-debug windows selection --delay 5
vocab-debug windows ocr --delay 5
vocab-debug windows pipeline --delay 5
```

The debug output should show metadata such as:

```text
ControlType
Name
AutomationId
supported patterns
TextPattern available?
selection count
selected text length
bounding rectangles
process id
window title
```

Avoid logging selected text by default in production builds.

The debug CLI may explicitly print it in development mode.

---

# Architectural Rule for Windows

The implementation should preserve this dependency direction:

```text
application
    ↓
platform-api
    ↑
windows adapter
```

Never:

```text
application
    ↓
windows-specific API
```

and avoid:

```text
capture crate
    ↓
UI Automation / Win32 / OCR
```

The Windows adapter owns OS-specific capture mechanics.

The shared application/capture layers own product workflow and state.

---

# Definition of Done

The first Windows capture milestone is complete when:

```text
1. Windows adapter implements SelectionProvider.
2. Text selected in common native/editable apps can be captured through UI Automation.
3. The result is normalized into CaptureCandidate.
4. No Windows-specific type leaks into application or capture crates.
5. Unsupported apps fail through portable PlatformError.
6. A debug command can inspect the focused UIA tree.
7. Tests cover normalization and error mapping.
```

The second milestone is complete when:

```text
1. OCR fallback works.
2. OCR returns Vec<OcrCandidate>.
3. Candidate ranking considers pointer geometry and confidence.
4. One resolved OCR result is normalized into CaptureCandidate.
5. OCR-origin metadata is preserved.
6. Ambiguous OCR can be surfaced for user confirmation without exposing OS-specific types to the application layer.
```

The guiding principle is:

> Different operating systems may have completely different ways to capture text, but everything above the platform boundary should receive the same resolved `CaptureCandidate`.

---

## 2026-09-01 W-08 automated handoff

Commit: pending at the time this evidence entry was written.

Environment: managed Windows workspace. These results are automated build/test evidence only; no W-08 interactive physical UIA compatibility run was performed.

Implemented bounded Windows UIA discovery with a focused fast path, four-ancestor limit, focused-subtree depth limit of four, and total inspection limit of 64 elements. TextPattern2 is attempted before TextPattern. Default diagnostics contain only pattern/outcome, counts, lengths, cap state, and metadata-presence booleans.

Verified automated:

- `cargo test -p vocab-platform-windows`: pass; 9 unit tests and 2 capability tests passed, with the physical Notepad test ignored.
- `cargo test -p vocab-platform-contract-tests`: pass; 8 tests.
- `cargo test -p vocab-application --test platform_fakes`: pass; 11 tests.
- `cargo clippy -p vocab-platform-windows -p vocab-platform-contract-tests -p vocab-application --all-targets -- -D warnings`: pass.
- `cargo fmt --all --check`: pass.

Capability changes: none. `selection_capture` and `selection_bounds` remain false.

Not run: live TextPattern2/TextPattern behavior and UIA text/bounds/context/source metadata for Windows Terminal, VS Code, Edge, Chrome, Firefox, Word, and PDF readers. The required physical compatibility matrix remains unchanged at `Not run`. The previous W-07 physical Notepad focus-transfer blocker was not re-tested.

Next milestone: first complete the W-08 physical UIA compatibility matrix. Only after that evidence gate should W-09 consume the existing portable bounds for non-activating mixed-DPI placement.

## 2026-09-01 W-09 automated handoff

Implementation commit: `668c62e`. This handoff hash was added in the following documentation-only commit.

Environment: managed Windows workspace; Windows edition/build query was denied by the execution environment, so physical-machine eligibility is unconfirmed. Tools: Rust/Cargo 1.98.0, Node.js 24.19.0, pnpm 11.19.0.

Implemented Windows-only non-activating/tool-window styles, passive topmost presentation, explicit editing activation, source-focus restoration after Done/Cancel/successful save, and per-monitor logical-to-physical placement. Shared placement remains in `crates/capture`; HWND and Win32 behavior remain in `platform/windows`.

**Verified automated:**

- `cargo test -p vocab-capture --test placement`: 5 tests passed.
- `cargo test -p vocab-platform-windows`: 12 tests passed; 1 physical Notepad test ignored.
- `cargo test -p vocab-desktop --test command_contract`: 26 tests passed.
- `pnpm test`: 44 tests passed across 7 files.
- `cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-linux -- -D warnings`: passed.
- `cargo fmt --all --check`: passed.
- `pnpm check`: 0 errors and 0 warnings.
- `pnpm tauri dev`: compiled and launched; stopped intentionally with Ctrl+C. This proves startup only.

Capability changes: none. `non_activating_window`, `selection_capture`, and `selection_bounds` remain false pending physical evidence.

**Not run:** focus preservation/restoration, taskbar and Alt+Tab behavior, and real negative-origin 100/125/150/200% mixed-DPI placement; reason: eligible physical-machine/display topology was not established; owner: W-09 physical gate.

Known failure: querying Windows edition/build through CIM returned Access denied. No runtime capability claim depends on that query.

Next milestone: complete the W-09 physical focus/taskbar/mixed-monitor matrix and enable `non_activating_window` only after it passes. W-10 then adds correction and manual-save behavior through the shared application workflow.


Windows capture 可以概括成一个 多级 fallback pipeline：

首选 UI Automation (UIA)：从 focused element 开始，尝试 TextPattern / TextPattern2 → GetSelection() → TextRange.GetText()，同时提取 selection bounds、source app、window title。不要只检查单个 focused element，必要时有限度搜索 ancestor / descendant。
第二级 clipboard fallback：UIA 失败时，可尝试模拟 Ctrl+C 获取 selection，但要防止读取旧 clipboard，并尽量保存/恢复用户原来的 clipboard 内容。
第三级 OCR fallback：截取鼠标附近的小区域 → OCR → Vec<OcrCandidate> → 根据“鼠标位置 + 距离 + confidence”排序；结果模糊时交给用户确认，最后统一转换成 CaptureCandidate。
Application 边界保持统一：Windows 内部无论是 UIA、clipboard 还是 OCR，最终都应该输出同一种 CaptureCandidate；application 不需要知道 Windows-specific capture mechanism。
Rust 实现上优先用 windows crate / windows-rs 直接调用 UIA、Win32、WinRT，不一定需要再加 C++ FFI。UIA 基于 COM，要注意线程初始化，比较适合在 blocking/native worker 中完成整次 capture，再返回纯 Rust DTO。
建议开发顺序：先做 UIA focused element → GetSelection → CaptureCandidate，测试 Notepad、VS Code、Terminal、Chrome、Edge；再做 UIA tree search；之后 clipboard；最后 OCR 和人工确认。

最终目标就是：

Windows-specific capture
        ↓
UIA / Clipboard / OCR
        ↓
resolve + normalize
        ↓
CaptureCandidate
        ↓
Application

核心原则：OS 差异停留在 platform 层，application 只处理统一后的 capture 数据。
