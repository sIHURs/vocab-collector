<script lang="ts">
  import { onMount } from 'svelte';
  import { isTauri } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import PanelLeft from '@lucide/svelte/icons/panel-left';
  import Plus from '@lucide/svelte/icons/plus';
  import Minus from '@lucide/svelte/icons/minus';
  import Square from '@lucide/svelte/icons/square';
  import Copy from '@lucide/svelte/icons/copy';
  import X from '@lucide/svelte/icons/x';

  export let sidebarOpen = true;
  export let search = '';
  export let onsearch: (value: string) => void;
  export let oncapture: (event: MouseEvent) => void;
  export let onerror: (message: string) => void;
  let maximized = false;
  let searchInput: HTMLInputElement | null = null;
  const native = isTauri();

  async function control(action: 'minimize' | 'toggleMaximize' | 'close') {
    if (!native) return;
    try { await getCurrentWindow()[action](); }
    catch (cause) { onerror(`Could not ${action === 'toggleMaximize' ? 'resize' : action} window: ${String(cause)}`); }
  }

  function shortcut(event: KeyboardEvent) {
    if (event.ctrlKey && event.key.toLowerCase() === 'k' && !document.querySelector('[role="dialog"], [role="alertdialog"]')) {
      event.preventDefault();
      searchInput?.focus();
      searchInput?.select();
    }
  }

  onMount(() => {
    if (!native) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const window = getCurrentWindow();
    const sync = async () => {
      try { const value = await window.isMaximized(); if (!disposed) maximized = value; }
      catch (cause) { if (!disposed) onerror(`Could not read window state: ${String(cause)}`); }
    };
    void sync();
    void window.onResized(sync).then(stop => { if (disposed) stop(); else unlisten = stop; })
      .catch(cause => { if (!disposed) onerror(`Could not track window state: ${String(cause)}`); });
    return () => { disposed = true; unlisten?.(); };
  });
</script>

<svelte:window onkeydown={shortcut} />

<div class="titlebar" data-tauri-drag-region>
  <Button variant="ghost" size="icon" aria-label={sidebarOpen ? 'Hide sidebar' : 'Show sidebar'} title={sidebarOpen ? 'Hide sidebar' : 'Show sidebar'} aria-expanded={sidebarOpen} aria-controls="windows-sidebar" onclick={() => sidebarOpen = !sidebarOpen}><PanelLeft data-icon="inline-start" /></Button>
  <div class="titlebar-search">
    <Input bind:ref={searchInput} type="search" aria-label="Search vocabulary" title="Search vocabulary (Ctrl+K)" placeholder="Search vocabulary…" value={search} oninput={(event) => onsearch(event.currentTarget.value)} />
  </div>
  <div class="drag-space" data-tauri-drag-region></div>
  <Button size="sm" variant="outline" aria-label="Manual capture" onclick={oncapture}><Plus data-icon="inline-start" />Add word</Button>
  <div class="controls" aria-label="Window controls">
    <Button variant="ghost" size="icon" aria-label="Minimize window" title="Minimize" disabled={!native} onclick={() => control('minimize')}><Minus data-icon="inline-start" /></Button>
    <Button variant="ghost" size="icon" aria-label={maximized ? 'Restore window' : 'Maximize window'} title={maximized ? 'Restore' : 'Maximize'} disabled={!native} onclick={() => control('toggleMaximize')}>{#if maximized}<Copy data-icon="inline-start" />{:else}<Square data-icon="inline-start" />{/if}</Button>
    <Button variant="ghost" size="icon" aria-label="Close window" title="Close" disabled={!native} onclick={() => control('close')}><X data-icon="inline-start" /></Button>
  </div>
</div>

<style>
  .titlebar { grid-column: 1 / -1; display:flex; align-items:center; gap:12px; min-width:0; padding:8px; background:var(--sidebar); border-bottom:1px solid var(--border); user-select:none; }
  .titlebar-search { width: min(340px, 38vw); min-width:120px; }
  .drag-space { flex:1; align-self:stretch; min-width:24px; }
  .controls { display:flex; gap:2px; }
  @media(max-width:560px) { .titlebar { gap:4px; } .titlebar-search { flex:1; min-width:0; } .drag-space { min-width:12px; flex:0; } }
</style>

