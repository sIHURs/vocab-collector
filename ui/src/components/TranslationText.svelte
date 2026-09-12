<script lang="ts">
  export let translation: string | undefined;
  export let language: string | undefined;
  export let targetLanguage: string | undefined;

  const names: Record<string, string> = {
    'zh-hans': '简体中文', 'zh-hant': '繁体中文',
    en: 'English', de: 'Deutsch', fr: 'Français', es: 'Español',
  };
  // Match the domain's legacy Chinese language normalization.
  const normalize = (value: string | undefined) => {
    const code = value?.trim().toLowerCase();
    return code === 'zh' ? 'zh-hans' : code;
  };
  $: normalizedLanguage = normalize(language);
  $: showLanguage = !!translation && !!normalizedLanguage && normalizedLanguage !== normalize(targetLanguage);
</script>

<span class="translation-text">{translation ?? 'No translation'}{#if showLanguage && normalizedLanguage}<small>{' '}({names[normalizedLanguage] ?? language})</small>{/if}</span>

<style>
  .translation-text { color:var(--foreground); }
  small { color:var(--muted-foreground); font-size:0.75rem; font-weight:400; }
</style>
