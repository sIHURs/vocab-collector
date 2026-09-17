-- Future sync representation; transport is not implemented yet.
-- Legacy Encounter snapshots remain NULL because their provenance is unknown.
alter table public.words add column translations jsonb not null default '[]'::jsonb;
alter table public.encounters add column saved_translation jsonb;
update public.words set translations = jsonb_build_array(jsonb_build_object(
  'targetLanguage', lower(target_language), 'text', translation, 'savedAt', updated_at
)) where translation is not null and trim(translation) <> '';
