<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { createBackend, type Backend } from "../lib/backend";
  import type { AchievedCaptureConflict, AchievedWordListItem, CaptureCard, GlobalInsight, ReviewCard, ReviewRating, ReviewResult, ReviewSessionInsight, Settings, SystemSettingsStatus, TodayView, WordDetail, WordListItem } from "../lib/types";
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Field from '$lib/components/ui/field';
  import * as Tabs from '$lib/components/ui/tabs';
  import { toast } from 'svelte-sonner';
  import { Toaster } from '$lib/components/ui/sonner';
  import UndoNotification from '../components/UndoNotification.svelte';
  import Plus from '@lucide/svelte/icons/plus';
  import SettingsForm from '../components/SettingsForm.svelte';
  import VocabularyTable from '../components/VocabularyTable.svelte';
  import VocabularyDetail from '../components/VocabularyDetail.svelte';
  import { applyAppearance } from '../lib/appearance';
  import CalendarDays from '@lucide/svelte/icons/calendar-days';
  import BookOpen from '@lucide/svelte/icons/book-open';
  import ChartNoAxesColumn from '@lucide/svelte/icons/chart-no-axes-column';
  import SettingsIcon from '@lucide/svelte/icons/settings';
  import Monitor from '@lucide/svelte/icons/monitor';
  import { localDate } from "../lib/vocabulary-log";
  import type { VocabularyLog } from "../lib/types";
  import TodayPanel from '../components/TodayPanel.svelte';
  import InsightsPanel from '../components/InsightsPanel.svelte';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { Progress } from '$lib/components/ui/progress';

  type Route = "Today" | "Vocabulary" | "Review" | "Insights" | "Settings";
  export let api: Backend = createBackend();
  const navigation = [{ title: "Today", icon: CalendarDays }, { title: "Vocabulary", icon: BookOpen }, { title: "Insights", icon: ChartNoAxesColumn }, { title: "Settings", icon: SettingsIcon }] as const;
  let route: Route = "Today";
  let today: TodayView | null = null;
  let words: WordListItem[] = [];
  let achievedWords: AchievedWordListItem[] = [];
  let vocabularyView: "active" | "mastered" | "achieved" = "active";
  let selectedAchievedIds = new Set<string>();
  let lastUnachievedIds: string[] = [];
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
  let achievedCaptureConflict: AchievedCaptureConflict | null = null;
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
  let globalInsight: GlobalInsight | null = null;
  let insightError = "";
  let vocabularyLog: VocabularyLog | null = null;
  let logLoading = false;
  let logError = "";
  let logRequest = 0;
  let lastLogDate = localDate();
  async function loadLog() {
    const request = ++logRequest;
    logLoading = true;
    logError = "";
    try { const result = await api.getVocabularyLog?.() ?? null; if (request === logRequest) vocabularyLog = result; }
    catch (cause) { if (request === logRequest) logError = cause instanceof Error ? cause.message : String(cause); }
    finally { if (request === logRequest) logLoading = false; }
  }
  function refreshLogDate() {
    const date = localDate();
    if (date !== lastLogDate) { lastLogDate = date; void loadLog(); }
  }
  let settingsDraft: Settings | null = null;
  let appliedSettings: Settings | null = null;
  let settingsSaving = false;
  let settingsError = "";
  let pendingSettings: Partial<Settings> = {};
  let initialLoadComplete = false;
  let systemStatus: SystemSettingsStatus = {};
  let captureTrigger: HTMLElement | null = null;
  let captureFirstField: HTMLInputElement | null = null;
  let captureDialog: HTMLDivElement | null = null;
  let detailTrigger: HTMLElement | null = null;
  let reviewCardElement: HTMLElement;
  let reviewCompleteHeading: HTMLHeadingElement;

  const savedToastId = crypto.randomUUID();
  const unachievedToastId = crypto.randomUUID();
  const settingsToastId = crypto.randomUUID();
  const notificationHostId = crypto.randomUUID();
  $: if (savedCard) toast.custom(UndoNotification, {
    id: savedToastId, toasterId: notificationHostId, duration: Infinity, dismissible: false,
    componentProps: { title: savedCard.displayForm, description: encounterLabel(savedCard.encounterCount), label: "Capture saved", dismissLabel: "Dismiss saved capture", onundo: undoSaved, ondismiss: () => { savedCard = null; } },
  }); else toast.dismiss(savedToastId);
  $: if (lastUnachievedIds.length) toast.custom(UndoNotification, {
    id: unachievedToastId, toasterId: notificationHostId, duration: Infinity, dismissible: false,
    componentProps: { title: lastUnachievedIds.length + " Unachieved", description: "Returned to Mastered", label: "Vocabulary Unachieved", dismissLabel: "Dismiss Unachieve result", onundo: undoUnachieve, ondismiss: () => { lastUnachievedIds = []; } },
  }); else toast.dismiss(unachievedToastId);
  onDestroy(() => { toast.dismiss(savedToastId); toast.dismiss(unachievedToastId); toast.dismiss(settingsToastId); });

  $: if (appliedSettings) applyAppearance(appliedSettings);
  $: visibleWords = words.filter((word) => vocabularyView === "active" ? word.status !== "mastered" : word.status === "mastered");
  $: filteredWords = visibleWords.filter((word) => `${word.displayForm} ${word.translation ?? ""}`.toLowerCase().includes(search.trim().toLowerCase()));
  $: filteredAchievedWords = achievedWords.filter((word) => `${word.displayForm} ${word.lemma} ${word.translation ?? ""}`.toLowerCase().includes(search.trim().toLowerCase()));
  $: allFilteredAchievedSelected = filteredAchievedWords.length > 0 && filteredAchievedWords.every((word) => selectedAchievedIds.has(word.id));
  $: vocabularyPageCount = Math.max(1, Math.ceil(filteredWords.length / vocabularyPageSize));
  $: if (vocabularyPage > vocabularyPageCount) vocabularyPage = vocabularyPageCount;
  $: vocabularyPageInput = String(vocabularyPage);
  $: pagedWords = filteredWords.slice((vocabularyPage - 1) * vocabularyPageSize, vocabularyPage * vocabularyPageSize);
  $: activeReview = today?.reviewQueue[reviewIndex] as ReviewCard | undefined;

  async function loadInsight() {
    insightError = "";
    try { return await api.getGlobalInsight?.() ?? null; }
    catch (cause) { insightError = cause instanceof Error ? cause.message : String(cause); return null; }
  }

  async function refresh() {
    loading = true;
    void loadLog();
    error = "";
    try {
      const [nextToday, nextWords, nextAchievedWords, nextSettings, nextGlobalInsight] = await Promise.all([api.getToday(), api.listWords(), api.listAchievedWords?.() ?? Promise.resolve([]), api.getSettings(), loadInsight()]);
      today = nextToday;
      words = nextWords;
      achievedWords = nextAchievedWords;
      if (!settingsDraft) {
        settingsDraft = { ...nextSettings };
        appliedSettings = { ...nextSettings };
      }
      globalInsight = nextGlobalInsight;
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
    achievedCaptureConflict = null;
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
    if (event.key !== "Tab" || !captureDialog) return;
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

  let captureLookupVersion = 0;
  $: void previewManualCapture(captureOpen, captureInput.selectedText);
  async function previewManualCapture(open: boolean, text: string) {
    const version = ++captureLookupVersion;
    achievedCaptureConflict = null;
    if (!open || !text.trim()) return;
    try {
      const match = await api.findAchievedCapture?.({ selectedText: text, sentence: "", captureOrigin: "manual" });
      if (version === captureLookupVersion && captureOpen) achievedCaptureConflict = match ?? null;
    } catch (cause) { if (version === captureLookupVersion) captureError = String(cause); }
  }

  async function saveCapture() {
    if (saving || !captureInput.selectedText.trim() || !captureInput.sentence.trim()) return;
    if (achievedCaptureConflict) { await restoreCapturedWordToLearning(); return; }
    saving = true;
    captureError = "";
    try {
      const input = { ...captureInput, translation: captureInput.translation.trim() || undefined, captureOrigin: "manual" as const };
      const conflict = await api.findAchievedCapture?.(input);
      if (conflict) {
        if (!api.restoreAchievedAndCapture) throw new Error("Achieved capture is unavailable");
        savedCard = await api.restoreAchievedAndCapture(conflict.wordId, input);
      } else savedCard = await api.capture(input);
      closeCapture();
      await refresh();
      reviewComplete = false;
    } catch (cause) { captureError = cause instanceof Error ? cause.message : String(cause); }
    finally { saving = false; }
  }

  async function restoreCapturedWordToLearning() {
    if (saving || !achievedCaptureConflict || !api.restoreAchievedAndCapture) return;
    saving = true;
    captureError = "";
    try {
      savedCard = await api.restoreAchievedAndCapture(achievedCaptureConflict.wordId, { ...captureInput, translation: captureInput.translation.trim() || undefined, captureOrigin: "manual" });
      achievedCaptureConflict = null;
      closeCapture();
      await refresh();
      reviewComplete = false;
    } catch (cause) { captureError = cause instanceof Error ? cause.message : String(cause); }
    finally { saving = false; }
  }

  async function showDetail(wordId: string, trigger: HTMLElement) {
    error = "";
    detailTrigger = trigger;
    try {
      selectedDetail = await api.getWord(wordId);
      await tick();
    }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }

  async function closeDetail() {
    selectedDetail = null;
    await tick();
    detailTrigger?.focus();
  }

  async function achieveSelectedWord() {
    if (!selectedDetail || selectedDetail.item.status !== "mastered") return;
    const retention = settingsDraft?.achievedRetentionDays ?? 30;
    const deletion = new Date(Date.now() + retention * 86_400_000).toLocaleDateString();
    if (!confirm(`Achieve ${selectedDetail.item.displayForm}? It will be permanently deleted on ${deletion}.`)) return;
    try { if (!api.achieveWord) throw new Error("Achieve is unavailable"); await api.achieveWord(selectedDetail.item.id); selectedDetail = null; vocabularyView = "achieved"; await refresh(); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }

  async function achieveWordFromRow(word: WordListItem) {
    const retention = settingsDraft?.achievedRetentionDays ?? 30;
    const deletion = new Date(Date.now() + retention * 86_400_000).toLocaleDateString();
    if (!confirm(`Achieve ${word.displayForm}? It will be permanently deleted on ${deletion}.`)) return;
    try { if (!api.achieveWord) throw new Error("Achieve is unavailable"); await api.achieveWord(word.id); vocabularyView = "achieved"; await refresh(); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }

  function toggleAchieved(wordId: string, checked: boolean) {
    const next = new Set(selectedAchievedIds); checked ? next.add(wordId) : next.delete(wordId); selectedAchievedIds = next;
  }

  function toggleAllAchieved(checked: boolean) {
    const next = new Set(selectedAchievedIds); for (const word of filteredAchievedWords) checked ? next.add(word.id) : next.delete(word.id); selectedAchievedIds = next;
  }

  async function unachieveSelected() {
    const ids = [...selectedAchievedIds]; if (!ids.length || !api.unachieveWords) return;
    try { await api.unachieveWords(ids); lastUnachievedIds = ids; selectedAchievedIds = new Set(); await refresh(); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }

  async function undoUnachieve() {
    if (!api.achieveWord) return;
    try { for (const id of lastUnachievedIds) await api.achieveWord(id); lastUnachievedIds = []; await refresh(); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }

  async function deleteSelectedAchieved() {
    const ids = [...selectedAchievedIds]; if (!ids.length || !api.deleteAchievedWords) return;
    if (!confirm(`Permanently delete ${ids.length} Achieved Vocabulary Item${ids.length === 1 ? "" : "s"}? Word-level history cannot be recovered.`)) return;
    try { await api.deleteAchievedWords(ids); selectedAchievedIds = new Set(); await refresh(); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }

  async function undoSaved() {
    if (!savedCard) return;
    error = "";
    try { await api.undoCapture(savedCard.encounterId); savedCard = null; await refresh(); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
  }

  async function startReview() {
    if (loading || !today?.reviewQueue.length || reviewRefreshRequired) return;
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
    route = "Today";
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

  function commitSettings(patch: Partial<Settings>) {
    pendingSettings = { ...pendingSettings, ...patch };
    void saveSettings();
  }

  async function saveSettings() {
    if (!settingsDraft || !appliedSettings || settingsSaving) return;
    settingsSaving = true;
    settingsError = "";
    let refreshToday = false;
    let savedChanges = false;
    const failedChanges = new Set<keyof Settings>();
    toast.dismiss(settingsToastId);
    try {
      while (Object.keys(pendingSettings).length) {
        const patch = pendingSettings;
        pendingSettings = {};
        const before = appliedSettings;
        const candidate = { ...before, ...patch };
        if (JSON.stringify(candidate) === JSON.stringify(before)) continue;
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
          // Runtime status includes historical errors. Judge this save by the
          // requested system changes and the values the backend actually applied.
          for (const key of ["selectionCaptureShortcut", "regionOcrCaptureShortcut", "launchAtLogin", "reviewTime"] as const) {
            if (candidate[key] !== before[key]) {
              if (persisted[key] !== candidate[key]) failedChanges.add(key);
              else failedChanges.delete(key);
            }
          }
          // Only reconcile submitted fields that have not been edited again.
          for (const key of Object.keys(patch) as (keyof Settings)[]) {
            if (!(key in pendingSettings) && settingsDraft[key] === candidate[key]) {
              settingsDraft = { ...settingsDraft, [key]: persisted[key] };
            }
          }
          appliedSettings = { ...persisted };
          savedChanges = true;
          refreshToday ||= before.dailyLimit !== persisted.dailyLimit || before.recentCapturesLimit !== persisted.recentCapturesLimit;
        } catch (cause) {
          pendingSettings = { ...patch, ...pendingSettings };
          settingsError = cause instanceof Error ? cause.message : String(cause);
          break;
        }
      }
      if (!settingsError && savedChanges) {
        const options = { id: settingsToastId, toasterId: notificationHostId, duration: 2000 };
        if (failedChanges.size) toast.warning("Some settings could not be applied", options);
        else toast.success("Settings saved", options);
      }
    } finally { settingsSaving = false; }
    if (refreshToday) {
      try { today = await api.getToday(); }
      catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
    }
  }

  async function selectRoute(item: Route) {
    if (route === "Review" && reviewOpen) {
      if (reviewSubmitting) return;
      await closeReview();
    }
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

  const encounterLabel = (count: number) => `Saved ${count} time${count === 1 ? "" : "s"}`;
  onMount(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;
    let unlistenManualCapture: (() => void) | undefined;
    void refresh();
    const dateTimer = setInterval(refreshLogDate, 30_000);
    window.addEventListener("focus", refreshLogDate);
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
      logRequest += 1;
      clearInterval(dateTimer);
      window.removeEventListener("focus", refreshLogDate);

      unlisten?.();
      unlistenManualCapture?.();
    };
  });
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="windows-presentation" data-settings-ready={initialLoadComplete} data-appearance={appliedSettings?.appearance ?? "system"} data-reduced-motion={appliedSettings?.reducedMotion ?? false}>
<div class="windows-shell" data-presentation="windows-main" data-appearance={appliedSettings?.appearance ?? "system"} data-reduced-motion={appliedSettings?.reducedMotion ?? false}>
  <aside>
    <div class="brand"><svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24"><path d="M3 4h6l3 5 3-5h6l-9 17Z M6 4l6 11 6-11" fill="none" stroke="currentColor" stroke-width="1.6" /></svg><strong>Vocab Collector</strong></div>
    <nav aria-label="Main navigation">{#each navigation as item}<button class:active={route === item.title || (route === "Review" && item.title === "Today")} aria-current={route === item.title || (route === "Review" && item.title === "Today") ? "page" : undefined} onclick={() => selectRoute(item.title)}><svelte:component this={item.icon} size={18} aria-hidden="true" />{item.title}</button>{/each}</nav>
    <div class="local-status"><Monitor size={16} aria-hidden="true" /><span>Local mode</span></div>
  </aside>
  <main>
    <header><div><h1 id="windows-page-title">{route}</h1><p>{route === "Today" ? "Your words, ready when you are" : route === "Settings" ? "Make Vocab Collector fit your reading." : route === "Vocabulary" ? "Every word, with the context you found it in." : route === "Insights" ? "Your learning, over time." : "One word at a time."}</p></div>{#if route === "Today" || route === "Vocabulary" || route === "Insights"}<Button aria-label="Manual capture" size="icon" onclick={openCapture}><Plus /></Button>{/if}</header>
    <section aria-labelledby="windows-page-title" aria-busy={loading}>
      {#if error}<div class="error" role="alert"><span>{error}</span><button onclick={retryError}>Try again</button></div>{/if}
      {#if route === "Today"}
        <TodayPanel {today} {loading} paused={reviewPaused} refreshRequired={reviewRefreshRequired} onstart={startReview} onretry={retryReviewRefresh} oncapture={openCapture} ondetail={showDetail} />
      {:else if loading && !initialLoadComplete}
        <div role="status">Loading your vocabulary...<Skeleton class="h-40 w-full" /></div>
      {:else if route === "Vocabulary"}
        <Tabs.Root value={vocabularyView} onValueChange={(value) => { vocabularyView = value as typeof vocabularyView; updateSearch(""); }}><Tabs.List aria-label="Vocabulary views"><Tabs.Trigger value="active">Active</Tabs.Trigger><Tabs.Trigger value="mastered">Mastered</Tabs.Trigger><Tabs.Trigger value="achieved">Achieved ({achievedWords.length})</Tabs.Trigger></Tabs.List></Tabs.Root>
        <div class="tools"><label><span>Search</span><Input aria-label="Search vocabulary" value={search} oninput={(event) => updateSearch(event.currentTarget.value)} placeholder="Word or translation" /></label><span>{vocabularyView === "achieved" ? filteredAchievedWords.length : filteredWords.length} items</span></div>
        {#if vocabularyView === "achieved"}<div class="list vocabulary-list">{#if filteredAchievedWords.length}<label class="select-all"><input type="checkbox" aria-label="Select all filtered Achieved vocabulary" checked={allFilteredAchievedSelected} onchange={(event) => toggleAllAchieved(event.currentTarget.checked)} /> Select all filtered</label>{/if}{#each filteredAchievedWords as word}<label class:urgent={word.urgency === "urgent"} class:warning={word.urgency === "warning"} class="achieved-row"><input type="checkbox" aria-label={`Select ${word.displayForm}`} checked={selectedAchievedIds.has(word.id)} onchange={(event) => toggleAchieved(word.id, event.currentTarget.checked)} /><span><strong>{word.displayForm}</strong><small>{word.translation ?? "No translation"}{#if word.translationLanguage}<small> · {word.translationLanguage}</small>{/if}</small><small>Achieved {new Date(word.achievedAt).toLocaleDateString()}</small></span><span>{word.remainingDays} days remaining · Deletes {new Date(word.deleteAfter).toLocaleDateString()}</span></label>{:else}<div class="state"><strong>{achievedWords.length ? "No matching vocabulary" : "No Achieved vocabulary"}</strong><span>{achievedWords.length ? "Try a different word or translation." : "Mastered words you Achieve will wait here before deletion."}</span></div>{/each}</div>{#if selectedAchievedIds.size}<div class="bulk-actions"><strong>{selectedAchievedIds.size} selected</strong><Button variant="outline" onclick={unachieveSelected}>Unachieve</Button><Button variant="destructiveOutline" onclick={deleteSelectedAchieved}>Delete permanently</Button></div>{/if}{:else}<div class="list vocabulary-list">{#if pagedWords.length}<VocabularyTable words={pagedWords} onSelect={showDetail} onAchieve={achieveWordFromRow} />{:else}{#if visibleWords.length}<div class="state"><strong>No matching vocabulary</strong><span>Try a different word or translation.</span></div>{:else}<div class="state"><strong>No {vocabularyView} vocabulary</strong><span>Vocabulary Items in this state will appear here.</span></div>{/if}{/if}</div>
        {#if filteredWords.length}<nav class="pagination" aria-label="Vocabulary pages"><button class="secondary" disabled={vocabularyPage === 1} onclick={() => goToVocabularyPage(1)}>First</button><button class="secondary" disabled={vocabularyPage === 1} onclick={() => goToVocabularyPage(vocabularyPage - 1)}>Previous</button><form aria-label="Go to vocabulary page" onsubmit={(event) => { event.preventDefault(); goToVocabularyPage(vocabularyPageInput); }}><label><span>Page</span><input aria-label="Page number" type="number" min="1" max={vocabularyPageCount} bind:value={vocabularyPageInput} onblur={() => goToVocabularyPage(vocabularyPageInput)} /><span>of {vocabularyPageCount}</span></label></form><button class="secondary" disabled={vocabularyPage === vocabularyPageCount} onclick={() => goToVocabularyPage(vocabularyPage + 1)}>Next</button><button class="secondary" disabled={vocabularyPage === vocabularyPageCount} onclick={() => goToVocabularyPage(vocabularyPageCount)}>Last</button></nav>{/if}{/if}
      {:else if route === "Review"}
        {#if reviewOpen && activeReview}
          <div class="review-card" aria-live="polite" bind:this={reviewCardElement}><div class="review-progress"><span>{reviewIndex + 1} of {today?.reviewQueue.length}</span><Button variant="ghost" size="icon" aria-label="Close review" disabled={reviewSubmitting} onclick={closeReview}>×</Button></div><Progress value={reviewIndex} max={today?.reviewQueue.length ?? 1} aria-label="Review progress" /><span class="eyebrow">Do you remember this word?</span><h2>{activeReview.displayForm}</h2><p>{activeReview.context ?? "No saved context"}</p>{#if reviewResult}<div class="review-translation" role="status"><small>{reviewResult.rating === "remembered" ? "Remembered" : "Forgot"}</small><strong>Next review {new Date(reviewResult.nextDueAt).toLocaleDateString()}</strong><span>{`Saved ${reviewResult.encounterCount} time${reviewResult.encounterCount === 1 ? "" : "s"}`}</span>{#if reviewResult.repeatedForgetting}<p>This Vocabulary Item has been repeatedly forgotten. Another context or a translation check may help.</p>{/if}</div>{:else if reviewRevealed}<div class="review-translation" role="status"><small>Translation</small><strong>{activeReview.translation ?? "Unavailable"}{#if activeReview.translationLanguage}<small> · {activeReview.translationLanguage}</small>{/if}</strong></div>{/if}{#if reviewError}<div class="dialog-error" role="alert">{reviewError}</div>{/if}<div class="review-actions">{#if reviewResult}{#if reviewResult.repeatedForgetting}<Button variant="outline" onclick={(event) => showDetail(activeReview.wordId, event.currentTarget)}>Review contexts</Button>{/if}<Button onclick={nextReview}>Next</Button>{:else if reviewRevealed}{#if reviewError && reviewSubmissionRating}<Button disabled={reviewSubmitting} onclick={retryReviewSubmission}>Retry {reviewSubmissionRating === "remembered" ? "Remembered" : "Forgot"}</Button>{:else}<Button variant="outline" disabled={reviewSubmitting} onclick={() => rateReview("forgot")}>Forgot</Button><Button disabled={reviewSubmitting} onclick={() => rateReview("remembered")}>Remembered</Button>{/if}{:else}<Button onclick={revealReview}>Show answer</Button>{/if}</div></div>
        {:else if reviewComplete && reviewSessionInsight}
          <div class="state" aria-live="polite"><h2 tabindex="-1" bind:this={reviewCompleteHeading}>Review complete</h2><strong>{reviewSessionInsight.reviewedCount} reviewed</strong><span>{reviewSessionInsight.rememberedCount} remembered · {reviewSessionInsight.forgottenCount} forgot</span><span>Estimated due by the end of tomorrow: {reviewSessionInsight.nextDayDueCount}</span>{#if reviewSessionInsight.attentionWordIds.length}<div><strong>Worth another context</strong>{#each reviewSessionInsight.attentionWordIds as wordId}<span>{attentionLabel(wordId)} may benefit from another context or a translation check.</span>{/each}</div>{/if}<Button onclick={returnToToday}>Back to Today</Button></div>
        {:else if reviewRefreshRequired}
          <div class="state"><strong>{reviewCompleting ? "Review saved" : "Review paused"}</strong><span>Refresh Today before continuing so the due queue stays current.</span><Button onclick={retryReviewRefresh}>Retry Review refresh</Button></div>
        {:else if today?.reviewQueue.length}
          <div class="state"><strong>{reviewPaused ? "Review paused" : `${today.plannedReviewCount} planned · ${today.totalDueCount} total due`}</strong><span>{reviewPaused ? `${today.reviewQueue.length} words remaining.` : `About ${today.estimatedMinutes} minute${today.estimatedMinutes === 1 ? "" : "s"}.`}</span><Button onclick={startReview}>{reviewPaused ? "Resume review" : "Start review"}</Button></div>
        {:else}
          <div class="state"><strong>Nothing due</strong><span>Your review queue is clear for today.</span></div>
        {/if}
      {:else if route === "Insights"}
        <InsightsPanel log={vocabularyLog} {logLoading} {logError} onlogretry={loadLog} insight={globalInsight} session={reviewSessionInsight} due={today?.totalDueCount ?? null} error={insightError} onretry={async () => { globalInsight = await loadInsight(); }} />
      {:else if settingsDraft}
        <SettingsForm bind:settingsDraft {systemStatus} {settingsError} {settingsSaving} oncommit={commitSettings} onretry={saveSettings} />
      {/if}
    </section>
  </main>
</div>


<Dialog.Root open={captureOpen} onOpenChange={(open) => { if (!open) closeCapture(); }}><Dialog.Content showCloseButton={false} class="manual-capture-content" onInteractOutside={(event) => event.preventDefault()} onOpenAutoFocus={(event) => { event.preventDefault(); captureFirstField?.focus(); }} onCloseAutoFocus={(event) => { event.preventDefault(); captureTrigger?.focus(); }} onkeydown={trapDialogFocus} bind:ref={captureDialog}><form onsubmit={(event) => { event.preventDefault(); saveCapture(); }}><div class="dialog-heading"><div><span class="eyebrow">Manual Capture</span><Dialog.Title>Save a reading context</Dialog.Title></div></div>{#if achievedCaptureConflict}<div class="achieved-capture-notice" role="status"><span>Achieved</span><p>Already learned and reviewed. Saving will restart learning and save this context.</p></div>{/if}{#if captureError}<div class="dialog-error" role="alert">{captureError}</div>{/if}<Field.FieldGroup><Field.Field><label>Word or phrase<Input bind:ref={captureFirstField} bind:value={captureInput.selectedText} /></label></Field.Field><Field.Field><label>Translation <small>Optional</small><Input bind:value={captureInput.translation} /></label></Field.Field><Field.Field><label>Context<Textarea bind:value={captureInput.sentence}></Textarea></label></Field.Field></Field.FieldGroup><div class="actions"><Button variant="outline" onclick={closeCapture}>Cancel</Button><Button type="submit" disabled={saving || !captureInput.selectedText.trim() || !captureInput.sentence.trim()}>{saving ? "Saving..." : "Save capture"}</Button></div></form></Dialog.Content></Dialog.Root>

<Toaster id={notificationHostId} position="bottom-right" />

<VocabularyDetail detail={selectedDetail} trigger={detailTrigger} onclose={closeDetail} onachieve={achieveSelectedWord} />
</div>

<style>
  :global(html), :global(body), :global(#app) { min-width: 100%; min-height: 100%; margin: 0; }
  :global(body) { background: transparent; } :global(*) { box-sizing: border-box; }
  button, input { font: inherit; } button:focus-visible, input:focus-visible { outline: 2px solid var(--ring); outline-offset: 2px; }
  .windows-presentation { min-height:100vh; color:var(--foreground); background:var(--background); font:0.875rem/1.45 var(--font-sans); }

  .windows-shell { min-height: 100vh; display: grid; grid-template-columns: 220px minmax(0, 1fr); color: var(--text); background: var(--page); }
  aside { display: flex; flex-direction: column; padding: 18px 12px 14px; border-right: 1px solid var(--line); background: var(--sidebar); }
  .brand { display: flex; align-items: center; gap: 10px; min-height: 36px; padding: 0 8px 18px; }.brand strong { font-size:0.8125rem; }
  nav { display: grid; gap: 3px; } nav button { min-height: 36px; padding: 0 10px; border: 0; border-radius: 6px; color: var(--muted-foreground); background: transparent; text-align: left; cursor: pointer; } nav button:hover, nav button.active { color: var(--text); background: var(--surface-raised); }
  .local-status { display: flex; align-items: center; gap: 8px; margin-top: auto; padding: 10px 8px; color: var(--muted-foreground); font-size:0.75rem; }
  main { min-width: 0; } header { display: flex; align-items: center; justify-content: space-between; height: 86px; padding: 0 28px; border-bottom: 1px solid var(--line); } h1, h2, p { margin: 0; } h1 { font-size:1.375rem; } header p { margin-top: 5px; color: var(--muted-foreground); font-size:0.75rem; } section { min-height: calc(100vh - 87px); padding: 26px 28px; }
  .secondary { min-height:32px; padding:0 14px; border-radius:10px; cursor:pointer; color: var(--text); border-color: var(--line); background: var(--surface-raised); }


  .list { overflow: hidden; border: 1px solid var(--line); border-radius: 7px; background: var(--surface); }

  .vocabulary-list { max-height: calc(100vh - 244px); overflow-y: auto; scrollbar-gutter: stable; }
  .vocabulary-list :global([data-slot=table-container]) { overflow:visible; }
  .pagination { display: flex; align-items: center; justify-content: flex-end; gap: 8px; margin-top: 12px; color: var(--muted-foreground); font-size:0.75rem; }
  .pagination form, .pagination label { display: flex; align-items: center; gap: 6px; }.pagination input { width: 54px; height: 32px; padding: 0 6px; border: 1px solid var(--line); border-radius: 5px; color: var(--text); background: var(--field); text-align: center; }
  .state { min-height: 220px; display: grid; place-content: center; justify-items: center; gap: 8px; color: var(--muted-foreground); text-align: center; }.state strong { color: var(--text); font-size:1rem; }
  .tools { margin-top:20px; display: flex; align-items: end; justify-content: space-between; margin-bottom: 12px; color: var(--muted-foreground); font-size:0.6875rem; }.tools label { display: grid; gap: 6px; }.tools :global(input) { width: 310px; height: 34px; padding: 0 10px; border: 1px solid var(--line); border-radius: 6px; color: var(--text); background: var(--surface); }
  .select-all,.achieved-row { display:grid; align-items:center; gap:12px; padding:12px 14px; border-bottom:1px solid var(--line); }.select-all { grid-template-columns:auto 1fr; color:var(--muted-foreground); }.achieved-row { grid-template-columns:auto minmax(0,1fr) auto; color:var(--text); }.achieved-row.warning { border-left:3px solid var(--warning); }.achieved-row.urgent { border-left:3px solid var(--destructive); }.achieved-row span { display:grid; gap:3px; overflow-wrap:anywhere; }.achieved-row > span:last-child { max-width:210px; font-size:0.75rem; color:var(--muted-foreground); }.achieved-row small { color:var(--muted-foreground); }.bulk-actions { flex-wrap:wrap; position:sticky; bottom:12px; display:flex; justify-content:flex-end; align-items:center; gap:10px; margin-top:12px; padding:12px; border:1px solid var(--line); border-radius:7px; background:var(--surface-raised); }
  .error { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-bottom: 14px; padding: 11px 13px; border: 1px solid var(--destructive); border-radius: 6px; color: var(--destructive); background: var(--card); }.error button { border: 0; color: var(--foreground); background: transparent; cursor: pointer; }
  .dialog-error { padding: 9px 10px; border: 1px solid var(--destructive); border-radius: 6px; color: var(--destructive); background: var(--card); font-size:0.75rem; }
  .achieved-capture-notice { padding: 10px; border: 1px solid var(--warning); border-radius: 6px; background: rgba(196, 145, 46, .1); }.achieved-capture-notice span { display: inline-block; padding: 2px 7px; border-radius: 999px; color: var(--warning); background: var(--card); font-size:0.625rem; font-weight: 700; text-transform: uppercase; }.achieved-capture-notice p { margin: 7px 0 0; color: var(--text); font-size:0.75rem; }


  .windows-presentation[data-reduced-motion="true"], .windows-presentation[data-reduced-motion="true"] * { scroll-behavior: auto !important; animation-duration: .01ms !important; animation-iteration-count: 1 !important; transition-duration: .01ms !important; }
  .review-card { width:100%; max-width: 620px; margin: 24px auto; padding: 28px; border: 1px solid var(--line); border-radius: 8px; background: var(--surface); }.review-progress { display: flex; align-items: center; justify-content: space-between; margin-bottom: 28px; color: var(--muted-foreground); font-size:0.6875rem; }.review-card h2 { overflow-wrap:anywhere; margin: 10px 0; font-size:2.25rem; }.review-card > p { font-size:1.125rem; overflow-wrap:anywhere; color: var(--muted-foreground); line-height: 1.6; }.review-translation { display: grid; gap: 5px; margin: 22px 0; padding: 14px; border-radius: 7px; background: var(--muted); }.review-translation small { color: var(--muted-foreground); }.review-translation strong { color: var(--foreground); font-size:1rem; }.review-actions { display: flex; flex-wrap:wrap; justify-content:flex-end; gap: 10px; margin-top: 18px; }
  .dialog-heading { display: flex; justify-content: space-between; }.eyebrow { color: var(--muted-foreground); font-size:0.75rem; font-weight: 700; text-transform: uppercase; }.actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }


  @media (prefers-reduced-motion: reduce) { .windows-presentation, .windows-presentation * { scroll-behavior: auto !important; animation-duration: .01ms !important; animation-iteration-count: 1 !important; transition-duration: .01ms !important; } }
  @media (forced-colors: active) { .windows-presentation { --page: Canvas; --sidebar: Canvas; --surface: Canvas; --surface-raised: Canvas; --field: Field; --text: CanvasText; --muted: CanvasText; --line: CanvasText; }  }
  @media (max-width: 760px), (min-resolution: 1.5dppx) and (max-width: 1100px) { .windows-shell { grid-template-columns: 10rem minmax(0,1fr); } header, section { height: auto; min-height: 5.4rem; padding-left: 1.125rem; padding-right: 1.125rem; } }
  @media (max-width: 560px) { .windows-shell { display: block; } aside { position: static; } nav { grid-template-columns: repeat(2, minmax(0, 1fr)); } .local-status { margin-top: 0; } header { align-items: flex-start; gap: 1rem; padding-top: 1rem; padding-bottom: 1rem; } section { min-height: auto; } .tools { align-items: stretch; flex-direction: column; gap: .75rem; } .tools :global(input) { width: 100%; } .pagination { flex-wrap: wrap; justify-content: center; }.pagination form { order: -1; width: 100%; justify-content: center; } }
  .windows-shell { grid-template-columns:184px minmax(0,1fr); height:100vh; overflow:hidden; border-radius:14px; }
  aside { padding:16px 8px; gap:24px; }
  .brand { padding:0 8px; min-height:32px; gap:8px; }
  .brand strong { font-size:0.875rem; }
  nav button { display:flex; align-items:center; gap:8px; border-radius:10px; }
  nav button.active { background:var(--accent); color:var(--accent-foreground); }
  main { overflow:auto; background:var(--background); }
  header { max-width:856px; margin:0 auto; height:auto; min-height:110px; padding:32px 48px 24px; border:0; }
  h1 { font-size:1.5rem; line-height:1.2; letter-spacing:-.025em; font-weight:600; }
  header p { font-size:0.875rem; line-height:1.45; }
  section { min-height:0; max-width:856px; padding:0 48px 32px; margin:0 auto; }
  @media(max-width:900px) { header { padding:24px; } section { padding:0 24px 24px; } }
  :global(.manual-capture-content) { max-width:460px; max-height:calc(100dvh - 32px); overflow:auto; padding:24px; box-shadow:var(--shadow-dialog); }
  :global(.manual-capture-content form) { display:flex; flex-direction:column; gap:20px; }
  :global(.manual-capture-content label) { display:flex; flex-direction:column; gap:6px; font-size:0.875rem; }
  :global(.manual-capture-content textarea) { min-height:96px; }
  :global(.manual-capture-content [data-slot=dialog-title]) { font-size:1.5rem; line-height:1.2; font-weight:600; margin-top:8px; }
</style>
