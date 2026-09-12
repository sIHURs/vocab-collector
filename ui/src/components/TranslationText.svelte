<script lang="ts">
  import { languageLabel } from '../lib/language-label';
  export let translation: string | undefined;
  export let language: string | undefined;
  export let targetLanguage: string | undefined;

  // Match the domain's legacy Chinese language normalization.
  const normalize = (value: string | undefined) => {
    const code = value?.trim().toLowerCase();
    return code === 'zh' ? 'zh-hans' : code;
  };
  $: normalizedLanguage = normalize(language);
  $: showLanguage = !!translation && !!normalizedLanguage && normalizedLanguage !== normalize(targetLanguage);
</script>

<span class="translation-text">{translation ?? 'No translation'}{#if showLanguage && normalizedLanguage}<small>{' '}({languageLabel(normalizedLanguage)})</small>{/if}</span>

<style>
  .translation-text { color:var(--foreground); }
  small { color:var(--muted-foreground); font-size:0.75rem; font-weight:400; }
</style>
