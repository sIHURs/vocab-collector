<script lang="ts">
  import '../native-capture.css';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Badge } from '$lib/components/ui/badge';
  import * as Field from '$lib/components/ui/field';
  import * as Alert from '$lib/components/ui/alert';
  import CaptureSource from '../components/CaptureSource.svelte';
  import { onMount } from "svelte";
  import type { AchievedCaptureConflict, CaptureCandidate, CaptureCard } from "../lib/types";
  import {
    tauriWindowsCaptureBackend,
    type OcrCandidate,
    type WindowsCaptureBackend,
  } from "./captureBackend";

  export let captureBackend: WindowsCaptureBackend = tauriWindowsCaptureBackend;
  let activeRequest = "";
  let hovered = false;
  let focusWithin = false;
  let candidate: CaptureCandidate | null = null;
  let editing = false;
  let selectedText = "";
  let sentence = "";
  let translation = "";
  let translationSource = "";
  let translationTarget = "";
  let translationAvailable = false;
  let translationFailed = false;
  let translationStale = false;
  let lastTranslatedText = "";
  let saved: CaptureCard | null = null;
  let achievedConflict: AchievedCaptureConflict | null = null;
  let busy = false;
  let error = "";
  let mounted = false;
  let dismissTimer: ReturnType<typeof setTimeout> | undefined;
  let ocrOffer = false;
  let ocrNeedsConfirmation = false;
  let ocrAvailable = false;
  let ocrEligibleFailure = false;
  let ocrCandidates: OcrCandidate[] = [];
  let ocrBusy = false;
  let ocrAttempted = false;

  function resetOcrState() {
    ocrOffer = false;
    ocrEligibleFailure = false;
    ocrNeedsConfirmation = false;
    ocrCandidates = [];
    ocrBusy = false;
    ocrAttempted = false;
  }

  function clearDismissTimer() {
    if (dismissTimer) clearTimeout(dismissTimer);
    dismissTimer = undefined;
  }

  function scheduleDismiss(requestId = activeRequest) {
    clearDismissTimer();
    if (!saved || !mounted || requestId !== activeRequest || hovered || focusWithin) return;
    dismissTimer = setTimeout(() => {
      if (mounted && requestId === activeRequest) void captureBackend.hide(requestId);
    }, 4_000);
  }

  async function beginEditing() {
    const requestId = activeRequest;
    await captureBackend.focus();
    if (!mounted || requestId !== activeRequest) return;
    editing = true;
  }

  function correction() {
    return {
      selectedText: selectedText.trim(),
      sentence: sentence.trim(),
      ...(translation.trim() ? { translation: translation.trim() } : {}),
    };
  }

  async function applyChanges() {
    if (!candidate || busy) return;
    const requestId = activeRequest;
    busy = true;
    error = "";
    try {
      await captureBackend.apply(requestId, correction());
      if (!mounted || requestId !== activeRequest) return;
      translationStale = Boolean(translation.trim()) && selectedText.trim() !== lastTranslatedText;
      editing = false;
    } catch (cause) {
      if (!mounted || requestId !== activeRequest) return;
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
  }

  async function save() {
    if (!candidate || busy) return;
    const requestId = activeRequest;
    busy = true;
    error = "";
    try {
      const conflict = await captureBackend.findAchieved(requestId);
      if (!mounted || requestId !== activeRequest) return;
      if (conflict) {
        achievedConflict = conflict;
        return;
      }
      const result = await captureBackend.save(requestId, !translation.trim());
      if (!mounted || requestId !== activeRequest) return;
      saved = result;
      editing = false;
      scheduleDismiss(requestId);
    } catch (cause) {
      if (!mounted || requestId !== activeRequest) return;
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
  }

  async function restoreToLearningAndSave() {
    if (!achievedConflict || busy) return;
    const requestId = activeRequest;
    const wordId = achievedConflict.wordId;
    busy = true;
    error = "";
    try {
      const result = await captureBackend.restoreAchievedAndSave(requestId, wordId, !translation.trim());
      if (!mounted || requestId !== activeRequest) return;
      achievedConflict = null;
      saved = result;
      editing = false;
      scheduleDismiss(requestId);
    } catch (cause) {
      if (mounted && requestId === activeRequest) error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
  }

  async function translateCandidate(requestId: string, text: string) {
    if (busy) return;
    busy = true;
    error = "";
    translationFailed = false;
    try {
      const [capabilities, currentSettings] = await Promise.all([
        captureBackend.getCapabilities(),
        captureBackend.getSettings(),
      ]);
      if (!mounted || requestId !== activeRequest) return;
      translationAvailable = capabilities.translation;
      if (!translationAvailable) return;
      const result = await captureBackend.translate(requestId, text, currentSettings.sourceLanguage, currentSettings.targetLanguage);
      if (!mounted || requestId !== activeRequest) return;
      translation = result.translatedText;
      lastTranslatedText = text;
      translationStale = false;
      translationSource = result.sourceLanguage;
      translationTarget = result.targetLanguage;
      await captureBackend.apply(requestId, correction());
    } catch {
      if (!mounted || requestId !== activeRequest) return;
      translationFailed = true;
      error = "Translation is temporarily unavailable. You can retry or continue editing.";
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
  }

  async function undo() {
    if (!saved || busy) return;
    const requestId = activeRequest;
    const encounterId = saved.encounterId;
    busy = true;
    error = "";
    try {
      await captureBackend.undo(requestId, encounterId);
      if (!mounted || requestId !== activeRequest) return;
      saved = null;
      achievedConflict = null;
      clearDismissTimer();
      await captureBackend.close(requestId);
    } catch (cause) {
      if (!mounted || requestId !== activeRequest) return;
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
  }

  async function confirmOcrCandidate() {
    if (busy) return;
    const requestId = activeRequest;
    if (!candidate || !selectedText.trim()) return;
    busy = true;
    try {
      await captureBackend.confirmOcr(requestId, selectedText, sentence);
      if (!mounted || requestId !== activeRequest) return;
      ocrNeedsConfirmation = false;
    } catch {
      if (mounted && requestId === activeRequest) error = "OCR confirmation failed. Please try again.";
      return;
    } finally {
      if (mounted && requestId === activeRequest) busy = false;
    }
    void translateCandidate(requestId, selectedText.trim());
  }

  async function startOcr() {
    if (!activeRequest || ocrBusy) return;
    const requestId = activeRequest;
    ocrBusy = true;
    ocrAttempted = true;
    error = "";
    try {
      await captureBackend.startRegionOcr();
    } catch (cause) {
      if (!mounted || requestId !== activeRequest) return;
      error = cause instanceof Error ? cause.message : String(cause);
      ocrOffer = true;
    } finally {
      if (mounted && requestId === activeRequest) ocrBusy = false;
    }
  }

  function useOcrDraft(candidates: OcrCandidate[]) {
    const text = candidates.map((item) => item.text.trim()).filter(Boolean).join(" ");
    const selected = candidates[0];
    if (!selected || !text) return;
    candidate = {
      selectedText: text,
      sentence: "",
      selectionBounds: selected.bounds,
      origin: "ocr",
    };
    selectedText = text;
    sentence = "";
  }

  async function cancel() {
    if (!activeRequest) return;
    try {
      await captureBackend.close(activeRequest);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function useManualCapture() {
    await captureBackend.openManualCapture();
    await cancel();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") void cancel();
  }

  onMount(() => {
    mounted = true;
    document.body.classList.add("windows-capture-document");
    const ready = captureBackend.listenReady((event) => {
      activeRequest = event.requestId;
      clearDismissTimer();
      candidate = event.candidate;
      selectedText = event.candidate.selectedText;
      sentence = event.candidate.sentence;
      translation = "";
      translationSource = "";
      translationTarget = "";
      translationFailed = false;
      translationStale = false;
      lastTranslatedText = "";
      saved = null;
      achievedConflict = null;
      error = "";
      editing = false;
      resetOcrState();
      if (event.candidate.origin !== "ocr") void translateCandidate(event.requestId, event.candidate.selectedText);
    });
    void captureBackend.getCapabilities().then((capabilities) => {
      if (mounted) {
        ocrAvailable = capabilities.screenshotOcr;
        ocrOffer = ocrEligibleFailure && ocrAvailable;
      }
    });
    const failed = captureBackend.listenError((event) => {
      activeRequest = event.requestId;
      candidate = null;
      saved = null;
      achievedConflict = null;
      error = event.failure.message;
      ocrEligibleFailure = event.failure.code === "empty_selection" || event.failure.code === "unsupported_element";
      ocrOffer = ocrAvailable && (ocrEligibleFailure || ocrAttempted);
      ocrNeedsConfirmation = false;
      ocrCandidates = [];
      ocrBusy = false;
    });
    const ocr = captureBackend.listenOcrCandidate((event) => {
      if (event.requestId !== activeRequest) return;
      ocrCandidates = event.candidates;
      useOcrDraft(event.candidates);
      error = "";
      ocrOffer = false;
      ocrNeedsConfirmation = true;
      void captureBackend.focus();
    });
    const regionOcr = captureBackend.listenRegionOcrStart((event) => {
      activeRequest = event.requestId;
      candidate = null;
      saved = null;
      error = "请框取词汇";
      ocrOffer = false;
      ocrAttempted = true;
    });
    return () => {
      mounted = false;
      clearDismissTimer();
      document.body.classList.remove("windows-capture-document");
      ready.then((unlisten) => unlisten());
      failed.then((unlisten) => unlisten());
      ocr.then((unlisten) => unlisten());
      regionOcr.then((unlisten) => unlisten());
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="windows-capture capture-surface" data-presentation="windows-capture" aria-label="Capture" onmouseenter={() => { hovered = true; clearDismissTimer(); }} onmouseleave={() => { hovered = false; scheduleDismiss(); }} onfocusin={() => { focusWithin = true; clearDismissTimer(); }} onfocusout={(event) => { focusWithin = event.currentTarget.contains(event.relatedTarget as Node | null); if (!focusWithin) scheduleDismiss(); }}>
  <header data-tauri-drag-region><span data-tauri-drag-region><i aria-hidden="true" data-tauri-drag-region></i>Vocab Collector</span>{#if !ocrNeedsConfirmation && !ocrOffer && !achievedConflict}<Button variant="ghost" size="icon" aria-label="Cancel capture" onclick={cancel}>×</Button>{/if}</header>
  <section class="capture-body" aria-live="polite" aria-busy={busy || ocrBusy}>
    {#if error}<Alert.Root variant="destructive"><Alert.Title>Capture needs attention</Alert.Title><Alert.Description>{error}</Alert.Description></Alert.Root>{/if}
    {#if saved}
      <Badge variant="secondary">Saved</Badge><h1>{saved.displayForm}</h1>
      <p class="translation">{saved.translation ?? "Saved without translation"}</p><p class="context">{saved.context}</p>
      <small>{saved.isExistingWord ? `Saved ${saved.encounterCount} times · New context saved` : "Saved for the first time"}</small>
      <CaptureSource app={candidate?.sourceApp} title={candidate?.sourceTitle} url={candidate?.sourceUrl} />
    {:else if candidate}
      <small>{ocrNeedsConfirmation ? "Confirm OCR text" : editing ? "Editing" : "Captured"}</small>
      {#if ocrNeedsConfirmation}
        {#if ocrCandidates.length > 1}<p role="status">识别到多个词，请保留你要收集的词汇。</p>{/if}
        <Field.FieldGroup><Field.Field><label>Vocabulary<Input bind:value={selectedText} /></label></Field.Field><Field.Field><label>Context sentence <small>Optional</small><Textarea bind:value={sentence} /></label></Field.Field></Field.FieldGroup>
      {:else if editing}
        <Field.FieldGroup><Field.Field><label>Selected text<Input aria-label="Selected text" bind:value={selectedText} /></label></Field.Field><Field.Field><label>Context<Textarea aria-label="Context" bind:value={sentence} /></label></Field.Field><Field.Field><label>Translation <small>Optional</small><Input aria-label="Translation (optional)" bind:value={translation} /></label></Field.Field></Field.FieldGroup>
      {:else}
        {#if achievedConflict}<Badge variant="secondary">Achieved</Badge>{/if}
        <h1>{selectedText}</h1>
        {#if translation}<p class="translation">{translation}</p><small>{translationSource} → {translationTarget}</small>
        {:else if busy && translationAvailable}<p>Translating…</p>
        {:else if !translationAvailable}<p>Automatic translation is unavailable. You can add a translation manually.</p>{/if}
        <p class="context">“{sentence}”</p><CaptureSource app={candidate.sourceApp} title={candidate.sourceTitle} url={candidate.sourceUrl} />
        {#if translationStale}<Alert.Root role="status"><Alert.Title>Vocabulary changed</Alert.Title><Alert.Description>Vocabulary changed. The translation may no longer match.</Alert.Description></Alert.Root>{/if}
        {#if achievedConflict}<Alert.Root role="status"><Alert.Title>Return to Learning?</Alert.Title><Alert.Description>This Vocabulary Item is Achieved. Return it to Learning and save this context?</Alert.Description></Alert.Root>{/if}
      {/if}
    {:else if !ocrOffer && !error}<p>Ready to capture selected text.</p>{/if}
  </section>
  <footer aria-label="Capture actions">
    {#if saved}<Button variant="outline" disabled={busy} onclick={undo}>Undo</Button>
    {:else if candidate}
      {#if ocrNeedsConfirmation}<Button variant="outline" onclick={cancel}>Cancel OCR</Button><Button disabled={busy || !selectedText.trim()} onclick={confirmOcrCandidate}>Confirm</Button>
      {:else if editing}<Button disabled={busy || !selectedText.trim()} onclick={applyChanges}>Apply changes</Button>
      {:else if achievedConflict}<Button variant="outline" disabled={busy} onclick={cancel}>Cancel</Button><Button disabled={busy} onclick={restoreToLearningAndSave}>Return to Learning</Button>
      {:else}
        {#if translationFailed}<Button variant="outline" disabled={busy} onclick={() => translateCandidate(activeRequest, selectedText.trim())}>Retry translation</Button>{/if}
        {#if translationStale}<Button variant="outline" disabled={busy} onclick={() => translateCandidate(activeRequest, selectedText.trim())}>Translate again</Button>{/if}
        <Button variant="outline" disabled={busy} onclick={beginEditing}>Edit capture</Button><Button disabled={busy} onclick={save}>{translation.trim() ? 'Save capture' : 'Save without translation'}</Button>
      {/if}
    {:else if ocrOffer}<Button variant="outline" onclick={cancel}>Cancel</Button><Button variant="outline" onclick={useManualCapture}>Manual Capture</Button><Button disabled={ocrBusy} onclick={startOcr}>{ocrBusy ? "Starting OCR..." : ocrAttempted ? "Try Again" : "Start OCR"}</Button>{/if}
  </footer>
</main>
<style>
:global(html),:global(body.windows-capture-document),:global(#app){width:100%;height:100%;margin:0;background:transparent;overflow:hidden}:global(body.windows-capture-document){min-width:0;min-height:0}
</style>
