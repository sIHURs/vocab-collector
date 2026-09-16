<script lang="ts">
  import LoaderCircleIcon from '@lucide/svelte/icons/loader-circle';
  import { cn } from '$lib/utils.js';
  import type { ComponentProps } from 'svelte';

  let { class: className, ...restProps }: ComponentProps<typeof LoaderCircleIcon> = $props();
</script>

<LoaderCircleIcon
  role="status"
  aria-label="Loading"
  data-slot="spinner"
  class={cn('size-4 animate-spin motion-reduce:animate-none', className)}
  {...restProps}
/>
