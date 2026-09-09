<script lang="ts">
  import type { WordListItem } from "../lib/types";
  export let word: WordListItem;
  export let onSelect: (wordId: string, trigger: HTMLButtonElement) => void;
  export let onAchieve: ((word: WordListItem) => void) | undefined = undefined;
  const encounters = (count: number) => `Saved ${count} time${count === 1 ? "" : "s"}`;
  const lastSeen = (value: string) => new Intl.DateTimeFormat(undefined, { dateStyle: "medium" }).format(new Date(value));
</script>

<div class="word-row-wrap"><button class="word-row" aria-label={`${word.displayForm}, ${encounters(word.encounterCount)}`} onclick={(event) => onSelect(word.id, event.currentTarget)}>
  <span class="glyph">{word.displayForm.slice(0, 1).toUpperCase()}</span>
  <span class="word"><strong>{word.displayForm}</strong><small>{word.translation ?? "No translation"}</small></span>
  <span class="status">{word.status}</span>
  <span class="metadata">{encounters(word.encounterCount)} · Last saved {lastSeen(word.lastSeenAt)}</span>
  <span aria-hidden="true">›</span>
</button>{#if word.status === "mastered" && onAchieve}<button class="achieve-action" aria-label={`Achieve ${word.displayForm}`} onclick={() => onAchieve?.(word)}>Achieve</button>{/if}</div>

<style>
.word-row-wrap{border-bottom:1px solid var(--border)}.word-row{width:100%;min-height:70px;display:grid;grid-template-columns:minmax(0,1fr) auto 12px;gap:12px;align-items:center;padding:12px 16px;border:0;background:transparent;color:var(--foreground);font:inherit;text-align:left;cursor:pointer}.word-row:hover{background:var(--accent)}.word-row:focus-visible{outline:2px solid var(--ring);outline-offset:-3px}.word{display:grid;gap:4px;overflow-wrap:anywhere}.word strong{font-weight:500}.word small,.metadata{font-size:.75rem;color:var(--muted-foreground)}.metadata{white-space:nowrap;text-align:right}.glyph,.status{display:none}.achieve-action{font:inherit;color:var(--foreground);background:var(--background);border:1px solid var(--border)}
</style>
