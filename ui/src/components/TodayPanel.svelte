<script lang="ts">
  import type { TodayView } from '../lib/types';
  import { Button } from '$lib/components/ui/button';
  import * as Alert from '$lib/components/ui/alert';
  import * as Card from '$lib/components/ui/card';
  import * as Empty from '$lib/components/ui/empty';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import WindowsWordRow from '../windows/WindowsWordRow.svelte';
  export let today: TodayView | null;
  export let loading = false;
  export let paused = false;
  export let refreshRequired = false;
  export let onstart: () => void;
  export let onretry: () => void;
  export let oncapture: (event: MouseEvent) => void;
  export let ondetail: (id: string, trigger: HTMLButtonElement) => void;
</script>
<div class="today">
  <Card.Root>
    <Card.Header><Card.Title>Today's plan</Card.Title><Card.Description>A little practice, at your own pace.</Card.Description></Card.Header>
    <Card.Content>
      {#if !today && loading}<div class="plan-loading" role="status"><span>Loading your vocabulary...</span><Skeleton class="h-12 w-32" /><Skeleton class="h-4 w-60" /></div>
      {:else if today}<div class="plan">{#if paused}<p>{today.reviewQueue.length} words remaining.</p>{/if}<strong>{today.plannedReviewCount}<span> words to review</span></strong><p>{today.totalDueCount} total due · About {today.estimatedMinutes} minute{today.estimatedMinutes === 1 ? '' : 's'}</p></div>{:else}<p>Today is unavailable. Try refreshing your vocabulary.</p>{/if}
    </Card.Content>
    <Card.Footer>
      {#if refreshRequired}<Alert.Root role="status"><Alert.Title>Review paused</Alert.Title><Alert.Description>Refresh Today before continuing so the due queue stays current.<Button disabled={loading} onclick={onretry}>Retry Review refresh</Button></Alert.Description></Alert.Root>
      {:else if paused || today?.reviewQueue.length}<Button disabled={loading} onclick={onstart}>{paused ? 'Resume review' : `Start review (${today?.plannedReviewCount})`}</Button>
      {:else if today}<p class="muted"><span>Nothing due</span>. You're all caught up.</p>{:else if loading}<Skeleton class="h-8 w-40" />{/if}
    </Card.Footer>
  </Card.Root>
  <div><h2>Recent captures</h2><p class="muted">New contexts appear here immediately after saving.</p></div>
  {#if !today && loading}<div class="recent-loading" aria-hidden="true">{#each [1,2,3] as row}<Skeleton class="h-16 w-full" />{/each}</div>
  {:else if today?.recentCaptures.length}<ScrollArea class="recent-captures-list h-[252px]" aria-label="Recent captures">{#each today.recentCaptures as word (word.id)}<WindowsWordRow {word} onSelect={ondetail} />{/each}</ScrollArea>
  {:else if today}<Empty.Root><Empty.Header><Empty.Title>No captures yet</Empty.Title><Empty.Description>Use Manual capture to save your first reading context.</Empty.Description></Empty.Header><Empty.Content><Button onclick={oncapture}>Manual capture</Button></Empty.Content></Empty.Root>{/if}
</div>
<style>
.today{display:flex;flex-direction:column;gap:24px}.plan strong{display:flex;align-items:baseline;gap:8px;flex-wrap:wrap;font-size:2.25rem;font-weight:600}.plan strong span{font-size:.875rem;font-weight:400}.plan p,.muted{color:var(--muted-foreground);font-size:.875rem;margin:4px 0 0}h2{font-size:1rem;font-weight:600;margin:0}.plan-loading,.recent-loading{display:grid;gap:12px}.plan-loading{min-height:80px}:global(.recent-captures-list){border:1px solid var(--border);border-radius:10px;overflow:hidden}
@media(max-height:650px){:global(.recent-captures-list){height:210px}}
</style>
