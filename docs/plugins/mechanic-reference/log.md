# Plugin Log: mechanic_reference

## Status: complete

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

### 2026-02-22 — ScaffoldAction string-based design

**Context**: Scaffold templates need to create entity types, enums, properties, CRT structures,
phases, and combat modifiers. These span `game_system` and `mechanics` contracts. **Decision**: Use
string-based `ScaffoldAction` variants (role as `"Cell"`/`"Token"`, prop_type as
`"Int"`/`"Enum(Name)"`, etc.) with conversion at application time in `editor_ui`. **Rationale**:
Avoids cross-contract type dependencies in the `mechanic_reference` contract. The converter
(`apply_scaffold_recipe`) lives in `editor_ui` where all registries are mutable. **Alternatives
rejected**: Typed ScaffoldAction using game_system types directly (would couple the
mechanic_reference contract to game_system internals).

## Test Results

### 2026-02-22 — Full audit pass

- `mise check:audit`: All checks pass
- 340 total tests (35 mechanic_reference + editor_ui Scope 5 tests)
- Zero clippy warnings, zero boundary violations, zero unwrap in production
- Coverage: 26 mechanic_reference tests + 9 editor_ui scaffold application tests

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
| (none)  |            |        |          |

## Deferred / Future Work

- [None yet]

## Status Updates

| Date       | Status   | Notes                                             |
| ---------- | -------- | ------------------------------------------------- |
| 2026-02-21 | speccing | Initial spec and log created                      |
| 2026-02-22 | complete | All 5 scopes built, audit passed, ready for merge |
