# Vocab Collector — Paper design handoff

Status: ready-for-review · App UI synchronized 2026-09-17 (a696b5e)

[Paper project / Page 1](https://app.paper.design/file/01M2AMNRXMP0TZ753GPD28ZSJV/1-0)

## Start here

Next phase: [development-plan.md](development-plan.md) — real App UI with browser-local sample data, selected 2026-09-17. This is the development plan; the five existing tickets remain Paper design deliverables. The interactive demo replaces recorded footage/hotspots as the primary implementation approach. References below to future recording describe optional supporting material, not a requirement for the first site.

| Artboards | Purpose |
| --- | --- |
| 44–45 | Complete desktop first-visit pages, dark/light |
| 46–47 | Complete phone first-visit pages, dark/light |
| 48 | Current design contract and journey index |
| 01–10 | Hero, anchored Capture, Saved/Undo and reading samples |
| 11–21 | Initial/saved Vocabulary, details, repeated Encounter and handoff |
| 22–37 | Review entry, recall, reveal, rating feedback and completion; phone reading/control crops |
| 38 | Second-word progress, pause/resume, reset confirmations and behavior |
| 39–42 | Benefits, final download and footer, desktop/phone dark/light |
| 43 | Navigation, control states and destination placeholders |

The three page sections are Capture, Vocabulary/Review, and benefits/download/footer. Review states replace the second-section content; they are not additional homepage sections. The full pages start with four seed Vocabulary Items and no selected word or Capture window.

## Current rules

- Audience: readers of foreign-language articles and technical material. Capture quickly without breaking reading; Review supports later recall.
- English website copy, Simplified Chinese translations. Vocab Collector is provisional branding.
- Style A: Paper & highlighter. Fraunces 500 for marketing headings; Inter for body and native App UI. No film branding or photographic background.
- First visit is dark; theme choice is remembered. Reload resets the demo dataset.
- No initial autoplay. Clicking a prepared word starts its Capture sequence; saving requires explicit confirmation. No separate demo Capture launcher.
- Capture stays beside the selected word, preferring above then below, with 12px separation and 8px viewport clearance. Saved/Undo stays in place and hides after 2 seconds; Undo closes the window. The current native implementation does not pause this timer on hover/focus. Saving does not auto-scroll to Vocabulary.
- New saves update Vocabulary. Repeated capture adds an Encounter to the same Vocabulary Item. Undo removes the latest save.
- Review uses serendipitous and resilient as previously due seeds in one batch; newly captured words do not join. Separate rings show batches completed and words completed. Show answer precedes Forgot/Remembered. Rating feedback stays until Next. Completion reflects the actual choices and ends with All due words reviewed / End review for this fixture.
- Website tab switching preserves the current frame, revealed answer, rating and pan/selection. Native App close follows native pause/resume behavior.
- Replay restages only the current hero sample and does not mutate data. Resolve an open Capture draft before Replay.
- Reset confirmation restores the first sample, Capture, original four words/Encounters and two-word Review. Preserve theme and active website tab; stop playback. Cancel preserves data and leaves playback paused. See 38 for focus rules.
- Logo targets the page top; How it works targets the Capture demo. Neither starts playback. All Windows download CTAs share the eventual release destination.

## Visual values

| Role | Value |
| --- | --- |
| Marketing background | Dark #20201E; light #FFFFFF |
| Primary/secondary text | Dark #FAFAFA / #C8C8C8; light #20201E / #666666 |
| Accent | #E8D98C |
| Marketing cards | Dark #262624 / border #45453F; light #FFFFFF / border #DDDDD7 |
| Native App | Existing theme: dark background #171717, surface #202020; preserve App status/control colors |
| Desktop type | Hero 64/66px; sections 48/54px; benefits 28/34px; body 18/28px |
| Phone type | Hero 44/48px; sections 36/40px; benefits 28/34px; body 16/24px |
| Marketing heading weight | Fraunces 500; headline tracking -0.025em |
| Layout | Desktop 1440px, content 1120px, native demo 1040 × 750; phone 390px, content 358px |
| Controls | Website touch targets at least 44px; native App controls retain original dimensions |

Spacing follows 8/12/16/24/32/48/56/72/80px values in Paper. Phone benefit cards stack vertically. Native App previews use 1× pan crops; Review has reading and action positions plus a proposed expanded player. This adds horizontal movement but avoids shrinking native text. Capture/detail retain their dedicated narrow states. Preserve the marketing/App boundary.

## Coverage and verification

Reviewed the four complete compositions, selected native Capture/detail states, Review states and handoff. Checked reading order, section gaps, dark/light contrast, headline/body hierarchy, English/CJK wrapping and intentional cropping. Corrected phone first-visit data and normalized website tab/pan touch targets across references and full pages. Full-page sizes are 1440 × 3269 desktop and 390 × 3978 phone.

The artifact contains editable static layouts and behavior annotations. It does not implement hotspots, playback, data storage, theme switching, pan/zoom, reset or downloads. No functional tests, App code, recordings or deployment are included.

## Remaining decisions

- Final product name and publication copy sign-off.
- Exact V monologue excerpt and final three reading samples/word translations. The displayed first article is explicitly labeled original placeholder prose.
- Actual Windows installer/release destination and supported platform/package details.
- Owner-approved Contact, Privacy and License destinations.
- Real seeded Review dates and branch-specific completion results for the recording; present dates/counts are illustrative fixtures.

## Future App updates

### Verified UI revision — 2026-09-17

Read-only source audit against repository HEAD `a696b5e`: `WindowsApp.svelte`, `BatchProgressRing.svelte`, `WindowsFloatingCapture.svelte`, `native-capture.css`, `CaptureSource.svelte`, `VocabularyDetail.svelte`, `VocabularyTable.svelte`, `TranslationText.svelte`, button variants and theme tokens.

- Updated all 14 Review card states, including both themes, phone crops and second-word examples: 48px batch/word rings; fixed 620 × 480px card at the 1040 × 750 recording size; content scroll region and fixed actions. Native height is responsive (`clamp(360px, 100dvh - 220px, 480px)`). Phone preview height is 514px, preserving the existing horizontal pan approach.
- Entry reads `2 total due · 2 in this batch`. Completion shows `All due words reviewed`, `1 of 1 batches completed`, `0 words still due`, actual results, and `End review`. More-due batch continuation is documented, not falsely shown for the two-word fixture.
- Next flips out and in for 180ms each; reduced motion skips animation. Paper records this as behavior only.
- Added saved translations beneath each Encounter sentence in all six detail/history examples. A target-matching Chinese translation has no extra language label; the App labels translations in other languages when applicable.
- Corrected all six Capture examples to 32px close controls and current floating shadow, with source metadata in the clipped scroll body and persistent action footer. Corrected the old four-second/hover-pause instruction to the current two-second behavior.
- Corrected Chinese App text to Microsoft YaHei, matching the App fallback rather than Paper's serif fallback, including the four complete-page clones. English App text remains Inter.
- Audited the first-visit full pages (44–47): Vocabulary remains the initial state; Review changes belong to its alternate states, not a new homepage section. Existing four demo words remain curated fixtures, not the App's first-use Starter Vocabulary Item.

Screenshot review covers desktop/mobile, dark/light Review, Capture and detail states, repeated history, and the complete-page App previews. These are editable static Paper designs, not a running App or an interactive recording. No application code was modified.

Record the released App using fixed sample data, window dimensions and theme. Keep the App version, sample text, steps and hotspot regions together. When UI or flow changes, re-record affected states and verify the complete Capture → Vocabulary → Review route in both themes. Review marketing-only changes separately. Paper does not automatically synchronize with subsequent App builds.

This handoff supersedes conflicting early interview proposals in spec.md and the original ticket-plan.md. It does not authorize or begin implementation work.
