//! Shared Shortcuts types. See `docs/contracts/shortcuts.md`.
//!
//! Types for the centralized keyboard shortcut registry, command execution
//! events, and command palette state.

use std::collections::HashMap;
use std::fmt;

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Core Types
// ---------------------------------------------------------------------------

/// Identifies a registered command.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandId(pub &'static str);

impl fmt::Display for CommandId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// Modifier key flags.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub cmd: bool,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        cmd: false,
        shift: false,
        alt: false,
        ctrl: false,
    };

    pub const CMD: Self = Self {
        cmd: true,
        shift: false,
        alt: false,
        ctrl: false,
    };

    pub const CMD_SHIFT: Self = Self {
        cmd: true,
        shift: true,
        alt: false,
        ctrl: false,
    };
}

/// A key combination: a primary key plus optional modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: Modifiers,
}

impl KeyBinding {
    #[must_use]
    pub const fn new(key: KeyCode, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    /// Returns a human-readable display string (e.g., "Cmd+S", "Shift+F").
    #[must_use]
    pub fn display_string(&self) -> String {
        let mut parts = Vec::new();
        if self.modifiers.cmd {
            parts.push("\u{2318}"); // ⌘
        }
        if self.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.shift {
            parts.push("\u{21e7}"); // ⇧
        }
        parts.push(keycode_display_name(self.key));
        parts.join("+")
    }
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
    /// Whether this is a continuous (held) command vs discrete (`just_pressed`).
    pub continuous: bool,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Central registry of all commands and their bindings.
#[derive(Resource, Debug, Default)]
pub struct ShortcutRegistry {
    commands: Vec<CommandEntry>,
    /// Lookup: binding -> command ID index (for fast matching on key press).
    binding_map: HashMap<KeyBinding, usize>,
}

impl ShortcutRegistry {
    /// Register a command with its bindings. If a binding already exists,
    /// the new registration wins and a warning is logged.
    pub fn register(&mut self, entry: CommandEntry) {
        let index = self.commands.len();

        for binding in &entry.bindings {
            if let Some(&existing_idx) = self.binding_map.get(binding) {
                let existing_id = &self.commands[existing_idx].id;
                warn!(
                    "Shortcut conflict: {} overwrites {} for {:?}",
                    entry.id, existing_id, binding
                );
            }
            self.binding_map.insert(binding.clone(), index);
        }

        self.commands.push(entry);
    }

    /// Look up which command a key binding maps to.
    #[must_use]
    pub fn lookup(&self, binding: &KeyBinding) -> Option<&CommandEntry> {
        self.binding_map
            .get(binding)
            .map(|&idx| &self.commands[idx])
    }

    /// Returns all registered commands.
    #[must_use]
    pub fn commands(&self) -> &[CommandEntry] {
        &self.commands
    }

    /// Returns all key codes bound to a given command ID (ignoring modifiers).
    #[must_use]
    pub fn bindings_for(&self, command_id: &str) -> Vec<KeyCode> {
        for entry in &self.commands {
            if entry.id.0 == command_id {
                return entry.bindings.iter().map(|b| b.key).collect();
            }
        }
        Vec::new()
    }

    /// Returns true if a command's bound key is currently pressed AND
    /// the active modifier state matches the binding's required modifiers.
    ///
    /// Use this for continuous (held) commands like camera panning. For
    /// discrete commands, the `match_shortcuts` system fires
    /// `CommandExecutedEvent` automatically.
    #[must_use]
    pub fn is_pressed(&self, command_id: &str, keys: &ButtonInput<KeyCode>) -> bool {
        let current_mods = Modifiers {
            cmd: keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]),
            shift: keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
            alt: keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]),
            ctrl: keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]),
        };

        for entry in &self.commands {
            if entry.id.0 == command_id {
                return entry
                    .bindings
                    .iter()
                    .any(|b| keys.pressed(b.key) && b.modifiers == current_mods);
            }
        }
        false
    }

    /// Returns all discrete (non-continuous) commands, for command palette display.
    #[must_use]
    pub fn discrete_commands(&self) -> Vec<&CommandEntry> {
        self.commands.iter().filter(|e| !e.continuous).collect()
    }

    /// Replaces the bindings for an existing command. Used by config file
    /// loading to apply user overrides after all plugins have registered defaults.
    /// Returns `true` if the command was found and updated.
    pub fn override_bindings(&mut self, command_id: &str, new_bindings: Vec<KeyBinding>) -> bool {
        let Some(idx) = self.commands.iter().position(|e| e.id.0 == command_id) else {
            return false;
        };

        // Remove old binding_map entries that point to this command.
        self.binding_map.retain(|_, &mut cmd_idx| cmd_idx != idx);

        // Insert new bindings.
        for binding in &new_bindings {
            if let Some(&existing_idx) = self.binding_map.get(binding) {
                let existing_id = &self.commands[existing_idx].id;
                warn!(
                    "Config override conflict: {} overwrites {} for {:?}",
                    command_id, existing_id, binding
                );
            }
            self.binding_map.insert(binding.clone(), idx);
        }

        self.commands[idx].bindings = new_bindings;
        true
    }
}

/// Resource controlling command palette visibility and navigation state.
#[derive(Resource, Debug, Default)]
pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
    /// Currently highlighted row in the palette results list.
    pub selected_index: usize,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Observer event fired when a command is executed (via shortcut or palette).
#[derive(Event, Debug, Clone)]
pub struct CommandExecutedEvent {
    pub command_id: CommandId,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns a short human-readable name for a `KeyCode`.
#[must_use]
pub fn keycode_display_name(key: KeyCode) -> &'static str {
    match key {
        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",
        KeyCode::Digit0 => "0",
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",
        KeyCode::Equal => "=",
        KeyCode::Minus => "-",
        KeyCode::Escape => "Esc",
        KeyCode::ArrowUp => "\u{2191}",    // ↑
        KeyCode::ArrowDown => "\u{2193}",  // ↓
        KeyCode::ArrowLeft => "\u{2190}",  // ←
        KeyCode::ArrowRight => "\u{2192}", // →
        KeyCode::Space => "Space",
        KeyCode::Enter => "Enter",
        KeyCode::Backspace => "\u{232b}", // ⌫
        KeyCode::Delete => "\u{2326}",    // ⌦
        KeyCode::Tab => "Tab",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_command(id: &'static str, key: KeyCode, modifiers: Modifiers) -> CommandEntry {
        CommandEntry {
            id: CommandId(id),
            name: id.to_string(),
            description: String::new(),
            bindings: vec![KeyBinding::new(key, modifiers)],
            category: CommandCategory::File,
            continuous: false,
        }
    }

    #[test]
    fn command_id_display() {
        let id = CommandId("file.save");
        assert_eq!(format!("{id}"), "file.save");
    }

    #[test]
    fn key_binding_display_string_cmd_s() {
        let binding = KeyBinding::new(KeyCode::KeyS, Modifiers::CMD);
        let display = binding.display_string();
        assert!(display.contains('\u{2318}')); // ⌘
        assert!(display.contains('S'));
    }

    #[test]
    fn key_binding_display_string_cmd_shift() {
        let binding = KeyBinding::new(KeyCode::KeyZ, Modifiers::CMD_SHIFT);
        let display = binding.display_string();
        assert!(display.contains('\u{2318}')); // ⌘
        assert!(display.contains('\u{21e7}')); // ⇧
        assert!(display.contains('Z'));
    }

    #[test]
    fn key_binding_display_string_no_modifiers() {
        let binding = KeyBinding::new(KeyCode::Escape, Modifiers::NONE);
        let display = binding.display_string();
        assert_eq!(display, "Esc");
    }

    #[test]
    fn key_binding_display_string_ctrl_alt() {
        let binding = KeyBinding::new(
            KeyCode::KeyA,
            Modifiers {
                cmd: false,
                shift: false,
                alt: true,
                ctrl: true,
            },
        );
        let display = binding.display_string();
        assert!(display.contains("Ctrl"));
        assert!(display.contains("Alt"));
        assert!(display.contains('A'));
    }

    #[test]
    fn shortcut_registry_register_and_lookup() {
        let mut registry = ShortcutRegistry::default();
        registry.register(test_command("file.save", KeyCode::KeyS, Modifiers::CMD));

        let binding = KeyBinding::new(KeyCode::KeyS, Modifiers::CMD);
        let found = registry.lookup(&binding);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, CommandId("file.save"));
    }

    #[test]
    fn shortcut_registry_lookup_miss() {
        let registry = ShortcutRegistry::default();
        let binding = KeyBinding::new(KeyCode::KeyX, Modifiers::NONE);
        assert!(registry.lookup(&binding).is_none());
    }

    #[test]
    fn shortcut_registry_commands_returns_all() {
        let mut registry = ShortcutRegistry::default();
        registry.register(test_command("file.save", KeyCode::KeyS, Modifiers::CMD));
        registry.register(test_command("file.open", KeyCode::KeyO, Modifiers::CMD));
        assert_eq!(registry.commands().len(), 2);
    }

    #[test]
    fn shortcut_registry_bindings_for() {
        let mut registry = ShortcutRegistry::default();
        registry.register(test_command("file.save", KeyCode::KeyS, Modifiers::CMD));
        let bindings = registry.bindings_for("file.save");
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0], KeyCode::KeyS);
    }

    #[test]
    fn shortcut_registry_bindings_for_missing_command() {
        let registry = ShortcutRegistry::default();
        let bindings = registry.bindings_for("nonexistent");
        assert!(bindings.is_empty());
    }

    #[test]
    fn shortcut_registry_discrete_commands() {
        let mut registry = ShortcutRegistry::default();
        registry.register(test_command("file.save", KeyCode::KeyS, Modifiers::CMD));
        registry.register(CommandEntry {
            id: CommandId("camera.pan"),
            name: "Pan".to_string(),
            description: String::new(),
            bindings: vec![],
            category: CommandCategory::Camera,
            continuous: true,
        });
        let discrete = registry.discrete_commands();
        assert_eq!(discrete.len(), 1);
        assert_eq!(discrete[0].id, CommandId("file.save"));
    }

    #[test]
    fn shortcut_registry_override_bindings() {
        let mut registry = ShortcutRegistry::default();
        registry.register(test_command("file.save", KeyCode::KeyS, Modifiers::CMD));

        let new_binding = KeyBinding::new(KeyCode::KeyP, Modifiers::CMD);
        let result = registry.override_bindings("file.save", vec![new_binding.clone()]);
        assert!(result);

        // Old binding should no longer work.
        let old = KeyBinding::new(KeyCode::KeyS, Modifiers::CMD);
        assert!(registry.lookup(&old).is_none());

        // New binding should work.
        assert!(registry.lookup(&new_binding).is_some());

        // Bindings_for should reflect the change.
        let bindings = registry.bindings_for("file.save");
        assert_eq!(bindings, vec![KeyCode::KeyP]);
    }

    #[test]
    fn shortcut_registry_override_bindings_missing_command() {
        let mut registry = ShortcutRegistry::default();
        let result = registry.override_bindings("nonexistent", vec![]);
        assert!(!result);
    }

    #[test]
    fn command_palette_state_default() {
        let state = CommandPaletteState::default();
        assert!(!state.open);
        assert!(state.query.is_empty());
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn keycode_display_name_letters() {
        assert_eq!(keycode_display_name(KeyCode::KeyA), "A");
        assert_eq!(keycode_display_name(KeyCode::KeyZ), "Z");
    }

    #[test]
    fn keycode_display_name_digits() {
        assert_eq!(keycode_display_name(KeyCode::Digit0), "0");
        assert_eq!(keycode_display_name(KeyCode::Digit9), "9");
    }

    #[test]
    fn keycode_display_name_special() {
        assert_eq!(keycode_display_name(KeyCode::Space), "Space");
        assert_eq!(keycode_display_name(KeyCode::Enter), "Enter");
        assert_eq!(keycode_display_name(KeyCode::Tab), "Tab");
        assert_eq!(keycode_display_name(KeyCode::Equal), "=");
        assert_eq!(keycode_display_name(KeyCode::Minus), "-");
    }

    #[test]
    fn keycode_display_name_arrows() {
        assert_eq!(keycode_display_name(KeyCode::ArrowUp), "\u{2191}");
        assert_eq!(keycode_display_name(KeyCode::ArrowDown), "\u{2193}");
        assert_eq!(keycode_display_name(KeyCode::ArrowLeft), "\u{2190}");
        assert_eq!(keycode_display_name(KeyCode::ArrowRight), "\u{2192}");
    }

    #[test]
    fn keycode_display_name_unknown_returns_question_mark() {
        assert_eq!(keycode_display_name(KeyCode::F24), "?");
    }

    #[test]
    fn command_executed_event_construction() {
        let evt = CommandExecutedEvent {
            command_id: CommandId("test.cmd"),
        };
        assert_eq!(evt.command_id, CommandId("test.cmd"));
    }

    #[test]
    fn shortcut_registry_is_pressed_matching() {
        let mut registry = ShortcutRegistry::default();
        registry.register(CommandEntry {
            id: CommandId("camera.pan"),
            name: "Pan".to_string(),
            description: String::new(),
            bindings: vec![KeyBinding::new(KeyCode::KeyW, Modifiers::NONE)],
            category: CommandCategory::Camera,
            continuous: true,
        });

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        assert!(registry.is_pressed("camera.pan", &keys));
    }

    #[test]
    fn shortcut_registry_is_pressed_wrong_key() {
        let mut registry = ShortcutRegistry::default();
        registry.register(CommandEntry {
            id: CommandId("camera.pan"),
            name: "Pan".to_string(),
            description: String::new(),
            bindings: vec![KeyBinding::new(KeyCode::KeyW, Modifiers::NONE)],
            category: CommandCategory::Camera,
            continuous: true,
        });

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyA);
        assert!(!registry.is_pressed("camera.pan", &keys));
    }

    #[test]
    fn shortcut_registry_is_pressed_missing_command() {
        let registry = ShortcutRegistry::default();
        let keys = ButtonInput::<KeyCode>::default();
        assert!(!registry.is_pressed("nonexistent", &keys));
    }

    #[test]
    fn shortcut_registry_is_pressed_with_modifiers() {
        let mut registry = ShortcutRegistry::default();
        registry.register(CommandEntry {
            id: CommandId("file.save"),
            name: "Save".to_string(),
            description: String::new(),
            bindings: vec![KeyBinding::new(KeyCode::KeyS, Modifiers::CMD)],
            category: CommandCategory::File,
            continuous: false,
        });

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyS);
        keys.press(KeyCode::SuperLeft);
        assert!(registry.is_pressed("file.save", &keys));
    }

    #[test]
    fn shortcut_registry_is_pressed_modifier_mismatch() {
        let mut registry = ShortcutRegistry::default();
        registry.register(CommandEntry {
            id: CommandId("file.save"),
            name: "Save".to_string(),
            description: String::new(),
            bindings: vec![KeyBinding::new(KeyCode::KeyS, Modifiers::CMD)],
            category: CommandCategory::File,
            continuous: false,
        });

        // Press S without Cmd — should not match.
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyS);
        assert!(!registry.is_pressed("file.save", &keys));
    }

    #[test]
    fn shortcut_registry_register_conflict_overwrites() {
        let mut registry = ShortcutRegistry::default();
        registry.register(test_command("cmd.a", KeyCode::KeyS, Modifiers::CMD));
        // Register another command with the same binding — last wins.
        registry.register(test_command("cmd.b", KeyCode::KeyS, Modifiers::CMD));

        let binding = KeyBinding::new(KeyCode::KeyS, Modifiers::CMD);
        let found = registry.lookup(&binding).unwrap();
        assert_eq!(found.id, CommandId("cmd.b"));
    }

    #[test]
    fn shortcut_registry_override_conflict_warns() {
        let mut registry = ShortcutRegistry::default();
        registry.register(test_command("cmd.a", KeyCode::KeyS, Modifiers::CMD));
        registry.register(test_command("cmd.b", KeyCode::KeyP, Modifiers::CMD));

        // Override cmd.b to use same binding as cmd.a — should overwrite.
        let new_binding = KeyBinding::new(KeyCode::KeyS, Modifiers::CMD);
        let result = registry.override_bindings("cmd.b", vec![new_binding.clone()]);
        assert!(result);
        // Now cmd.b should own the binding.
        let found = registry.lookup(&new_binding).unwrap();
        assert_eq!(found.id, CommandId("cmd.b"));
    }

    #[test]
    fn modifiers_constants() {
        let none = Modifiers::NONE;
        assert!(!none.cmd);
        assert!(!none.shift);
        assert!(!none.alt);
        assert!(!none.ctrl);

        let cmd = Modifiers::CMD;
        assert!(cmd.cmd);
        assert!(!cmd.shift);

        let cmd_shift = Modifiers::CMD_SHIFT;
        assert!(cmd_shift.cmd);
        assert!(cmd_shift.shift);
        assert!(!cmd_shift.alt);
    }

    #[test]
    fn command_category_all_variants() {
        let cats = [
            CommandCategory::Camera,
            CommandCategory::File,
            CommandCategory::Edit,
            CommandCategory::View,
            CommandCategory::Tool,
            CommandCategory::Mode,
        ];
        for cat in cats {
            assert!(!format!("{cat:?}").is_empty());
        }
    }

    #[test]
    fn keycode_display_name_all_letters() {
        let pairs = [
            (KeyCode::KeyA, "A"),
            (KeyCode::KeyB, "B"),
            (KeyCode::KeyC, "C"),
            (KeyCode::KeyD, "D"),
            (KeyCode::KeyE, "E"),
            (KeyCode::KeyF, "F"),
            (KeyCode::KeyG, "G"),
            (KeyCode::KeyH, "H"),
            (KeyCode::KeyI, "I"),
            (KeyCode::KeyJ, "J"),
            (KeyCode::KeyK, "K"),
            (KeyCode::KeyL, "L"),
            (KeyCode::KeyM, "M"),
            (KeyCode::KeyN, "N"),
            (KeyCode::KeyO, "O"),
            (KeyCode::KeyP, "P"),
            (KeyCode::KeyQ, "Q"),
            (KeyCode::KeyR, "R"),
            (KeyCode::KeyS, "S"),
            (KeyCode::KeyT, "T"),
            (KeyCode::KeyU, "U"),
            (KeyCode::KeyV, "V"),
            (KeyCode::KeyW, "W"),
            (KeyCode::KeyX, "X"),
            (KeyCode::KeyY, "Y"),
            (KeyCode::KeyZ, "Z"),
        ];
        for (key, expected) in pairs {
            assert_eq!(keycode_display_name(key), expected);
        }
    }

    #[test]
    fn keycode_display_name_all_digits() {
        let pairs = [
            (KeyCode::Digit0, "0"),
            (KeyCode::Digit1, "1"),
            (KeyCode::Digit2, "2"),
            (KeyCode::Digit3, "3"),
            (KeyCode::Digit4, "4"),
            (KeyCode::Digit5, "5"),
            (KeyCode::Digit6, "6"),
            (KeyCode::Digit7, "7"),
            (KeyCode::Digit8, "8"),
            (KeyCode::Digit9, "9"),
        ];
        for (key, expected) in pairs {
            assert_eq!(keycode_display_name(key), expected);
        }
    }

    #[test]
    fn keycode_display_name_backspace_delete() {
        assert_eq!(keycode_display_name(KeyCode::Backspace), "\u{232b}");
        assert_eq!(keycode_display_name(KeyCode::Delete), "\u{2326}");
    }
}
