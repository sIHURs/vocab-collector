<script lang="ts">
  let { value, onRecorded }: { value: string; onRecorded: (shortcut: string) => void } = $props();
  let recording = $state(false);
  let message = $state("");
  let current = $state("");
  $effect(() => { if (!recording) current = value; });

  const display = (shortcut: string) => shortcut.split("+").map((part) => ({
    Meta: "⌘", Alt: "⌥", Control: "⌃", Shift: "⇧",
  }[part] ?? part)).join(" ");

  function keydown(event: KeyboardEvent) {
    if (!recording) return;
    event.preventDefault();
    if (event.key === "Escape") { recording = false; message = ""; return; }
    if (["Alt", "Control", "Meta", "Shift"].includes(event.key)) return;
    const modifiers = [event.metaKey && "Meta", event.altKey && "Alt",
      event.ctrlKey && "Control", event.shiftKey && "Shift"].filter(Boolean) as string[];
    if (!modifiers.length) { message = "Add a modifier such as ⌥, ⌘, or ⌃."; return; }
    const key = event.key === " " ? "Space" : event.key.toUpperCase();
    current = [...modifiers, key].join("+");
    recording = false;
    message = "";
    onRecorded(current);
  }
</script>

<svelte:window onkeydown={keydown} />
<div class="shortcut-recorder">
  <kbd>{display(current)}</kbd>
  <button type="button" aria-label="Record shortcut" onclick={() => { recording = true; message = "Press your shortcut"; }}>
    {recording ? "Recording…" : "Record shortcut"}
  </button>
  {#if message}<small>{message}</small>{/if}
</div>

<style>
  .shortcut-recorder { display: grid; grid-template-columns: 1fr auto; gap: .55rem; align-items: center; }
  kbd { min-height: 2.35rem; display: flex; align-items: center; padding: 0 .75rem; border: 1px solid var(--line); border-radius: 7px; background: var(--surface-raised, #24242c); color: var(--text); font: inherit; }
  button { min-height: 2.35rem; padding: 0 .75rem; border: 1px solid var(--line); border-radius: 7px; background: transparent; color: var(--text); cursor: pointer; }
  small { grid-column: 1 / -1; color: var(--muted); }
</style>
