//! TOML config file loading for shortcut overrides.

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use serde::Deserialize;

use hexorder_contracts::shortcuts::{KeyBinding, Modifiers, ShortcutRegistry};

/// TOML file structure for shortcut overrides.
#[derive(Deserialize, Debug)]
struct ShortcutConfig {
    #[serde(default)]
    bindings: HashMap<String, toml::Value>,
}

/// Returns the config directory based on compile-time feature flags.
fn config_dir() -> PathBuf {
    #[cfg(feature = "macos-app")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join("Library/Application Support/hexorder")
    }

    #[cfg(not(feature = "macos-app"))]
    {
        PathBuf::from("config")
    }
}

/// Returns the full path to the shortcuts config file.
fn config_path() -> PathBuf {
    config_dir().join("shortcuts.toml")
}

/// Startup system: reads the TOML config file and applies binding overrides.
pub fn apply_config_overrides(mut registry: ResMut<ShortcutRegistry>) {
    let path = config_path();

    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!("No shortcut config at {}, using defaults", path.display());
            return;
        }
        Err(e) => {
            warn!("Failed to read shortcut config {}: {e}", path.display());
            return;
        }
    };

    let config: ShortcutConfig = match toml::from_str(&contents) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to parse shortcut config {}: {e}", path.display());
            return;
        }
    };

    let mut applied = 0;
    for (command_id, value) in &config.bindings {
        let binding_strings = match value {
            toml::Value::String(s) => vec![s.clone()],
            toml::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            _ => {
                warn!(
                    "Shortcut config: invalid value type for '{}', expected string or array",
                    command_id
                );
                continue;
            }
        };

        let bindings: Vec<KeyBinding> = binding_strings
            .iter()
            .filter(|s| !s.is_empty()) // empty string = unbind
            .filter_map(|s| {
                if let Some(b) = parse_binding(s) {
                    Some(b)
                } else {
                    warn!(
                        "Shortcut config: invalid binding '{}' for '{}'",
                        s, command_id
                    );
                    None
                }
            })
            .collect();

        if registry.override_bindings(command_id, bindings) {
            applied += 1;
        } else {
            warn!(
                "Shortcut config: unknown command '{}', skipping",
                command_id
            );
        }
    }

    if applied > 0 {
        info!(
            "Applied {} shortcut override(s) from {}",
            applied,
            path.display()
        );
    }
}

/// Parses a binding string like `cmd+key_s` into a [`KeyBinding`].
pub fn parse_binding(s: &str) -> Option<KeyBinding> {
    let parts: Vec<&str> = s.split('+').map(str::trim).collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::NONE;
    let mut key_part = None;

    for part in &parts {
        match part.to_lowercase().as_str() {
            "cmd" | "super" => modifiers.cmd = true,
            "shift" => modifiers.shift = true,
            "alt" | "option" => modifiers.alt = true,
            "ctrl" | "control" => modifiers.ctrl = true,
            _ => {
                // Last non-modifier part is the key.
                if key_part.is_some() {
                    return None; // Multiple non-modifier parts.
                }
                key_part = Some(*part);
            }
        }
    }

    let key_name = key_part?;
    let key = key_name_to_keycode(key_name)?;
    Some(KeyBinding::new(key, modifiers))
}

/// Maps a lowercase key name string to a Bevy `KeyCode`.
fn key_name_to_keycode(name: &str) -> Option<KeyCode> {
    match name.to_lowercase().as_str() {
        // Letters
        "key_a" | "a" => Some(KeyCode::KeyA),
        "key_b" | "b" => Some(KeyCode::KeyB),
        "key_c" | "c" => Some(KeyCode::KeyC),
        "key_d" | "d" => Some(KeyCode::KeyD),
        "key_e" | "e" => Some(KeyCode::KeyE),
        "key_f" | "f" => Some(KeyCode::KeyF),
        "key_g" | "g" => Some(KeyCode::KeyG),
        "key_h" | "h" => Some(KeyCode::KeyH),
        "key_i" | "i" => Some(KeyCode::KeyI),
        "key_j" | "j" => Some(KeyCode::KeyJ),
        "key_k" | "k" => Some(KeyCode::KeyK),
        "key_l" | "l" => Some(KeyCode::KeyL),
        "key_m" | "m" => Some(KeyCode::KeyM),
        "key_n" | "n" => Some(KeyCode::KeyN),
        "key_o" | "o" => Some(KeyCode::KeyO),
        "key_p" | "p" => Some(KeyCode::KeyP),
        "key_q" | "q" => Some(KeyCode::KeyQ),
        "key_r" | "r" => Some(KeyCode::KeyR),
        "key_s" | "s" => Some(KeyCode::KeyS),
        "key_t" | "t" => Some(KeyCode::KeyT),
        "key_u" | "u" => Some(KeyCode::KeyU),
        "key_v" | "v" => Some(KeyCode::KeyV),
        "key_w" | "w" => Some(KeyCode::KeyW),
        "key_x" | "x" => Some(KeyCode::KeyX),
        "key_y" | "y" => Some(KeyCode::KeyY),
        "key_z" | "z" => Some(KeyCode::KeyZ),
        // Digits
        "digit0" | "0" => Some(KeyCode::Digit0),
        "digit1" | "1" => Some(KeyCode::Digit1),
        "digit2" | "2" => Some(KeyCode::Digit2),
        "digit3" | "3" => Some(KeyCode::Digit3),
        "digit4" | "4" => Some(KeyCode::Digit4),
        "digit5" | "5" => Some(KeyCode::Digit5),
        "digit6" | "6" => Some(KeyCode::Digit6),
        "digit7" | "7" => Some(KeyCode::Digit7),
        "digit8" | "8" => Some(KeyCode::Digit8),
        "digit9" | "9" => Some(KeyCode::Digit9),
        // Symbols
        "equal" | "=" => Some(KeyCode::Equal),
        "minus" | "-" => Some(KeyCode::Minus),
        // Navigation
        "escape" | "esc" => Some(KeyCode::Escape),
        "arrow_up" | "up" => Some(KeyCode::ArrowUp),
        "arrow_down" | "down" => Some(KeyCode::ArrowDown),
        "arrow_left" | "left" => Some(KeyCode::ArrowLeft),
        "arrow_right" | "right" => Some(KeyCode::ArrowRight),
        "space" => Some(KeyCode::Space),
        "enter" | "return" => Some(KeyCode::Enter),
        "backspace" => Some(KeyCode::Backspace),
        "delete" => Some(KeyCode::Delete),
        "tab" => Some(KeyCode::Tab),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_key() {
        let b = parse_binding("key_s").expect("should parse");
        assert_eq!(b.key, KeyCode::KeyS);
        assert_eq!(b.modifiers, Modifiers::NONE);
    }

    #[test]
    fn parse_short_key_name() {
        let b = parse_binding("s").expect("should parse");
        assert_eq!(b.key, KeyCode::KeyS);
    }

    #[test]
    fn parse_with_modifier() {
        let b = parse_binding("cmd+key_s").expect("should parse");
        assert_eq!(b.key, KeyCode::KeyS);
        assert!(b.modifiers.cmd);
        assert!(!b.modifiers.shift);
    }

    #[test]
    fn parse_multiple_modifiers() {
        let b = parse_binding("cmd+shift+key_z").expect("should parse");
        assert_eq!(b.key, KeyCode::KeyZ);
        assert!(b.modifiers.cmd);
        assert!(b.modifiers.shift);
        assert!(!b.modifiers.alt);
    }

    #[test]
    fn parse_arrow_key() {
        let b = parse_binding("arrow_up").expect("should parse");
        assert_eq!(b.key, KeyCode::ArrowUp);
    }

    #[test]
    fn parse_escape() {
        let b = parse_binding("escape").expect("should parse");
        assert_eq!(b.key, KeyCode::Escape);
    }

    #[test]
    fn parse_digit() {
        let b = parse_binding("digit1").expect("should parse");
        assert_eq!(b.key, KeyCode::Digit1);
    }

    #[test]
    fn parse_invalid_key_returns_none() {
        assert!(parse_binding("nonexistent_key").is_none());
    }

    #[test]
    fn parse_empty_returns_none() {
        assert!(parse_binding("").is_none());
    }

    #[test]
    fn parse_case_insensitive() {
        let b = parse_binding("CMD+Key_S").expect("should parse");
        assert_eq!(b.key, KeyCode::KeyS);
        assert!(b.modifiers.cmd);
    }

    #[test]
    fn override_replaces_bindings() {
        let mut registry = ShortcutRegistry::default();
        registry.register(hexorder_contracts::shortcuts::CommandEntry {
            id: hexorder_contracts::shortcuts::CommandId("test.cmd"),
            name: "Test".to_string(),
            description: String::new(),
            bindings: vec![KeyBinding::new(KeyCode::KeyA, Modifiers::NONE)],
            category: hexorder_contracts::shortcuts::CommandCategory::Edit,
            continuous: false,
        });

        // Original binding works.
        assert!(
            registry
                .lookup(&KeyBinding::new(KeyCode::KeyA, Modifiers::NONE))
                .is_some()
        );

        // Override to KeyB.
        let replaced = registry.override_bindings(
            "test.cmd",
            vec![KeyBinding::new(KeyCode::KeyB, Modifiers::NONE)],
        );
        assert!(replaced);

        // Old binding gone, new binding works.
        assert!(
            registry
                .lookup(&KeyBinding::new(KeyCode::KeyA, Modifiers::NONE))
                .is_none()
        );
        assert!(
            registry
                .lookup(&KeyBinding::new(KeyCode::KeyB, Modifiers::NONE))
                .is_some()
        );
    }

    #[test]
    fn override_unknown_command_returns_false() {
        let registry = ShortcutRegistry::default();
        // Need mutable for override_bindings but registry is empty.
        let mut registry = registry;
        assert!(!registry.override_bindings("nonexistent", vec![]));
    }

    #[test]
    fn override_with_empty_unbinds() {
        let mut registry = ShortcutRegistry::default();
        registry.register(hexorder_contracts::shortcuts::CommandEntry {
            id: hexorder_contracts::shortcuts::CommandId("test.cmd"),
            name: "Test".to_string(),
            description: String::new(),
            bindings: vec![KeyBinding::new(KeyCode::KeyA, Modifiers::NONE)],
            category: hexorder_contracts::shortcuts::CommandCategory::Edit,
            continuous: false,
        });

        // Unbind by passing empty vec.
        let replaced = registry.override_bindings("test.cmd", vec![]);
        assert!(replaced);

        // Binding removed.
        assert!(
            registry
                .lookup(&KeyBinding::new(KeyCode::KeyA, Modifiers::NONE))
                .is_none()
        );
        // Command still exists but with no bindings.
        assert_eq!(registry.commands()[0].bindings.len(), 0);
    }
}
