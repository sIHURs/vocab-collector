const languageNames: Record<string, string> = {
  auto: 'Auto detect',
  en: 'English',
  de: 'German',
  fr: 'French',
  es: 'Spanish',
  zh: '简体中文',
  'zh-hans': '简体中文',
  'zh-hant': '繁体中文',
};

export function languageLabel(code: string): string {
  const normalized = code.trim().toLowerCase();
  if (!normalized) return 'Unknown language';
  if (languageNames[normalized]) return languageNames[normalized];
  try {
    return new Intl.DisplayNames(['en'], { type: 'language' }).of(normalized) ?? code;
  } catch {
    return code;
  }
}
