<script lang="ts">
  import type { WordListItem } from '../lib/types';
  import * as Table from '$lib/components/ui/table';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  export let words: WordListItem[];
  export let onSelect: (id: string, trigger: HTMLButtonElement) => void;
  export let onAchieve: (word: WordListItem) => void;
  const date = (value: string) => new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' }).format(new Date(value));
</script>

<Table.Root aria-label="Vocabulary">
  <Table.Header><Table.Row>
    <Table.Head>Word</Table.Head><Table.Head>Translation</Table.Head><Table.Head>Status</Table.Head><Table.Head>Encounters</Table.Head><Table.Head>Last Seen</Table.Head>
    {#if words.some(word => word.status === 'mastered')}<Table.Head><span class="sr-only">Actions</span></Table.Head>{/if}
  </Table.Row></Table.Header>
  <Table.Body>
    {#each words as word (word.id)}
      <Table.Row onclick={(event) => { if (!(event.target as HTMLElement).closest('button')) { const trigger = event.currentTarget.querySelector('button'); if (trigger) onSelect(word.id, trigger); } }}>
        <Table.Cell><button class="word-link" aria-label={`${word.displayForm}, ${word.encounterCount} encounter${word.encounterCount === 1 ? '' : 's'}`} onclick={(event) => onSelect(word.id, event.currentTarget)}>{word.displayForm}</button></Table.Cell>
        <Table.Cell><span class="translation">{word.translation ?? 'No translation'}</span></Table.Cell>
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
