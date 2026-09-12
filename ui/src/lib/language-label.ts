const languageNames: Record<string, string> = {
  auto: 'Auto detect',
  en: 'English',
  de: 'German',
  fr: 'French',
  es: 'Spanish',
  zh: 'Chinese (Simplified)',
  'zh-hans': 'Chinese (Simplified)',
  'zh-hant': 'Chinese (Traditional)',
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
