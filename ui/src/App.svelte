<script lang="ts">
  import { onMount } from "svelte";
  import { createBackend } from "./lib/backend";
  import ShortcutRecorder from "./components/ShortcutRecorder.svelte";
  import type { CaptureCard, ReviewCard, Settings, TodayView, WordDetail, WordListItem } from "./lib/types";

  type Route = "Today" | "Vocabulary" | "Progress" | "Settings";
  const api = createBackend();
  const navigation: Array<{ label: Route; icon: string }> = [
    { label: "Today", icon: "◫" }, { label: "Vocabulary", icon: "Aa" },
    { label: "Progress", icon: "↗" }, { label: "Settings", icon: "⚙" },
  ];
  let route: Route = "Today";
  let today: TodayView | null = null;
  let words: WordListItem[] = [];
  let settings: Settings | null = null;
  let savedShortcut = "";
  let search = "";
  let captureOpen = false;
  let reviewOpen = false;
  let reviewIndex = 0;
  let savedCard: CaptureCard | null = null;
  let selectedDetail: WordDetail | null = null;
  let error = "";
  let captureInput = { selectedText: "", translation: "", sentence: "", sourceApp: "Browser" };

  $: filteredWords = words.filter((word) =>
    `${word.displayForm} ${word.translation ?? ""}`.toLowerCase().includes(search.toLowerCase()));
  $: activeReview = today?.reviewQueue[reviewIndex] as ReviewCard | undefined;

  async function refresh() {
    try {
      [today, words, settings] = await Promise.all([api.getToday(), api.listWords(), api.getSettings()]);
      savedShortcut = settings.selectionCaptureShortcut;
    } catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }

  async function saveCapture() {
    if (!captureInput.selectedText.trim() || !captureInput.sentence.trim()) return;
    savedCard = await api.capture({ ...captureInput, translation: captureInput.translation || undefined });
    captureInput = { selectedText: "", translation: "", sentence: "", sourceApp: "Browser" };
    await refresh();
  }

  async function undoSaved() {
    if (!savedCard) return;
    await api.undoCapture(savedCard.encounterId);
    savedCard = null;
    await refresh();
  }

  async function rate(rating: "forgot" | "remembered") {
    if (!activeReview) return;
    await api.submitReview(activeReview.wordId, rating);
    if (reviewIndex + 1 < (today?.reviewQueue.length ?? 0)) reviewIndex += 1;
    else { reviewOpen = false; reviewIndex = 0; await refresh(); }
  }

  async function showDetail(id: string) { selectedDetail = await api.getWord(id); }
  async function saveSettings() {
    if (!settings) return;
    try {
      if (settings.selectionCaptureShortcut !== savedShortcut) settings = await api.replaceShortcut(settings.selectionCaptureShortcut);
      await api.updateSettings(settings);
      await refresh();
    } catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }
  const relative = (iso: string) => {
    const hours = Math.max(0, Math.round((Date.now() - new Date(iso).getTime()) / 3_600_000));
    return hours < 1 ? "just now" : hours === 1 ? "1 hour ago" : `${hours} hours ago`;
  };
  onMount(refresh);
</script>

<div class="window-shell">
  <aside class="sidebar">
    <div class="traffic-spacer" aria-hidden="true"></div>
    <div class="brand"><span class="brand-mark">V</span><span>Vocab Collector</span></div>
    <nav aria-label="Main navigation">
      {#each navigation as item}
        <button class:active={route === item.label} aria-current={route === item.label ? "page" : undefined}
          onclick={() => { route = item.label; selectedDetail = null; }}>
          <span class="nav-icon" aria-hidden="true">{item.icon}</span>{item.label}
        </button>
      {/each}
    </nav>
    <div class="account-card"><span class="status-dot"></span><div><strong>Local mode</strong><small>Everything is saved</small></div></div>
  </aside>

  <main>
    <header class="toolbar">
      <div><h1>{route}</h1><p>{route === "Today" ? "Your quiet learning rhythm" : `Manage your ${route.toLowerCase()}`}</p></div>
      <div class="toolbar-actions">
        <button class="secondary compact" aria-label="Quick capture" onclick={() => (captureOpen = true)}>＋ Quick capture</button>
        <button class="avatar" aria-label="Account">YW</button>
      </div>
    </header>

    <section class="content" aria-label={`${route} content`}>
      {#if error}<div class="error-banner">{error}</div>{/if}
      {#if route === "Today"}
        <div class="review-hero panel">
          <div><span class="eyebrow">Daily review</span><h2>{today?.plannedReviewCount ?? 0} words ready</h2>
            <p>{today?.plannedReviewCount ? `About ${today.estimatedMinutes} minute${today.estimatedMinutes === 1 ? "" : "s"} · ${today.totalDueCount} total due.` : "You’re caught up for today."}</p></div>
          <button class="primary" disabled={!today?.plannedReviewCount} onclick={() => { reviewOpen = true; reviewIndex = 0; }}>Review {today?.plannedReviewCount ?? 0} words</button>
        </div>
        <div class="section-heading"><div><h2>Recent captures</h2><p>Words saved while you read</p></div><button class="text-button" onclick={() => (route = "Vocabulary")}>View all</button></div>
        <div class="word-list panel">
          {#each today?.recentCaptures ?? [] as word}
            <button class="word-row" onclick={() => showDetail(word.id)}><span class="word-glyph">{word.displayForm.slice(0, 1).toUpperCase()}</span>
              <span class="word-copy"><strong>{word.displayForm}</strong><small>{word.translation ?? "Translation unavailable"}</small></span>
              <span class="source">Saved {word.encounterCount} time{word.encounterCount === 1 ? "" : "s"}</span><time>{relative(word.lastSeenAt)}</time><span class="chevron">›</span></button>
          {:else}<div class="empty">Capture a word to begin your collection.</div>{/each}
        </div>
      {:else if route === "Vocabulary"}
        <div class="list-tools"><label class="search"><span>⌕</span><input aria-label="Search vocabulary" bind:value={search} placeholder="Search words or translations" /></label><span>{filteredWords.length} words</span></div>
        <div class="table panel"><div class="table-head"><span>Word</span><span>Translation</span><span>Status</span><span>Times saved</span><span>Last saved</span></div>
          {#each filteredWords as word}<button class="table-row" onclick={() => showDetail(word.id)}><strong>{word.displayForm}</strong><span>{word.translation ?? "—"}</span><span class="status-pill">{word.status}</span><span>{word.encounterCount}</span><span>{relative(word.lastSeenAt)}</span></button>{/each}
        </div>
      {:else if route === "Progress"}
        <div class="metrics"><div class="panel metric"><span>Captured</span><strong>{words.length}</strong><small>This week</small></div><div class="panel metric"><span>Total due</span><strong>{today?.totalDueCount ?? 0}</strong><small>{today?.plannedReviewCount ?? 0} planned today</small></div><div class="panel metric"><span>Habit</span><strong>Quiet</strong><small>No streak pressure</small></div></div>
        <div class="panel chart-card"><div class="section-heading"><div><h2>Weekly rhythm</h2><p>Captured and reviewed</p></div></div><div class="bars">{#each [38,62,48,78,54,88,66] as height}<span style={`height:${height}%`}></span>{/each}</div></div>
      {:else if settings}
        <div class="settings-grid"><div class="panel setting-card"><span class="eyebrow">Languages</span><h2>Translation</h2><label>Source language<input value="English" disabled /></label><label>Translate into<select bind:value={settings.targetLanguage}><option value="de">German</option><option value="fr">French</option><option value="es">Spanish</option><option value="zh">Chinese</option></select></label></div>
          <div class="panel setting-card"><span class="eyebrow">Review</span><h2>Daily rhythm</h2><label>Review time<input type="time" bind:value={settings.reviewTime} /></label><label>Daily limit<input type="number" min="1" max="5" bind:value={settings.dailyLimit} /></label></div>
          <div class="panel setting-card"><span class="eyebrow">Capture</span><h2>Reading flow</h2><label>Selection Capture shortcut<ShortcutRecorder value={settings.selectionCaptureShortcut} onRecorded={(shortcut) => { if (settings) settings.selectionCaptureShortcut = shortcut; }} /></label><label>Region OCR Capture shortcut<ShortcutRecorder value={settings.regionOcrCaptureShortcut} onRecorded={(shortcut) => { if (settings) settings.regionOcrCaptureShortcut = shortcut; }} /></label><label class="toggle-row"><span>Launch at login</span><input type="checkbox" bind:checked={settings.launchAtLogin} /></label></div>
          <div class="panel setting-card"><span class="eyebrow">Appearance</span><h2>Comfort</h2><label>Theme<select bind:value={settings.appearance}><option value="system">System</option><option value="dark">Dark</option><option value="light">Light</option></select></label><label class="toggle-row"><span>Reduce motion</span><input type="checkbox" bind:checked={settings.reducedMotion} /></label></div>
        </div><button class="primary save-settings" onclick={saveSettings}>Save settings</button>
      {/if}
    </section>
  </main>
</div>

{#if captureOpen}
  <div class="floating-card capture-card" role="dialog" aria-label="Quick capture"><div class="float-header"><span><i class="status-dot"></i> Quick capture</span><button aria-label="Close capture" onclick={() => { captureOpen = false; savedCard = null; }}>×</button></div>
    {#if savedCard}<div class="saved-state"><span class="saved-check">✓</span><div><span class="eyebrow">Saved · Undo</span><h2>{savedCard.displayForm}</h2><strong>{savedCard.translation ?? "Translation unavailable"}</strong><p>“{savedCard.context}”</p><small>{savedCard.isExistingWord ? `Saved ${savedCard.encounterCount} times · New context saved` : "Added to your review queue"}</small></div></div><button class="text-button undo" onclick={undoSaved}>Undo</button>
    {:else}<form onsubmit={(event) => { event.preventDefault(); saveCapture(); }}><label>Word<input bind:value={captureInput.selectedText} placeholder="Selected word" /></label><label>Translation<input bind:value={captureInput.translation} placeholder="Automatic or manual" /></label><label>Context<textarea bind:value={captureInput.sentence} placeholder="Sentence around the word"></textarea></label><div class="form-actions"><span>Auto-saves locally</span><button class="primary" type="submit">Save word</button></div></form>{/if}
  </div>
{/if}

{#if reviewOpen && activeReview}
  <div class="modal-backdrop"><div class="floating-card review-card" role="dialog" aria-label="Daily review"><div class="review-progress"><span>Review {reviewIndex + 1} of {today?.reviewQueue.length}</span><div><i style={`width:${((reviewIndex + 1)/(today?.reviewQueue.length ?? 1))*100}%`}></i></div><button aria-label="Close review" onclick={() => (reviewOpen = false)}>×</button></div><span class="eyebrow">Do you remember this word?</span><h2>{activeReview.displayForm}</h2><p>“{activeReview.context}”</p><div class="translation-reveal"><small>Translation</small><strong>{activeReview.translation ?? "Unavailable"}</strong></div><div class="review-actions"><button class="secondary" onclick={() => rate("forgot")}>Forgot</button><button class="primary" onclick={() => rate("remembered")}>Remembered</button></div></div></div>
{/if}

{#if selectedDetail}
  <div class="detail-drawer"><button class="drawer-close" aria-label="Close word detail" onclick={() => (selectedDetail = null)}>×</button><span class="eyebrow">Word detail</span><h2>{selectedDetail.item.displayForm}</h2><strong class="detail-translation">{selectedDetail.item.translation ?? "No translation"}</strong><span class="status-pill">{selectedDetail.item.status}</span><div class="timeline"><h3>Capture history · {selectedDetail.encounters.length}</h3>{#each selectedDetail.encounters as encounter}<article><i></i><p>“{encounter.sentence}”</p><small>{encounter.sourceApp ?? "Unknown source"} · {relative(encounter.capturedAt)}</small></article>{/each}</div></div>
{/if}
