<script lang="ts">
  import { tick } from 'svelte';
  import type { VocabularyLog } from '../lib/types';
  import { activityLevel, calendarDate, dayLabel, shiftDate } from '../lib/vocabulary-log';
  import * as Tooltip from '$lib/components/ui/tooltip';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import * as Empty from '$lib/components/ui/empty';
  import * as Alert from '$lib/components/ui/alert';
  import { Button } from '$lib/components/ui/button';
  export let log: VocabularyLog | null = null;
  export let loading = false;
  export let error = '';
  export let onretry: () => void;
  let viewport: HTMLElement | null = null;
  let grid: HTMLDivElement;
  let focusedDate = '';
  let positionedEnd = '';
  $: if (log && log.endDate !== positionedEnd) {
    positionedEnd = log.endDate;
    focusedDate = log.endDate;
    void tick().then(() => { if (viewport) viewport.scrollLeft = viewport.scrollWidth; });
  }
  $: offset = log ? calendarDate(log.startDate).getUTCDay() : 0;
  $: weeks = log ? Array.from({ length: Math.ceil((offset + log.days.length) / 7) }, (_, week) => Array.from({ length: 7 }, (_, day) => log?.days[week * 7 + day - offset] ?? null)) : [];
  const month = (date: string) => calendarDate(date).toLocaleDateString(undefined, { month: 'short', timeZone: 'UTC' });
  async function navigate(event: KeyboardEvent, date: string) {
    const delta = { ArrowLeft: -7, ArrowRight: 7, ArrowUp: -1, ArrowDown: 1 }[event.key];
    if (delta == null || !log) return;
    event.preventDefault();
    const next = shiftDate(date, delta);
    focusedDate = next < log.startDate ? log.startDate : next > log.endDate ? log.endDate : next;
    await tick();
    const cell = grid.querySelector<HTMLButtonElement>(`[data-date="${focusedDate}"]`);
    cell?.focus(); cell?.scrollIntoView?.({ block: 'nearest', inline: 'nearest' });
  }
</script>
<div class="vocabulary-log" aria-busy={loading}>
  <div class="heading"><h2>Vocabulary log</h2><span>Past 12 months</span></div>
  {#if error}<Alert.Root variant="destructive"><Alert.Title>Vocabulary log unavailable</Alert.Title><Alert.Description>{error}<Button variant="outline" onclick={onretry}>Retry Vocabulary log</Button></Alert.Description></Alert.Root>{/if}
  {#if loading && !log}<div class="placeholder" role="status" aria-label="Loading Vocabulary log"><Skeleton class="h-28 w-full" /></div>
  {:else if !log}<Empty.Root><Empty.Header><Empty.Title>Vocabulary log unavailable</Empty.Title><Empty.Description>Daily capture history is unavailable.</Empty.Description></Empty.Header></Empty.Root>
  {:else}
    <p id="log-instructions">Successful captures, including repeats, minus Undo. Use ↑ ↓ for days and ← → for weeks.</p>
    <div class="calendar-shell">
      <div class="weekdays" aria-hidden="true">{#each ['Sun','','Tue','','Thu','','Sat'] as label}<span>{label}</span>{/each}</div>
      <ScrollArea orientation="horizontal" bind:viewportRef={viewport} class="min-w-0 flex-1 pb-3">
        <Tooltip.Provider delayDuration={100}>
          <div class="calendar" bind:this={grid} role="grid" aria-label="Daily captures" aria-describedby="log-instructions">
            {#each weeks as week}
              <div class="week" role="row">
                <span class="month" aria-hidden="true">{#if week.some(day => day?.date.endsWith('-01'))}{month(week.find(day => day?.date.endsWith('-01'))!.date)}{/if}</span>
                {#each week as day}
                  {#if day}<div role="gridcell"><Tooltip.Root><Tooltip.Trigger class="day" data-date={day.date} data-level={activityLevel(day.count ?? 0)} data-coverage={day.coverage} tabindex={focusedDate === day.date ? 0 : -1} aria-label={dayLabel(day)} onfocus={() => { focusedDate = day.date; }} onkeydown={(event) => navigate(event, day.date)} /><Tooltip.Content>{dayLabel(day)}</Tooltip.Content></Tooltip.Root></div>
                  {:else}<div class="blank" role="gridcell" aria-hidden="true"></div>{/if}
                {/each}
              </div>
            {/each}
          </div>
        </Tooltip.Provider>
      </ScrollArea>
    </div>
    <div class="legend" aria-label="Capture count levels"><span>Less</span>{#each ['0','1–2','3–5','6–9','10+'] as count, index}<span class="legend-item"><i data-level={index}></i>{count}</span>{/each}<span>More</span></div>
    <p>Unknown days have no recorded history. Outlined days have partial history; counts may be incomplete.</p>
  {/if}
</div>
<style>
.vocabulary-log{min-width:0}.heading{display:flex;justify-content:space-between;align-items:center;gap:12px}h2{font-size:1rem;margin:0;font-weight:600}.heading span,p,.legend{font-size:.75rem;color:var(--muted-foreground)}p{margin:8px 0 16px}.placeholder{min-height:168px;padding:20px 0}.calendar-shell{display:flex;gap:8px;min-width:0}.weekdays{display:grid;grid-template-rows:repeat(7,10px);gap:3px;padding-top:22px;font-size:.625rem;line-height:10px;flex-shrink:0}.calendar{display:flex;width:max-content;gap:3px;padding:2px 4px 6px}.week{display:flex;position:relative;flex-direction:column;gap:3px;padding-top:20px}.month{position:absolute;top:0;left:0;font-size:.625rem;color:var(--muted-foreground);white-space:nowrap}.week>div{width:10px;height:10px}:global(.day),.legend i{display:block;width:10px;height:10px;border-radius:2px;border:0;padding:0;background:var(--activity-0);cursor:default}:global(.day[data-level='1']),.legend i[data-level='1']{background:var(--activity-1)}:global(.day[data-level='2']),.legend i[data-level='2']{background:var(--activity-2)}:global(.day[data-level='3']),.legend i[data-level='3']{background:var(--activity-3)}:global(.day[data-level='4']),.legend i[data-level='4']{background:var(--activity-4)}:global(.day[data-coverage='unknown']){background:transparent;border:1px dotted var(--muted-foreground)}:global(.day[data-coverage='partial']){outline:1px solid var(--muted-foreground);outline-offset:0}:global(.day:focus-visible){outline:2px solid var(--ring);outline-offset:2px}.legend{display:flex;align-items:center;justify-content:flex-end;gap:8px;margin:8px 0;flex-wrap:wrap}.legend-item{display:flex;gap:4px;align-items:center}
@media(forced-colors:active){:global(.day),.legend i{forced-color-adjust:none;background:Canvas;border:1px solid CanvasText}:global(.day[data-level='1']),:global(.day[data-level='2']){background:ButtonFace}:global(.day[data-level='3']),:global(.day[data-level='4']){background:Highlight}:global(.day:focus-visible){outline-color:Highlight}}
</style>
