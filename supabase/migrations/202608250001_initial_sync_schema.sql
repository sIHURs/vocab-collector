create extension if not exists pgcrypto;

create table public.profiles (
  id uuid primary key references auth.users(id) on delete cascade,
  created_at timestamptz not null default now(), updated_at timestamptz not null default now()
);
create table public.devices (
  id uuid primary key, user_id uuid not null references auth.users(id) on delete cascade,
  name text not null, platform text not null, app_version text not null,
  last_sync_at timestamptz, created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);
create table public.words (
  id uuid primary key, user_id uuid not null references auth.users(id) on delete cascade,
  lemma text not null, display_form text not null, source_language text not null,
  target_language text not null, translation text, part_of_speech text,
  status text not null check (status in ('learning', 'mastered', 'paused')),
  review_state jsonb, created_at timestamptz not null, updated_at timestamptz not null,
  deleted_at timestamptz, unique (user_id, lemma, source_language, target_language)
);
create table public.encounters (
  id uuid primary key, user_id uuid not null references auth.users(id) on delete cascade,
  word_id uuid not null references public.words(id) on delete cascade,
  selected_text text not null, sentence text not null, source_app text, source_title text,
  source_url text, captured_at timestamptz not null, updated_at timestamptz not null,
  deleted_at timestamptz
);
create table public.review_logs (
  id uuid primary key, user_id uuid not null references auth.users(id) on delete cascade,
  word_id uuid not null references public.words(id) on delete cascade,
  rating text not null check (rating in ('forgot', 'remembered')),
  reviewed_at timestamptz not null, received_at timestamptz not null default now(),
  device_id uuid not null
);
create table public.user_settings (
  user_id uuid primary key references auth.users(id) on delete cascade,
  payload jsonb not null default '{}'::jsonb, updated_at timestamptz not null default now()
);

create index words_sync_cursor on public.words(user_id, updated_at, id);
create index encounters_sync_cursor on public.encounters(user_id, updated_at, id);
create index encounters_word_history on public.encounters(user_id, word_id, captured_at desc);
create index review_logs_sync_cursor on public.review_logs(user_id, received_at, id);
create index review_logs_word_history on public.review_logs(user_id, word_id, reviewed_at);
create index devices_owner on public.devices(user_id, updated_at, id);

alter table public.profiles enable row level security;
alter table public.devices enable row level security;
alter table public.words enable row level security;
alter table public.encounters enable row level security;
alter table public.review_logs enable row level security;
alter table public.user_settings enable row level security;

create policy "profiles_owner_all" on public.profiles for all
  using (auth.uid() = id) with check (auth.uid() = id);
create policy "devices_owner_all" on public.devices for all
  using (auth.uid() = user_id) with check (auth.uid() = user_id);
create policy "words_owner_all" on public.words for all
  using (auth.uid() = user_id) with check (auth.uid() = user_id);
create policy "encounters_owner_all" on public.encounters for all
  using (auth.uid() = user_id) with check (auth.uid() = user_id);
create policy "review_logs_owner_all" on public.review_logs for all
  using (auth.uid() = user_id) with check (auth.uid() = user_id);
create policy "settings_owner_all" on public.user_settings for all
  using (auth.uid() = user_id) with check (auth.uid() = user_id);
