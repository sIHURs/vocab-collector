<script lang="ts">
  import { onMount } from "svelte";
  import { createBackend, type Backend } from "../lib/backend";
  import type { CaptureCard, ReviewCard, ReviewRating, TodayView, WordDetail, WordListItem } from "../lib/types";
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

  $: filteredWords = words.filter((word) => `${word.displayForm} ${word.translation ?? ""}`.toLowerCase().includes(search.trim().toLowerCase()));
  $: activeReview = today?.reviewQueue[reviewIndex] as ReviewCard | undefined;

  async function refresh() {
    loading = true;
    error = "";
    try {
      [today, words] = await Promise.all([api.getToday(), api.listWords()]);
      if (selectedDetail) selectedDetail = await api.getWord(selectedDetail.item.id);
      return true;
    } catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
    finally { loading = false; }
    return false;
  }

  function openCapture() {
    captureInput = { selectedText: "", translation: "", sentence: "" };
    captureError = "";
    savedCard = null;
    captureOpen = true;
  }

  async function saveCapture() {
    if (!captureInput.selectedText.trim() || !captureInput.sentence.trim()) return;
    saving = true;
    captureError = "";
    try {
      savedCard = await api.capture({ ...captureInput, translation: captureInput.translation.trim() || undefined, captureOrigin: "manual" });
      captureOpen = false;
      await refresh();
      reviewComplete = false;
    } catch (cause) { captureError = cause instanceof Error ? cause.message : String(cause); }
    finally { saving = false; }
  }

  async function showDetail(wordId: string) {
    error = "";
    try { selectedDetail = await api.getWord(wordId); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }

  async function undoSaved() {
    if (!savedCard) return;
    error = "";
    try { await api.undoCapture(savedCard.encounterId); savedCard = null; await refresh(); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }

  function startReview() {
    if (!today?.reviewQueue.length || reviewRefreshRequired) return;
    route = "Review";
    reviewOpen = true;
    reviewComplete = false;
    reviewError = "";
  }

  async function closeReview() {
    if (reviewSubmitting) return;
    reviewOpen = false;
    reviewPaused = true;
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
    }
  }

  async function retryError() {
    if (reviewRefreshRequired) await retryReviewRefresh();
    else await refresh();
  }

  async function rateReview(rating: ReviewRating) {
    if (!activeReview || reviewSubmitting) return;
    reviewSubmitting = true;
    reviewError = "";
    try {
      await api.submitReview(activeReview.wordId, rating);
      if (reviewIndex + 1 < (today?.reviewQueue.length ?? 0)) reviewIndex += 1;
      else {
        reviewOpen = false;
        reviewPaused = false;
        reviewCompleting = true;
        if (await refresh()) {
          reviewIndex = 0;
          reviewCompleting = false;
          reviewComplete = true;
        } else reviewRefreshRequired = true;
      }
    } catch (cause) { reviewError = cause instanceof Error ? cause.message : String(cause); }
    finally { reviewSubmitting = false; }
  }

  const encounterLabel = (count: number) => `${count} encounter${count === 1 ? "" : "s"}`;
  onMount(refresh);
</script>

<div class="windows-shell" data-presentation="windows-main">
  <aside>
    <div class="brand"><span aria-hidden="true">V</span><strong>Vocab Collector</strong></div>
    <nav aria-label="Main navigation">{#each navigation as item}<button class:active={route === item} aria-current={route === item ? "page" : undefined} onclick={() => { route = item; selectedDetail = null; }}>{item}</button>{/each}</nav>
    <div class="local-status"><i aria-hidden="true"></i><span>Local mode</span></div>
  </aside>
  <main>
    <header><div><h1>{route}</h1><p>{route === "Today" ? "Your words, ready when you are" : "Vocab Collector for Windows"}</p></div><button class="primary" onclick={openCapture}>Manual capture</button></header>
    <section aria-label={`${route} content`}>
      {#if error}<div class="error" role="alert"><span>{error}</span><button onclick={retryError}>Try again</button></div>{/if}
      {#if loading}
        <div class="state" aria-live="polite">Loading your vocabulary...</div>
      {:else if route === "Today"}
        <div class="summary"><div><span>Due today</span><strong>{today?.dueCount ?? 0}</strong><small>{today?.dueCount ? `About ${today?.estimatedMinutes ?? 0} minute${today?.estimatedMinutes === 1 ? "" : "s"}` : "Review queue is clear"}</small>{#if today?.dueCount}<button class="primary" onclick={startReview}>Start review ({today.dueCount})</button>{/if}</div><div><span>Recent captures</span><strong>{today?.recentCaptures.length ?? 0}</strong><small>Stored locally</small></div></div>
        <div class="section-heading"><h2>Recent captures</h2><p>New contexts appear here immediately after saving.</p></div>
        <div class="list">{#each today?.recentCaptures ?? [] as word}<WindowsWordRow {word} onSelect={showDetail} />{:else}<div class="state"><strong>No captures yet</strong><span>Use Manual capture to save your first reading context.</span><button class="primary" onclick={openCapture}>Manual capture</button></div>{/each}</div>
      {:else if route === "Vocabulary"}
        <div class="tools"><label><span>Search</span><input aria-label="Search vocabulary" bind:value={search} placeholder="Word or translation" /></label><span>{filteredWords.length} items</span></div>
        <div class="list">{#each filteredWords as word}<WindowsWordRow {word} onSelect={showDetail} />{:else}{#if words.length}<div class="state"><strong>No matching vocabulary</strong><span>Try a different word or translation.</span></div>{:else}<div class="state"><strong>Your vocabulary is empty</strong><span>Saved words will appear here.</span></div>{/if}{/each}</div>
      {:else if route === "Review"}
        {#if reviewOpen && activeReview}
          <div class="review-card" aria-live="polite"><div class="review-progress"><span>{reviewIndex + 1} of {today?.reviewQueue.length}</span><button class="icon" aria-label="Close review" disabled={reviewSubmitting} onclick={closeReview}>×</button></div><span class="eyebrow">Do you remember this word?</span><h2>{activeReview.displayForm}</h2><p>{activeReview.context ?? "No saved context"}</p><div class="review-translation"><small>Translation</small><strong>{activeReview.translation ?? "Unavailable"}</strong></div>{#if reviewError}<div class="dialog-error" role="alert">{reviewError}</div>{/if}<div class="review-actions"><button class="secondary" disabled={reviewSubmitting} onclick={() => rateReview("forgot")}>Forgot</button><button class="primary" disabled={reviewSubmitting} onclick={() => rateReview("remembered")}>Remembered</button></div></div>
        {:else if reviewComplete}
          <div class="state"><h2>Review complete</h2><span>Today is refreshed. Your next due dates come from the shared review schedule.</span><button class="primary" onclick={() => (route = "Today")}>Back to Today</button></div>
        {:else if reviewRefreshRequired}
          <div class="state"><strong>{reviewCompleting ? "Review saved" : "Review paused"}</strong><span>Refresh Today before continuing so the due queue stays current.</span><button class="primary" onclick={retryReviewRefresh}>Retry Review refresh</button></div>
        {:else if today?.reviewQueue.length}
          <div class="state"><strong>{reviewPaused ? "Review paused" : `${today.dueCount} words ready`}</strong><span>{reviewPaused ? `${today.reviewQueue.length} words remaining.` : `About ${today.estimatedMinutes} minute${today.estimatedMinutes === 1 ? "" : "s"}.`}</span><button class="primary" onclick={startReview}>{reviewPaused ? "Resume review" : "Start review"}</button></div>
        {:else}
          <div class="state"><strong>Nothing due</strong><span>Your review queue is clear for today.</span></div>
        {/if}
      {:else}
        <div class="state"><strong>{route}</strong><span>This area is scheduled for a later Windows ticket.</span></div>
      {/if}
    </section>
  </main>
</div>

{#if captureOpen}<div class="backdrop"><div class="dialog" role="dialog" aria-modal="true" aria-label="Manual capture"><form onsubmit={(event) => { event.preventDefault(); saveCapture(); }}><div class="dialog-heading"><div><span class="eyebrow">Manual Capture</span><h2>Save a reading context</h2></div><button type="button" class="icon" aria-label="Close manual capture" onclick={() => (captureOpen = false)}>×</button></div>{#if captureError}<div class="dialog-error" role="alert">{captureError}</div>{/if}<label>Word or phrase<input bind:value={captureInput.selectedText} /></label><label>Translation <small>Optional</small><input bind:value={captureInput.translation} /></label><label>Context<textarea bind:value={captureInput.sentence}></textarea></label><div class="actions"><button type="button" class="secondary" onclick={() => (captureOpen = false)}>Cancel</button><button class="primary" disabled={saving || !captureInput.selectedText.trim() || !captureInput.sentence.trim()}>{saving ? "Saving..." : "Save capture"}</button></div></form></div></div>{/if}

{#if savedCard}<div class="toast" role="dialog" aria-label="Capture saved"><span class="saved-mark" aria-hidden="true">✓</span><div><strong>{savedCard.displayForm}</strong><span>{encounterLabel(savedCard.encounterCount)}</span></div><button onclick={undoSaved}>Undo</button><button class="icon" aria-label="Dismiss saved capture" onclick={() => (savedCard = null)}>×</button></div>{/if}

{#if selectedDetail}<div class="drawer" role="dialog" aria-label="Vocabulary detail"><button class="icon close" aria-label="Close vocabulary detail" onclick={() => (selectedDetail = null)}>×</button><span class="eyebrow">Vocabulary detail</span><h2>{selectedDetail.item.displayForm}</h2><strong class="translation">{selectedDetail.item.translation ?? "No translation"}</strong><span class="count">{selectedDetail.item.status} · {encounterLabel(selectedDetail.item.encounterCount)}</span><div class="timeline"><h3>Contexts</h3>{#each selectedDetail.encounters as encounter}<article><p>{encounter.sentence}</p><small>{[encounter.sourceApp, encounter.sourceTitle, encounter.sourceUrl].filter(Boolean).join(" · ") || "Manual entry"}</small></article>{/each}</div></div>{/if}

<style>
  :global(html), :global(body), :global(#app) { min-width: 100%; min-height: 100%; margin: 0; }
  :global(body) { background: #15161c; } :global(*) { box-sizing: border-box; }
  button, input, textarea { font: inherit; } button:focus-visible, input:focus-visible, textarea:focus-visible { outline: 2px solid #9aa5ff; outline-offset: 2px; }
  .windows-shell { min-height: 100vh; display: grid; grid-template-columns: 220px minmax(0, 1fr); color: #eeeef3; background: #15161c; font: 14px "Segoe UI Variable", "Segoe UI", sans-serif; }
  aside { display: flex; flex-direction: column; padding: 18px 12px 14px; border-right: 1px solid #2c2e38; background: #191a21; }
  .brand { display: flex; align-items: center; gap: 10px; min-height: 36px; padding: 0 8px 18px; }.brand > span { display: grid; place-items: center; width: 28px; height: 28px; border-radius: 6px; background: #7584ef; color: #fff; font-weight: 700; }.brand strong { font-size: 13px; }
  nav { display: grid; gap: 3px; } nav button { min-height: 36px; padding: 0 10px; border: 0; border-radius: 6px; color: #a9acb8; background: transparent; text-align: left; cursor: pointer; } nav button:hover, nav button.active { color: #fff; background: #2a2d3b; }
  .local-status { display: flex; align-items: center; gap: 8px; margin-top: auto; padding: 10px 8px; color: #8f93a1; font-size: 12px; }.local-status i { width: 7px; height: 7px; border-radius: 50%; background: #78c8a7; }
  main { min-width: 0; } header { display: flex; align-items: center; justify-content: space-between; height: 86px; padding: 0 28px; border-bottom: 1px solid #2c2e38; } h1, h2, h3, p { margin: 0; } h1 { font-size: 22px; } header p { margin-top: 5px; color: #8f93a1; font-size: 12px; } section { min-height: calc(100vh - 87px); padding: 26px 28px; }
  .primary, .secondary { min-height: 34px; padding: 0 14px; border-radius: 6px; border: 1px solid transparent; cursor: pointer; }.primary { color: #fff; background: #7584ef; }.primary:disabled { opacity: .45; cursor: default; }.secondary { color: #eeeef3; border-color: #3a3d49; background: #262832; }
  .summary { display: grid; grid-template-columns: repeat(2, minmax(0, 190px)); gap: 12px; }.summary div { display: grid; gap: 8px; padding: 18px; border: 1px solid #30323d; border-radius: 7px; background: #20212a; }.summary span, .summary small { color: #9094a3; font-size: 12px; }.summary strong { font-size: 26px; }
  .section-heading { margin: 24px 0 10px; }.section-heading h2 { font-size: 15px; }.section-heading p { margin-top: 4px; color: #8f93a1; font-size: 11px; }
  .list { overflow: hidden; border: 1px solid #30323d; border-radius: 7px; background: #20212a; }
  .state { min-height: 220px; display: grid; place-content: center; justify-items: center; gap: 8px; color: #9195a4; text-align: center; }.state strong { color: #eeeef3; font-size: 16px; }.state .primary { margin-top: 8px; }
  .tools { display: flex; align-items: end; justify-content: space-between; margin-bottom: 12px; color: #9195a4; font-size: 11px; }.tools label { display: grid; gap: 6px; }.tools input { width: 310px; height: 34px; padding: 0 10px; border: 1px solid #3a3d49; border-radius: 6px; color: #eeeef3; background: #20212a; }
  .error { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-bottom: 14px; padding: 11px 13px; border: 1px solid #724747; border-radius: 6px; color: #f0b4b4; background: #321f24; }.error button { border: 0; color: #cad0ff; background: transparent; cursor: pointer; }
  .dialog-error { padding: 9px 10px; border: 1px solid #724747; border-radius: 6px; color: #f0b4b4; background: #321f24; font-size: 12px; }
  .review-card { max-width: 620px; margin: 24px auto; padding: 28px; border: 1px solid #30323d; border-radius: 8px; background: #20212a; }.review-progress { display: flex; align-items: center; justify-content: space-between; margin-bottom: 28px; color: #9195a4; font-size: 11px; }.review-card h2 { margin: 10px 0; font-size: 30px; }.review-card > p { color: #b6b9c4; line-height: 1.6; }.review-translation { display: grid; gap: 5px; margin: 22px 0; padding: 14px; border-radius: 7px; background: rgba(117,132,239,.12); }.review-translation small { color: #9195a4; }.review-translation strong { color: #c5caff; font-size: 16px; }.review-actions { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-top: 18px; }
  .backdrop { position: fixed; z-index: 30; inset: 0; display: grid; place-items: center; background: rgba(8,9,13,.65); }.dialog { width: min(460px, calc(100vw - 32px)); padding: 20px; border: 1px solid #3a3d49; border-radius: 8px; color: #eeeef3; background: #20212a; box-shadow: 0 24px 70px rgba(0,0,0,.5); }.dialog form { display: grid; gap: 13px; }.dialog-heading { display: flex; justify-content: space-between; }.dialog h2 { margin-top: 5px; font-size: 19px; }.eyebrow { color: #9da7ff; font-size: 10px; font-weight: 700; text-transform: uppercase; }.dialog label { display: grid; gap: 6px; color: #a6a9b5; font-size: 11px; }.dialog label small { margin-left: 4px; }.dialog input, .dialog textarea { width: 100%; padding: 9px 10px; border: 1px solid #3a3d49; border-radius: 6px; color: #eeeef3; background: #181920; }.dialog textarea { min-height: 80px; resize: vertical; }.actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
  .icon { width: 32px; height: 32px; border: 0; color: #a6a9b5; background: transparent; cursor: pointer; font-size: 20px; }.toast { position: fixed; z-index: 35; right: 22px; bottom: 22px; min-width: 320px; display: grid; grid-template-columns: 28px 1fr auto 32px; gap: 10px; align-items: center; padding: 13px; border: 1px solid #3a3d49; border-radius: 8px; color: #eeeef3; background: #262832; box-shadow: 0 18px 50px rgba(0,0,0,.45); }.toast > div { display: grid; gap: 2px; }.toast span { color: #a6a9b5; font-size: 11px; }.toast button:not(.icon) { border: 0; color: #aeb6ff; background: transparent; cursor: pointer; }.saved-mark { display: grid; place-items: center; width: 26px; height: 26px; border-radius: 50%; color: #78c8a7!important; background: rgba(120,200,167,.14); }
  .drawer { position: fixed; z-index: 25; top: 0; right: 0; width: min(400px, 100vw); height: 100vh; overflow: auto; padding: 62px 24px 24px; border-left: 1px solid #3a3d49; color: #eeeef3; background: #1d1e26; box-shadow: -20px 0 60px rgba(0,0,0,.38); }.close { position: absolute; top: 18px; right: 18px; }.drawer h2 { margin: 7px 0 5px; font-size: 27px; }.translation { display: block; color: #b8c0ff; }.count { display: block; margin-top: 10px; color: #9296a4; font-size: 11px; }.timeline { margin-top: 28px; }.timeline h3 { color: #9296a4; font-size: 11px; text-transform: uppercase; }.timeline article { margin-top: 12px; padding: 12px; border-left: 2px solid #7584ef; background: #242630; }.timeline article p { line-height: 1.5; }.timeline article small { display: block; margin-top: 7px; color: #9296a4; }
  @media (max-width: 760px) { .windows-shell { grid-template-columns: 170px minmax(0,1fr); } header, section { padding-left: 18px; padding-right: 18px; }.summary { grid-template-columns: 1fr 1fr; } }
</style>
