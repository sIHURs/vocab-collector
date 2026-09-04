# Issue tracker: Local Markdown

Issues and specs for this repo live as Markdown files in `.scratch/`.

## Conventions

- One feature per directory: `.scratch/<feature-slug>/`
- The spec is `.scratch/<feature-slug>/spec.md`
- Implementation issues use one file per ticket:
  `.scratch/<feature-slug>/issues/<NN>-<slug>.md`
- Ticket numbers begin at `01` and follow dependency order
- `Blocked by:` records blocking ticket numbers and titles
- `Status:` records the ticket state; agent-ready implementation tickets use
  `ready-for-agent`
- Comments and conversation history may be appended under `## Comments`

## Publishing

When a skill says “publish to the issue tracker,” create the corresponding
files under `.scratch/<feature-slug>/`, creating the directory when necessary.

Do not create GitHub, GitLab, Linear, or other remote issues for this repository.

## Fetching

When a skill needs a ticket, read its referenced local Markdown file. The user
will normally provide its path or ticket number.

