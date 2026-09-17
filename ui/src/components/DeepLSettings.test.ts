import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import DeepLSettings from './DeepLSettings.svelte';
import { toast } from 'svelte-sonner';
vi.mock('svelte-sonner', () => ({ toast: { custom: vi.fn(), dismiss: vi.fn() } }));

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const empty = { supported: true, configured: false, storageError: false };
const saved = { ...empty, configured: true };
beforeEach(() => { vi.mocked(invoke).mockReset(); vi.mocked(invoke).mockResolvedValue(empty); });

describe('Personal DeepL key', () => {
  it('blocks duplicate saves during validation and asks for a new key on rejection', async () => {
    vi.mocked(invoke).mockResolvedValue(saved);
    render(DeepLSettings, { toasterId: 'settings-host' });
    const input = screen.getByLabelText('DeepL API key');
    await waitFor(() => expect(input).toBeEnabled());
    await fireEvent.input(input, { target: { value: 'invalid-secret' } });
    let reject!: (reason: string) => void;
    vi.mocked(invoke).mockImplementation(() => new Promise((_, fail) => { reject = fail; }));
    await fireEvent.click(screen.getByRole('button', { name: 'Replace key' }));
    expect(screen.getByRole('button', { name: 'Verifying…' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Remove key' })).toBeDisabled();
    expect(input).toBeDisabled();
    reject('invalid_key');
    expect(await screen.findByRole('alert')).toHaveTextContent('Please re-enter a valid key');
    expect(toast.custom).toHaveBeenLastCalledWith(expect.anything(), expect.objectContaining({
      toasterId: 'settings-host', duration: 6000, closeButton: false,
      componentProps: expect.objectContaining({ tone: 'error', description: expect.stringContaining('Please re-enter a valid key') }),
    }));
    await waitFor(() => expect(input).toHaveFocus());
    expect(screen.getByText('A DeepL key is saved on this device.')).toBeVisible();
    expect(input).toHaveValue('invalid-secret');
  });

  it.each(['validation_network', 'validation_timeout', 'validation_unavailable'])('shows retry feedback for %s without calling the key invalid', async (reason) => {
    render(DeepLSettings);
    const input = screen.getByLabelText('DeepL API key');
    await waitFor(() => expect(input).toBeEnabled());
    await fireEvent.input(input, { target: { value: 'new-key:fx' } });
    vi.mocked(invoke).mockRejectedValue(reason);
    await fireEvent.click(screen.getByRole('button', { name: 'Save key' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(/try again/i);
    expect(screen.getByRole('alert')).not.toHaveTextContent('re-enter');
    expect(screen.getByText('No personal DeepL key is saved.')).toBeVisible();
  });

  it('masks input, saves a trimmed key separately, and clears it after saving', async () => {
    render(DeepLSettings);
    const input = screen.getByLabelText('DeepL API key');
    await waitFor(() => expect(input).toBeEnabled());
    expect(input).toHaveAttribute('type', 'password');
    expect(screen.getByRole('button', { name: 'Save key' })).toBeDisabled();
    await fireEvent.input(input, { target: { value: ' test-secret:fx ' } });
    vi.mocked(invoke).mockResolvedValue(saved);
    await fireEvent.click(screen.getByRole('button', { name: 'Save key' }));
    await waitFor(() => expect(input).toHaveValue(''));
    expect(invoke).toHaveBeenLastCalledWith('save_deepl_key', { key: 'test-secret:fx' });
    expect(screen.getByRole('button', { name: 'Replace key' })).toBeDisabled();
    expect(screen.getByText(/Key verified and saved/)).toBeVisible();
  });

  it('retains saved status on failed removal without displaying raw errors', async () => {
    vi.mocked(invoke).mockResolvedValue(saved);
    render(DeepLSettings);
    const remove = screen.getByRole('button', { name: 'Remove key' });
    await waitFor(() => expect(remove).toBeEnabled());
    vi.mocked(invoke).mockRejectedValue('sensitive-error-payload');
    await fireEvent.click(remove);
    expect(await screen.findByRole('alert')).toHaveTextContent('Could not remove the key');
    expect(screen.queryByText(/sensitive-error-payload/)).not.toBeInTheDocument();
    expect(screen.getByText('A DeepL key is saved on this device.')).toBeVisible();
    vi.mocked(invoke).mockResolvedValue(empty);
    await fireEvent.click(remove);
    await waitFor(() => expect(remove).toBeDisabled());
    expect(invoke).toHaveBeenLastCalledWith('remove_deepl_key');
  });

  it('keeps replacement input available for retry when storage fails', async () => {
    render(DeepLSettings);
    const input = screen.getByLabelText('DeepL API key');
    await waitFor(() => expect(input).toBeEnabled());
    await fireEvent.input(input, { target: { value: 'replacement:fx' } });
    vi.mocked(invoke).mockRejectedValue('secret');
    await fireEvent.click(screen.getByRole('button', { name: 'Save key' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Could not save the key');
    expect(input).toHaveValue('replacement:fx');
  });

  it('disables key entry in a browser preview', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('no native bridge'));
    render(DeepLSettings);
    expect(await screen.findByText('Open the Windows desktop app to manage your DeepL key.')).toBeVisible();
    expect(screen.getByLabelText('DeepL API key')).toBeDisabled();
  });
});
