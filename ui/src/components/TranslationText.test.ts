import { render } from '@testing-library/svelte';
import { expect, it } from 'vitest';
import TranslationText from './TranslationText.svelte';

it('shows the saved language name only when it differs from the current preference', async () => {
  const { container, rerender } = render(TranslationText, {
    translation: '今天', language: 'de', targetLanguage: 'zh-Hans',
  });
  expect(container.textContent).toBe('今天 (Deutsch)');
  await rerender({ translation: '今天', language: 'de', targetLanguage: 'de' });
  expect(container.textContent).toBe('今天');
});

it('treats legacy zh as simplified Chinese when comparing and displaying languages', async () => {
  const { container, rerender } = render(TranslationText, {
    translation: '修理', language: 'zh', targetLanguage: 'zh-Hans',
  });
  expect(container.textContent).toBe('修理');
  await rerender({ translation: '修理', language: 'zh', targetLanguage: 'en' });
  expect(container.textContent).toBe('修理 (简体中文)');
  await rerender({ translation: '修理', language: 'zh-Hant', targetLanguage: 'zh' });
  expect(container.textContent).toBe('修理 (繁体中文)');
});
