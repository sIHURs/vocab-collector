alter table public.encounters
  add column capture_origin text not null default 'manual'
  check (capture_origin in ('manual', 'accessibility', 'ocr'));
