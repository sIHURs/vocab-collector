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
  await fireEvent.change(screen.getByRole('combobox', { name: 'Learning status' }), { target: { value: 'mastered' } });
  await waitFor(() => expect(screen.getByRole('combobox', { name: 'Learning status' })).toHaveValue('mastered'));
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
  await fireEvent.change(await screen.findByRole('combobox', { name: 'Learning status' }), { target: { value: 'paused' } });
  expect(screen.getByRole('combobox', { name: 'Learning status' })).toBeDisabled();
  reject(new Error('Could not save status'));
  expect(await screen.findByRole('alert')).toHaveTextContent('Could not save status');
  expect(screen.getByRole('combobox', { name: 'Learning status' })).toHaveValue('learning');
  await fireEvent.click(screen.getByRole('button', { name: 'Retry status update' }));
  await waitFor(() => expect(screen.getByRole('combobox', { name: 'Learning status' })).toHaveValue('paused'));
  expect(screen.queryByRole('alert')).toBeNull();
});

it('reports a saved status separately from a failed refresh', async () => {
  const api = new DemoBackend();
  const getToday = api.getToday.bind(api);
  render(WindowsApp, { api });
  await openVocabulary();
  api.getToday = vi.fn().mockRejectedValueOnce(new Error('Offline')).mockImplementation(getToday);
  await fireEvent.change(await screen.findByRole('combobox', { name: 'Learning status' }), { target: { value: 'mastered' } });
  expect(await screen.findByText('Status saved. Could not refresh vocabulary. Retry to refresh.')).toBeVisible();
  expect(screen.getByRole('combobox', { name: 'Learning status' })).toHaveValue('mastered');
  await fireEvent.click(screen.getByRole('button', { name: 'Retry status update' }));
  await waitFor(() => expect(screen.queryByText('Status saved. Could not refresh vocabulary. Retry to refresh.')).toBeNull());
});

it('undoes a status change through the existing bottom-right notification', async () => {
  const api = new DemoBackend();
  render(WindowsApp, { api });
  await openVocabulary();
  await fireEvent.change(await screen.findByRole('combobox', { name: 'Learning status' }), { target: { value: 'mastered' } });
  const notification = await screen.findByRole('dialog', { name: 'Learning status changed' });
  expect(notification.closest('[data-sonner-toaster]')).toHaveAttribute('data-y-position', 'bottom');
  expect(notification.closest('[data-sonner-toaster]')).toHaveAttribute('data-x-position', 'right');
  await waitFor(() => expect(within(notification).getByRole('button', { name: 'Undo' })).toBeEnabled());
  await fireEvent.click(within(notification).getByRole('button', { name: 'Undo' }));
  await waitFor(() => expect(screen.getByRole('combobox', { name: 'Learning status' })).toHaveValue('learning'));
  expect((await api.getToday()).totalDueCount).toBe(3);
});

it('shows a failed Undo in its notification without falsely restoring the status', async () => {
  const api = new DemoBackend();
  let reject!: (reason: Error) => void;
  api.undoLearningStatus = vi.fn().mockImplementationOnce(() => new Promise<string>((_, fail) => { reject = fail; }));
  render(WindowsApp, { api });
  await openVocabulary();
  await fireEvent.change(await screen.findByRole('combobox', { name: 'Learning status' }), { target: { value: 'paused' } });
  const notification = await screen.findByRole('dialog', { name: 'Learning status changed' });
  await waitFor(() => expect(within(notification).getByRole('button', { name: 'Undo' })).toBeEnabled());
  await fireEvent.click(within(notification).getByRole('button', { name: 'Undo' }));
  expect(within(notification).getByRole('button', { name: 'Undo' })).toBeDisabled();
  reject(new Error('This Vocabulary Item changed after the status update'));
  expect(await within(notification).findByRole('alert')).toHaveTextContent('This Vocabulary Item changed');
  expect(screen.getByRole('combobox', { name: 'Learning status' })).toHaveValue('paused');
});

