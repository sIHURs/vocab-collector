<script lang="ts">
  import type { WordDetail } from '../lib/types';
  import * as Sheet from '$lib/components/ui/sheet';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import X from '@lucide/svelte/icons/x';
  export let detail: WordDetail | null;
  export let trigger: HTMLElement | null;
  export let onclose: () => void;
  export let onachieve: () => void;
  let closeButton: HTMLButtonElement;
</script>

<Sheet.Root open={detail !== null} onOpenChange={(open) => { if (!open) onclose(); }}>
  <Sheet.Content showCloseButton={false} class="vocabulary-detail" onInteractOutside={(event) => event.preventDefault()} onOpenAutoFocus={(event) => { event.preventDefault(); closeButton?.focus(); }} onCloseAutoFocus={(event) => { event.preventDefault(); trigger?.focus(); }}>
    {#if detail}
      <div class="detail-heading"><span>Vocabulary detail</span><button bind:this={closeButton} class="close" aria-label="Close vocabulary detail" onclick={onclose}><X size={18} /></button></div>
      <Sheet.Title>{detail.item.displayForm}</Sheet.Title>
      <Sheet.Description>{detail.item.translation ?? 'No translation'}</Sheet.Description>
      <div class="detail-status"><Badge variant="secondary"><span class="status">{detail.item.status}</span></Badge><span>{detail.item.encounterCount} encounter{detail.item.encounterCount === 1 ? '' : 's'}</span></div>
      {#if detail.item.status === 'mastered'}<div><Button variant="destructiveOutline" onclick={onachieve}>Achieve</Button></div>{/if}
      <section class="timeline" aria-label="Encounter contexts"><h3>Contexts</h3>
        {#each detail.encounters as encounter (encounter.id)}
          <article><p>{encounter.sentence}</p><small>{[encounter.sourceApp, encounter.sourceTitle, encounter.sourceUrl].filter(Boolean).join(' · ') || 'Manual entry'}</small></article>
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
