# WaveDSL Agent

Before starting any task, read `instructions/base-instructions.md`.

## Documentation Index

| Resource | Purpose | When to read |
|----------|---------|--------------|
| `instructions/base-instructions.md` | Persona, coding rules, project conventions | Every session start |
| `wavedsl-spec.md` | WaveDSL language specification v0.2 | Before any implementation or design change |
| `docs/skills-index.md` | Which skill to load for a given task | When selecting a skill |
| `docs/plan/` | Committed implementation plans | Before and during multi-step work (>3 steps) |
| `docs/progress/` | Development diaries, status tracking | Every session start and end |
| `docs/doc/` | Design documents, ADRs, specifications | Before architecture or design changes |

## Skill Loading

Read `docs/skills-index.md` to select the appropriate skill, then load the skill by reading its `SKILL.md`.

## Contribution Protocol

After completing a task, update the relevant doc:

| Outcome | Action |
|---------|--------|
| New lesson or root cause found | Record in `docs/progress/diary_<YYYYMMDD>.md` |
| Plan committed | Write `docs/plan/plan_<feature>_<YYYYMMDD>.md` before implementation |
| Design decision made | Write `docs/doc/adr_<NNN>_<title>.md` |
| New skill added | Update `docs/skills-index.md` |
