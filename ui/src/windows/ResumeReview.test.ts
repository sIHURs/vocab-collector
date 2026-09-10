import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import { DemoBackend } from '../lib/backend';
import WindowsApp from './WindowsApp.svelte';

async function pauseAfterOneReview(api: DemoBackend) {
  render(WindowsApp, { api });
  await fireEvent.click(await screen.findByRole('button', { name: /Start review/ }));
  await fireEvent.click(screen.getByRole('button', { name: 'Show answer' }));
  await fireEvent.click(screen.getByRole('button', { name: 'Remembered' }));
  await screen.findByRole('button', { name: /Next/ });
  await fireEvent.click(screen.getByRole('button', { name: 'Close review' }));
  await screen.findByRole('button', { name: 'Resume review' });
}

it('resumes from current status and completes with earlier Review results intact', async () => {
  const api = new DemoBackend();
  await pauseAfterOneReview(api);
  for (const item of (await api.getToday()).reviewQueue) await api.changeLearningStatus(item.wordId, 'mastered');
  await fireEvent.click(screen.getByRole('button', { name: 'Resume review' }));
  expect(await screen.findByText('Review complete')).toBeVisible();
  expect(screen.getByText(/1 reviewed/)).toBeVisible();
  expect(screen.queryByRole('button', { name: 'Show answer' })).toBeNull();
});

it('includes a newly due Learning item and excludes Paused and Mastered within the existing limit', async () => {
  const api = new DemoBackend();
  const extra = await api.capture({ selectedText: 'renewed', sentence: 'A renewed context.', translation: 'erneuert' });
  await api.changeLearningStatus(extra.wordId, 'mastered');
  await pauseAfterOneReview(api);
  const remaining = (await api.getToday()).reviewQueue;
  await api.changeLearningStatus(remaining[0].wordId, 'paused');
  await api.changeLearningStatus(remaining[1].wordId, 'mastered');
  await api.changeLearningStatus(extra.wordId, 'learning');
  await api.updateSettings({ ...(await api.getSettings()), dailyLimit: 1 });
  await fireEvent.click(screen.getByRole('button', { name: 'Resume review' }));
  expect(await screen.findByRole('heading', { name: 'renewed' })).toBeVisible();
  expect(screen.getByText('1 of 1')).toBeVisible();
  await fireEvent.click(screen.getByRole('button', { name: 'Show answer' }));
  await fireEvent.click(screen.getByRole('button', { name: 'Remembered' }));
  await fireEvent.click(await screen.findByRole('button', { name: 'Next' }));
  expect(await screen.findByText('2 reviewed')).toBeVisible();
});

it('blocks stale Resume on refresh failure and retries without losing completed results', async () => {
  const api = new DemoBackend();
  await pauseAfterOneReview(api);
  const getToday = api.getToday.bind(api);
  api.getToday = vi.fn().mockRejectedValueOnce(new Error('Refresh unavailable')).mockImplementation(getToday);
  await fireEvent.click(screen.getByRole('button', { name: 'Resume review' }));
  expect(await screen.findByRole('button', { name: 'Retry Review refresh' })).toBeVisible();
  expect(screen.queryByRole('button', { name: 'Show answer' })).toBeNull();
  for (const item of (await api.getToday()).reviewQueue) await api.changeLearningStatus(item.wordId, 'paused');
  await fireEvent.click(screen.getByRole('button', { name: 'Retry Review refresh' }));
  await waitFor(() => expect(screen.getByRole('button', { name: 'Resume review' })).toBeEnabled());
  await fireEvent.click(screen.getByRole('button', { name: 'Resume review' }));
  expect(await screen.findByText('1 reviewed')).toBeVisible();
});

it('uses persisted Undo results when resuming', async () => {
  const api = new DemoBackend();
  await pauseAfterOneReview(api);
  const item = (await api.getToday()).reviewQueue[0];
  const token = await api.changeLearningStatus(item.wordId, 'mastered');
  await api.undoLearningStatus(token!);
  await fireEvent.click(screen.getByRole('button', { name: 'Resume review' }));
  expect(await screen.findByRole('heading', { name: item.displayForm })).toBeVisible();
  expect(screen.getByRole('button', { name: 'Show answer' })).toBeVisible();
});

it('fills the remaining plan even when an already reviewed item becomes due again', async () => {
  vi.useFakeTimers({ toFake: ['Date'] });
  try {
    vi.setSystemTime(new Date('2026-09-10T12:00:00Z'));
    const api = new DemoBackend();
    await pauseAfterOneReview(api);
    const remaining = (await api.getToday()).reviewQueue;
    vi.setSystemTime(new Date('2026-09-11T12:00:00Z'));
    await api.submitReview(remaining[0].wordId, 'remembered');
    await api.changeLearningStatus(remaining[1].wordId, 'paused');
    await api.updateSettings({ ...(await api.getSettings()), dailyLimit: 1 });
    vi.setSystemTime(new Date('2026-09-15T12:00:00Z'));
    await fireEvent.click(screen.getByRole('button', { name: 'Resume review' }));
    expect(await screen.findByRole('heading', { name: remaining[0].displayForm })).toBeVisible();
    expect(screen.getByText('1 of 1')).toBeVisible();
  } finally { vi.useRealTimers(); }
});

it('keeps the submitted card paired with its result when its status is edited from Review contexts', async () => {
  const api = new DemoBackend();
  const submit = api.submitReview.bind(api);
  api.submitReview = async (...args) => ({ ...(await submit(...args)), repeatedForgetting: true });
  render(WindowsApp, { api });
  await fireEvent.click(await screen.findByRole('button', { name: /Start review/ }));
  await fireEvent.click(screen.getByRole('button', { name: 'Show answer' }));
  await fireEvent.click(screen.getByRole('button', { name: 'Forgot' }));
  await fireEvent.click(await screen.findByRole('button', { name: 'Review contexts' }));
  await fireEvent.change(await screen.findByRole('combobox', { name: 'Learning status' }), { target: { value: 'paused' } });
  await waitFor(() => expect(screen.getByRole('combobox', { name: 'Learning status' })).toBeEnabled());
  await fireEvent.click(screen.getByRole('button', { name: 'Close vocabulary detail' }));
  expect(screen.getByRole('heading', { name: 'serendipity' })).toBeVisible();
  await fireEvent.click(screen.getByRole('button', { name: 'Next' }));
  expect(await screen.findByRole('heading', { name: 'nuance' })).toBeVisible();
  expect(screen.getByRole('button', { name: 'Show answer' })).toBeVisible();
  expect(screen.queryByText(/Next review/)).toBeNull();
});

