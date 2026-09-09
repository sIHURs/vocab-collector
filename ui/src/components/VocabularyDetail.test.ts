import { render, screen } from '@testing-library/svelte';
import { expect, it } from 'vitest';
import VocabularyDetail from './VocabularyDetail.svelte';
it('shows saved target languages and the translation saved with an Encounter', () => {
  render(VocabularyDetail, { detail: { item: { id:'w', displayForm:'robust', status:'learning', encounterCount:1, lastSeenAt:'2026-09-10' }, lemma:'robust', translations:[{targetLanguage:'de',text:'kräftig',savedAt:'2026-09-10'}], encounters:[{id:'e',wordId:'w',selectedText:'robust',sentence:'A robust design.',captureOrigin:'manual',capturedAt:'2026-09-10',updatedAt:'2026-09-10',savedTranslation:{targetLanguage:'de',text:'beständig',savedAt:'2026-09-09'}}] }, trigger:null, onclose:()=>{}, onachieve:()=>{} });
  expect(screen.getByText('kräftig')).toBeVisible();
  expect(screen.getByText('beständig')).toBeVisible();
});
