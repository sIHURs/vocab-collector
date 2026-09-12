<script lang="ts">
  import type { Settings, SystemSettingsStatus } from '../lib/types';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Switch } from '$lib/components/ui/switch';
  import * as Field from '$lib/components/ui/field';
  import SettingsSelect from './SettingsSelect.svelte';
  import ReviewTimePicker from './ReviewTimePicker.svelte';
  export let settingsDraft: Settings;
  export let systemStatus: SystemSettingsStatus = {};
  export let settingsError = '';
  export let settingsSaving = false;
  export let oncommit: (patch: Partial<Settings>) => void;
  export let onretry: () => void;
  let validation: Partial<Record<keyof Settings, string>> = {};

  type ShortcutField = 'selectionCaptureShortcut' | 'regionOcrCaptureShortcut';
  function shortcutIdentity(value: string) {
    return value.toLowerCase().split('+').map((part) => {
      const key = part.trim();
      return ({ ctrl: 'control', cmd: 'meta', command: 'meta', option: 'alt' } as Record<string, string>)[key] ?? key;
    }).sort().join('+');
  }
  function recordShortcut(key: ShortcutField, event: KeyboardEvent) {
    if (event.key === 'Tab') return;
    event.preventDefault();
    event.stopPropagation();
    if (event.repeat || event.isComposing) return;
    if (event.key === 'Escape') {
      validation = { ...validation, [key]: '' };
      (event.currentTarget as HTMLInputElement).blur();
      return;
    }
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)) return;
    const modifiers = [event.ctrlKey && 'Control', event.altKey && 'Alt', event.shiftKey && 'Shift', event.metaKey && 'Meta'].filter(Boolean);
    const clear = !modifiers.length && ['Backspace', 'Delete'].includes(event.key);
    if (!clear && (!modifiers.length || event.getModifierState('AltGraph'))) {
      validation = { ...validation, [key]: 'Use Ctrl, Alt, Shift, or Windows with another key.' };
      return;
    }
    const physicalKey = /^(Key[A-Z]|Digit[0-9])$/.test(event.code) ? event.code.replace(/^(Key|Digit)/, '') : event.key;
    const mainKey = physicalKey === ' ' ? 'Space' : physicalKey;
    if (!clear && !/^(\w+|,|\.|\/|;|'|\[|\]|\\|`|-|=)$/.test(mainKey)) {
      validation = { ...validation, [key]: 'This key is not supported. Choose another combination.' };
      return;
    }
    const candidate = clear ? '' : [...modifiers, mainKey.length === 1 ? mainKey.toUpperCase() : mainKey].join('+');
    const other: ShortcutField = key === 'selectionCaptureShortcut' ? 'regionOcrCaptureShortcut' : 'selectionCaptureShortcut';
    if (candidate && shortcutIdentity(candidate) === shortcutIdentity(settingsDraft[other])) {
      validation = { ...validation, [key]: 'Capture shortcuts must be different.' };
      return;
    }
    validation = { ...validation, selectionCaptureShortcut: '', regionOcrCaptureShortcut: '' };
    if (candidate !== settingsDraft[key]) {
      settingsDraft = { ...settingsDraft, [key]: candidate };
      oncommit({ [key]: candidate });
    }
  }

  function commitInput(key: keyof Settings, input: HTMLInputElement) {
    let message = '';
    if (key === 'dailyLimit' || key === 'recentCapturesLimit') {
      const max = key === 'dailyLimit' ? 50 : 100;
      if (!input.value || !Number.isInteger(Number(input.value)) || Number(input.value) < 1 || Number(input.value) > max)
        message = `Enter a whole number from 1 to ${max}.`;
    }
    validation = { ...validation, [key]: message };
    if (!message) {
      const value = input.type === 'number' ? Number(input.value) : input.value.trim();
      settingsDraft = { ...settingsDraft, [key]: value };
      oncommit({ [key]: value });
    }
  }
  function finishInput(event: KeyboardEvent) {
    if (event.key === 'Enter') { event.preventDefault(); (event.currentTarget as HTMLInputElement).blur(); }
  }
</script>
        <form class="settings" onsubmit={(event) => event.preventDefault()}>
          {#if settingsError}<div class="dialog-error settings-message" role="alert">{settingsError} <Button type="button" variant="outline" size="sm" disabled={settingsSaving} onclick={onretry}>Retry saving</Button></div>{/if}
          <Field.FieldGroup>
            <Field.FieldSet><Field.FieldLegend>Languages</Field.FieldLegend><p>Used for capture and translation.</p>
              <Field.Field><label>Source language<SettingsSelect label="Source language" value={String(settingsDraft.sourceLanguage ?? '')} options={[{ value: 'auto', label: 'Auto detect' }, { value: 'en', label: 'English' }, { value: 'de', label: 'German' }, { value: 'fr', label: 'French' }, { value: 'es', label: 'Spanish' }, { value: 'zh-Hans', label: 'Chinese (Simplified)' }, { value: 'zh-Hant', label: 'Chinese (Traditional)' }]} onchange={(value) => { const patch = { sourceLanguage: value as Settings["sourceLanguage"] }; settingsDraft = { ...settingsDraft, ...patch }; oncommit(patch); }} /></label></Field.Field>
              <Field.Field><label>Translate into<SettingsSelect label="Translate into" value={String(settingsDraft.targetLanguage ?? '')} options={[{ value: 'en', label: 'English' }, { value: 'de', label: 'German' }, { value: 'fr', label: 'French' }, { value: 'es', label: 'Spanish' }, { value: 'zh-Hans', label: 'Chinese (Simplified)' }, { value: 'zh-Hant', label: 'Chinese (Traditional)' }]} onchange={(value) => { const patch = { targetLanguage: value as Settings["targetLanguage"] }; settingsDraft = { ...settingsDraft, ...patch }; oncommit(patch); }} /></label></Field.Field>
            </Field.FieldSet>
            <Field.FieldSet><Field.FieldLegend>Review</Field.FieldLegend><p>Set the size of your daily session.</p>
              <Field.Field><div class="time-row"><span>Review time <small>(24-hour)</small></span><ReviewTimePicker value={settingsDraft.reviewTime} onchange={(reviewTime) => { settingsDraft = { ...settingsDraft, reviewTime }; oncommit({ reviewTime }); }} /></div></Field.Field>
              {#if systemStatus.notificationError}<small class="field-error" role="alert">{systemStatus.notificationError}</small>{/if}
              <Field.Field data-invalid={!!validation.dailyLimit}><label>Daily limit<Input type="number" min="1" max="50" bind:value={settingsDraft.dailyLimit} aria-invalid={!!validation.dailyLimit} onblur={(event) => commitInput("dailyLimit", event.currentTarget)} onkeydown={finishInput} /></label>{#if validation.dailyLimit}<small class="field-error" role="alert">{validation.dailyLimit}</small>{/if}</Field.Field>
              <Field.Field data-invalid={!!validation.recentCapturesLimit}><label>Recent captures<Input type="number" min="1" max="100" bind:value={settingsDraft.recentCapturesLimit} aria-invalid={!!validation.recentCapturesLimit} onblur={(event) => commitInput("recentCapturesLimit", event.currentTarget)} onkeydown={finishInput} /></label>{#if validation.recentCapturesLimit}<small class="field-error" role="alert">{validation.recentCapturesLimit}</small>{/if}</Field.Field>
              <Field.Field><label>Keep achieved words for<SettingsSelect label="Keep achieved words for" value={String(settingsDraft.achievedRetentionDays ?? '')} options={[{ value: '10', label: '10 days' }, { value: '20', label: '20 days' }, { value: '30', label: '30 days' }, { value: '60', label: '60 days' }]} onchange={(value) => { const patch = { achievedRetentionDays: Number(value) as Settings["achievedRetentionDays"] }; settingsDraft = { ...settingsDraft, ...patch }; oncommit(patch); }} /></label></Field.Field>
              <small>Retention changes apply only to words Achieved after this setting changes.</small>
              <Field.Field><label class="toggle-row"><span><strong>Automatically achieve Mastered words</strong><small>Includes existing words Mastered for 30 uninterrupted days. Achieved words are permanently deleted after their retention period; switching this off does not undo earlier Achieve actions.</small></span><Switch aria-label="Automatically achieve Mastered words after 30 days" checked={settingsDraft.automaticAchieveEnabled} onCheckedChange={(value) => { settingsDraft = { ...settingsDraft, automaticAchieveEnabled: value }; oncommit({ automaticAchieveEnabled: value }); }} /></label></Field.Field>
            </Field.FieldSet>
            <Field.FieldSet><Field.FieldLegend>Capture shortcuts</Field.FieldLegend><p>Selection Capture is recommended. Click a shortcut and press a key combination. Esc cancels; Backspace or Delete disables it. Changes are saved automatically.</p>
              <Field.Field data-invalid={!!validation.selectionCaptureShortcut}><label>Selection Capture · Recommended<Input aria-label="Selection Capture shortcut" readonly value={settingsDraft.selectionCaptureShortcut} aria-invalid={!!validation.selectionCaptureShortcut} onkeydown={(event) => recordShortcut("selectionCaptureShortcut", event)} /></label>{#if validation.selectionCaptureShortcut}<small class="field-error" role="alert">{validation.selectionCaptureShortcut}</small>{/if}</Field.Field>
              {#if systemStatus.selectionShortcutError}<small class="field-error" role="alert">{systemStatus.selectionShortcutError}</small>{/if}
              <Field.Field data-invalid={!!validation.regionOcrCaptureShortcut}><label>Region OCR Capture<Input aria-label="Region OCR Capture shortcut" readonly value={settingsDraft.regionOcrCaptureShortcut} aria-invalid={!!validation.regionOcrCaptureShortcut} onkeydown={(event) => recordShortcut("regionOcrCaptureShortcut", event)} /></label>{#if validation.regionOcrCaptureShortcut}<small class="field-error" role="alert">{validation.regionOcrCaptureShortcut}</small>{/if}</Field.Field>
              {#if systemStatus.regionOcrShortcutError}<small class="field-error" role="alert">{systemStatus.regionOcrShortcutError}</small>{/if}
              <Field.Field><label class="toggle-row"><span><strong>Launch at login</strong><small>Start hidden and remain available in the system tray</small></span><Switch aria-label="Launch at login" checked={settingsDraft.launchAtLogin} onCheckedChange={(value) => { settingsDraft = { ...settingsDraft, launchAtLogin: value }; oncommit({ launchAtLogin: value }); }} /></label></Field.Field>
              {#if systemStatus.autostartError}<small class="field-error" role="alert">{systemStatus.autostartError}</small>{/if}
            </Field.FieldSet>
            <Field.FieldSet><Field.FieldLegend>Appearance</Field.FieldLegend><p>Changes are saved automatically.</p>
              <Field.Field><label>Theme<SettingsSelect label="Theme" value={String(settingsDraft.appearance ?? '')} options={[{ value: 'system', label: 'System' }, { value: 'light', label: 'Light' }, { value: 'dark', label: 'Dark' }]} onchange={(value) => { const patch = { appearance: value as Settings["appearance"] }; settingsDraft = { ...settingsDraft, ...patch }; oncommit(patch); }} /></label></Field.Field>
              <Field.Field><label class="toggle-row"><span><strong>Reduce motion</strong><small>Minimize non-essential interface motion</small></span><Switch aria-label="Reduce motion" checked={settingsDraft.reducedMotion} onCheckedChange={(value) => { settingsDraft = { ...settingsDraft, reducedMotion: value }; oncommit({ reducedMotion: value }); }} /></label></Field.Field>
            </Field.FieldSet>
          </Field.FieldGroup>
        </form>
<style>
  .settings { width:100%; }
  .time-row { display:flex; align-items:center; justify-content:space-between; flex-wrap:wrap; gap:12px; min-height:36px; }
  .settings :global([data-slot=field-group]) { gap:24px; }
  .settings :global(fieldset) { min-width:0; border:0; border-bottom:1px solid var(--border); padding:0 0 24px; gap:12px; }
  .settings :global(fieldset:last-child) { border-bottom:0; padding-bottom:0; }
  .settings :global(legend) { font-size:1rem; line-height:1.45; font-weight:600; padding:0; margin:0; }
  .settings :global(fieldset > p), small { color:var(--muted-foreground); font-size:0.75rem; line-height:1.45; }
  label { display:flex; align-items:center; justify-content:space-between; gap:12px; min-height:36px; width:100%; font-size:0.875rem; line-height:1.45; }
  label :global([data-slot=input]), label :global([data-slot=native-select-wrapper]) { width:224px; flex-shrink:0; }
  .toggle-row span { display:flex; flex-direction:column; gap:4px; }
  strong { font-weight:500; }
  .field-error, .dialog-error { color:var(--destructive); }
  @media(max-width:900px) { label { flex-wrap:wrap; } }
</style>

