# Landing page design

Status: confirmed-paper-design-in-review

## Current handoff — 2026-09-13

The current design is documented in [handoff.md](handoff.md) and Paper Page 1 artboard 48. Full pages are 44–47. Use these and the delivered ticket notes as the implementation-facing source of truth.

Confirmed changes since the early interview: Style A (Paper & highlighter), Fraunces marketing headings, Inter App UI, no photographic background, no page-load autoplay, no separate demo Capture launcher. Click a prepared word to start Capture and explicitly save. How it works targets the capture demo. The real Windows App UI is authoritative; phone App previews use readable pan viewports. No Beta, pricing or trial claims are included. Release and legal/contact destinations remain placeholders.

## Historical interview record

The material below preserves earlier discussion. Proposals for photos, autoplay, a separate Capture launcher, section-two anchor destinations or unconfirmed release labels are superseded by the current handoff above; they are not additional requirements.

## Latest hero review — 2026-09-12

The confirmed ticket 01 review supersedes earlier autoplay and demo-Capture-control proposals below. Use a simplified browser with three context tabs and inline highlighted vocabulary. Clicking a word starts its capture sequence; page load waits for input. Do not show a separate Capture launcher. Anchor the current Windows app capture window to the selected word (centered, above if space permits, otherwise below, 12px gap, viewport-clamped). Preserve the article and reading position on desktop and mobile. Match the current implemented window where older Paper references differ. Saving shows Saved and Undo in place; dismiss after four seconds unless hovered or focused. These are Paper design states and intended behavior, not implemented interactions.

## Confirmed direction

- Audience: people reading foreign-language articles, including developers reading foreign-language technical material.
- Primary value: quickly and conveniently collect unfamiliar vocabulary with minimal interruption to reading. Review comes later and is a supporting benefit.
- Positioning: a practical tool that helps readers collect unfamiliar vocabulary; avoid guaranteed learning outcomes.
- First version: English throughout.
- Visual direction: a large product demonstration, generous whitespace, and lightly bordered detail cards. Start with a quiet reading-scene photograph; decide on background video later.
- Reference: Inkdrop's hero and compact detail section. The desired page is substantially shorter than Inkdrop's home page.
- Brand independence: Vocab Collector has no film theme or affiliation. V's introductory monologue is reading material chosen to spark interest in English learning, not a source for branding, palette, imagery, or product positioning.
- Theme: dark on first visit, with a light/dark switch and remembered preference. Product demonstrations follow the selected theme. Use app-aligned colors rather than film-inspired colors.

## Interview status

Rounds 1 through 6 completed. The design decisions below are confirmed individually; the consolidated outline and provisional English copy await final overall confirmation before implementation. Exact assets and publication details can be resolved during production.

## Confirmed page structure

1. Hero: a staged reading scene demonstrates selecting unfamiliar vocabulary, initiating capture, explicitly saving, and returning to reading. Do not extend this autoplay into the library or Review.
2. Product exploration: one interactive app window with Vocabulary and Review switches, defaulting to Vocabulary. Visitors can inspect context and complete a short curated Review session.
3. Value explanation: three concise benefits followed by the final download CTA.

## Confirmed interaction and release direction

- Hero automatically demonstrates the capture flow once, supports visitor participation by clicking prepared vocabulary and using a demo-local Capture control, and provides Replay.
- Windows-first download direction accepted. Use Download for Windows for an available installer; the actual release readiness and whether a Beta label applies remain to be confirmed before publication.

## Round 3 decisions

- Hero participation: visitors click a highlighted word in a preset article, use a demo-local Capture control, and explicitly confirm saving. Identify the control as a web-demo aid and explain the desktop shortcut separately.
- Link hero and library through session-local demo data, conditional on low implementation complexity. Inspection supports this approach: DemoBackend already saves and lists words, and WindowsApp accepts an injected Backend. Add a demo change notification/refresh mechanism; sharing an instance alone does not refresh mounted UI.
- Saved vocabulary appears prominently in the library without automatically scrolling visitors away from reading.
- Review uses a curated set of previously due example vocabulary. Current DemoBackend marks newly saved words due immediately, so the demo needs an explicit curated Review scope; do not silently change production scheduling to achieve this presentation.
- Second section: one large app demonstration with Vocabulary and Review switches. Default to Vocabulary; switch only on visitor action. Allow opening vocabulary details and viewing sentence/source context; Review supports reveal and remembered/forgotten for a short example session.
- Third section: concise benefits, provisionally titled Keep your reading flow, Learn from your own reading, and Come back when you're ready, followed by the download CTA. Avoid another large feature inventory.

## Open decisions

- Approval of provisional English copy and final overall outline.
- Production details: exact reading excerpts and prepared vocabulary, final typefaces and color values, and background photograph. Use placeholders in Paper until exact reading assets are selected.
- Publication prerequisites: actual installer URL/readiness, Beta status, contact and policy links, final licensed/original background asset. These need not block Paper layout.

## Round 4 decisions

- All website interface copy is English; vocabulary translations use Simplified Chinese. The initial everyday/technology article proposal is superseded by the round 5 reading-material direction below.
- Mobile keeps the demonstration interactive with a narrow layout, not a scaled-down desktop window. Vocabulary details open separately; Review remains a separate switch.
- Replay only replays the hero demonstration and does not mutate visitor data. Keep scripted playback state separate from the session data that user captures modify.
- Reset demo restores initial data across both demonstrations. Reloading the page also resets demo data. Only the theme preference is persisted; vocabulary and Review state are not.
- Minimal navigation: logo/name, How it works linking to section two, and Download for Windows.
- Footer: contact and the privacy/license information needed for publication. No blog, community, testimonials, or standalone pricing section in version one.

## Round 5 and 6 decisions

- Product name remains Vocab Collector provisionally; naming is not finalized for publication.
- The first reading example is a short excerpt from V's introductory monologue from V for Vendetta, with three prepared vocabulary choices. This is solely reading material to encourage English learning; do not associate the app brand with the film. Exact excerpt and publication asset selection remain open; no film dialogue has been added to this document.
- A quiet Reading sample selector offers V's introduction (default), Everyday reading, and Tech reading. Changing the sample preserves saved vocabulary and does not restart autoplay. Use placeholders for exact text in Paper until assets are selected.
- Use serif marketing headings and sans-serif body copy; preserve the app's own typography inside the product demonstration.
- Default to dark on first visit; offer light/dark switching and remember the preference. Product demonstrations follow the selected theme. Explicitly reject film-derived palettes, masks, posters, logos, and other thematic branding.
- Reuse actual app components for product windows. Paper defines marketing layout and interaction states; allow limited demo navigation, explanatory affordances, and narrow-screen adaptation.
- Shared UI changes flow through website rebuilds; flow changes require demo script updates; public website releases follow released app versions.

## Paper design outline

Design desktop and mobile layouts for the same three sections. Product UI references should use actual app captures; marketing chrome and responsive demo framing can be designed separately. Show relevant interaction states as additional frames, not additional page sections. The following English copy is provisional, not user-approved final wording.

### Navigation

- Product mark/name: Vocab Collector (provisional name).
- How it works anchor to section two.
- Download for Windows primary CTA. Beta annotation only if applicable at publication.
- Accessible light/dark theme toggle.

### Section 1: capture and return to reading

- Heading: Capture words. Keep reading.
- Supporting copy: Quickly save unfamiliar words as you read. Come back to review them later.
- Quiet reading-scene photograph with sufficient overlay/contrast; a large, legible staged article window is the focal point.
- Reading sample selector: V's introduction, Everyday reading, Tech reading. Default to the monologue excerpt; show three prepared vocabulary choices. No film branding or imagery.
- Autoplay once: select vocabulary, initiate capture, display confirmation, save, return to article. The scripted run must not modify the visitor's shared dataset.
- Try it prompt: Try it — select a highlighted word.
- Visitor flow: click prepared word, activate demo Capture control, inspect capture draft with Simplified Chinese translation, explicitly save, return to article.
- Distinguish the demo-local Capture aid from the actual desktop shortcut. Use verified desktop shortcut information when implementing.
- Replay control. Main download CTA with clear visual priority over demo controls.

### Section 2: vocabulary library and Review

- Heading: Your words, ready when you are.
- Supporting copy: Revisit words with their original context, then make time for a short review.
- One large product demonstration with Vocabulary and Review switches; Vocabulary is the default.
- Vocabulary: seed examples, newly captured words prominently visible, selectable details with sentence and source.
- Review: curated example session, unrevealed/revealed states, remembered/forgotten decisions, completion state.
- Share visitor capture data with section one; do not auto-scroll or auto-switch when saving.
- Provide a quiet Reset demo control that resets both sections.

### Section 3: value and final CTA

- Three concise benefits in a lightly bordered, generously spaced card layout.
- Keep your reading flow: Save an unfamiliar word, then return to what you were reading.
- Learn from your own reading: Build a vocabulary collection from the articles you read, with the context you found it in.
- Come back when you're ready: Collect words now. Return to them for review later.
- Final Download for Windows CTA followed by a compact footer.

### Required design frames

- Desktop and mobile full pages, each in dark and light themes. Dark is the primary presentation.
- Hero: autoplay, ready to try, selected vocabulary, capture confirmation, saved feedback.
- Vocabulary: initial list, visitor-saved word, word detail.
- Review: question, revealed answer, completion.
- Mobile: capture confirmation, word detail, and Review layouts at readable sizes.
- Implementation notes to resolve: keyboard access, focus management, reduced-motion behavior, and loading/fallback presentation. Use conventional accessible defaults without expanding the marketing content.

## Verified constraints

- Capture is explicitly initiated and saving requires user confirmation; do not imply passive monitoring or silent background saving.
- Selection Capture and Region OCR Capture are distinct actions.
- Browser demonstrations must simulate native capture within a staged reading scene.
- Existing Svelte UI has a Backend interface and DemoBackend that can support reuse; a dedicated Windows demo entry is still needed.
- Do not assume published pricing, trial terms, public download availability, or equal platform readiness.

## References

- https://www.inkdrop.app/
- https://github.com/craftzdog/interactive-demo-tutorial
- CONTEXT.md
- docs/adr/0001-independent-windows-presentation.md
- docs/adr/0002-azure-translator-development-provider.md
