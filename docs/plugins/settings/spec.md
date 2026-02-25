# Plugin: Settings

## Summary

Provides a layered settings infrastructure that merges defaults, user config, and project overrides
into a single typed registry. Enables custom themes and rebindable shortcuts without hardcoding
preferences.

## Plugin

- Module: `src/settings/`
- Plugin struct: `SettingsPlugin`
- Schedule: `Startup` (load config files), `Update` (react to changes)

## Appetite

- **Size**: Big Batch (full cycle)
- **Pitch**: #173

## Dependencies

- **Contracts consumed**: `persistence` (Workspace resource for project-level overrides),
  `shortcuts` (ShortcutRegistry for binding overrides), `editor_ui` (BrandTheme for default theme)
- **Contracts produced**: `settings` (SettingsRegistry, SettingsChanged, ThemeDefinition)
- **Crate dependencies**: `toml` (already in deps), `dirs` (already in deps)

## Scope

1. Settings infrastructure — SettingsRegistry resource, TOML parsing, three-layer merge,
   SettingsPlugin
2. Preference migration — move font_size and workspace_preset to settings, backward compat
3. Custom themes — ThemeDefinition struct, theme loading, View menu dropdown, brand palette as
   default
4. Rebindable shortcuts — keymap.toml loading, ShortcutRegistry override merge, shortcuts reference
   panel
5. Settings contract — shared types in src/contracts/settings.rs and docs/contracts/settings.md

## Success Criteria

- [ ] [SC-1] SettingsRegistry loads and merges three layers (defaults, user TOML, project overrides)
- [ ] [SC-2] font_size and workspace_preset migrate to settings without breaking existing project
      files
- [ ] [SC-3] Custom themes load from ~/.config/hexorder/themes/ and apply to egui Visuals
- [ ] [SC-4] Shortcut overrides load from keymap.toml and merge into ShortcutRegistry
- [ ] [SC-5] Keyboard shortcuts reference panel displays current bindings
- [ ] [SC-6] Settings contract types in src/contracts/settings.rs with matching spec
- [ ] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [ ] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [ ] [SC-TEST] `cargo test` passes (all tests, not just this plugin's)
- [ ] [SC-BOUNDARY] No imports from other plugins' internals — all cross-plugin types come from
      `crate::contracts::`

## UAT Checklist

- [ ] [UAT-1] Launch app, verify font size and workspace preset persist across project open/save/new
      cycles
- [ ] [UAT-2] Place a custom theme TOML in themes dir, restart, verify theme appears in View menu
      and applies correctly
- [ ] [UAT-3] Place a keymap.toml with overrides, restart, verify shortcuts work with new bindings
- [ ] [UAT-4] Open Keyboard Shortcuts panel from View menu, verify all registered commands display
      with current bindings

## Constraints

- No in-app settings editor UI (TOML file editing only)
- No hot-reload of user config files (restart required)
- ThemeDefinition limited to ~10 high-impact color fields, not full egui Visuals serialization
- Existing project files must load without errors (serde defaults for backward compat)

## Open Questions

- [None yet]

## Deferred Items

- [None yet]
