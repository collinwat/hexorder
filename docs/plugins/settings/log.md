# Plugin Log: Settings

## Status: building

## Decision Log

### 2026-02-24 — Config directory strategy

**Context**: Need to decide where user settings files live on disk. **Decision**: Reuse the existing
`StorageConfig` pattern from `persistence/storage.rs` — macOS uses
`~/Library/Application Support/hexorder/`, default uses `./config/`. Shortcuts config already uses
this pattern in `shortcuts/config.rs`. **Rationale**: Consistency with existing config file
locations (dock_layout.ron, shortcuts.toml). Users already have config files there. **Alternatives
rejected**: XDG-only (~/.config/hexorder/) — would diverge from existing macOS convention used by
shortcuts and dock layout.

### 2026-02-24 — Existing infrastructure inventory

**Context**: Orientation for settings pitch. Need to understand what already exists. **Decision**:
Documented the following existing patterns to build on:

- `ShortcutRegistry.override_bindings()` already supports config overrides
- `shortcuts/config.rs` already loads and parses `shortcuts.toml` from config dir
- `BrandTheme` in `editor_ui/components.rs` has hardcoded color constants
- `Workspace` in `contracts/persistence.rs` holds `font_size_base` and `workspace_preset`
- `EditorState` syncs font_size bidirectionally with Workspace
- `DockLayoutState` persists to `dock_layout.ron` in config dir **Rationale**: Build on existing
  patterns rather than replacing them.

### 2026-02-24 — Scope 1+5: Settings infrastructure + contract

**Context**: First piece of the settings pitch — build the registry and contract end-to-end.
**Decision**: Typed struct fields for `SettingsRegistry` (not string-keyed dynamic). Simpler,
type-safe, aligns with "no plugin registration API" no-go. **Decision**: Three partial layers with
`Option<T>` fields. Merge is field-by-field `project.or(user).or(defaults)`. **Decision**: Config
dir reuses same `#[cfg(feature)]` pattern as `shortcuts/config.rs`. **Abstraction check**: No
abstraction needed — the `PartialSettings`/merge pattern is simple, direct, and unlikely to be
reused by other plugins.

### 2026-02-24 — Scope 2: Preference migration

**Context**: Migrate editor_ui restore systems to read from SettingsRegistry instead of Workspace.
**Decision**: Added `SettingsReady` SystemSet to the settings contract for cross-plugin system
ordering. `apply_project_layer` runs in `SettingsReady`, editor_ui restore systems run
`.after(SettingsReady)`. **Abstraction check**: `SettingsReady` is a clean cross-plugin ordering
mechanism — reusable by any future consumer of SettingsRegistry.

## Test Results

### 2026-02-24 — Scope 2

- 406 tests pass (full suite, no regressions)
- `cargo clippy --all-targets`: zero warnings
- `mise check:boundary`: no violations

### 2026-02-24 — Scope 1+5

- 10 tests pass: merge priority (5), TOML deserialization (3), edge cases (1), default (1)
- `cargo clippy --all-targets`: zero warnings
- `mise check:boundary`: no violations

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
|         |            |        |          |

## Deferred / Future Work

- [None yet]

## Status Updates

| Date       | Status   | Notes                                          |
| ---------- | -------- | ---------------------------------------------- |
| 2026-02-24 | speccing | Initial spec created                           |
| 2026-02-24 | building | Scope 1+5 complete — infrastructure + contract |
| 2026-02-24 | building | Scope 2 complete — preference migration        |
