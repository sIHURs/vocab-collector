<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$lib/components/ui/button';
  import * as Field from '$lib/components/ui/field';
  import { Spinner } from '$lib/components/ui/spinner';
  import DownloadIcon from '@lucide/svelte/icons/download';
  import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';

  type Report = { status: string; issues: string[]; coverage: string[]; elapsedMs: number };
  type Snapshot = { id: string; running: boolean; stage: string; report: Report | null };
  let check: Snapshot | null = null;
  let error = '';
  let message = '';
  let saving: 'report' | 'csv' | null = null;
  let starting = false;
  let cancelling = false;
  let alive = true;
  let polling = false;
  let generation = 0;
  const stages: Record<string, string> = { structure: 'Checking application structure', sqlite: 'Checking database structure', relationships: 'Checking record relationships', formats: 'Checking data formats', incomplete: 'Check could not complete' };
  const statuses: Record<string, string> = { passed: 'No issues found', issues: 'Issues found', incomplete: 'Check incomplete', cancelled: 'Check cancelled' };
  const problems: Record<string, string> = {
    unsupported_schema_version: 'This database version is not supported by this check.',
    sqlite_structure_error: 'The database structure has errors.',
    foreign_key_violation: 'Some records refer to missing records.',
    invalid_vocabulary_format: 'Some vocabulary records have invalid fields.',
    invalid_timestamp: 'Some saved dates cannot be read.',
    invalid_translation_snapshot: 'Some saved translations cannot be read.',
    invalid_settings: 'Some settings cannot be read.',
    invalid_review_rating: 'Some review ratings cannot be read.',
    database_busy: 'The database is busy. Try again after the current operation.',
    database_unavailable: 'The database could not be opened.',
    database_unreadable: 'The database file could not be read.',
    check_time_limit: 'The check reached its time limit.',
    check_could_not_complete: 'Some checks could not finish. Save the report for troubleshooting.',
  };
  async function refresh() {
    if (polling || starting) return;
    polling = true;
    const current = generation;
    try { const next = await invoke<Snapshot>('get_database_check'); if (alive && !starting && current === generation) check = next; }
    catch { /* Browser previews have no native database. Actions explain this if used. */ }
    finally { polling = false; }
  }
  onMount(() => {
    alive = true;
    void refresh();
    const timer = setInterval(() => { void refresh(); }, 500);
    return () => { alive = false; clearInterval(timer); };
  });
  async function start() {
    if (starting || check?.running || saving) return;
    generation += 1;
    starting = true; error = ''; message = '';
    try { const next = await invoke<Snapshot>('start_database_check'); if (alive) check = next; }
    catch { if (alive) error = 'Could not start the check. Open the desktop app and try again.'; }
    finally { if (alive) starting = false; }
  }
  async function cancel() {
    if (!check || cancelling) return;
    cancelling = true;
    try { await invoke('cancel_database_check', { id: check.id }); }
    catch { if (alive) error = 'Could not cancel the check.'; }
    finally { if (alive) cancelling = false; }
  }
  async function save(report: boolean) {
    if (saving) return;
    saving = report ? 'report' : 'csv'; error = ''; message = '';
    try {
      if (report) {
        const saved = await invoke<boolean>('save_database_check_report', { id: check?.id });
        if (alive) message = saved ? 'Diagnostic report saved.' : 'Save cancelled.';
      } else {
        const count = await invoke<number | null>('export_vocabulary_csv');
        if (alive) message = count === null ? 'Export cancelled.' : `Exported ${count} vocabulary items.`;
      }
    } catch { if (alive) error = 'Could not save the file. Check the destination and try again.'; }
    finally { if (alive) saving = null; }
  }
</script>

<Field.FieldSet>
  <Field.FieldLegend>Local data</Field.FieldLegend>
  <Field.FieldGroup>
    <Field.Field>
      <div class="flex flex-wrap items-start justify-between gap-4">
        <div class="flex min-w-0 flex-[1_1_18rem] flex-col gap-2">
          <Field.FieldTitle>Data health</Field.FieldTitle>
          <Field.FieldDescription>Check database structure, record relationships and basic data formats. This does not repair data or verify that past data has never been lost.</Field.FieldDescription>
        </div>
        <div class="flex shrink-0 items-center gap-2">
          <Button type="button" variant="outline" disabled={starting || check?.running || !!saving} aria-busy={starting || check?.running} onclick={start}>
            {#if starting || check?.running}<Spinner aria-hidden="true" data-icon="inline-start" />{:else}<ShieldCheckIcon data-icon="inline-start" />{/if}
            Check data
          </Button>
          {#if check?.running}<Button type="button" variant="ghost" disabled={cancelling} aria-busy={cancelling} onclick={cancel}>{#if cancelling}<Spinner aria-hidden="true" data-icon="inline-start" />{/if}{cancelling ? 'Cancelling…' : 'Cancel check'}</Button>{/if}
        </div>
      </div>
      <div role="status" aria-live="polite" class="text-sm">
        {#if starting}<p>Starting check…</p>
        {:else if check?.running}{stages[check.stage] ?? 'Checking data'}…
        {:else if check?.report}
          <p>{statuses[check.report.status] ?? 'Check incomplete'} · {check.report.elapsedMs} ms</p>
          {#if check.report.issues.length}
            <ul>{#each check.report.issues as issue}<li>{problems[issue] ?? 'Some checks could not finish.'}</li>{/each}</ul>
            <p>No data was repaired or deleted.</p>
          {/if}
        {:else if check?.stage === 'incomplete'}<p>Check incomplete. Please try again.</p>{/if}
      </div>
      {#if check?.report && !check.running && !starting}
        <div class="flex flex-col items-start gap-1">
          <Button type="button" variant="link" size="sm" class="px-0" disabled={!!saving} aria-busy={saving === 'report'} onclick={() => save(true)}>
            {#if saving === 'report'}<Spinner aria-hidden="true" data-icon="inline-start" />{:else}<DownloadIcon data-icon="inline-start" />{/if}
            {saving === 'report' ? 'Saving report…' : 'Save diagnostic report'}
          </Button>
          <Field.FieldDescription>Reports contain technical check results, not vocabulary, context or source URLs. Nothing is uploaded.</Field.FieldDescription>
        </div>
      {/if}
    </Field.Field>
    <Field.Field>
      <div class="flex flex-wrap items-start justify-between gap-4">
        <div class="flex min-w-0 flex-[1_1_18rem] flex-col gap-2">
          <Field.FieldTitle>Vocabulary export</Field.FieldTitle>
          <Field.FieldDescription>Exports all non-achieved vocabulary with the displayed translation. Context, review history and settings are excluded. This is not a backup.</Field.FieldDescription>
        </div>
        <div class="shrink-0">
          <Button type="button" variant="outline" disabled={!!saving} aria-busy={saving === 'csv'} onclick={() => save(false)}>
            {#if saving === 'csv'}<Spinner aria-hidden="true" data-icon="inline-start" />{:else}<DownloadIcon data-icon="inline-start" />{/if}
            {saving === 'csv' ? 'Exporting…' : 'Export vocabulary CSV'}
          </Button>
        </div>
      </div>
    </Field.Field>
  </Field.FieldGroup>
  <div role="status" aria-live="polite" class="text-sm">{#if message}<p>{message}</p>{/if}</div>
  {#if error}<p role="alert">{error}</p>{/if}
</Field.FieldSet>
