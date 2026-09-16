import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import DatabaseSettings from './DatabaseSettings.svelte';
import { invoke } from '@tauri-apps/api/core';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const idle = { id: '', running: false, stage: '', report: null };
beforeEach(() => { vi.mocked(invoke).mockReset(); vi.mocked(invoke).mockResolvedValue(idle); });
describe('Local data controls', () => {
  it('starts and cancels one identified check', async () => {
    vi.mocked(invoke).mockImplementation(async (command) => command === 'start_database_check'
      ? { id: 'check-1', running: true, stage: 'structure', report: null } : idle);
    render(DatabaseSettings);
    await fireEvent.click(screen.getByRole('button', { name: 'Check data' }));
    expect(await screen.findByText('Checking application structure…')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Check data' })).toBeDisabled();
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel check' }));
    expect(invoke).toHaveBeenCalledWith('cancel_database_check', { id: 'check-1' });
  });
  it('saves a completed report without exposing issue payloads', async () => {
    vi.mocked(invoke).mockImplementation(async (command) => command === 'get_database_check'
      ? { id: 'check-2', running: false, stage: 'formats', report: { status: 'issues', issues: ['invalid_timestamp'], coverage: [], elapsedMs: 15 } }
      : true);
    render(DatabaseSettings);
    await screen.findByText('Issues found · 15 ms');
    await fireEvent.click(screen.getByRole('button', { name: 'Save diagnostic report' }));
    expect(invoke).toHaveBeenCalledWith('save_database_check_report', { id: 'check-2' });
    await screen.findByText('Diagnostic report saved.');
  });
  it('distinguishes export cancellation from success', async () => {
    vi.mocked(invoke).mockImplementation(async (command) => command === 'export_vocabulary_csv' ? null : idle);
    render(DatabaseSettings);
    await fireEvent.click(screen.getByRole('button', { name: 'Export vocabulary CSV' }));
    await screen.findByText('Export cancelled.');
    vi.mocked(invoke).mockImplementation(async (command) => command === 'export_vocabulary_csv' ? 7 : idle);
    await fireEvent.click(screen.getByRole('button', { name: 'Export vocabulary CSV' }));
    await screen.findByText('Exported 7 vocabulary items.');
  });
  it('shows failures and allows retry', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('private path'));
    render(DatabaseSettings);
    await fireEvent.click(screen.getByRole('button', { name: 'Check data' }));
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('Could not start'));
    expect(screen.queryByText('private path')).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Check data' })).not.toBeDisabled();
  });
});
