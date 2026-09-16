<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { toast } from 'svelte-sonner';
  import UndoNotification from './UndoNotification.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import * as Field from '$lib/components/ui/field';

  type Status = { supported: boolean; configured: boolean; storageError: boolean };
  export let toasterId: string | undefined = undefined;
  let keyInput: HTMLInputElement | null = null;
  let removing = false;
  const validationErrors: Record<string, string> = {
    invalid_key: 'DeepL could not validate this API key. Please re-enter a valid key.',
    validation_timeout: 'DeepL validation timed out. Please try again.',
    validation_network: 'Could not reach DeepL. Check your connection and try again.',
    validation_unavailable: 'DeepL could not validate the key right now. Please try again later.',
  };
  let status: Status | null = null;
  let key = '';
  let busy = false;
  let loading = true;
  let error = '';
  let availabilityMessage = '';
  let message = '';
  onMount(() => {
    void invoke<Status>('get_deepl_settings').then(value => { status = value; })
      .catch(() => { availabilityMessage = 'Open the Windows desktop app to manage your DeepL key.'; })
      .finally(() => { loading = false; });
  });
  async function update(remove = false) {
    if (busy || (!remove && !key.trim())) return;
    busy = true; removing = remove; error = ''; message = '';
    try {
      status = remove
        ? await invoke<Status>('remove_deepl_key')
        : await invoke<Status>('save_deepl_key', { key: key.trim() });
      key = '';
      message = remove ? 'Key removed. The default translation configuration is now active.' : 'Key verified and saved. DeepL will use it for the next translation.';
    } catch (cause) {
      error = remove ? 'Could not remove the key. Please try again.' : (typeof cause === 'string' && Object.hasOwn(validationErrors, cause) ? validationErrors[cause] : 'Could not save the key. Check Windows credential storage and try again.');
      const id = toast.custom(UndoNotification, {
        toasterId, duration: 6000, closeButton: false,
        componentProps: {
          title: remove ? 'Key removal failed' : 'Key verification failed',
          description: error,
          tone: 'error' as const,
          label: 'DeepL API key notification',
          dismissLabel: 'Dismiss DeepL API key notification',
          ondismiss: () => toast.dismiss(id),
        },
      });
    } finally { busy = false; }
    if (error && !remove) {
      await tick();
      keyInput?.focus();
      keyInput?.select();
    }
  }
</script>

<Field.FieldSet>
  <Field.FieldLegend>DeepL translation</Field.FieldLegend>
  <Field.FieldDescription>Use your own DeepL API key for automatic translation. No Vocab Collector account is needed. Your DeepL plan and quota apply.</Field.FieldDescription>
  {#if availabilityMessage}<p>{availabilityMessage}</p>{/if}
  <p>{loading ? 'Loading key settings…' : status?.configured ? 'A DeepL key is saved on this device.' : 'No personal DeepL key is saved.'}</p>
  {#if status?.storageError}<p role="alert">The saved key could not be loaded. Save it again or remove it.</p>{/if}
  <Field.Field data-disabled={loading || busy || !status?.supported} data-invalid={!!error}>
    <Field.FieldLabel for="deepl-api-key">DeepL API key</Field.FieldLabel>
    <Input id="deepl-api-key" type="password" bind:ref={keyInput} bind:value={key} autocomplete="off" spellcheck={false} maxlength={1024} disabled={loading || busy || !status?.supported} aria-invalid={!!error} placeholder={status?.configured ? 'Enter a replacement key' : 'Enter your API key'} onkeydown={(event) => { if (event.key === 'Enter') { event.preventDefault(); void update(); } }} />
    <Field.FieldDescription>Stored in Windows Credential Manager, separately from your vocabulary. Only vocabulary text is sent to DeepL for translation. API Free and API Pro keys are supported.</Field.FieldDescription>
  </Field.Field>
  <div class="actions">
    <Button type="button" disabled={loading || busy || !status?.supported || !key.trim()} onclick={() => update()}>{busy ? removing ? 'Removing…' : 'Verifying…' : status?.configured ? 'Replace key' : 'Save key'}</Button>
    <Button type="button" variant="outline" disabled={loading || busy || !status?.supported || (!status?.configured && !status?.storageError)} onclick={() => update(true)}>Remove key</Button>
  </div>
  {#if error}<p role="alert">{error}</p>{/if}
  {#if message}<p role="status">{message}</p>{/if}
</Field.FieldSet>

<style>
  .actions { display: flex; gap: 8px; flex-wrap: wrap; }
  p { color: var(--muted-foreground); font-size: 0.75rem; }
  p[role='alert'] { color: var(--destructive); }
</style>
