# Shadcn-Svelte UI Design Specification

## Purpose

This document defines how Vocab Collector should evolve from its current custom-styled Svelte interface into a coherent shadcn-svelte-style desktop application.

It has two uses:

1. guide the user and an agent while creating the first visual system and screen concepts in Paper;
2. guide later implementation without changing the existing product behavior, Tauri boundary, or Rust application services.

This is a visual-system and component-architecture specification, not a feature redesign. Existing workflows, accessible names, keyboard behavior, persistence rules, and platform-specific window behavior remain authoritative.

## Reference and implementation policy

- Use the legacy [shadcn-svelte example site](https://0162389a.shadcn-svelte.pages.dev/docs) for visual inspiration only.
- Use the maintained [shadcn-svelte component documentation](https://www.shadcn-svelte.com/docs/components) and [Vite installation guide](https://www.shadcn-svelte.com/docs/installation/vite) for implementation.
- Copy only the components the application needs. Do not introduce the entire catalog.
- Keep project-owned product components. Compose them from shadcn-svelte primitives rather than forcing product workflows into generic components.
- Prefer semantic design tokens over hard-coded colors inside pages.
- The intended result is “shadcn-svelte style adapted to Vocab Collector,” not a pixel-for-pixel copy of the shadcn demo dashboard.

## Current UI boundary

The frontend currently selects one of several presentations in `ui/src/main.ts`:

| Window/presentation | Current component | Purpose |
| --- | --- | --- |
| Shared main window | `App.svelte` | Today, Vocabulary, Insights, Settings |
| Windows main window | `windows/WindowsApp.svelte` | Windows-specific complete main presentation |
| Shared capture window | `FloatingCapture.svelte` | Native selection/OCR capture feedback |
| Windows capture window | `windows/WindowsFloatingCapture.svelte` | Windows capture workflow and recovery |
| Windows OCR overlay | `windows/WindowsOcrOverlay.svelte` | Full-screen region selection |

The visual system should be shared. Presentation-specific behavior may remain separate until the codebase deliberately consolidates it.

The current UI uses custom CSS for almost every primitive. Only `ShortcutRecorder.svelte` and `WindowsWordRow.svelte` are meaningfully extracted. Buttons, fields, tabs, dialogs, sheets, alerts, toasts, tables, empty states, and progress indicators are otherwise repeated in page components.

## Visual direction

The application should feel like a focused reading companion rather than an analytics dashboard.

Desired qualities:

- calm and compact desktop density;
- strong hierarchy for vocabulary, translation, and reading context;
- quiet neutral surfaces with one restrained accent color;
- clear keyboard focus and accessible state changes;
- subtle elevation for temporary capture and review surfaces;
- light and dark themes derived from the same semantic tokens;
- minimal motion, with full support for reduced-motion preferences;
- platform-appropriate window behavior while maintaining one product identity.

Avoid:

- excessive cards around every piece of content;
- oversized SaaS-dashboard metrics;
- decorative gradients that reduce text clarity;
- replacing familiar desktop controls with unusual interactions;
- using color as the only indicator of status;
- copying shadcn demo content or layout when it does not fit the learning workflow.

## Design-token foundation

Paper and the Svelte implementation should use the same semantic token names. Exact values are chosen during Paper exploration and then committed to CSS.

### Required color tokens

```text
background
foreground
card
card-foreground
popover
popover-foreground
primary
primary-foreground
secondary
secondary-foreground
muted
muted-foreground
accent
accent-foreground
destructive
destructive-foreground
border
input
ring
success
success-foreground
warning
warning-foreground
sidebar
sidebar-foreground
sidebar-accent
sidebar-accent-foreground
```

The `success` and `warning` tokens are product additions. Shadcn's standard destructive token does not cover saved, remembered, pending-translation, or recoverable-warning states.

### Required structural tokens

```text
radius-sm
radius-md
radius-lg
shadow-floating
shadow-dialog
space-1 through space-8
font-sans
font-size-caption
font-size-body
font-size-title
font-size-word
sidebar-width
content-max-width
capture-width
```

### Theme requirements

- Define light, dark, and system behavior.
- Verify all text/background pairs for readable contrast.
- Preserve Windows forced-colors support; theme colors must yield to system colors in forced-colors mode.
- Do not encode theme values directly in page components.
- The OCR selection rectangle must remain visible independently of the normal application theme.

## Component inventory

### Components available directly from shadcn-svelte

The following current needs have direct equivalents in the maintained component catalog.

| Project need | Use | Notes |
| --- | --- | --- |
| Primary, secondary, ghost, icon and destructive actions | `Button`, `Button Group` | Define product-specific size and density variants. |
| Text, number, time and search fields | `Input`, `Input Group`, `Field`, `Label` | Keep native input types where they provide OS behavior. |
| Reading context and capture text | `Textarea` | Provide compact and regular sizes. |
| Language, retention and appearance choices | `Native Select` initially; `Select` when richer behavior is justified | Native Select is safer for compact desktop settings and system accessibility. |
| Boolean settings | `Switch` | Retain explicit labels and descriptions. |
| Content grouping | `Card`, `Separator` | Avoid wrapping every row in a card. |
| Vocabulary views | `Tabs` | Active, Mastered, Achieved. |
| Vocabulary status | `Badge` | Always include readable text, not color alone. |
| Manual Capture | `Dialog` | Preserve focus trap, Escape behavior and return focus. |
| Vocabulary Detail | `Sheet` | Right-side desktop detail surface. |
| Saved/error feedback | `Sonner`, `Alert` | Toast for transient confirmation; Alert for actionable or persistent problems. |
| Loading | `Skeleton`, `Spinner` | Use Skeleton only when the final layout is predictable. |
| Empty collections and completed queues | `Empty` | Product copy and actions remain custom. |
| Review progress | `Progress` | Add text such as “3 of 10”; do not rely on the bar alone. |
| Vocabulary pagination | `Pagination` | Preserve direct page entry if still required. |
| Shortcut presentation | `Kbd` | Recording behavior remains project-owned. |
| Explanatory help | `Tooltip`, optionally `Popover` | Tooltips must not contain essential instructions. |
| Vocabulary grid | `Table`; optionally `Data Table` later | Start with Table. Data Table is unnecessary until sorting/filtering actions justify it. |
| Insights chart | `Chart` | Use only when Insights remains in product scope and real data exists. |
| Sidebar structure | `Sidebar` or project shell composed with `Button` and `Separator` | A custom compact shell may be simpler for Tauri windows. |

### Product components composed from shadcn-svelte

These components do not exist as useful one-to-one library components. They must remain named project concepts.

#### AppShell

Composed from `Sidebar`, `Button`, `Separator`, and project window chrome.

Responsibilities:

- brand and main navigation;
- active-route state;
- Local mode indicator;
- page toolbar and Manual Capture action;
- Tauri-safe sizing and platform-specific title/drag regions.

#### VocabularyRow

Composed from `Button` behavior or a semantic interactive row, `Badge`, and typography primitives.

Content:

- display form;
- translation fallback;
- lifecycle status;
- encounter count;
- last-seen metadata;
- navigation affordance.

Variants:

- default;
- hover/focus;
- mastered;
- achieved/pending deletion;
- compact layout at narrow width.

#### ReviewCard

Composed from `Card`, `Progress`, `Alert`, and `Button Group`.

States:

- question;
- answer revealed;
- submitting;
- remembered result;
- forgotten result;
- repeated-forgetting guidance;
- recoverable submission error;
- paused;
- complete.

#### CaptureCard

Composed from `Card`, `Field`, `Input`, `Textarea`, `Alert`, `Spinner`, and buttons.

States:

- ready;
- captured candidate;
- translating;
- translation unavailable/failed;
- stale translation;
- editing;
- OCR confirmation;
- multiple OCR candidates;
- saving;
- saved with Undo;
- permission required;
- empty selection;
- recoverable error.

This component has compact floating-window constraints and cannot simply reuse the normal page Dialog layout.

#### ShortcutRecorder

Composed from `Button`, `Kbd`, `Field`, and `Alert`.

The existing recording state machine remains project-owned. Design states:

- configured;
- listening;
- invalid shortcut;
- registration conflict;
- disabled/unavailable capability.

#### EncounterTimeline

Composed from project markup, `Separator`, and typography primitives. A generic Timeline component is not required.

#### SummaryStat

Composed from `Card` and typography. It should emphasize the review plan rather than imitate revenue-dashboard cards.

#### SettingsSection

Composed from `Card`, `Field`, `Native Select`, `Switch`, `Input`, `ShortcutRecorder`, and inline `Alert` messages.

Sections:

- Languages;
- Review;
- Capture;
- Appearance.

#### OcrRegionOverlay

Project-owned overlay with no shadcn equivalent.

Requirements:

- transparent full-screen surface;
- clear selection rectangle;
- concise instruction callout;
- Escape-to-cancel behavior;
- forced-colors visibility;
- no dependency on Card/Dialog positioning that could interfere with capture coordinates.

## Screen specification for Paper

Create one Paper page named `Vocab Collector — shadcn-svelte exploration` with the following sections.

### 1. Foundations

Artboards:

- `Foundations / Light`
- `Foundations / Dark`

Show:

- all color tokens with names;
- typography hierarchy;
- spacing scale;
- radius and elevation samples;
- focus ring;
- success, warning and destructive treatments.

### 2. Primitive UI kit

Create component specimen artboards for:

- Button: primary, secondary, outline, ghost, destructive, icon;
- Input and Textarea: default, focus, filled, disabled, invalid;
- Native Select and Select;
- Switch: on, off, disabled;
- Badge: active, mastered, achieved, warning;
- Alert: info, success, warning, destructive;
- Card: default and interactive;
- Tabs;
- Progress;
- Skeleton and Spinner;
- Dialog;
- Sheet;
- Toast;
- Empty state;
- Kbd.

Every interactive primitive should show at least default, hover/focus, and disabled states where applicable.

### 3. Product component kit

Create artboards for:

- `VocabularyRow / states`
- `ReviewCard / states`
- `CaptureCard / states`
- `ShortcutRecorder / states`
- `EncounterTimeline`
- `SummaryStat`
- `SettingsSection`
- `OcrRegionOverlay / instruction and active selection`

Use realistic vocabulary content instead of lorem ipsum. Long German/English/Chinese text should be included to expose wrapping problems.

### 4. Main screens

Create both light and dark variants after the component direction is approved.

#### Today

- application sidebar;
- title and Manual Capture action;
- today's review plan as the dominant element;
- recent captures list;
- empty and loading variants.

#### Vocabulary

- Active/Mastered/Achieved tabs;
- search field and result count;
- vocabulary table/list;
- status badges;
- pagination;
- open Vocabulary Detail sheet;
- no-results and empty-library variants.

#### Review

- question;
- revealed answer;
- result and next-review feedback;
- repeated-forgetting guidance;
- completion screen.

#### Settings

- all four settings sections;
- field help and validation;
- shortcut listening state;
- save-in-progress and saved confirmation.

#### Manual Capture

- normal form;
- validation error;
- saving state.

### 5. Floating and native windows

Use artboards matching the actual compact window dimensions when known. Do not scale a desktop page down to approximate these surfaces.

Create:

- Shared Capture: ready, candidate, translation failure, saved;
- Windows Capture: OCR confirmation, edit, translating, saved, failure recovery;
- OCR overlay: instruction and active selection;
- minimum-width and 150% text-scale stress cases.

## Paper workflow for user and agent

### Preparation

1. Open a Paper Desktop file and connect the official Paper MCP to the agent.
2. Create the page and artboards listed above.
3. Import selected shadcn-svelte examples with Paper Snapshot only as references.
4. Place imported references in a clearly labelled, locked `References` area.
5. Do not treat imported webpage layers as the production component system.

### Agent responsibilities

The agent should:

- read current UI copy and states from the repository;
- create foundations and component specimens using semantic token names;
- use realistic product data;
- build screens by duplicating and composing approved specimens;
- keep layers and artboards consistently named;
- produce light/dark and accessibility stress cases;
- take screenshots and compare hierarchy, spacing, clipping, contrast, and consistency;
- record unresolved decisions rather than silently inventing product behavior.

### User responsibilities

The user should decide:

- base neutral family and primary accent;
- light-first, dark-first, or equal-priority direction;
- preferred density;
- sidebar character;
- degree of elevation and translucency;
- whether Insights remains a first-class screen;
- which Today and Capture explorations best represent the product.

### Review checkpoints

Do not design all screens before review. Use these checkpoints:

1. Foundations approval.
2. Button, field, Card, VocabularyRow, and CaptureCard approval.
3. Today and Capture vertical-slice approval.
4. Remaining main screens.
5. Light/dark and accessibility stress review.

## Implementation sequence after Paper approval

### Phase 1: Foundation

- Add Tailwind and initialize current shadcn-svelte for the Vite project.
- Preserve existing Vite/Tauri configuration.
- Add semantic light/dark tokens.
- Add only the first approved primitives.
- Keep the existing UI operational throughout migration.

### Phase 2: Low-risk vertical slice

- Implement SettingsSection and Manual Capture Dialog.
- Verify keyboard navigation, focus return, validation, light/dark mode, reduced motion, and Windows forced colors.
- Adjust tokens and primitive variants before wider adoption.

### Phase 3: Core product components

- Extract and implement VocabularyRow, ReviewCard, CaptureCard, ShortcutRecorder, EncounterTimeline, and SummaryStat.
- Preserve existing component events and backend calls.
- Add component-level tests for critical states.

### Phase 4: Main screens

- Migrate Today.
- Migrate Vocabulary and Vocabulary Detail Sheet.
- Migrate Review.
- Migrate Settings completely.
- Migrate Insights only if it remains in scope.

### Phase 5: Native/floating windows

- Apply shared tokens and approved product components to capture windows.
- Keep Tauri drag regions, window sizing, focus behavior, dismissal timing, and platform-specific accessibility CSS.
- Style the OCR overlay separately.

### Phase 6: Consolidation

- Remove superseded page-level primitive CSS.
- Audit duplicate shared/Windows visual rules.
- Capture visual baselines for important states.
- Run Svelte checks, UI tests, production build, and physical Tauri checks.

## Acceptance criteria

The initial Paper design is complete when:

- foundations, primitive kit, and product component kit exist;
- Today, Vocabulary, Review, Settings, Manual Capture, Capture window, and OCR overlay are represented;
- critical loading, empty, success, warning, error, disabled, and focus states are shown;
- one approved light theme and one approved dark theme exist;
- layouts have been checked with long content and increased text size;
- imported references are separated from original Vocab Collector components;
- every product component maps to either a documented shadcn primitive or a composition described here.

The implementation is complete when:

- new features can use project-owned primitives without adding page-local button, field, dialog, badge, or toast styles;
- existing workflows and tests remain valid;
- keyboard focus, screen-reader semantics, reduced motion, forced colors, and compact-window behavior are preserved;
- the application has one recognizable visual language across shared and Windows presentations;
- the UI looks intentionally designed for vocabulary capture and review, not like an unmodified shadcn example application.

## Initial recommended component set

Begin with this bounded set:

```text
Button
Card
Input
Input Group
Textarea
Field
Label
Native Select
Switch
Tabs
Badge
Alert
Dialog
Sheet
Sonner
Progress
Skeleton
Spinner
Empty
Pagination
Kbd
Separator
```

Defer `Data Table`, `Chart`, `Command`, `Popover`, and the full `Sidebar` component until a demonstrated product need outweighs their additional complexity.
