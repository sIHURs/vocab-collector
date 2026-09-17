<script lang="ts">
  export let app: string | undefined = undefined;
  export let title: string | undefined = undefined;
  export let url: string | undefined = undefined;
</script>
{#if app || title || url}<div class="capture-source"><span>{app ?? 'Current application'}</span>{#if title}<span>{title}</span>{/if}{#if url}<span>{url}</span>{/if}</div>{/if}
<style>.capture-source{display:grid;gap:2px;font-size:.75rem;line-height:1.4;color:var(--muted-foreground);overflow-wrap:anywhere;margin:12px 0 0}</style>
