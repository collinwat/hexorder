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

### 2026-02-24 — Scope 3: Custom themes

**Context**: Wire ThemeDefinition loading from TOML files into the editor. **Decision**: Separate
`ThemeLibrary` resource in the contract (not on `SettingsRegistry`) — themes don't participate in
three-layer merge, they're loaded once at startup. **Decision**: Theme selector placed in the
Settings dock tab rather than the View menu — `editor_dock_system` already has 16 parameters (Bevy
maximum) and cannot accept more resources. EditorState bridges theme state between the dock tab and
`configure_theme`. **Decision**: `widget_noninteractive` derived from `widget_inactive - 10` rather
than adding a 15th field to ThemeDefinition — keeps the contract simpler, matches brand palette
exactly. **Abstraction check**: No abstraction needed — the theme loading is straightforward
file-scan-and-parse with a hardcoded brand fallback.

## Test Results

### 2026-02-24 — Scope 4

**Context**: Add keyboard shortcuts reference panel and verify rebindable shortcuts. **Decision**:
Existing `shortcuts/config.rs` already loads `shortcuts.toml` with `apply_config_overrides` — no
separate `keymap.toml` needed, the functionality is identical. SC-4 is satisfied by the existing
shortcuts plugin. **Decision**: Keyboard shortcuts data cached on `EditorState.shortcut_entries`
rather than passing `ShortcutRegistry` through the 16-parameter dock system. Populated by
`restore_shortcuts` on editor entry. **Decision**: `DockTab::Shortcuts` added to MapEditing,
UnitDesign, and RuleAuthoring layouts (alongside Settings tab). Not in Playtesting (minimal panels).
**Abstraction check**: No abstraction needed — the reference panel is a simple read-only list
renderer with no reuse potential.

### 2026-02-24 — Scope 3+4

- 411 tests pass (full suite, 5 new theme tests, no regressions)
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
| 2026-02-24 | building | Scope 3 complete — custom themes               |
| 2026-02-24 | building | Scope 4 complete — rebindable shortcuts        |
