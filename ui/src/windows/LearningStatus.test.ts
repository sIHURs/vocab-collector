import { fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import { DemoBackend } from '../lib/backend';
import WindowsApp from './WindowsApp.svelte';

async function openVocabulary() {
  await fireEvent.click(await screen.findByRole('button', { name: 'Vocabulary' }));
  await fireEvent.click(await screen.findByRole('button', { name: /serendipity, Saved/ }));
}

it('saves the selected Learning Status and moves the item without leaving its detail', async () => {
  const api = new DemoBackend();
  render(WindowsApp, { api });
  await openVocabulary();
  await fireEvent.keyDown(await screen.findByLabelText('Learning status'), { key: 'ArrowDown' });
  await fireEvent.pointerUp(await screen.findByRole('option', { name: 'Mastered' }));
  await waitFor(() => expect(screen.getByLabelText('Learning status')).toHaveTextContent('Mastered'));
  expect(screen.getByRole('heading', { name: 'serendipity' })).toBeVisible();
  await fireEvent.click(screen.getByRole('button', { name: 'Close vocabulary detail' }));
  expect(screen.queryByRole('button', { name: /serendipity, Saved/ })).toBeNull();
  await fireEvent.click(screen.getByRole('tab', { name: 'Mastered' }));
  expect(await screen.findByRole('button', { name: /serendipity, Saved/ })).toBeVisible();
});

it('retains the saved state on write failure and allows retry while preventing repeat submission', async () => {
  const api = new DemoBackend();
  const change = api.changeLearningStatus.bind(api);
  let reject!: (reason: Error) => void;
  api.changeLearningStatus = vi.fn().mockImplementationOnce(() => new Promise<void>((_, fail) => { reject = fail; })).mockImplementation(change);
  render(WindowsApp, { api });
  await openVocabulary();
  await fireEvent.keyDown(await screen.findByLabelText('Learning status'), { key: 'ArrowDown' });
  await fireEvent.pointerUp(await screen.findByRole('option', { name: 'Paused' }));
  expect(screen.getByLabelText('Learning status')).toBeDisabled();
  reject(new Error('Could not save status'));
  expect(await screen.findByRole('alert')).toHaveTextContent('Could not save status');
  expect(screen.getByLabelText('Learning status')).toHaveTextContent('Learning');
  await fireEvent.click(screen.getByRole('button', { name: 'Retry status update' }));
  await waitFor(() => expect(screen.getByLabelText('Learning status')).toHaveTextContent('Paused'));
  expect(screen.queryByRole('alert')).toBeNull();
});

it('reports a saved status separately from a failed refresh', async () => {
  const api = new DemoBackend();
  const getToday = api.getToday.bind(api);
  render(WindowsApp, { api });
  await openVocabulary();
  api.getToday = vi.fn().mockRejectedValueOnce(new Error('Offline')).mockImplementation(getToday);
  await fireEvent.keyDown(await screen.findByLabelText('Learning status'), { key: 'ArrowDown' });
  await fireEvent.pointerUp(await screen.findByRole('option', { name: 'Mastered' }));
  expect(await screen.findByText('Status saved. Could not refresh vocabulary. Retry to refresh.')).toBeVisible();
  expect(screen.getByLabelText('Learning status')).toHaveTextContent('Mastered');
  await fireEvent.click(screen.getByRole('button', { name: 'Retry status update' }));
  await waitFor(() => expect(screen.queryByText('Status saved. Could not refresh vocabulary. Retry to refresh.')).toBeNull());
});

it('undoes a status change through the existing bottom-right notification', async () => {
  const api = new DemoBackend();
  render(WindowsApp, { api });
  await openVocabulary();
  await fireEvent.keyDown(await screen.findByLabelText('Learning status'), { key: 'ArrowDown' });
  await fireEvent.pointerUp(await screen.findByRole('option', { name: 'Mastered' }));
  const notification = await screen.findByRole('dialog', { name: 'Learning status changed' });
  expect(notification.closest('[data-sonner-toaster]')).toHaveAttribute('data-y-position', 'bottom');
  expect(notification.closest('[data-sonner-toaster]')).toHaveAttribute('data-x-position', 'right');
  await waitFor(() => expect(within(notification).getByRole('button', { name: 'Undo' })).toBeEnabled());
  await fireEvent.click(within(notification).getByRole('button', { name: 'Undo' }));
  await waitFor(() => expect(screen.getByLabelText('Learning status')).toHaveTextContent('Learning'));
  expect((await api.getToday()).totalDueCount).toBe(3);
});

it('shows a failed Undo in its notification without falsely restoring the status', async () => {
  const api = new DemoBackend();
  let reject!: (reason: Error) => void;
  api.undoLearningStatus = vi.fn().mockImplementationOnce(() => new Promise<string>((_, fail) => { reject = fail; }));
  render(WindowsApp, { api });
  await openVocabulary();
  await fireEvent.keyDown(await screen.findByLabelText('Learning status'), { key: 'ArrowDown' });
  await fireEvent.pointerUp(await screen.findByRole('option', { name: 'Paused' }));
  const notification = await screen.findByRole('dialog', { name: 'Learning status changed' });
  await waitFor(() => expect(within(notification).getByRole('button', { name: 'Undo' })).toBeEnabled());
  await fireEvent.click(within(notification).getByRole('button', { name: 'Undo' }));
  expect(within(notification).getByRole('button', { name: 'Undo' })).toBeDisabled();
  reject(new Error('This Vocabulary Item changed after the status update'));
  expect(await within(notification).findByRole('alert')).toHaveTextContent('This Vocabulary Item changed');
  expect(screen.getByLabelText('Learning status')).toHaveTextContent('Paused');
});


it('automatically dismisses Unachieve feedback after two seconds without undoing the change', async () => {
  const api = new DemoBackend();
  let achieved = [{ id: 'toast-word', lemma: 'word', displayForm: 'word', encounterCount: 1, achievedAt: '2026-09-01T00:00:00Z', deleteAfter: '2026-10-01T00:00:00Z', remainingDays: 20, urgency: 'normal' as const }];
  api.listAchievedWords = async () => achieved;
  api.unachieveWords = vi.fn(async () => { achieved = []; return 1; });
  render(WindowsApp, { api });
  await fireEvent.click(await screen.findByRole('button', { name: 'Vocabulary' }));
  await fireEvent.click(await screen.findByRole('tab', { name: 'Achieved (1)' }));
  await fireEvent.click(screen.getByLabelText('Select word'));
  await fireEvent.click(screen.getByRole('button', { name: 'Unachieve' }));
  await screen.findByRole('dialog', { name: 'Vocabulary Unachieved' });
  await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Vocabulary Unachieved' })).not.toBeInTheDocument(), { timeout: 3500 });
  expect(api.unachieveWords).toHaveBeenCalledTimes(1);
  await fireEvent.click(screen.getByRole('button', { name: 'Today' }));
  expect(screen.queryByRole('dialog', { name: 'Vocabulary Unachieved' })).not.toBeInTheDocument();
});
