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

    /// Returns all bindings for a given command ID.
    #[must_use]
    pub fn bindings_for(&self, command_id: &str) -> Vec<KeyCode> {
        for entry in &self.commands {
            if entry.id.0 == command_id {
                return entry.bindings.iter().map(|b| b.key).collect();
            }
        }
        Vec::new()
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
