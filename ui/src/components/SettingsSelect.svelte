<script lang="ts">
  import * as Select from '$lib/components/ui/select';
  export let value: string;
  export let label: string;
  export let options: { value: string; label: string }[];
  export let onchange: (value: string) => void;
</script>

<Select.Root type="single" {value} onValueChange={onchange}>
  <Select.Trigger aria-label={label} class="w-56 shrink-0">
    {options.find(option => option.value === value)?.label ?? value}
  </Select.Trigger>
  <Select.Content>
    <Select.Group>
      {#each options as option}
        <Select.Item value={option.value} label={option.label} />
      {/each}
    </Select.Group>
  </Select.Content>
</Select.Root>
