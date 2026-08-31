<script lang="ts">
  import { onMount } from "svelte";

  onMount(() => {
    document.body.classList.add("windows-capture-document");
    return () => document.body.classList.remove("windows-capture-document");
  });
</script>

<main class="windows-capture" data-presentation="windows-capture" aria-label="Capture">
  <header><i aria-hidden="true"></i><span>Vocab Collector</span></header>
  <section aria-live="polite"></section>
</main>

<style>
  :global(html), :global(body.windows-capture-document), :global(#app) { width: 100%; height: 100%; margin: 0; background: transparent; overflow: hidden; }
  :global(body.windows-capture-document) { min-width: 0; min-height: 0; }
  .windows-capture { box-sizing: border-box; width: 100%; height: 100%; padding: 14px 16px; border: 1px solid #4b4f60; border-radius: 8px; color: #eeeef3; background: rgba(30, 31, 40, .98); box-shadow: 0 18px 48px rgba(0, 0, 0, .42); font: 14px "Segoe UI Variable", "Segoe UI", sans-serif; }
  header { display: flex; align-items: center; gap: 7px; color: #aaadba; font-size: 12px; }
  header i { width: 7px; height: 7px; border-radius: 50%; background: #7584ef; }
  section { min-height: 190px; }
</style>
