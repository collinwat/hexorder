# Plugin Log: mechanic_reference

## Status: speccing

## Decision Log

### 2026-02-21 — Plugin naming and placement

**Context**: Pitch #100 describes a "Mechanic Reference Library" as a browsable catalog panel in the
editor. Need to decide whether this is a standalone plugin or part of editor_ui. **Decision**:
Standalone plugin `mechanic_reference` — owns catalog data and scaffolding logic. Editor_ui
integration renders the panel but the data and template logic live in this plugin. **Rationale**:
Separation of concerns — the catalog data model and scaffolding templates are distinct from the
editor UI. This allows the catalog to be consumed by other systems (e.g., a future CLI export or
API) without coupling to the editor. **Alternatives rejected**: Embedding in editor_ui (would bloat
an already large module).

### 2026-02-21 — Research foundation

**Context**: Checked wiki for prior research. **Decision**: Draw catalog content from Hex Wargame
Mechanics Survey (50+ mechanics across 6 areas) and Game Mechanics Discovery research (Engelstein
taxonomy, 203 mechanisms across 13 categories). **Rationale**: Substantial research already exists —
no new investigation needed. The survey provides descriptions, example games, and data model
implications.

## Test Results

(none yet)

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
| (none)  |            |        |          |

## Deferred / Future Work

- [None yet]

## Status Updates

| Date       | Status   | Notes                        |
| ---------- | -------- | ---------------------------- |
| 2026-02-21 | speccing | Initial spec and log created |
