<script lang="ts">
  import type { WordListItem } from "../lib/types";
  export let word: WordListItem;
  export let onSelect: (wordId: string, trigger: HTMLButtonElement) => void;
  const encounters = (count: number) => `${count} encounter${count === 1 ? "" : "s"}`;
  const lastSeen = (value: string) => new Intl.DateTimeFormat(undefined, { dateStyle: "medium" }).format(new Date(value));
</script>

<button class="word-row" aria-label={`${word.displayForm}, ${encounters(word.encounterCount)}`} onclick={(event) => onSelect(word.id, event.currentTarget)}>
  <span class="glyph">{word.displayForm.slice(0, 1).toUpperCase()}</span>
  <span class="word"><strong>{word.displayForm}</strong><small>{word.translation ?? "No translation"}</small></span>
  <span class="status">{word.status}</span>
  <span class="metadata">{encounters(word.encounterCount)} · Last seen {lastSeen(word.lastSeenAt)}</span>
  <span aria-hidden="true">›</span>
</button>

<style>
  .word-row { width: 100%; min-height: 60px; display: grid; grid-template-columns: 32px minmax(140px, 1fr) 76px 210px 16px; gap: 12px; align-items: center; padding: 8px 14px; border: 0; border-bottom: 1px solid var(--line, #30323d); color: var(--text, #eeeef3); background: transparent; text-align: left; cursor: pointer; font: inherit; }
  .word-row:last-child { border-bottom: 0; }.word-row:hover { background: var(--surface-raised, #272934); }.word-row:focus-visible { outline: 2px solid #9aa5ff; outline-offset: -2px; }
  .glyph { display: grid; place-items: center; width: 30px; height: 30px; border-radius: 6px; color: #b8c0ff; background: rgba(117,132,239,.16); font-weight: 700; }
  .word { display: grid; gap: 3px; }.word small, .metadata { color: var(--muted, #9195a4); font-size: 11px; }
  .status { justify-self: start; padding: 3px 7px; border-radius: 10px; color: #b8c0ff; background: rgba(117,132,239,.16); font-size: 10px; text-transform: capitalize; }
  @media (max-width: 900px) { .word-row { grid-template-columns: 32px minmax(0,1fr) 90px 16px; }.status { display: none; }.metadata { text-align: right; } }
</style>
