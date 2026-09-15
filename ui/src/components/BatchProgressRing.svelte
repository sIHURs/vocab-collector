<script lang="ts">
  import { Progress as ProgressPrimitive } from 'bits-ui';
  export let completed = 0;
  export let total = 1;
  export let label = 'Batches completed';
  export let accessibleName = 'Overall batch progress';
  export let countText: string | undefined = undefined;
  $: maximum = Math.max(1, total);
  $: value = Math.min(maximum, Math.max(0, completed));
</script>

<div class="batch-summary">
  <ProgressPrimitive.Root class="batch-ring" {value} max={maximum} aria-label={accessibleName} aria-valuetext={`${completed} of ${total} ${label.toLowerCase()}`}>
    <svg viewBox="0 0 48 48" aria-hidden="true">
      <circle class="track" cx="24" cy="24" r="21" />
      <circle class="fill" cx="24" cy="24" r="21" pathLength="100" stroke-dasharray="100" stroke-dashoffset={100 - value / maximum * 100} />
    </svg>
    <span class="count" aria-hidden="true">{countText ?? `${completed} / ${total}`}</span>
  </ProgressPrimitive.Root>
  <span class="label">{label}</span>
</div>

<style>
  .batch-summary { display: flex; align-items: center; gap: 10px; min-width: 0; text-align: left; }
  :global(.batch-ring) { position: relative; display: grid; place-items: center; width: 48px; height: 48px; flex-shrink: 0; }
  svg { position: absolute; inset: 0; width: 100%; height: 100%; transform: rotate(-90deg); }
  circle { fill: none; stroke-width: 3; }
  .track { stroke: var(--muted); }
  .fill { stroke: var(--primary); transition: stroke-dashoffset 180ms ease; }
  .count { font-size: .625rem; font-variant-numeric: tabular-nums; color: var(--foreground); }
  .label { font-size: .75rem; color: var(--muted-foreground); }
  @media (prefers-reduced-motion: reduce) { .fill { transition: none; } }
</style>
