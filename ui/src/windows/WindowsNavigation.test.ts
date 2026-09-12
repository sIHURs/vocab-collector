import { fireEvent, render, screen } from '@testing-library/svelte';
import { expect, it } from 'vitest';
import { DemoBackend } from '../lib/backend';
import WindowsApp from './WindowsApp.svelte';

it('searches from Today, toggles navigation and opens capture from the titlebar', async () => {
  render(WindowsApp, { api: new DemoBackend() });
  await screen.findByRole('heading', { name: 'Today' });
  await fireEvent.click(screen.getByRole('button', { name: 'Hide sidebar' }));
  expect(screen.queryByRole('navigation', { name: 'Main navigation' })).not.toBeInTheDocument();
  await fireEvent.click(screen.getByRole('button', { name: 'Show sidebar' }));
  expect(screen.getByRole('navigation', { name: 'Main navigation' })).toBeVisible();
  await fireEvent.keyDown(window, { key: 'k', ctrlKey: true });
  const search = screen.getByRole('searchbox', { name: 'Search vocabulary' });
  expect(search).toHaveFocus();
  await fireEvent.input(search, { target: { value: 'no-such-vocabulary' } });
  await screen.findByRole('heading', { name: 'Vocabulary' });
  await screen.findByText('No matching vocabulary');
  await fireEvent.click(screen.getByRole('button', { name: 'Manual capture' }));
  await screen.findByRole('dialog', { name: 'Save a reading context' });
});


