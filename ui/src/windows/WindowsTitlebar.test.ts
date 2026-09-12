import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import WindowsTitlebar from './WindowsTitlebar.svelte';

const windowApi = vi.hoisted(() => ({
  minimize: vi.fn().mockResolvedValue(undefined),
  toggleMaximize: vi.fn().mockResolvedValue(undefined),
  close: vi.fn().mockResolvedValue(undefined),
  isMaximized: vi.fn().mockResolvedValue(true),
  onResized: vi.fn().mockResolvedValue(() => {}),
}));
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }));
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => windowApi }));

it('connects window controls, reflects maximized state and keeps controls out of drag regions', async () => {
  const onerror = vi.fn();
  render(WindowsTitlebar, { onsearch: vi.fn(), oncapture: vi.fn(), onerror });
  await fireEvent.click(await screen.findByRole('button', { name: 'Restore window' }));
  await fireEvent.click(screen.getByRole('button', { name: 'Minimize window' }));
  await fireEvent.click(screen.getByRole('button', { name: 'Close window' }));
  expect(windowApi.toggleMaximize).toHaveBeenCalledOnce();
  expect(windowApi.minimize).toHaveBeenCalledOnce();
  expect(windowApi.close).toHaveBeenCalledOnce();
  for (const button of screen.getAllByRole('button')) expect(button).not.toHaveAttribute('data-tauri-drag-region');
  windowApi.minimize.mockRejectedValueOnce(new Error('Unavailable'));
  await fireEvent.click(screen.getByRole('button', { name: 'Minimize window' }));
  await waitFor(() => expect(onerror).toHaveBeenCalledWith(expect.stringContaining('Unavailable')));
});
