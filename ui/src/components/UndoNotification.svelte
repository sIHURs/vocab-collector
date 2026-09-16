<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import Check from '@lucide/svelte/icons/check';
  import CircleX from '@lucide/svelte/icons/circle-x';
  import X from '@lucide/svelte/icons/x';
  export let title: string;
  export let description: string;
  export let label: string;
  export let dismissLabel: string;
  export let onundo: (() => void) | undefined = undefined;
  export let tone: 'success' | 'error' = 'success';
  export let ondismiss: () => void;
  export let busy = false;
  export let error = '';
</script>

<div class="notification" role="dialog" aria-label={label}>
  {#if tone === 'error'}<CircleX size={18} aria-hidden="true" />{:else}<Check size={18} aria-hidden="true" />{/if}
  <div><strong>{title}</strong><span>{description}</span>{#if error}<span role="alert">{error}</span>{/if}</div>
  {#if onundo}<Button variant="outline" disabled={busy} onclick={onundo}>Undo</Button>{/if}
  <Button variant="ghost" size="icon" disabled={busy} aria-label={dismissLabel} onclick={ondismiss}><X size={16} /></Button>
</div>

<style>
  .notification { display:flex; align-items:center; gap:12px; padding:16px; width:100%; border:1px solid var(--border); border-radius:10px; background:var(--popover); color:var(--foreground); box-shadow:var(--shadow-floating); }
  .notification > div { flex:1; min-width:0; display:grid; gap:4px; overflow-wrap:anywhere; }
  strong { font-size:0.875rem; font-weight:500; }
  span { font-size:0.75rem; color:var(--muted-foreground); }
</style>
