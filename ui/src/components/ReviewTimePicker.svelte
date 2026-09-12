<script lang="ts">
  import * as Select from '$lib/components/ui/select';
  export let value: string;
  export let onchange: (value: string) => void;
  const hours = Array.from({ length: 24 }, (_, i) => String(i).padStart(2, '0'));
  const minutes = Array.from({ length: 60 }, (_, i) => String(i).padStart(2, '0'));
  $: [hour, minute] = value.split(':');
  function change(part: 'hour' | 'minute', next: string) {
    if (!next) return;
    const time = part === 'hour' ? `${next}:${minute}` : `${hour}:${next}`;
    if (time !== value) onchange(time);
  }
</script>

<div role="group" aria-label="Review time" class="flex w-56 shrink-0 items-center gap-2">
  <Select.Root type="single" value={hour} onValueChange={(next) => change('hour', next)}>
    <Select.Trigger aria-label="Review hour" class="flex-1">{hour}</Select.Trigger>
    <Select.Content class="max-h-64">
      <Select.Group><Select.Label>Hour (24-hour)</Select.Label>
        {#each hours as option}<Select.Item value={option} label={option} />{/each}
      </Select.Group>
    </Select.Content>
  </Select.Root>
  <span aria-hidden="true">:</span>
  <Select.Root type="single" value={minute} onValueChange={(next) => change('minute', next)}>
    <Select.Trigger aria-label="Review minute" class="flex-1">{minute}</Select.Trigger>
    <Select.Content class="max-h-64">
      <Select.Group><Select.Label>Minute</Select.Label>
        {#each minutes as option}<Select.Item value={option} label={option} />{/each}
      </Select.Group>
    </Select.Content>
  </Select.Root>
</div>
