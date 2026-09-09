<script lang="ts">
  import type { GlobalInsight, ReviewSessionInsight, VocabularyLog as VocabularyLogData } from '../lib/types';
  import * as Card from '$lib/components/ui/card';
  import * as Alert from '$lib/components/ui/alert';
  import { Button } from '$lib/components/ui/button';
  import VocabularyLog from './VocabularyLog.svelte';
  export let log: VocabularyLogData | null = null;
  export let logLoading = false;
  export let logError = '';
  export let onlogretry: () => void;
  export let insight: GlobalInsight | null;
  export let session: ReviewSessionInsight | null;
  export let due: number | null;
  export let error = '';
  export let onretry: () => void;
</script>
<div class="insights">
  {#if error}<Alert.Root variant="destructive"><Alert.Title>Insights unavailable</Alert.Title><Alert.Description>{error}<Button variant="outline" onclick={onretry}>Retry Insights</Button></Alert.Description></Alert.Root>{/if}
  <section aria-label="All-time progress">
    <h2>All-time progress</h2><p>Includes anonymous totals retained after permanent deletion.</p>
    <div class="metrics">{#each [['Captured', insight?.lifetimeVocabularyCount], ['Reviewed', insight?.lifetimeReviewCount], ['Due', due]] as [label, value]}
      <Card.Root><Card.Header><Card.Title>{label}</Card.Title></Card.Header><Card.Content><strong class="metric">{value ?? 'Unavailable'}</strong></Card.Content></Card.Root>
    {/each}</div>
    {#if insight}<dl><div><dt>Encounters saved</dt><dd>{insight.lifetimeEncounterCount}</dd></div><div><dt>Remembered</dt><dd>{insight.lifetimeRememberedCount}</dd></div><div><dt>Forgot</dt><dd>{insight.lifetimeForgottenCount}</dd></div><div><dt>Currently achieved</dt><dd>{insight.currentAchievedCount}</dd></div></dl>
    {#if !insight.lifetimeRatingBreakdownComplete}<p>Remembered and Forgot totals exclude anonymous review history deleted before this app version.</p>{/if}
    {:else}<p>All-time history is unavailable. No totals have been assumed.</p>{/if}
  </section>
  <section aria-label="Vocabulary log"><VocabularyLog {log} loading={logLoading} error={logError} onretry={onlogretry} /></section>
  <section aria-label="Latest review session"><h2>Latest review session</h2>
    {#if session}<p>{session.reviewedCount} reviewed · {session.rememberedCount} remembered · {session.forgottenCount} forgot</p><p>Estimated due by the end of tomorrow: {session.nextDayDueCount}</p>
    {:else}<p>No review session yet. Complete a review to see its summary here.</p>{/if}
  </section>
</div>
<style>
.insights{display:flex;flex-direction:column;gap:28px}section{min-width:0}h2{font-size:1rem;font-weight:600;margin:0 0 8px}p{font-size:.875rem;color:var(--muted-foreground);margin:6px 0 16px}.metrics{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px}.metric{font-size:1.75rem;line-height:1.2;font-weight:600;overflow-wrap:anywhere}dl{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;margin:20px 0 0}dl div{display:flex;justify-content:space-between;gap:12px}dt{color:var(--muted-foreground)}dd{margin:0;font-variant-numeric:tabular-nums}@media(max-width:600px){.metrics,dl{grid-template-columns:1fr}}
</style>
