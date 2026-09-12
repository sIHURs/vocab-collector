import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import VocabularyLog from './VocabularyLog.svelte';
import { DemoBackend } from '../lib/backend';
import { localDate, logStart, activityLevel, dayLabel } from '../lib/vocabulary-log';

describe('Vocabulary Activity', () => {
  it('reports genuine counts for save, repeat and repeated Undo', async () => {
    const api = new DemoBackend(false);
    const a = await api.capture({ selectedText: 'word', sentence: 'a word' });
    const b = await api.capture({ selectedText: 'word', sentence: 'another word' });
    expect((await api.getVocabularyLog()).days.at(-1)?.count).toBe(2);
    await expect(api.undoCapture(a.encounterId)).rejects.toThrow("Cannot undo");
    await api.undoCapture(b.encounterId);
    await expect(api.undoCapture(b.encounterId)).rejects.toThrow();
    expect((await api.getVocabularyLog()).days.at(-1)?.count).toBe(1);
    expect((await api.getVocabularyLog()).days[0].count).toBeNull();
  });
  it('does not undo anonymous totals after permanent deletion', async () => {
    const api = new DemoBackend(false);
    const capture = await api.capture({ selectedText: 'retained total', sentence: 'A context.' });
    // The demo scheduler never promotes words; arrange a Mastered fixture only.
    (api as unknown as { words: Array<{ item: { status: string } }> }).words[0].item.status = 'mastered';
    await api.achieveWord(capture.wordId);
    await api.deleteAchievedWords([capture.wordId]);
    await expect(api.undoCapture(capture.encounterId)).rejects.toThrow("Cannot undo");
    expect((await api.getVocabularyLog()).days.at(-1)?.count).toBe(1);
  });
  it('uses one tab stop, moves through days and weeks, labels recorded and unknown days', async () => {
    const api = new DemoBackend(false);
    const { container } = render(VocabularyLog, { log: await api.getVocabularyLog(), onretry() {} });
    const cells = [...container.querySelectorAll<HTMLButtonElement>('[data-date]')];
    expect(cells.filter(c => c.tabIndex === 0)).toHaveLength(1);
    const today = screen.getByRole('button', { name: dayLabel({ date: localDate(), count: 0, coverage: 'complete' }) });
    today.focus(); await fireEvent.keyDown(today, { key: 'ArrowUp' });
    await waitFor(() => expect(document.activeElement).toBe(cells.at(-2)));
    await fireEvent.keyDown(document.activeElement!, { key: 'ArrowLeft' });
    await waitFor(() => expect(document.activeElement).toBe(cells.at(-9)));
    expect(cells[0]).toHaveAccessibleName(/No activity records are available for this day/);
    expect(cells.filter(c => c.tabIndex === 0)).toHaveLength(1);
  }, 15000);
  it('distinguishes loading and unavailable from zero', () => {
    render(VocabularyLog, { loading: true, onretry() {} });
    expect(screen.getByRole('status', { name: 'Loading Vocabulary Activity' })).toBeVisible();
    expect(screen.queryByRole('grid')).toBeNull();
  });
  it('shows complete zero days and preserves real data alongside retryable errors', async () => {
    const retry = vi.fn();
    render(VocabularyLog, { log: { startDate:'2026-09-08', endDate:'2026-09-09', days:[
      {date:'2026-09-08',count:0,coverage:'complete'},
      {date:'2026-09-09',count:10,coverage:'complete'},
    ] }, error:'Refresh failed', onretry:retry });
    expect(screen.getByRole('button', {name:'September 8, 2026 - No vocabulary saves recorded.'})).toBeVisible();
    expect(screen.getByRole('button', {name:'September 9, 2026 - You saved vocabulary 10 times.'})).toHaveAttribute('data-level','1');
    await fireEvent.click(screen.getByRole('button', {name:'Retry Vocabulary Activity'}));
    expect(retry).toHaveBeenCalledOnce();
  });
  it('handles leap-day ranges and all five intensity levels', () => {
    expect(logStart('2024-02-29')).toBe('2023-03-01');
    expect([0,1,24,25,49,50,99,100,500].map(activityLevel)).toEqual([0,1,1,2,2,3,3,4,4]);
  });
});
