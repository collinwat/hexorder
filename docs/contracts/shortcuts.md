# Contract: shortcuts

## Purpose

Defines the centralized keyboard shortcut registry, command execution events, and command palette
state shared between the shortcuts plugin (producer) and all plugins that register or respond to
commands (consumers).

## Consumers

- camera (registers pan/zoom/view shortcuts, observes commands)
- hex_grid (registers deselect shortcut, observes commands)
- persistence (registers file shortcuts, observes commands)
- editor_ui (renders command palette, registers tool/view/mode shortcuts, observes commands)

## Producers

- shortcuts (inserts registry and palette state resources, fires command events)

## Types

### Enums / Utility Types

```rust
/// Identifies a registered command.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandId(pub &'static str);

/// Modifier key flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub cmd: bool,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

/// A key combination: a primary key plus optional modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: Modifiers,
}

/// Command grouping for palette display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Camera,
    File,
    Edit,
    View,
    Tool,
    Mode,
}

/// A registered command with metadata.
#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub id: CommandId,
    pub name: String,
    pub description: String,
    pub bindings: Vec<KeyBinding>,
    pub category: CommandCategory,
    pub continuous: bool,
}
```

### Resources

```rust
/// Central registry of all commands and their bindings.
#[derive(Resource, Debug, Default)]
pub struct ShortcutRegistry {
    // Internal: commands list + binding->command lookup map
}

/// Resource controlling command palette visibility and navigation state.
#[derive(Resource, Debug, Default)]
pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
    pub selected_index: usize,
}
```

### Events

```rust
/// Observer event fired when a command is executed (via shortcut or palette).
#[derive(Event, Debug, Clone)]
pub struct CommandExecutedEvent {
    pub command_id: CommandId,
}
```

## Invariants

- `ShortcutRegistry` is inserted during `ShortcutsPlugin::build()` (immediate, before consumers)
- `CommandPaletteState` is inserted during `ShortcutsPlugin::build()` (immediate)
- `ShortcutsPlugin` must be registered before all consumer plugins in `main.rs`
- Duplicate bindings: last-registered wins, warning logged
- Continuous commands (e.g., WASD pan) register in the registry for discoverability but are not
  fired via `CommandExecutedEvent` — consumers read bindings directly

## Changelog

| Date       | Change                                        | Reason                                    |
| ---------- | --------------------------------------------- | ----------------------------------------- |
| 2026-02-16 | Initial definition                            | Pitch #80 — keyboard-first command access |
| 2026-02-16 | Add `selected_index` to `CommandPaletteState` | Palette keyboard navigation               |
