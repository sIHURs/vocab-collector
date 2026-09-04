<script lang="ts">
  import { onMount, tick } from "svelte";
  import { createBackend, type Backend } from "../lib/backend";
  import type { CaptureCard, ReviewCard, ReviewRating, ReviewResult, ReviewSessionInsight, Settings, SystemSettingsStatus, TodayView, WordDetail, WordListItem } from "../lib/types";
  import WindowsWordRow from "./WindowsWordRow.svelte";

  type Route = "Today" | "Vocabulary" | "Review" | "Settings";
  export let api: Backend = createBackend();
  const navigation: Route[] = ["Today", "Vocabulary", "Review", "Settings"];
  let route: Route = "Today";
  let today: TodayView | null = null;
  let words: WordListItem[] = [];
  let selectedDetail: WordDetail | null = null;
  let savedCard: CaptureCard | null = null;
  let captureOpen = false;
  let loading = true;
  let saving = false;
  let error = "";
  let search = "";
  let vocabularyPage = 1;
  let vocabularyPageInput = "1";
  const vocabularyPageSize = 10;
  let captureError = "";
  let captureInput = { selectedText: "", translation: "", sentence: "" };
  let reviewIndex = 0;
  let reviewOpen = false;
  let reviewComplete = false;
  let reviewPaused = false;
  let reviewRefreshRequired = false;
  let reviewCompleting = false;
  let reviewSubmitting = false;
  let reviewError = "";
  let reviewRevealed = false;
  let reviewResult: ReviewResult | null = null;
  let reviewSubmissionId: string | null = null;
  let reviewSubmissionRating: ReviewRating | null = null;
  let reviewRequestVersion = 0;
  let reviewSessionResults: ReviewResult[] = [];
  let reviewSessionCards: ReviewCard[] = [];
  let reviewSessionInsight: ReviewSessionInsight | null = null;
  let settingsDraft: Settings | null = null;
  let appliedSettings: Settings | null = null;
  let settingsSaving = false;
  let settingsError = "";
  let settingsSaved = false;
  let settingsSavedTimer: ReturnType<typeof setTimeout> | undefined;
  let initialLoadComplete = false;
  let systemStatus: SystemSettingsStatus = {};
  let captureTrigger: HTMLElement | null = null;
  let captureFirstField: HTMLInputElement;
  let captureDialog: HTMLElement;
  let detailTrigger: HTMLElement | null = null;
  let detailCloseButton: HTMLButtonElement;
  let reviewCardElement: HTMLElement;
  let reviewCompleteHeading: HTMLHeadingElement;

  $: filteredWords = words.filter((word) => `${word.displayForm} ${word.translation ?? ""}`.toLowerCase().includes(search.trim().toLowerCase()));
  $: vocabularyPageCount = Math.max(1, Math.ceil(filteredWords.length / vocabularyPageSize));
  $: if (vocabularyPage > vocabularyPageCount) vocabularyPage = vocabularyPageCount;
  $: vocabularyPageInput = String(vocabularyPage);
  $: pagedWords = filteredWords.slice((vocabularyPage - 1) * vocabularyPageSize, vocabularyPage * vocabularyPageSize);
  $: activeReview = today?.reviewQueue[reviewIndex] as ReviewCard | undefined;

  async function refresh() {
    loading = true;
    error = "";
    try {
      const [nextToday, nextWords, nextSettings] = await Promise.all([api.getToday(), api.listWords(), api.getSettings()]);
      today = nextToday;
      words = nextWords;
      settingsDraft = { ...nextSettings };
      appliedSettings = { ...nextSettings };
      if (api.getWindowsSettingsStatus) systemStatus = await api.getWindowsSettingsStatus();
      if (selectedDetail) selectedDetail = await api.getWord(selectedDetail.item.id);
      return true;
    } catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
    finally { loading = false; initialLoadComplete = true; }
    return false;
  }

  async function openCapture(event?: MouseEvent) {
    captureTrigger = event?.currentTarget instanceof HTMLElement ? event.currentTarget : document.activeElement instanceof HTMLElement ? document.activeElement : null;
    captureInput = { selectedText: "", translation: "", sentence: "" };
    captureError = "";
    savedCard = null;
    captureOpen = true;
    await tick();
    captureFirstField?.focus();
  }

  function closeCapture() {
    captureOpen = false;
    void tick().then(() => captureTrigger?.focus());
  }

  function trapDialogFocus(event: KeyboardEvent) {
    if (event.key !== "Tab") return;
    const focusable = [...captureDialog.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), textarea:not(:disabled), select:not(:disabled), [tabindex]:not([tabindex="-1"])')];
    const first = focusable[0];
    const last = focusable.at(-1);
    if (!first || !last) return;
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    if (captureOpen) {
      event.preventDefault();
      void closeCapture();
    } else if (selectedDetail) {
      event.preventDefault();
      void closeDetail();
    }
  }

  async function saveCapture() {
    if (!captureInput.selectedText.trim() || !captureInput.sentence.trim()) return;
    saving = true;
    captureError = "";
    try {
      savedCard = await api.capture({ ...captureInput, translation: captureInput.translation.trim() || undefined, captureOrigin: "manual" });
      closeCapture();
      await refresh();
      reviewComplete = false;
    } catch (cause) { captureError = cause instanceof Error ? cause.message : String(cause); }
    finally { saving = false; }
  }

  async function showDetail(wordId: string, trigger: HTMLButtonElement) {
    error = "";
    detailTrigger = trigger;
    try {
      selectedDetail = await api.getWord(wordId);
      await tick();
      detailCloseButton?.focus();
    }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }

  async function closeDetail() {
    selectedDetail = null;
    await tick();
    detailTrigger?.focus();
  }

  async function undoSaved() {
    if (!savedCard) return;
    error = "";
    try { await api.undoCapture(savedCard.encounterId); savedCard = null; await refresh(); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }

  async function startReview() {
    if (!today?.reviewQueue.length || reviewRefreshRequired) return;
    route = "Review";
    reviewOpen = true;
    reviewComplete = false;
    reviewError = "";
    reviewRevealed = false;
    reviewResult = null;
    reviewSubmissionId = null;
    reviewSubmissionRating = null;
    if (!reviewPaused) {
      reviewSessionResults = [];
      reviewSessionCards = [...(today?.reviewQueue ?? [])];
      reviewSessionInsight = null;
    }
    await tick();
    reviewCardElement?.querySelector<HTMLButtonElement>(".review-actions button")?.focus();
  }

  async function revealReview() {
    reviewRevealed = true;
    await tick();
    reviewCardElement?.querySelector<HTMLButtonElement>(".review-actions button")?.focus();
  }

  async function closeReview() {
    if (reviewSubmitting) return;
    reviewOpen = false;
    reviewRequestVersion += 1;
    reviewPaused = true;
    reviewRevealed = false;
    reviewResult = null;
    reviewSubmissionId = null;
    reviewSubmissionRating = null;
    if (await refresh()) reviewIndex = 0;
    else reviewRefreshRequired = true;
  }

  async function retryReviewRefresh() {
    if (!(await refresh())) return;
    reviewIndex = 0;
    reviewRefreshRequired = false;
    if (reviewCompleting) {
      reviewCompleting = false;
      reviewComplete = true;
      await tick();
      reviewCompleteHeading?.focus();
    }
  }

  async function retryError() {
    if (reviewRefreshRequired) await retryReviewRefresh();
    else await refresh();
  }

  async function rateReview(rating: ReviewRating) {
    if (!activeReview || reviewSubmitting) return;
    const wordId = activeReview.wordId;
    if (reviewSubmissionRating && reviewSubmissionRating !== rating) return;
    reviewSubmissionRating = rating;
    reviewSubmissionId ??= globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;
    const requestVersion = ++reviewRequestVersion;
    reviewSubmitting = true;
    reviewError = "";
    try {
      const result = await api.submitReview(wordId, rating, reviewSubmissionId);
      if (requestVersion === reviewRequestVersion && activeReview?.wordId === wordId) {
        reviewResult = result;
        if (!reviewSessionResults.some((item) => item.wordId === result.wordId)) {
          reviewSessionResults = [...reviewSessionResults, result];
        }
        await tick();
        reviewCardElement?.querySelector<HTMLButtonElement>(".review-actions button:last-child")?.focus();
      }
    } catch (cause) {
      if (requestVersion === reviewRequestVersion) {
        reviewError = cause instanceof Error ? cause.message : String(cause);
      }
    } finally {
      if (requestVersion === reviewRequestVersion) {
        reviewSubmitting = false;
        if (reviewError) {
          await tick();
          reviewCardElement?.querySelector<HTMLButtonElement>(".review-actions button")?.focus();
        }
      }
    }
  }

  async function retryReviewSubmission() {
    if (reviewSubmissionRating) await rateReview(reviewSubmissionRating);
  }

  async function nextReview() {
    if (!reviewResult) return;
    if (reviewIndex + 1 < (today?.reviewQueue.length ?? 0)) {
      reviewIndex += 1;
      reviewRevealed = false;
      reviewResult = null;
      reviewError = "";
      reviewSubmissionId = null;
      reviewSubmissionRating = null;
      await tick();
      reviewCardElement?.querySelector<HTMLButtonElement>(".review-actions button")?.focus();
    } else {
        const nextDayEnd = new Date();
        nextDayEnd.setDate(nextDayEnd.getDate() + 2);
        nextDayEnd.setHours(0, 0, 0, 0);
        try {
          reviewSessionInsight = await api.getReviewSessionInsight(
            reviewSessionResults.map((result) => result.submissionId), nextDayEnd.toISOString()
          );
        } catch (cause) {
          reviewError = cause instanceof Error ? cause.message : String(cause);
          return;
        }
        reviewOpen = false;
        reviewPaused = false;
        reviewCompleting = true;
        if (await refresh()) {
          reviewIndex = 0;
          reviewCompleting = false;
          reviewComplete = true;
          await tick();
          reviewCompleteHeading?.focus();
        } else reviewRefreshRequired = true;
    }
  }

  async function returnToToday() {
    await refresh();
    route = "Today";
  }

  const attentionLabel = (wordId: string) =>
    reviewSessionCards.find((card) => card.wordId === wordId)?.displayForm ?? "Vocabulary Item";

  async function saveSettings() {
    if (!settingsDraft || settingsSaving) return;
    clearTimeout(settingsSavedTimer);
    settingsSaving = true;
    settingsError = "";
    settingsSaved = false;
    error = "";
    const candidate = {
      ...settingsDraft,
      dailyLimit: Number(settingsDraft.dailyLimit),
      recentCapturesLimit: Number(settingsDraft.recentCapturesLimit),
    };
    try {
      let persisted: Settings;
      if (api.applyWindowsSettings) {
        const result = await api.applyWindowsSettings(candidate);
        persisted = result.settings;
        systemStatus = {
          selectionShortcutError: result.selectionShortcutError,
          regionOcrShortcutError: result.regionOcrShortcutError,
          autostartError: result.autostartError,
          notificationError: result.notificationError,
        };
      } else {
        await api.updateSettings(candidate);
        persisted = await api.getSettings();
        systemStatus = {};
      }
      settingsDraft = { ...persisted };
      appliedSettings = { ...persisted };
      settingsSaved = true;
      settingsSavedTimer = setTimeout(() => { settingsSaved = false; }, 1200);
      try { today = await api.getToday(); }
      catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
    } catch (cause) { settingsError = cause instanceof Error ? cause.message : String(cause); }
    finally { settingsSaving = false; }
  }

  async function selectRoute(item: Route) {
    route = item;
    selectedDetail = null;
    if (item === "Settings" && api.getWindowsSettingsStatus) {
      try { systemStatus = await api.getWindowsSettingsStatus(); }
      catch (cause) { settingsError = cause instanceof Error ? cause.message : String(cause); }
    }
  }


  function updateSearch(value: string) {
    search = value;
    vocabularyPage = 1;
  }

  function goToVocabularyPage(value: number | string) {
    const requestedPage = Number(value);
    vocabularyPage = Number.isFinite(requestedPage)
      ? Math.min(vocabularyPageCount, Math.max(1, Math.floor(requestedPage)))
      : vocabularyPage;
    vocabularyPageInput = String(vocabularyPage);
  }

  const encounterLabel = (count: number) => `${count} encounter${count === 1 ? "" : "s"}`;
  onMount(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;
    let unlistenManualCapture: (() => void) | undefined;
    void refresh();
    if (api.listenOpenManualCapture) {
      void api.listenOpenManualCapture(() => { void openCapture(); }).then((nextUnlisten) => {
        if (mounted) unlistenManualCapture = nextUnlisten;
        else nextUnlisten();
      });
    }
    if (api.listenLibraryChanged) {
      void api.listenLibraryChanged(() => { void refresh(); }).then((nextUnlisten) => {
        if (mounted) unlisten = nextUnlisten;
        else nextUnlisten();
      });
    }
    return () => {
      mounted = false;
      clearTimeout(settingsSavedTimer);
      unlisten?.();
      unlistenManualCapture?.();
    };
  });
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="windows-presentation" data-settings-ready={initialLoadComplete} data-appearance={appliedSettings?.appearance ?? "system"} data-reduced-motion={appliedSettings?.reducedMotion ?? false}>
<div class="windows-shell" data-presentation="windows-main" data-appearance={appliedSettings?.appearance ?? "system"} data-reduced-motion={appliedSettings?.reducedMotion ?? false}>
  <aside>
    <div class="brand"><span aria-hidden="true">V</span><strong>Vocab Collector</strong></div>
    <nav aria-label="Main navigation">{#each navigation as item}<button class:active={route === item} aria-current={route === item ? "page" : undefined} onclick={() => selectRoute(item)}>{item}</button>{/each}</nav>
    <div class="local-status"><i aria-hidden="true"></i><span>Local mode</span></div>
  </aside>
  <main>
    <header><div><h1 id="windows-page-title">{route}</h1><p>{route === "Today" ? "Your words, ready when you are" : "Vocab Collector for Windows"}</p></div><button class="primary" onclick={openCapture}>Manual capture</button></header>
    <section aria-labelledby="windows-page-title" aria-busy={loading}>
      {#if error}<div class="error" role="alert"><span>{error}</span><button onclick={retryError}>Try again</button></div>{/if}
      {#if loading}
        <div class="state" role="status">Loading your vocabulary...</div>
      {:else if route === "Today"}
        <div class="summary"><div><span>Today's plan</span><strong>{today?.plannedReviewCount ?? 0}</strong><small>{today?.plannedReviewCount ? `${today.totalDueCount} total due · About ${today?.estimatedMinutes ?? 0} minute${today?.estimatedMinutes === 1 ? "" : "s"}` : "Review queue is clear"}</small>{#if today?.plannedReviewCount}<button class="primary" onclick={startReview}>Start review ({today.plannedReviewCount})</button>{/if}</div><div><span>Recent captures</span><strong>{today?.recentCaptures.length ?? 0}</strong><small>Stored locally</small></div></div>
        <div class="section-heading"><h2>Recent captures</h2><p>New contexts appear here immediately after saving.</p></div>
        <div class="list recent-captures-list">{#each today?.recentCaptures ?? [] as word}<WindowsWordRow {word} onSelect={showDetail} />{:else}<div class="state"><strong>No captures yet</strong><span>Use Manual capture to save your first reading context.</span><button class="primary" onclick={openCapture}>Manual capture</button></div>{/each}</div>
      {:else if route === "Vocabulary"}
        <div class="tools"><label><span>Search</span><input aria-label="Search vocabulary" value={search} oninput={(event) => updateSearch(event.currentTarget.value)} placeholder="Word or translation" /></label><span>{filteredWords.length} items</span></div>
        <div class="list vocabulary-list">{#each pagedWords as word}<WindowsWordRow {word} onSelect={showDetail} />{:else}{#if words.length}<div class="state"><strong>No matching vocabulary</strong><span>Try a different word or translation.</span></div>{:else}<div class="state"><strong>Your vocabulary is empty</strong><span>Saved words will appear here.</span></div>{/if}{/each}</div>
        {#if filteredWords.length}<nav class="pagination" aria-label="Vocabulary pages"><button class="secondary" disabled={vocabularyPage === 1} onclick={() => goToVocabularyPage(1)}>First</button><button class="secondary" disabled={vocabularyPage === 1} onclick={() => goToVocabularyPage(vocabularyPage - 1)}>Previous</button><form aria-label="Go to vocabulary page" onsubmit={(event) => { event.preventDefault(); goToVocabularyPage(vocabularyPageInput); }}><label><span>Page</span><input aria-label="Page number" type="number" min="1" max={vocabularyPageCount} bind:value={vocabularyPageInput} onblur={() => goToVocabularyPage(vocabularyPageInput)} /><span>of {vocabularyPageCount}</span></label></form><button class="secondary" disabled={vocabularyPage === vocabularyPageCount} onclick={() => goToVocabularyPage(vocabularyPage + 1)}>Next</button><button class="secondary" disabled={vocabularyPage === vocabularyPageCount} onclick={() => goToVocabularyPage(vocabularyPageCount)}>Last</button></nav>{/if}
      {:else if route === "Review"}
        {#if reviewOpen && activeReview}
          <div class="review-card" aria-live="polite" bind:this={reviewCardElement}><div class="review-progress"><span>{reviewIndex + 1} of {today?.reviewQueue.length}</span><button class="icon" aria-label="Close review" disabled={reviewSubmitting} onclick={closeReview}>×</button></div><span class="eyebrow">Do you remember this word?</span><h2>{activeReview.displayForm}</h2><p>{activeReview.context ?? "No saved context"}</p>{#if reviewResult}<div class="review-translation" role="status"><small>{reviewResult.rating === "remembered" ? "Remembered" : "Forgot"}</small><strong>Next review {new Date(reviewResult.nextDueAt).toLocaleDateString()}</strong><span>{`Encountered ${reviewResult.encounterCount} time${reviewResult.encounterCount === 1 ? "" : "s"}`}</span>{#if reviewResult.repeatedForgetting}<p>This Vocabulary Item has been repeatedly forgotten. Another context or a translation check may help.</p>{/if}</div>{:else if reviewRevealed}<div class="review-translation" role="status"><small>Translation</small><strong>{activeReview.translation ?? "Unavailable"}</strong></div>{/if}{#if reviewError}<div class="dialog-error" role="alert">{reviewError}</div>{/if}<div class="review-actions">{#if reviewResult}{#if reviewResult.repeatedForgetting}<button class="secondary" onclick={(event) => showDetail(activeReview.wordId, event.currentTarget)}>Review contexts</button>{/if}<button class="primary" onclick={nextReview}>Next</button>{:else if reviewRevealed}{#if reviewError && reviewSubmissionRating}<button class="primary" disabled={reviewSubmitting} onclick={retryReviewSubmission}>Retry {reviewSubmissionRating === "remembered" ? "Remembered" : "Forgot"}</button>{:else}<button class="secondary" disabled={reviewSubmitting} onclick={() => rateReview("forgot")}>Forgot</button><button class="primary" disabled={reviewSubmitting} onclick={() => rateReview("remembered")}>Remembered</button>{/if}{:else}<button class="primary" onclick={revealReview}>Show answer</button>{/if}</div></div>
        {:else if reviewComplete && reviewSessionInsight}
          <div class="state" aria-live="polite"><h2 tabindex="-1" bind:this={reviewCompleteHeading}>Review complete</h2><strong>{reviewSessionInsight.reviewedCount} reviewed</strong><span>{reviewSessionInsight.rememberedCount} remembered · {reviewSessionInsight.forgottenCount} forgot</span><span>Estimated due by the end of tomorrow: {reviewSessionInsight.nextDayDueCount}</span>{#if reviewSessionInsight.attentionWordIds.length}<div><strong>Worth another context</strong>{#each reviewSessionInsight.attentionWordIds as wordId}<span>{attentionLabel(wordId)} may benefit from another context or a translation check.</span>{/each}</div>{/if}<button class="primary" onclick={returnToToday}>Back to Today</button></div>
        {:else if reviewRefreshRequired}
          <div class="state"><strong>{reviewCompleting ? "Review saved" : "Review paused"}</strong><span>Refresh Today before continuing so the due queue stays current.</span><button class="primary" onclick={retryReviewRefresh}>Retry Review refresh</button></div>
        {:else if today?.reviewQueue.length}
          <div class="state"><strong>{reviewPaused ? "Review paused" : `${today.plannedReviewCount} planned · ${today.totalDueCount} total due`}</strong><span>{reviewPaused ? `${today.reviewQueue.length} words remaining.` : `About ${today.estimatedMinutes} minute${today.estimatedMinutes === 1 ? "" : "s"}.`}</span><button class="primary" onclick={startReview}>{reviewPaused ? "Resume review" : "Start review"}</button></div>
        {:else}
          <div class="state"><strong>Nothing due</strong><span>Your review queue is clear for today.</span></div>
        {/if}
      {:else if settingsDraft}
        <form class="settings" onsubmit={(event) => { event.preventDefault(); saveSettings(); }}>
          {#if settingsError}<div class="dialog-error settings-message" role="alert">{settingsError}</div>{/if}
          <div class="settings-grid">
            <fieldset><legend>Languages</legend><p>Used for capture and translation.</p>
              <label>Source language<select bind:value={settingsDraft.sourceLanguage}><option value="auto">Auto detect</option><option value="en">English</option><option value="de">German</option><option value="fr">French</option><option value="es">Spanish</option><option value="zh-Hans">Chinese (Simplified)</option><option value="zh-Hant">Chinese (Traditional)</option></select></label>
              <label>Translate into<select bind:value={settingsDraft.targetLanguage}><option value="en">English</option><option value="de">German</option><option value="fr">French</option><option value="es">Spanish</option><option value="zh-Hans">Chinese (Simplified)</option><option value="zh-Hant">Chinese (Traditional)</option></select></label>
            </fieldset>
            <fieldset><legend>Review</legend><p>Set the size of your daily session.</p>
              <label>Review time<input type="time" bind:value={settingsDraft.reviewTime} /></label>
              {#if systemStatus.notificationError}<small class="field-error" role="alert">{systemStatus.notificationError}</small>{/if}
              <label>Daily limit<input type="number" min="1" max="50" bind:value={settingsDraft.dailyLimit} /></label>
              <label>Recent captures<input type="number" min="1" max="100" bind:value={settingsDraft.recentCapturesLimit} /></label>
            </fieldset>
            <fieldset><legend>Capture shortcuts</legend><p>Selection Capture is recommended. Leave a shortcut blank to disable it.</p>
              <label>Selection Capture · Recommended<input aria-label="Selection Capture shortcut" bind:value={settingsDraft.selectionCaptureShortcut} /></label>
              {#if systemStatus.selectionShortcutError}<small class="field-error" role="alert">{systemStatus.selectionShortcutError}</small>{/if}
              <label>Region OCR Capture<input aria-label="Region OCR Capture shortcut" bind:value={settingsDraft.regionOcrCaptureShortcut} /></label>
              {#if systemStatus.regionOcrShortcutError}<small class="field-error" role="alert">{systemStatus.regionOcrShortcutError}</small>{/if}
              <label class="toggle-row"><span><strong>Launch at login</strong><small>Start hidden and remain available in the system tray</small></span><input aria-label="Launch at login" type="checkbox" bind:checked={settingsDraft.launchAtLogin} /></label>
              {#if systemStatus.autostartError}<small class="field-error" role="alert">{systemStatus.autostartError}</small>{/if}
            </fieldset>
            <fieldset><legend>Appearance</legend><p>Visual preferences apply after a successful save.</p>
              <label>Theme<select bind:value={settingsDraft.appearance}><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></label>
              <label class="toggle-row"><span><strong>Reduce motion</strong><small>Minimize non-essential interface motion</small></span><input aria-label="Reduce motion" type="checkbox" bind:checked={settingsDraft.reducedMotion} /></label>
            </fieldset>
          </div>
          <div class="settings-actions"><button class="primary" disabled={settingsSaving}>{settingsSaving ? "Saving..." : "Save settings"}</button></div>
        </form>
      {/if}
    </section>
  </main>
</div>
{#if settingsSaved}<div class="settings-success settings-toast" role="status">Settings saved</div>{/if}

{#if captureOpen}<div class="backdrop"><div bind:this={captureDialog} class="dialog" role="dialog" tabindex="-1" aria-modal="true" aria-labelledby="manual-capture-title" onkeydown={trapDialogFocus}><form onsubmit={(event) => { event.preventDefault(); saveCapture(); }}><div class="dialog-heading"><div><span class="eyebrow">Manual Capture</span><h2 id="manual-capture-title">Save a reading context</h2></div><button type="button" class="icon" aria-label="Close manual capture" onclick={closeCapture}>×</button></div>{#if captureError}<div class="dialog-error" role="alert">{captureError}</div>{/if}<label>Word or phrase<input bind:this={captureFirstField} bind:value={captureInput.selectedText} /></label><label>Translation <small>Optional</small><input bind:value={captureInput.translation} /></label><label>Context<textarea bind:value={captureInput.sentence}></textarea></label><div class="actions"><button type="button" class="secondary" onclick={closeCapture}>Cancel</button><button class="primary" disabled={saving || !captureInput.selectedText.trim() || !captureInput.sentence.trim()}>{saving ? "Saving..." : "Save capture"}</button></div></form></div></div>{/if}

{#if savedCard}<div class="toast" role="dialog" aria-label="Capture saved"><span class="saved-mark" aria-hidden="true">✓</span><div><strong>{savedCard.displayForm}</strong><span>{encounterLabel(savedCard.encounterCount)}</span></div><button onclick={undoSaved}>Undo</button><button class="icon" aria-label="Dismiss saved capture" onclick={() => (savedCard = null)}>×</button></div>{/if}

{#if selectedDetail}<div class="drawer" role="dialog" aria-modal="true" aria-labelledby="vocabulary-detail-title"><button bind:this={detailCloseButton} class="icon close" aria-label="Close vocabulary detail" onclick={closeDetail}>×</button><span class="eyebrow">Vocabulary detail</span><h2 id="vocabulary-detail-title">{selectedDetail.item.displayForm}</h2><strong class="translation">{selectedDetail.item.translation ?? "No translation"}</strong><span class="count">{selectedDetail.item.status} · {encounterLabel(selectedDetail.item.encounterCount)}</span><div class="timeline"><h3>Contexts</h3>{#each selectedDetail.encounters as encounter}<article><p>{encounter.sentence}</p><small>{[encounter.sourceApp, encounter.sourceTitle, encounter.sourceUrl].filter(Boolean).join(" · ") || "Manual entry"}</small></article>{/each}</div></div>{/if}
</div>

<style>
  :global(html), :global(body), :global(#app) { min-width: 100%; min-height: 100%; margin: 0; }
  :global(body) { background: transparent; } :global(*) { box-sizing: border-box; }
  button, input, select, textarea { font: inherit; } button:focus-visible, input:focus-visible, select:focus-visible, textarea:focus-visible { outline: 2px solid #9aa5ff; outline-offset: 2px; }
  .windows-presentation { --page: #15161c; --sidebar: #191a21; --surface: #20212a; --surface-raised: #262832; --field: #181920; --text: #eeeef3; --muted: #9195a4; --line: #30323d; min-height: 100vh; color: var(--text); background: var(--page); font: 14px "Segoe UI Variable", "Segoe UI", sans-serif; }
  .windows-presentation[data-appearance="light"] { --page: #f5f6fa; --sidebar: #eceef4; --surface: #fff; --surface-raised: #f1f2f7; --field: #fff; --text: #20212a; --muted: #606474; --line: #d6d9e2; }
  .windows-presentation[data-settings-ready="false"] { visibility: hidden; }
  .windows-shell { min-height: 100vh; display: grid; grid-template-columns: 220px minmax(0, 1fr); color: var(--text); background: var(--page); }
  aside { display: flex; flex-direction: column; padding: 18px 12px 14px; border-right: 1px solid var(--line); background: var(--sidebar); }
  .brand { display: flex; align-items: center; gap: 10px; min-height: 36px; padding: 0 8px 18px; }.brand > span { display: grid; place-items: center; width: 28px; height: 28px; border-radius: 6px; background: #7584ef; color: #fff; font-weight: 700; }.brand strong { font-size: 13px; }
  nav { display: grid; gap: 3px; } nav button { min-height: 36px; padding: 0 10px; border: 0; border-radius: 6px; color: var(--muted); background: transparent; text-align: left; cursor: pointer; } nav button:hover, nav button.active { color: var(--text); background: var(--surface-raised); }
  .local-status { display: flex; align-items: center; gap: 8px; margin-top: auto; padding: 10px 8px; color: var(--muted); font-size: 12px; }.local-status i { width: 7px; height: 7px; border-radius: 50%; background: #78c8a7; }
  main { min-width: 0; } header { display: flex; align-items: center; justify-content: space-between; height: 86px; padding: 0 28px; border-bottom: 1px solid var(--line); } h1, h2, h3, p { margin: 0; } h1 { font-size: 22px; } header p { margin-top: 5px; color: var(--muted); font-size: 12px; } section { min-height: calc(100vh - 87px); padding: 26px 28px; }
  .primary, .secondary { min-height: 34px; padding: 0 14px; border-radius: 6px; border: 1px solid transparent; cursor: pointer; }.primary { color: #fff; background: #7584ef; }.primary:disabled { opacity: .45; cursor: default; }.secondary { color: var(--text); border-color: var(--line); background: var(--surface-raised); }
  .summary { display: grid; grid-template-columns: repeat(2, minmax(0, 190px)); gap: 12px; }.summary div { display: grid; gap: 8px; padding: 18px; border: 1px solid var(--line); border-radius: 7px; background: var(--surface); }.summary span, .summary small { color: var(--muted); font-size: 12px; }.summary strong { font-size: 26px; }
  .section-heading { margin: 24px 0 10px; }.section-heading h2 { font-size: 15px; }.section-heading p { margin-top: 4px; color: var(--muted); font-size: 11px; }
  .list { overflow: hidden; border: 1px solid var(--line); border-radius: 7px; background: var(--surface); }
  .recent-captures-list { max-height: min(55vh, 540px); overflow-y: auto; scrollbar-gutter: stable; }
  .vocabulary-list { max-height: calc(100vh - 244px); overflow-y: auto; scrollbar-gutter: stable; }
  .pagination { display: flex; align-items: center; justify-content: flex-end; gap: 8px; margin-top: 12px; color: var(--muted); font-size: 12px; }
  .pagination form, .pagination label { display: flex; align-items: center; gap: 6px; }.pagination input { width: 54px; height: 32px; padding: 0 6px; border: 1px solid var(--line); border-radius: 5px; color: var(--text); background: var(--field); text-align: center; }
  .state { min-height: 220px; display: grid; place-content: center; justify-items: center; gap: 8px; color: var(--muted); text-align: center; }.state strong { color: var(--text); font-size: 16px; }.state .primary { margin-top: 8px; }
  .tools { display: flex; align-items: end; justify-content: space-between; margin-bottom: 12px; color: var(--muted); font-size: 11px; }.tools label { display: grid; gap: 6px; }.tools input { width: 310px; height: 34px; padding: 0 10px; border: 1px solid var(--line); border-radius: 6px; color: var(--text); background: var(--surface); }
  .error { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-bottom: 14px; padding: 11px 13px; border: 1px solid #724747; border-radius: 6px; color: #f0b4b4; background: #321f24; }.error button { border: 0; color: #cad0ff; background: transparent; cursor: pointer; }
  .dialog-error { padding: 9px 10px; border: 1px solid #724747; border-radius: 6px; color: #f0b4b4; background: #321f24; font-size: 12px; }
  .settings { max-width: 900px; margin: 0 auto; }.settings-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }.settings fieldset { min-width: 0; display: grid; align-content: start; gap: 14px; margin: 0; padding: 18px; border: 1px solid var(--line); border-radius: 7px; background: var(--surface); }.settings legend { padding: 0; color: var(--text); font-size: 15px; font-weight: 700; }.settings fieldset > p { color: var(--muted); font-size: 11px; }.settings label { display: grid; gap: 6px; color: var(--muted); font-size: 11px; }.settings input:not([type="checkbox"]), .settings select { width: 100%; min-height: 36px; padding: 0 10px; border: 1px solid var(--line); border-radius: 6px; color: var(--text); background: var(--field); }.toggle-row { grid-template-columns: 1fr auto; align-items: center; }.toggle-row span { display: grid; gap: 3px; }.toggle-row strong { color: var(--text); font-size: 12px; }.toggle-row small { color: var(--muted); }.toggle-row input { width: 18px; height: 18px; accent-color: #7584ef; }.settings-actions { display: flex; justify-content: flex-end; margin-top: 14px; }.settings-message { margin-bottom: 12px; }.settings-success { padding: 9px 10px; border: 1px solid #3f755f; border-radius: 6px; color: #28624d; background: #dff4e9; font-size: 12px; }.settings-toast { position: fixed; z-index: 40; top: 18px; left: 50%; width: min(520px, calc(100vw - 32px)); margin: 0; box-shadow: 0 12px 36px rgba(0,0,0,.24); transform: translateX(-50%); animation: settings-toast-out 200ms ease 1s forwards; }
  @keyframes settings-toast-out { to { opacity: 0; transform: translate(-50%, -6px); } }
  .field-error { color: #f0b4b4; font-size: 11px; line-height: 1.4; }
  .windows-presentation[data-reduced-motion="true"], .windows-presentation[data-reduced-motion="true"] * { scroll-behavior: auto !important; animation-duration: .01ms !important; animation-iteration-count: 1 !important; transition-duration: .01ms !important; }
  .review-card { max-width: 620px; margin: 24px auto; padding: 28px; border: 1px solid var(--line); border-radius: 8px; background: var(--surface); }.review-progress { display: flex; align-items: center; justify-content: space-between; margin-bottom: 28px; color: var(--muted); font-size: 11px; }.review-card h2 { margin: 10px 0; font-size: 30px; }.review-card > p { color: var(--muted); line-height: 1.6; }.review-translation { display: grid; gap: 5px; margin: 22px 0; padding: 14px; border-radius: 7px; background: rgba(117,132,239,.12); }.review-translation small { color: var(--muted); }.review-translation strong { color: #7a86e8; font-size: 16px; }.review-actions { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-top: 18px; }
  .backdrop { position: fixed; z-index: 30; inset: 0; display: grid; place-items: center; background: rgba(8,9,13,.65); }.dialog { width: min(460px, calc(100vw - 32px)); padding: 20px; border: 1px solid var(--line); border-radius: 8px; color: var(--text); background: var(--surface); box-shadow: 0 24px 70px rgba(0,0,0,.5); }.dialog form { display: grid; gap: 13px; }.dialog-heading { display: flex; justify-content: space-between; }.dialog h2 { margin-top: 5px; font-size: 19px; }.eyebrow { color: #7a86e8; font-size: 10px; font-weight: 700; text-transform: uppercase; }.dialog label { display: grid; gap: 6px; color: var(--muted); font-size: 11px; }.dialog label small { margin-left: 4px; }.dialog input, .dialog textarea { width: 100%; padding: 9px 10px; border: 1px solid var(--line); border-radius: 6px; color: var(--text); background: var(--field); }.dialog textarea { min-height: 80px; resize: vertical; }.actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
  .icon { width: 32px; height: 32px; border: 0; color: var(--muted); background: transparent; cursor: pointer; font-size: 20px; }.toast { position: fixed; z-index: 35; right: 22px; bottom: 22px; min-width: 320px; display: grid; grid-template-columns: 28px 1fr auto 32px; gap: 10px; align-items: center; padding: 13px; border: 1px solid var(--line); border-radius: 8px; color: var(--text); background: var(--surface-raised); box-shadow: 0 18px 50px rgba(0,0,0,.45); }.toast > div { display: grid; gap: 2px; }.toast span { color: var(--muted); font-size: 11px; }.toast button:not(.icon) { border: 0; color: #7584ef; background: transparent; cursor: pointer; }.saved-mark { display: grid; place-items: center; width: 26px; height: 26px; border-radius: 50%; color: #78c8a7!important; background: rgba(120,200,167,.14); }
  .drawer { position: fixed; z-index: 25; top: 0; right: 0; width: min(400px, 100vw); height: 100vh; overflow: auto; padding: 62px 24px 24px; border-left: 1px solid var(--line); color: var(--text); background: var(--surface); box-shadow: -20px 0 60px rgba(0,0,0,.38); }.close { position: absolute; top: 18px; right: 18px; }.drawer h2 { margin: 7px 0 5px; font-size: 27px; }.translation { display: block; color: #7a86e8; }.count { display: block; margin-top: 10px; color: var(--muted); font-size: 11px; }.timeline { margin-top: 28px; }.timeline h3 { color: var(--muted); font-size: 11px; text-transform: uppercase; }.timeline article { margin-top: 12px; padding: 12px; border-left: 2px solid #7584ef; background: var(--surface-raised); }.timeline article p { line-height: 1.5; }.timeline article small { display: block; margin-top: 7px; color: var(--muted); }
  @media (prefers-color-scheme: light) { .windows-presentation[data-appearance="system"] { --page: #f5f6fa; --sidebar: #eceef4; --surface: #fff; --surface-raised: #f1f2f7; --field: #fff; --text: #20212a; --muted: #606474; --line: #d6d9e2; } }
  @media (prefers-reduced-motion: reduce) { .windows-presentation, .windows-presentation * { scroll-behavior: auto !important; animation-duration: .01ms !important; animation-iteration-count: 1 !important; transition-duration: .01ms !important; } }
  @media (forced-colors: active) { .windows-presentation { --page: Canvas; --sidebar: Canvas; --surface: Canvas; --surface-raised: Canvas; --field: Field; --text: CanvasText; --muted: CanvasText; --line: CanvasText; } .primary, .secondary, :global(.word-row), .dialog, .drawer, .toast { border: 1px solid ButtonText; } }
  @media (max-width: 760px), (min-resolution: 1.5dppx) and (max-width: 1100px) { .windows-shell { grid-template-columns: 10rem minmax(0,1fr); } header, section { height: auto; min-height: 5.4rem; padding-left: 1.125rem; padding-right: 1.125rem; }.summary, .settings-grid { grid-template-columns: 1fr; } }
  @media (max-width: 560px) { .windows-shell { display: block; } aside { position: static; } nav { grid-template-columns: repeat(2, minmax(0, 1fr)); } .local-status { margin-top: 0; } header { align-items: flex-start; gap: 1rem; padding-top: 1rem; padding-bottom: 1rem; } section { min-height: auto; } .tools { align-items: stretch; flex-direction: column; gap: .75rem; } .tools input { width: 100%; } .pagination { flex-wrap: wrap; justify-content: center; }.pagination form { order: -1; width: 100%; justify-content: center; }.toast { right: 1rem; bottom: 1rem; left: 1rem; min-width: 0; } }
</style>
