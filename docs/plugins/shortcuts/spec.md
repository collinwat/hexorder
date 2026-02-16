# Plugin: Shortcuts

## Summary

Provides a centralized keyboard shortcut registry, a Cmd+K command palette for discoverability, and
TOML-based shortcut customization. Migrates all existing scattered shortcuts to a single registry.

## Plugin

- Module: `src/shortcuts/`
- Plugin struct: `ShortcutsPlugin`
- Schedule: PreUpdate (Cmd+K intercept), Update (shortcut matching)

## Appetite

- **Size**: Small Batch (1-2 weeks)
- **Pitch**: #80

## Dependencies

- **Contracts consumed**: editor_ui (EditorTool for tool mode switching)
- **Contracts produced**: shortcuts (ShortcutRegistry, CommandExecutedEvent, CommandPaletteState)
- **Crate dependencies**: toml (config parsing), sublime_fuzzy or nucleo (fuzzy matching)

## Scope

1. [SCOPE-1] ShortcutRegistry resource with command registration API
2. [SCOPE-2] match_shortcuts system — detects key presses, fires CommandExecutedEvent
3. [SCOPE-3] Migrate persistence plugin shortcuts (Cmd+S, Cmd+Shift+S, Cmd+O, Cmd+N)
4. [SCOPE-4] Migrate camera discrete shortcuts (zoom, center, fit, reset)
5. [SCOPE-5] Migrate camera continuous shortcuts (WASD/arrow pan) to registry lookups
6. [SCOPE-6] Migrate hex_grid Escape deselect
7. [SCOPE-7] Command palette UI (Cmd+K, fuzzy search, execute on Enter/click)
8. [SCOPE-8] TOML config file loading for shortcut overrides
9. [SCOPE-9] New commands: tool switching (1/2/3), view toggles, mode switching
10. [SCOPE-10] Shortcut contract spec and code

## Success Criteria

- [ ] [SC-1] ShortcutRegistry registers commands and resolves bindings
- [ ] [SC-2] match_shortcuts fires CommandExecutedEvent for just-pressed bindings
- [ ] [SC-3] Cmd+S/O/N still trigger save/open/new via registry (regression)
- [ ] [SC-4] Camera zoom/center/fit/reset work via CommandExecutedEvent observers
- [ ] [SC-5] WASD pan reads bound keys from registry (customizable)
- [ ] [SC-6] Escape deselect works via CommandExecutedEvent observer
- [ ] [SC-7] Cmd+K opens palette; typing filters commands; Enter executes; Esc closes
- [ ] [SC-8] Missing config file uses defaults; invalid entries log warnings
- [ ] [SC-9] Number keys 1/2/3 switch tool modes
- [ ] [SC-10] shortcuts contract spec matches code in src/contracts/shortcuts.rs
- [ ] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [ ] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [ ] [SC-TEST] `cargo test` passes (all tests, not just this plugin's)
- [ ] [SC-BOUNDARY] No imports from other plugins' internals

## UAT Checklist

- [ ] [UAT-1] Launch app, press Cmd+K — command palette appears centered with search field focused
- [ ] [UAT-2] Type "sv" in palette — fuzzy match shows Save/Save As with shortcut hints; Enter saves
- [ ] [UAT-3] Press Cmd+S — file save works (regression from before shortcuts plugin)
- [ ] [UAT-4] Press 1/2/3 — tool mode switches between Select/Paint/Place
- [ ] [UAT-5] Press Escape — deselects hex (when palette not open) / closes palette (when open)

## Constraints

- No chord bindings (single key combinations only)
- No context-sensitive shortcuts (global bindings only)
- No visual shortcut editor UI (hand-edit TOML config file)
- Cmd+K must work even when egui text fields have focus

## Open Questions

- Research spike on Bevy input crates (#25) — evaluate before committing to custom registry

## Deferred Items

- Visual shortcut editor UI (#80 No Gos)
- Chord/sequence bindings (#80 No Gos)
- Context-sensitive shortcuts (#80 No Gos)
- Mouse gesture customization (#80 No Gos)
- Macro recording (#80 No Gos)
