import type { WordListItem } from './types';

export type VocabularySortKey = 'displayForm' | 'encounterCount' | 'lastSeenAt';
export type SortDirection = 'ascending' | 'descending';

export function sortVocabulary(words: WordListItem[], key: VocabularySortKey, direction: SortDirection): WordListItem[] {
  const multiplier = direction === 'ascending' ? 1 : -1;
  return [...words].sort((a, b) => {
    const primary = key === 'displayForm'
      ? a.displayForm.localeCompare(b.displayForm, undefined, { sensitivity: 'base' })
      : key === 'encounterCount'
        ? a.encounterCount - b.encounterCount
        : Date.parse(a.lastSeenAt) - Date.parse(b.lastSeenAt);
    return primary * multiplier
      || Date.parse(b.lastSeenAt) - Date.parse(a.lastSeenAt)
      || a.id.localeCompare(b.id);
  });
}
