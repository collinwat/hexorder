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

    let applied = apply_overrides_from_str(&mut registry, &contents);
    if applied > 0 {
        info!(
            "Applied {applied} shortcut override(s) from {}",
            path.display()
        );
    }
}

/// Parse TOML shortcut config and apply overrides to the registry.
/// Returns the number of successfully applied overrides.
#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn apply_overrides_from_str(registry: &mut ShortcutRegistry, contents: &str) -> usize {
    let config: ShortcutConfig = match toml::from_str(contents) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to parse shortcut config: {e}");
            return 0;
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

    applied
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

    // -----------------------------------------------------------------------
    // parse_binding: modifier coverage
    // -----------------------------------------------------------------------

    #[test]
    fn parse_alt_modifier() {
        let b = parse_binding("alt+key_a").expect("should parse");
        assert_eq!(b.key, KeyCode::KeyA);
        assert!(b.modifiers.alt);
        assert!(!b.modifiers.cmd);
    }

    #[test]
    fn parse_option_modifier_alias() {
        let b = parse_binding("option+key_b").expect("should parse");
        assert_eq!(b.key, KeyCode::KeyB);
        assert!(b.modifiers.alt);
    }

    #[test]
    fn parse_ctrl_modifier() {
        let b = parse_binding("ctrl+key_c").expect("should parse");
        assert_eq!(b.key, KeyCode::KeyC);
        assert!(b.modifiers.ctrl);
        assert!(!b.modifiers.cmd);
    }

    #[test]
    fn parse_control_modifier_alias() {
        let b = parse_binding("control+key_d").expect("should parse");
        assert_eq!(b.key, KeyCode::KeyD);
        assert!(b.modifiers.ctrl);
    }

    #[test]
    fn parse_super_modifier_alias() {
        let b = parse_binding("super+key_e").expect("should parse");
        assert_eq!(b.key, KeyCode::KeyE);
        assert!(b.modifiers.cmd);
    }

    #[test]
    fn parse_all_modifiers() {
        let b = parse_binding("cmd+shift+alt+ctrl+key_f").expect("should parse");
        assert_eq!(b.key, KeyCode::KeyF);
        assert!(b.modifiers.cmd);
        assert!(b.modifiers.shift);
        assert!(b.modifiers.alt);
        assert!(b.modifiers.ctrl);
    }

    #[test]
    fn parse_multiple_non_modifier_parts_returns_none() {
        // Two non-modifier parts should fail.
        assert!(parse_binding("key_a+key_b").is_none());
    }

    // -----------------------------------------------------------------------
    // key_name_to_keycode: extended key coverage
    // -----------------------------------------------------------------------

    #[test]
    fn key_name_all_letters() {
        for (name, expected) in [
            ("a", KeyCode::KeyA),
            ("b", KeyCode::KeyB),
            ("c", KeyCode::KeyC),
            ("d", KeyCode::KeyD),
            ("e", KeyCode::KeyE),
            ("f", KeyCode::KeyF),
            ("g", KeyCode::KeyG),
            ("h", KeyCode::KeyH),
            ("i", KeyCode::KeyI),
            ("j", KeyCode::KeyJ),
            ("k", KeyCode::KeyK),
            ("l", KeyCode::KeyL),
            ("m", KeyCode::KeyM),
            ("n", KeyCode::KeyN),
            ("o", KeyCode::KeyO),
            ("p", KeyCode::KeyP),
            ("q", KeyCode::KeyQ),
            ("r", KeyCode::KeyR),
            ("s", KeyCode::KeyS),
            ("t", KeyCode::KeyT),
            ("u", KeyCode::KeyU),
            ("v", KeyCode::KeyV),
            ("w", KeyCode::KeyW),
            ("x", KeyCode::KeyX),
            ("y", KeyCode::KeyY),
            ("z", KeyCode::KeyZ),
        ] {
            let b = parse_binding(name).unwrap_or_else(|| panic!("should parse '{name}'"));
            assert_eq!(b.key, expected, "key for '{name}'");
        }
    }

    #[test]
    fn key_name_all_digits() {
        for (name, expected) in [
            ("0", KeyCode::Digit0),
            ("1", KeyCode::Digit1),
            ("2", KeyCode::Digit2),
            ("3", KeyCode::Digit3),
            ("4", KeyCode::Digit4),
            ("5", KeyCode::Digit5),
            ("6", KeyCode::Digit6),
            ("7", KeyCode::Digit7),
            ("8", KeyCode::Digit8),
            ("9", KeyCode::Digit9),
        ] {
            let b = parse_binding(name).unwrap_or_else(|| panic!("should parse '{name}'"));
            assert_eq!(b.key, expected, "key for '{name}'");
        }
    }

    #[test]
    fn key_name_symbols_and_navigation() {
        for (name, expected) in [
            ("equal", KeyCode::Equal),
            ("=", KeyCode::Equal),
            ("-", KeyCode::Minus),
            ("minus", KeyCode::Minus),
            ("esc", KeyCode::Escape),
            ("up", KeyCode::ArrowUp),
            ("down", KeyCode::ArrowDown),
            ("left", KeyCode::ArrowLeft),
            ("right", KeyCode::ArrowRight),
            ("space", KeyCode::Space),
            ("enter", KeyCode::Enter),
            ("return", KeyCode::Enter),
            ("backspace", KeyCode::Backspace),
            ("delete", KeyCode::Delete),
            ("tab", KeyCode::Tab),
        ] {
            let b = parse_binding(name).unwrap_or_else(|| panic!("should parse '{name}'"));
            assert_eq!(b.key, expected, "key for '{name}'");
        }
    }

    #[test]
    fn key_name_unknown_returns_none() {
        assert!(key_name_to_keycode("potato").is_none());
    }

    // -----------------------------------------------------------------------
    // apply_config_overrides: integration tests with temp config
    // -----------------------------------------------------------------------

    #[test]
    fn config_dir_returns_path() {
        let dir = config_dir();
        // Just verify it returns something (exact value depends on feature flags).
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn config_path_includes_shortcuts_toml() {
        let path = config_path();
        assert!(
            path.to_str().unwrap_or_default().contains("shortcuts.toml"),
            "config path should contain shortcuts.toml"
        );
    }

    #[test]
    fn apply_config_overrides_noop_when_no_file() {
        // Without a config file at the expected path, apply should be a no-op.
        // This exercises the NotFound branch.
        use bevy::prelude::*;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ShortcutRegistry>();
        app.add_systems(Startup, apply_config_overrides);
        app.update();

        // Registry should still be empty (default).
        let registry = app.world().resource::<ShortcutRegistry>();
        assert!(registry.commands().is_empty());
    }

    // -----------------------------------------------------------------------
    // apply_overrides_from_str: TOML parsing and application
    // -----------------------------------------------------------------------

    fn test_registry() -> ShortcutRegistry {
        let mut r = ShortcutRegistry::default();
        r.register(hexorder_contracts::shortcuts::CommandEntry {
            id: hexorder_contracts::shortcuts::CommandId("test.save"),
            name: "Save".to_string(),
            description: String::new(),
            bindings: vec![KeyBinding::new(KeyCode::KeyS, Modifiers::CMD)],
            category: hexorder_contracts::shortcuts::CommandCategory::File,
            continuous: false,
        });
        r.register(hexorder_contracts::shortcuts::CommandEntry {
            id: hexorder_contracts::shortcuts::CommandId("test.undo"),
            name: "Undo".to_string(),
            description: String::new(),
            bindings: vec![KeyBinding::new(KeyCode::KeyZ, Modifiers::CMD)],
            category: hexorder_contracts::shortcuts::CommandCategory::Edit,
            continuous: false,
        });
        r
    }

    #[test]
    fn apply_overrides_single_string_binding() {
        let mut registry = test_registry();
        let toml = r#"
[bindings]
"test.save" = "ctrl+key_s"
"#;
        let applied = apply_overrides_from_str(&mut registry, toml);
        assert_eq!(applied, 1);

        // Old binding replaced.
        assert!(
            registry
                .lookup(&KeyBinding::new(KeyCode::KeyS, Modifiers::CMD))
                .is_none()
        );
        // New binding works.
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        };
        assert!(
            registry
                .lookup(&KeyBinding::new(KeyCode::KeyS, ctrl))
                .is_some()
        );
    }

    #[test]
    fn apply_overrides_array_bindings() {
        let mut registry = test_registry();
        let toml = r#"
[bindings]
"test.save" = ["ctrl+key_s", "cmd+shift+key_s"]
"#;
        let applied = apply_overrides_from_str(&mut registry, toml);
        assert_eq!(applied, 1);

        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        };
        assert!(
            registry
                .lookup(&KeyBinding::new(KeyCode::KeyS, ctrl))
                .is_some()
        );
        let cmd_shift = Modifiers {
            cmd: true,
            shift: true,
            ..Modifiers::NONE
        };
        assert!(
            registry
                .lookup(&KeyBinding::new(KeyCode::KeyS, cmd_shift))
                .is_some()
        );
    }

    #[test]
    fn apply_overrides_empty_string_unbinds() {
        let mut registry = test_registry();
        let toml = r#"
[bindings]
"test.save" = ""
"#;
        let applied = apply_overrides_from_str(&mut registry, toml);
        assert_eq!(applied, 1);

        // Old binding gone, no new binding.
        assert!(
            registry
                .lookup(&KeyBinding::new(KeyCode::KeyS, Modifiers::CMD))
                .is_none()
        );
    }

    #[test]
    fn apply_overrides_invalid_toml_returns_zero() {
        let mut registry = test_registry();
        let applied = apply_overrides_from_str(&mut registry, "{{not valid toml}}");
        assert_eq!(applied, 0);
    }

    #[test]
    fn apply_overrides_unknown_command_skipped() {
        let mut registry = test_registry();
        let toml = r#"
[bindings]
"nonexistent.command" = "key_a"
"#;
        let applied = apply_overrides_from_str(&mut registry, toml);
        assert_eq!(applied, 0);
    }

    #[test]
    fn apply_overrides_invalid_value_type_skipped() {
        let mut registry = test_registry();
        let toml = r#"
[bindings]
"test.save" = 42
"#;
        let applied = apply_overrides_from_str(&mut registry, toml);
        assert_eq!(applied, 0);
    }

    #[test]
    fn apply_overrides_invalid_binding_string_skipped() {
        let mut registry = test_registry();
        let toml = r#"
[bindings]
"test.save" = "not_a_real_key"
"#;
        let applied = apply_overrides_from_str(&mut registry, toml);
        // Override is applied (binding is replaced) but with empty vec
        // since the invalid binding is filtered out.
        assert_eq!(applied, 1);
    }

    #[test]
    fn apply_overrides_multiple_commands() {
        let mut registry = test_registry();
        let toml = r#"
[bindings]
"test.save" = "ctrl+key_s"
"test.undo" = "ctrl+key_z"
"#;
        let applied = apply_overrides_from_str(&mut registry, toml);
        assert_eq!(applied, 2);
    }

    #[test]
    fn apply_overrides_empty_bindings_section() {
        let mut registry = test_registry();
        let toml = r"
[bindings]
";
        let applied = apply_overrides_from_str(&mut registry, toml);
        assert_eq!(applied, 0);
    }
}
