<script lang="ts">
  import type { WordDetail, WordStatus } from '../lib/types';
  import * as NativeSelect from '$lib/components/ui/native-select';
  import * as Field from '$lib/components/ui/field';
  import * as Alert from '$lib/components/ui/alert';
  import * as Sheet from '$lib/components/ui/sheet';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import X from '@lucide/svelte/icons/x';
  export let detail: WordDetail | null;
  export let trigger: HTMLElement | null;
  export let onclose: () => void;
  export let onachieve: () => void;
  export let onstatuschange: ((status: WordStatus) => Promise<void>) | undefined = undefined;
  export let statusSaving = false;
  export let statusError = '';
  export let onstatusretry: (() => void) | undefined = undefined;
  let selectedStatus: WordStatus = 'learning';
  $: if (detail && !statusSaving) selectedStatus = detail.item.status;
  let closeButton: HTMLButtonElement;
</script>

<Sheet.Root open={detail !== null} onOpenChange={(open) => { if (!open) onclose(); }}>
  <Sheet.Content showCloseButton={false} class="vocabulary-detail" onInteractOutside={(event) => event.preventDefault()} onOpenAutoFocus={(event) => { event.preventDefault(); closeButton?.focus(); }} onCloseAutoFocus={(event) => { event.preventDefault(); trigger?.focus(); }}>
    {#if detail}
      <div class="detail-heading"><span>Vocabulary detail</span><button bind:this={closeButton} class="close" aria-label="Close vocabulary detail" onclick={onclose}><X size={18} /></button></div>
      <Sheet.Title>{detail.item.displayForm}</Sheet.Title>
      <Sheet.Description>{detail.item.translation ?? 'No translation'}</Sheet.Description>
      <section aria-label="Saved translations">{#each detail.translations ?? [] as translation}<p><small>{translation.targetLanguage}</small> <span>{translation.text}</span></p>{/each}</section>
      <div class="detail-status"><Badge variant="secondary"><span class="status">{detail.item.status}</span></Badge><span>Saved {detail.item.encounterCount} time{detail.item.encounterCount === 1 ? '' : 's'}</span></div>
      {#if onstatuschange && !detail.item.achievedAt}
        <Field.FieldGroup><Field.Field data-disabled={statusSaving} data-invalid={!!statusError}>
          <Field.FieldLabel for="learning-status">Learning status</Field.FieldLabel>
          <NativeSelect.Root id="learning-status" bind:value={selectedStatus} disabled={statusSaving} aria-invalid={!!statusError} onchange={() => onstatuschange?.(selectedStatus)}>
            <NativeSelect.Option value="learning">Learning</NativeSelect.Option>
            <NativeSelect.Option value="mastered">Mastered</NativeSelect.Option>
            <NativeSelect.Option value="paused">Paused</NativeSelect.Option>
          </NativeSelect.Root>
        </Field.Field></Field.FieldGroup>
        {#if statusError}<Alert.Root variant="destructive"><Alert.Title>Learning status</Alert.Title><Alert.Description>{statusError}{#if onstatusretry}<Button variant="outline" disabled={statusSaving} onclick={onstatusretry}>Retry status update</Button>{/if}</Alert.Description></Alert.Root>{/if}
      {/if}
      {#if detail.item.status === 'mastered' && !detail.item.achievedAt}<div><Button variant="destructiveOutline" disabled={statusSaving} onclick={onachieve}>Achieve</Button></div>{/if}
      <section class="timeline" aria-label="Capture history"><h3>Capture history</h3>
        {#each detail.encounters as encounter (encounter.id)}
          <article><p>{encounter.sentence}</p>{#if encounter.savedTranslation}<p><small>{encounter.savedTranslation.targetLanguage}</small> <span>{encounter.savedTranslation.text}</span></p>{/if}<small>{[encounter.sourceApp, encounter.sourceTitle, encounter.sourceUrl].filter(Boolean).join(' · ') || 'Manual entry'}</small></article>
        {/each}
      </section>
    {/if}
  </Sheet.Content>
</Sheet.Root>

<style>
  :global(.vocabulary-detail) { width:min(440px, calc(100vw - 32px)); max-width:440px; padding:24px; overflow:auto; gap:20px; }
  :global(.vocabulary-detail [data-slot=sheet-title]) { font-size:1.5rem; line-height:1.2; font-weight:600; overflow-wrap:anywhere; }
  :global(.vocabulary-detail [data-slot=sheet-description]) { font-size:1.125rem; line-height:1.45; color:var(--foreground); overflow-wrap:anywhere; }
  .detail-heading { display:flex; justify-content:space-between; align-items:center; color:var(--muted-foreground); font-size:0.75rem; }
  .close { display:grid; place-items:center; width:32px; height:32px; border:0; border-radius:10px; background:transparent; color:var(--foreground); cursor:pointer; }
  .detail-status { display:flex; align-items:center; gap:8px; color:var(--muted-foreground); font-size:0.75rem; }
  .status { text-transform:capitalize; }
  .timeline { border-top:1px solid var(--border); padding-top:24px; margin:0; }
  h3 { font-size:1rem; line-height:1.45; font-weight:600; margin:0 0 20px; text-transform:none; letter-spacing:normal; }
  article { margin:0 0 24px; padding-left:16px; border-left:2px solid var(--border); overflow-wrap:anywhere; }
  p { margin:0 0 8px; font-size:0.875rem; line-height:1.5; }
  small { color:var(--muted-foreground); font-size:0.75rem; line-height:1.5; }
</style>
