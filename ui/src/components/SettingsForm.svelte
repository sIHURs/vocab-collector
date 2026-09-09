<script lang="ts">
  import type { Settings, SystemSettingsStatus } from '../lib/types';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Switch } from '$lib/components/ui/switch';
  import * as Field from '$lib/components/ui/field';
  import * as NativeSelect from '$lib/components/ui/native-select';
  export let settingsDraft: Settings;
  export let systemStatus: SystemSettingsStatus = {};
  export let settingsError = '';
  export let settingsSaving = false;
  export let oncommit: (patch: Partial<Settings>) => void;
  export let onretry: () => void;
  let validation: Partial<Record<keyof Settings, string>> = {};

  function commitInput(key: keyof Settings, input: HTMLInputElement) {
    let message = '';
    if (key === 'dailyLimit' || key === 'recentCapturesLimit') {
      const max = key === 'dailyLimit' ? 50 : 100;
      if (!input.value || !Number.isInteger(Number(input.value)) || Number(input.value) < 1 || Number(input.value) > max)
        message = `Enter a whole number from 1 to ${max}.`;
    } else if (key === 'reviewTime' && !/^([01]\d|2[0-3]):[0-5]\d$/.test(input.value)) {
      message = 'Enter a valid review time.';
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
              <Field.Field><label>Source language<NativeSelect.Root bind:value={settingsDraft.sourceLanguage} onchange={(event) => oncommit({ sourceLanguage: event.currentTarget.value as Settings["sourceLanguage"] })}><option value="auto">Auto detect</option><option value="en">English</option><option value="de">German</option><option value="fr">French</option><option value="es">Spanish</option><option value="zh-Hans">Chinese (Simplified)</option><option value="zh-Hant">Chinese (Traditional)</option></NativeSelect.Root></label></Field.Field>
              <Field.Field><label>Translate into<NativeSelect.Root bind:value={settingsDraft.targetLanguage} onchange={(event) => oncommit({ targetLanguage: event.currentTarget.value as Settings["targetLanguage"] })}><option value="en">English</option><option value="de">German</option><option value="fr">French</option><option value="es">Spanish</option><option value="zh-Hans">Chinese (Simplified)</option><option value="zh-Hant">Chinese (Traditional)</option></NativeSelect.Root></label></Field.Field>
            </Field.FieldSet>
            <Field.FieldSet><Field.FieldLegend>Review</Field.FieldLegend><p>Set the size of your daily session.</p>
              <Field.Field data-invalid={!!validation.reviewTime}><label>Review time<Input type="time" bind:value={settingsDraft.reviewTime} aria-invalid={!!validation.reviewTime} onblur={(event) => commitInput("reviewTime", event.currentTarget)} onkeydown={finishInput} /></label>{#if validation.reviewTime}<small class="field-error" role="alert">{validation.reviewTime}</small>{/if}</Field.Field>
              {#if systemStatus.notificationError}<small class="field-error" role="alert">{systemStatus.notificationError}</small>{/if}
              <Field.Field data-invalid={!!validation.dailyLimit}><label>Daily limit<Input type="number" min="1" max="50" bind:value={settingsDraft.dailyLimit} aria-invalid={!!validation.dailyLimit} onblur={(event) => commitInput("dailyLimit", event.currentTarget)} onkeydown={finishInput} /></label>{#if validation.dailyLimit}<small class="field-error" role="alert">{validation.dailyLimit}</small>{/if}</Field.Field>
              <Field.Field data-invalid={!!validation.recentCapturesLimit}><label>Recent captures<Input type="number" min="1" max="100" bind:value={settingsDraft.recentCapturesLimit} aria-invalid={!!validation.recentCapturesLimit} onblur={(event) => commitInput("recentCapturesLimit", event.currentTarget)} onkeydown={finishInput} /></label>{#if validation.recentCapturesLimit}<small class="field-error" role="alert">{validation.recentCapturesLimit}</small>{/if}</Field.Field>
              <Field.Field><label>Keep achieved words for<NativeSelect.Root bind:value={settingsDraft.achievedRetentionDays} onchange={(event) => oncommit({ achievedRetentionDays: Number(event.currentTarget.value) as Settings["achievedRetentionDays"] })}><option value={10}>10 days</option><option value={20}>20 days</option><option value={30}>30 days</option><option value={60}>60 days</option></NativeSelect.Root></label></Field.Field>
              <small>Retention changes apply only to words Achieved after this setting changes.</small>
              <Field.Field><label class="toggle-row"><span><strong>Automatically achieve Mastered words</strong><small>Includes existing words Mastered for 30 uninterrupted days. Achieved words are permanently deleted after their retention period; switching this off does not undo earlier Achieve actions.</small></span><Switch aria-label="Automatically achieve Mastered words after 30 days" checked={settingsDraft.automaticAchieveEnabled} onCheckedChange={(value) => { settingsDraft = { ...settingsDraft, automaticAchieveEnabled: value }; oncommit({ automaticAchieveEnabled: value }); }} /></label></Field.Field>
            </Field.FieldSet>
            <Field.FieldSet><Field.FieldLegend>Capture shortcuts</Field.FieldLegend><p>Selection Capture is recommended. Leave a shortcut blank to disable it.</p>
              <Field.Field data-invalid={!!validation.selectionCaptureShortcut}><label>Selection Capture · Recommended<Input aria-label="Selection Capture shortcut" bind:value={settingsDraft.selectionCaptureShortcut} aria-invalid={!!validation.selectionCaptureShortcut} onblur={(event) => commitInput("selectionCaptureShortcut", event.currentTarget)} onkeydown={finishInput} /></label>{#if validation.selectionCaptureShortcut}<small class="field-error" role="alert">{validation.selectionCaptureShortcut}</small>{/if}</Field.Field>
              {#if systemStatus.selectionShortcutError}<small class="field-error" role="alert">{systemStatus.selectionShortcutError}</small>{/if}
              <Field.Field data-invalid={!!validation.regionOcrCaptureShortcut}><label>Region OCR Capture<Input aria-label="Region OCR Capture shortcut" bind:value={settingsDraft.regionOcrCaptureShortcut} aria-invalid={!!validation.regionOcrCaptureShortcut} onblur={(event) => commitInput("regionOcrCaptureShortcut", event.currentTarget)} onkeydown={finishInput} /></label>{#if validation.regionOcrCaptureShortcut}<small class="field-error" role="alert">{validation.regionOcrCaptureShortcut}</small>{/if}</Field.Field>
              {#if systemStatus.regionOcrShortcutError}<small class="field-error" role="alert">{systemStatus.regionOcrShortcutError}</small>{/if}
              <Field.Field><label class="toggle-row"><span><strong>Launch at login</strong><small>Start hidden and remain available in the system tray</small></span><Switch aria-label="Launch at login" checked={settingsDraft.launchAtLogin} onCheckedChange={(value) => { settingsDraft = { ...settingsDraft, launchAtLogin: value }; oncommit({ launchAtLogin: value }); }} /></label></Field.Field>
              {#if systemStatus.autostartError}<small class="field-error" role="alert">{systemStatus.autostartError}</small>{/if}
            </Field.FieldSet>
            <Field.FieldSet><Field.FieldLegend>Appearance</Field.FieldLegend><p>Changes are saved automatically.</p>
              <Field.Field><label>Theme<NativeSelect.Root bind:value={settingsDraft.appearance} onchange={(event) => oncommit({ appearance: event.currentTarget.value as Settings["appearance"] })}><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></NativeSelect.Root></label></Field.Field>
              <Field.Field><label class="toggle-row"><span><strong>Reduce motion</strong><small>Minimize non-essential interface motion</small></span><Switch aria-label="Reduce motion" checked={settingsDraft.reducedMotion} onCheckedChange={(value) => { settingsDraft = { ...settingsDraft, reducedMotion: value }; oncommit({ reducedMotion: value }); }} /></label></Field.Field>
            </Field.FieldSet>
          </Field.FieldGroup>
        </form>
<style>
  .settings { width:100%; }
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
