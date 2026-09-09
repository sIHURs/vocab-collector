-- Future sync schema only: there is no deployed sync transport in this app.
-- Existing remote duplicates, if any, require a separately verified consolidation;
-- this migration deliberately fails rather than deleting their history.
do $$
declare identity_constraint text;
begin
  select conname into identity_constraint from pg_constraint
  where conrelid = 'public.words'::regclass and contype = 'u'
    and pg_get_constraintdef(oid) like '%target_language%';
  if identity_constraint is not null then
    execute format('alter table public.words drop constraint %I', identity_constraint);
  end if;
end $$;
create unique index words_active_identity on public.words(user_id, lemma, source_language)
  where deleted_at is null;
