import { expect, it } from 'vitest';
import { channel, isThemeMessage } from './protocol';

it('accepts only versioned light/dark theme messages', () => {
  expect(isThemeMessage({ channel, type: 'theme', theme: 'dark' })).toBe(true);
  expect(isThemeMessage({ channel, type: 'theme', theme: 'light' })).toBe(true);
  for (const value of [null, 'dark', { channel, type: 'theme', theme: 'system' },
    { channel: 'another-demo', type: 'theme', theme: 'dark' }, { channel, type: 'reset' }]) {
    expect(isThemeMessage(value)).toBe(false);
  }
});
