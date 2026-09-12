<script lang="ts">
  import type { WordListItem } from '../lib/types';
  import TranslationText from './TranslationText.svelte';
  import type { VocabularySortKey, SortDirection } from '../lib/vocabulary-sort';
  import * as Table from '$lib/components/ui/table';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  export let words: WordListItem[];
  export let targetLanguage: string | undefined = undefined;
  export let sortKey: VocabularySortKey = 'lastSeenAt';
  export let sortDirection: SortDirection = 'descending';
  export let onSort: (key: VocabularySortKey) => void;
  export let onSelect: (id: string, trigger: HTMLButtonElement) => void;
  export let onAchieve: (word: WordListItem) => void;
  const date = (value: string) => new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' }).format(new Date(value));
</script>

<Table.Root aria-label="Vocabulary">
  <Table.Header><Table.Row>
    {#each [{ key: 'displayForm', label: 'Word' }, { key: null, label: 'Translation' }, { key: null, label: 'Status' }, { key: 'encounterCount', label: 'Times saved' }, { key: 'lastSeenAt', label: 'Last saved' }] as column}
      <Table.Head aria-sort={column.key === sortKey ? sortDirection : undefined}>
        {#if column.key}
          <Button variant="ghost" size="sm" onclick={() => onSort(column.key as VocabularySortKey)}>
            {column.label}{#if column.key === sortKey}<span aria-hidden="true">{sortDirection === 'ascending' ? '↑' : '↓'}</span>{/if}
          </Button>
        {:else}{column.label}{/if}
      </Table.Head>
    {/each}
    {#if words.some(word => word.status === 'mastered')}<Table.Head><span class="sr-only">Actions</span></Table.Head>{/if}
  </Table.Row></Table.Header>
  <Table.Body>
    {#each words as word (word.id)}
      <Table.Row onclick={(event) => { if (!(event.target as HTMLElement).closest('button')) { const trigger = event.currentTarget.querySelector('button'); if (trigger) onSelect(word.id, trigger); } }}>
        <Table.Cell><button class="word-link" aria-label={`${word.displayForm}, Saved ${word.encounterCount} time${word.encounterCount === 1 ? '' : 's'}`} onclick={(event) => onSelect(word.id, event.currentTarget)}>{word.displayForm}</button></Table.Cell>
        <Table.Cell><span class="translation"><TranslationText translation={word.translation} language={word.translationLanguage} {targetLanguage} /></span></Table.Cell>
        <Table.Cell><Badge variant="secondary"><span class="status">{word.status}</span></Badge></Table.Cell>
        <Table.Cell>{word.encounterCount}</Table.Cell>
        <Table.Cell><time datetime={word.lastSeenAt}>{date(word.lastSeenAt)}</time></Table.Cell>
        {#if word.status === 'mastered'}<Table.Cell><Button variant="destructiveOutline" aria-label={`Achieve ${word.displayForm}`} onclick={() => onAchieve(word)}>Achieve</Button></Table.Cell>{/if}
      </Table.Row>
    {/each}
  </Table.Body>
</Table.Root>

<style>
  .word-link { border:0; background:transparent; text-align:left; font:inherit; font-weight:500; color:var(--foreground); padding:6px 0; cursor:pointer; overflow-wrap:anywhere; white-space:normal; min-width:100px; max-width:200px; }
  .translation { display:block; min-width:100px; max-width:230px; white-space:normal; overflow-wrap:anywhere; color:var(--muted-foreground); }
  .status { text-transform:capitalize; }
  time { color:var(--muted-foreground); font-size:0.75rem; }
</style>
