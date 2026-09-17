import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import SettingsForm from './SettingsForm.svelte';

function setup() {
  const oncommit = vi.fn();
  render(SettingsForm, { settingsDraft: {
    sourceLanguage: 'en', targetLanguage: 'de', selectionCaptureShortcut: 'Alt+Shift+V',
    regionOcrCaptureShortcut: 'Shift+Ctrl+O', reviewTime: '09:00', dailyLimit: 10,
    recentCapturesLimit: 10, launchAtLogin: false, appearance: 'system', reducedMotion: false,
  }, oncommit, onretry: vi.fn() });
  return { oncommit, input: screen.getByLabelText('Selection Capture shortcut') };
}

describe('Settings shortcut recording', () => {
  it('saves hour and minute choices in 24-hour format without losing minute precision', async () => {
    const { oncommit } = setup();
    await fireEvent.keyDown(screen.getByLabelText('Review hour'), { key: 'ArrowDown' });
    await fireEvent.pointerUp(await screen.findByRole('option', { name: '23' }));
    expect(oncommit).toHaveBeenLastCalledWith({ reviewTime: '23:00' });
    await fireEvent.keyDown(screen.getByLabelText('Review minute'), { key: 'ArrowDown' });
    await fireEvent.pointerUp(await screen.findByRole('option', { name: '17' }));
    expect(oncommit).toHaveBeenLastCalledWith({ reviewTime: '23:17' });
  });
  it('records physical keys with modifiers and saves immediately', async () => {
    const { input, oncommit } = setup();
    await fireEvent.keyDown(input, { key: '!', code: 'Digit1', ctrlKey: true, shiftKey: true });
    expect(oncommit).toHaveBeenCalledWith({ selectionCaptureShortcut: 'Control+Shift+1' });
    expect(input).toHaveValue('Control+Shift+1');
    await fireEvent.keyDown(input, { key: '!', code: 'Digit1', ctrlKey: true, shiftKey: true, repeat: true });
    expect(oncommit).toHaveBeenCalledTimes(1);
  });

  it('rejects equivalent shortcuts regardless of modifier order and aliases', async () => {
    const { input, oncommit } = setup();
    await fireEvent.keyDown(input, { key: 'o', code: 'KeyO', ctrlKey: true, shiftKey: true });
    expect(screen.getByText('Capture shortcuts must be different.')).toBeVisible();
    expect(input).toHaveValue('Alt+Shift+V');
    expect(oncommit).not.toHaveBeenCalled();
  });

  it('ignores modifiers, cancels, permits Tab navigation, and clears with Delete', async () => {
    const { input, oncommit } = setup();
    for (const key of ['Control', 'v', 'Escape', 'Tab']) await fireEvent.keyDown(input, { key });
    expect(oncommit).not.toHaveBeenCalled();
    expect(input).toHaveValue('Alt+Shift+V');
    await fireEvent.keyDown(input, { key: 'Delete' });
    expect(oncommit).toHaveBeenCalledWith({ selectionCaptureShortcut: '' });
    expect(input).toHaveValue('');
  });
});

