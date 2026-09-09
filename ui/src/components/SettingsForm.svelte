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
  export let onsave: () => void;
</script>
        <form class="settings" onsubmit={(event) => { event.preventDefault(); onsave(); }}>
          {#if settingsError}<div class="dialog-error settings-message" role="alert">{settingsError}</div>{/if}
          <Field.FieldGroup>
            <Field.FieldSet><Field.FieldLegend>Languages</Field.FieldLegend><p>Used for capture and translation.</p>
              <Field.Field><label>Source language<NativeSelect.Root bind:value={settingsDraft.sourceLanguage}><option value="auto">Auto detect</option><option value="en">English</option><option value="de">German</option><option value="fr">French</option><option value="es">Spanish</option><option value="zh-Hans">Chinese (Simplified)</option><option value="zh-Hant">Chinese (Traditional)</option></NativeSelect.Root></label></Field.Field>
              <Field.Field><label>Translate into<NativeSelect.Root bind:value={settingsDraft.targetLanguage}><option value="en">English</option><option value="de">German</option><option value="fr">French</option><option value="es">Spanish</option><option value="zh-Hans">Chinese (Simplified)</option><option value="zh-Hant">Chinese (Traditional)</option></NativeSelect.Root></label></Field.Field>
            </Field.FieldSet>
            <Field.FieldSet><Field.FieldLegend>Review</Field.FieldLegend><p>Set the size of your daily session.</p>
              <Field.Field><label>Review time<Input type="time" bind:value={settingsDraft.reviewTime} /></label></Field.Field>
              {#if systemStatus.notificationError}<small class="field-error" role="alert">{systemStatus.notificationError}</small>{/if}
              <Field.Field><label>Daily limit<Input type="number" min="1" max="50" bind:value={settingsDraft.dailyLimit} /></label></Field.Field>
              <Field.Field><label>Recent captures<Input type="number" min="1" max="100" bind:value={settingsDraft.recentCapturesLimit} /></label></Field.Field>
              <Field.Field><label>Keep achieved words for<NativeSelect.Root bind:value={settingsDraft.achievedRetentionDays}><option value={10}>10 days</option><option value={20}>20 days</option><option value={30}>30 days</option><option value={60}>60 days</option></NativeSelect.Root></label></Field.Field>
              <small>Retention changes apply only to words Achieved after you save this setting.</small>
              <Field.Field><label class="toggle-row"><span><strong>Automatically achieve Mastered words</strong><small>After 30 uninterrupted days; manual Achieve remains available</small></span><Switch aria-label="Automatically achieve Mastered words after 30 days" bind:checked={settingsDraft.automaticAchieveEnabled} /></label></Field.Field>
            </Field.FieldSet>
            <Field.FieldSet><Field.FieldLegend>Capture shortcuts</Field.FieldLegend><p>Selection Capture is recommended. Leave a shortcut blank to disable it.</p>
              <Field.Field><label>Selection Capture · Recommended<Input aria-label="Selection Capture shortcut" bind:value={settingsDraft.selectionCaptureShortcut} /></label></Field.Field>
              {#if systemStatus.selectionShortcutError}<small class="field-error" role="alert">{systemStatus.selectionShortcutError}</small>{/if}
              <Field.Field><label>Region OCR Capture<Input aria-label="Region OCR Capture shortcut" bind:value={settingsDraft.regionOcrCaptureShortcut} /></label></Field.Field>
              {#if systemStatus.regionOcrShortcutError}<small class="field-error" role="alert">{systemStatus.regionOcrShortcutError}</small>{/if}
              <Field.Field><label class="toggle-row"><span><strong>Launch at login</strong><small>Start hidden and remain available in the system tray</small></span><Switch aria-label="Launch at login" bind:checked={settingsDraft.launchAtLogin} /></label></Field.Field>
              {#if systemStatus.autostartError}<small class="field-error" role="alert">{systemStatus.autostartError}</small>{/if}
            </Field.FieldSet>
            <Field.FieldSet><Field.FieldLegend>Appearance</Field.FieldLegend><p>Visual preferences apply after a successful save.</p>
              <Field.Field><label>Theme<NativeSelect.Root bind:value={settingsDraft.appearance}><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></NativeSelect.Root></label></Field.Field>
              <Field.Field><label class="toggle-row"><span><strong>Reduce motion</strong><small>Minimize non-essential interface motion</small></span><Switch aria-label="Reduce motion" bind:checked={settingsDraft.reducedMotion} /></label></Field.Field>
            </Field.FieldSet>
          </Field.FieldGroup>
          <div class="settings-actions"><Button type="submit" disabled={settingsSaving}>{settingsSaving ? "Saving..." : "Save settings"}</Button></div>
        </form>
<style>
  .settings { width:100%; }
  .settings :global([data-slot=field-group]) { gap:24px; }
  .settings :global(fieldset) { min-width:0; border:0; border-bottom:1px solid var(--border); padding:0 0 24px; gap:12px; }
  .settings :global(legend) { font-size:1rem; line-height:1.45; font-weight:600; padding:0; margin:0; }
  .settings :global(fieldset > p), small { color:var(--muted-foreground); font-size:0.75rem; line-height:1.45; }
  label { display:flex; align-items:center; justify-content:space-between; gap:12px; min-height:36px; width:100%; font-size:0.875rem; line-height:1.45; }
  label :global([data-slot=input]), label :global([data-slot=native-select-wrapper]) { width:224px; flex-shrink:0; }
  .toggle-row span { display:flex; flex-direction:column; gap:4px; }
  strong { font-weight:500; }
  .field-error, .dialog-error { color:var(--destructive); }
  .settings-actions { display:flex; justify-content:flex-end; padding-top:20px; }
  @media(max-width:900px) { label { flex-wrap:wrap; } }
</style>
