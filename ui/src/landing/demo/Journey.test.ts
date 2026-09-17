import { fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import Journey from './Journey.svelte';
import { LandingSession, preparedCapture } from './session';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Any accidental native call or provider request fails the complete UI journey.
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(() => { throw new Error('Native invocation in browser'); }), isTauri: () => false }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn(() => { throw new Error('Native listener in browser'); }) }));

it('connects shared Capture, live Vocabulary, detail and the two-word Review', async () => {
  const session = new LandingSession();
  const fetchSpy = vi.spyOn(globalThis, 'fetch').mockRejectedValue(new Error('Provider request in browser'));
  try {
    render(Journey, { session });
    await screen.findByText('4 items');
    expect(screen.queryByRole('button', { name: 'Settings' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Insights' })).not.toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'ephemeral' }));
    const capture = await screen.findByRole('dialog', { name: 'Capture a word' });
    await waitFor(() => expect(within(capture).getByRole('button', { name: 'Save capture' })).toBeEnabled());
    expect(document.body).not.toHaveClass('windows-capture-document');
    await fireEvent.click(within(capture).getByRole('button', { name: 'Save capture' }));
    await waitFor(async () => expect(await session.listWords()).toHaveLength(5));
    await fireEvent.click(within(capture).getByRole('button', { name: 'Cancel capture' }));
    await screen.findByText('5 items');
    const word = await screen.findByRole('button', { name: 'ephemeral, Saved 1 time' });
    await fireEvent.click(word);
    const detail = await screen.findByRole('dialog', { name: 'ephemeral' });
    expect(within(detail).getByText(preparedCapture.sentence)).toBeInTheDocument();
    expect(within(detail).getAllByText(preparedCapture.translation).length).toBeGreaterThan(0);
    expect(within(detail).getByText(/A quiet garden/)).toBeInTheDocument();
    await fireEvent.click(within(detail).getByRole('button', { name: 'Close vocabulary detail' }));
    await waitFor(() => expect(word).toHaveFocus());
    await fireEvent.click(screen.getByRole('button', { name: 'Review' }));
    await fireEvent.click(await screen.findByRole('button', { name: 'Start review' }));
    await screen.findByRole('heading', { name: 'serendipitous' });
    await fireEvent.click(screen.getByRole('button', { name: 'Show answer' }));
    await fireEvent.click(screen.getByRole('button', { name: 'Remembered' }));
    await fireEvent.click(await screen.findByRole('button', { name: 'Next' }));
    await screen.findByRole('heading', { name: 'resilient' });
    await fireEvent.click(screen.getByRole('button', { name: 'Show answer' }));
    await fireEvent.click(screen.getByRole('button', { name: 'Forgot' }));
    await fireEvent.click(await screen.findByRole('button', { name: 'Next' }));
    await screen.findByRole('heading', { name: 'All due words reviewed' });
    expect(screen.getByText('1 remembered · 1 forgot')).toBeInTheDocument();
    expect((await session.getToday()).reviewQueue).toHaveLength(0);
    expect(fetchSpy).not.toHaveBeenCalled();
    expect(invoke).not.toHaveBeenCalled();
    expect(listen).not.toHaveBeenCalled();
  } finally { fetchSpy.mockRestore(); }
});

it('keeps capture identity, undo, notifications and curated due scope coherent', async () => {
  const session = new LandingSession();
  const listener = vi.fn();
  const stop = await session.listenLibraryChanged(listener);
  const first = await session.capture(preparedCapture);
  const repeat = await session.capture(preparedCapture);
  expect(repeat.wordId).toBe(first.wordId);
  expect(repeat.encounterCount).toBe(2);
  expect(await session.listWords()).toHaveLength(5);
  await session.undoCapture(repeat.encounterId);
  expect((await session.getWord(first.wordId)).encounters).toHaveLength(1);
  expect((await session.getToday()).reviewQueue.map(word => word.wordId)).toEqual(['serendipitous', 'resilient']);
  await expect(session.submitReview(first.wordId, 'remembered')).rejects.toThrow('due demo scope');
  const result = await session.submitReview('serendipitous', 'remembered', 'submission');
  expect(await session.submitReview('serendipitous', 'remembered', 'submission')).toEqual(result);
  await expect(session.submitReview('serendipitous', 'forgot', 'submission')).rejects.toThrow('conflict');
  expect(listener).toHaveBeenCalledTimes(3);
  stop();
  await session.undoCapture(first.encounterId);
  expect(await session.listWords()).toHaveLength(4);
  expect(listener).toHaveBeenCalledTimes(3);
});
