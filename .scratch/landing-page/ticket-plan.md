# Paper landing page UX/UI ticket proposal

Status: approved-and-published

## Scope

The user approved the landing page design and narrowed the current work to UX/UI in Paper only. This proposal supersedes the earlier code-inclusive breakdown. No application code, backend work, tests, build configuration, hosting, or release work is included. Inspect existing app visuals as references without modifying the app.

Deliver Paper artboards and interaction annotations, with prototype connections only where Paper supports them. Do not promise live capture or data persistence in a design artifact. If Paper access is unavailable, report it rather than silently substituting tools. The user approved this breakdown; five local issue files have been published with Status: ready-for-agent.

## 01: Design the capture hero from first impression to saved feedback

**What to build:** A complete Paper journey showing how visitors understand the product and try collecting unfamiliar vocabulary while reading.

**Blocked by:** None (can start immediately).

- [ ] Design desktop/mobile hero layouts in dark/light themes, with dark as the first-visit presentation and a theme switch.
- [ ] Use provisional Vocab Collector branding, serif marketing headings, sans-serif body text, app-aligned colors, generous whitespace, and a quiet reading photograph. No film-derived branding, palette, or imagery.
- [ ] Compose Capture words. Keep reading., supporting copy, and Download for Windows, marking unresolved assets and release details as placeholders.
- [ ] Show Reading sample with V's introduction as default, Everyday reading, and Tech reading. Use a labeled placeholder for the exact monologue excerpt until available; reserve three prepared vocabulary choices and Simplified Chinese translations.
- [ ] Show a simplified browser with three context tabs and inline highlighted words. Clicking a word starts its capture sequence; page load waits for input.
- [ ] Anchor the current app capture window beside the selected word, with confirmation, Saved and Undo. Remove the separate demo Capture launcher; show pointer/touch controls and keyboard focus.
- [ ] Annotate visitor takeover, Replay, sample switching without data loss/autoplay restart, and reduced-motion/manual presentation as intended behavior, not implemented functionality.
- [ ] Use readable mobile capture layouts rather than shrinking a desktop window. Record the visual choices established by this section for reuse.

## 02: Design the linked Vocabulary library and detail experience

**What to build:** A second-section Paper journey where visitors find the word saved in the hero and inspect its sentence and source.

**Blocked by:** 01: Design the capture hero from first impression to saved feedback.

- [ ] Design Your words, ready when you are. and one product frame with Vocabulary/Review switches, defaulting to Vocabulary.
- [ ] Reference existing app library/detail visuals; limit demo navigation without inventing a separate product design language.
- [ ] Show the seeded library, a newly saved Vocabulary Item, selected detail, and Encounter sentence/source with Simplified Chinese translation.
- [ ] Annotate that a hero save appears prominently without auto-scrolling or switching sections. Repeated capture adds an Encounter to the same Vocabulary Item.
- [ ] Deliver desktop/mobile and dark/light states, including a readable mobile detail view and return action.
- [ ] Place a quiet Reset demo control with clear visual hierarchy.

## 03: Design the short Review journey and connected demo reset

**What to build:** A complete Paper journey through deliberate Review selection, answer reveal, remembered/forgotten choice, completion, and reset.

**Blocked by:** 02: Design the linked Vocabulary library and detail experience.

- [ ] Show Review entry, unrevealed vocabulary, revealed answer, remembered/forgotten actions, progress, and completion using existing app Review visuals as references.
- [ ] Use previously due example vocabulary; annotate that newly captured words do not enter this curated demonstration automatically.
- [ ] Deliver readable desktop/mobile and dark/light states.
- [ ] Specify view switching and returning to Vocabulary while preserving the visitor's place.
- [ ] Show Reset demo restoring initial sample, library, Review, and capture states while preserving theme choice. Distinguish Replay, which restages only the hero without changing visitor data.
- [ ] Annotate reset during unfinished capture, Review, or playback and appropriate focus destinations. Specify UX, not implementation architecture.

## 04: Design the value section and complete page navigation

**What to build:** A concise page ending that explains value and offers download, with consistent navigation across all sections.

**Blocked by:** 01: Design the capture hero from first impression to saved feedback.

- [ ] Design Keep your reading flow, Learn from your own reading, and Come back when you're ready as three concise benefits.
- [ ] Use generous whitespace and lightly bordered cards, with desktop/mobile compositions in both themes.
- [ ] Design the final Download for Windows CTA and contact/privacy/license footer. Mark missing destinations as placeholders; do not invent Beta, pricing, or trial claims.
- [ ] Complete desktop/mobile navigation: provisional logo/name, How it works anchor, theme switch, and download. Omit blog, community, testimonials, and pricing sections.
- [ ] Define relevant hover/focus/active states and distinguish download actions from demo controls.
- [ ] Keep English copy focused on convenient capture and later Review, without film associations or guaranteed learning outcomes.

## 05: Assemble and review the complete Paper design handoff

**What to build:** A coherent Paper package of full pages and annotated visitor journeys, ready for later implementation planning.

**Blocked by:** 03: Design the short Review journey and connected demo reset; 04: Design the value section and complete page navigation.

- [ ] Assemble complete desktop/mobile pages in dark/light themes, checking consistent typography, spacing, colors, framing, and section transitions.
- [ ] Organize named artboards for first visit, capture, saved-word inspection, Review, sample/theme switches, Replay, and reset. Provide ordered annotations or supported prototype connections.
- [ ] Visually check reading order, contrast, font sizes, focus visibility, touch targets, English headings, and Simplified Chinese wrapping; correct defects in Paper.
- [ ] Verify coverage of the approved outline and distinguish annotated future behavior from interactions supported by the Paper artifact.
- [ ] Collect visual tokens, component states, responsive rules, and unresolved copy/assets into a concise design handoff; preserve the boundary between marketing layout and app UI.
- [ ] Return usable Paper references and outstanding asset/content decisions. Do not produce app code or deployment tasks.

## Execution frontier

Start 01. Then 02 and 04 may proceed independently. 03 follows 02; 05 joins 03 and 04.

## Review questions

Is the granularity appropriate? Are the blocking edges correct? Should any tickets be merged or split before local publication?

