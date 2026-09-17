import { render, screen, within } from '@testing-library/svelte';
import { expect, it } from 'vitest';
import VocabularyDetail from './VocabularyDetail.svelte';
it('shows saved target languages and the translation saved with an Encounter', () => {
  render(VocabularyDetail, { detail: { item: { id:'w', displayForm:'robust', status:'learning', encounterCount:1, lastSeenAt:'2026-09-10' }, lemma:'robust', translations:[{targetLanguage:'de',text:'kräftig',savedAt:'2026-09-10'}], encounters:[{id:'e',wordId:'w',selectedText:'robust',sentence:'A robust design.',captureOrigin:'manual',capturedAt:'2026-09-10',updatedAt:'2026-09-10',savedTranslation:{targetLanguage:'de',text:'beständig',savedAt:'2026-09-09'}}] }, trigger:null, onclose:()=>{}, onachieve:()=>{} });
  expect(screen.getByText('kräftig')).toBeVisible();
  expect(screen.getByText('beständig')).toBeVisible();
});

it('shows a translation once with a friendly language and dates every Encounter', async () => {
  const capturedAt = ['2026-09-10T14:30:00Z', '2026-09-11T08:15:00Z'];
  const { rerender } = render(VocabularyDetail, { detail: {
    item: { id: 'w', displayForm: 'delta force', translation: '三角洲游戏', translationLanguage: 'zh-hans', status: 'learning', encounterCount: 2, lastSeenAt: capturedAt[1] },
    lemma: 'delta force',
    translations: [{ targetLanguage: 'zh-hans', text: '三角洲游戏', savedAt: capturedAt[0] }, { targetLanguage: 'de', text: 'Delta-Einheit', savedAt: capturedAt[1] }],
    encounters: capturedAt.map((date, index) => ({ id: String(index), wordId: 'w', selectedText: 'delta force', sentence: 'for testing', captureOrigin: 'manual' as const, capturedAt: date, updatedAt: date,
      ...(index === 1 ? { savedTranslation: { targetLanguage: 'zh-hans', text: '历史翻译', savedAt: date } } : {}) })),
  }, trigger: null, onclose: () => {}, onachieve: () => {} });
  expect(screen.getAllByText('三角洲游戏')).toHaveLength(1);
  expect(screen.queryByText('zh-hans')).not.toBeInTheDocument();
  expect(screen.getAllByText(/简体中文/).length).toBeGreaterThan(0);
  expect(screen.getByText('Delta-Einheit')).toBeVisible();
  const history = screen.getByRole('region', { name: 'Capture history' });
  expect(within(history).getByText('历史翻译')).toBeVisible();
  const dates = history.querySelectorAll('time');
  expect(dates).toHaveLength(2);
  dates.forEach((date, index) => {
    expect(date).toHaveAttribute('datetime', capturedAt[index]);
    expect(date).toHaveTextContent(new Intl.DateTimeFormat('en', { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(capturedAt[index])));
  });
  await rerender({ targetLanguage: 'zh-Hans' });
  expect(screen.queryByText(/简体中文/)).not.toBeInTheDocument();
  expect(screen.getByText(/German/)).toBeVisible();
  expect(screen.getByText('历史翻译')).toBeVisible();
  await rerender({ targetLanguage: 'de' });
  expect(screen.getAllByText(/简体中文/)).toHaveLength(2);
  expect(screen.queryByText(/German/)).not.toBeInTheDocument();
});
