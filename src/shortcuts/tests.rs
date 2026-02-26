//! Unit tests for the shortcuts plugin.

use bevy::input::keyboard::KeyCode;

use hexorder_contracts::shortcuts::{
    CommandCategory, CommandEntry, CommandId, KeyBinding, Modifiers, ShortcutRegistry,
};

fn test_entry(id: &'static str, bindings: Vec<KeyBinding>) -> CommandEntry {
    CommandEntry {
        id: CommandId(id),
        name: id.to_string(),
        description: String::new(),
        bindings,
        category: CommandCategory::Edit,
        continuous: false,
    }
}

#[test]
fn registry_accepts_and_stores_commands() {
    let mut registry = ShortcutRegistry::default();
    registry.register(test_entry(
        "file.save",
        vec![KeyBinding::new(KeyCode::KeyS, Modifiers::CMD)],
    ));
    registry.register(test_entry(
        "file.open",
        vec![KeyBinding::new(KeyCode::KeyO, Modifiers::CMD)],
    ));

    assert_eq!(registry.commands().len(), 2);
}

#[test]
fn lookup_returns_correct_command() {
    let mut registry = ShortcutRegistry::default();
    registry.register(test_entry(
        "file.save",
        vec![KeyBinding::new(KeyCode::KeyS, Modifiers::CMD)],
    ));

    let binding = KeyBinding::new(KeyCode::KeyS, Modifiers::CMD);
    let result = registry.lookup(&binding);
    assert!(result.is_some());
    assert_eq!(result.expect("should exist").id.0, "file.save");
}

#[test]
fn lookup_returns_none_for_unregistered_binding() {
    let registry = ShortcutRegistry::default();
    let binding = KeyBinding::new(KeyCode::KeyX, Modifiers::NONE);
    assert!(registry.lookup(&binding).is_none());
}

#[test]
fn lookup_distinguishes_modifiers() {
    let mut registry = ShortcutRegistry::default();
    registry.register(test_entry(
        "file.save",
        vec![KeyBinding::new(KeyCode::KeyS, Modifiers::CMD)],
    ));

    // Same key, no modifier — should NOT match.
    let no_mod = KeyBinding::new(KeyCode::KeyS, Modifiers::NONE);
    assert!(registry.lookup(&no_mod).is_none());

    // Same key, with cmd — should match.
    let with_cmd = KeyBinding::new(KeyCode::KeyS, Modifiers::CMD);
    assert!(registry.lookup(&with_cmd).is_some());
}

#[test]
fn duplicate_binding_last_registered_wins() {
    let mut registry = ShortcutRegistry::default();
    registry.register(test_entry(
        "old_command",
        vec![KeyBinding::new(KeyCode::KeyS, Modifiers::CMD)],
    ));
    registry.register(test_entry(
        "new_command",
        vec![KeyBinding::new(KeyCode::KeyS, Modifiers::CMD)],
    ));

    let binding = KeyBinding::new(KeyCode::KeyS, Modifiers::CMD);
    let result = registry.lookup(&binding).expect("should exist");
    assert_eq!(result.id.0, "new_command");
}

#[test]
fn multiple_bindings_per_command() {
    let mut registry = ShortcutRegistry::default();
    registry.register(test_entry(
        "camera.pan_up",
        vec![
            KeyBinding::new(KeyCode::KeyW, Modifiers::NONE),
            KeyBinding::new(KeyCode::ArrowUp, Modifiers::NONE),
        ],
    ));

    assert_eq!(
        registry
            .lookup(&KeyBinding::new(KeyCode::KeyW, Modifiers::NONE))
            .expect("should exist")
            .id
            .0,
        "camera.pan_up"
    );
    assert_eq!(
        registry
            .lookup(&KeyBinding::new(KeyCode::ArrowUp, Modifiers::NONE))
            .expect("should exist")
            .id
            .0,
        "camera.pan_up"
    );
}

#[test]
fn bindings_for_returns_bound_keys() {
    let mut registry = ShortcutRegistry::default();
    registry.register(test_entry(
        "camera.pan_up",
        vec![
            KeyBinding::new(KeyCode::KeyW, Modifiers::NONE),
            KeyBinding::new(KeyCode::ArrowUp, Modifiers::NONE),
        ],
    ));

    let keys = registry.bindings_for("camera.pan_up");
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&KeyCode::KeyW));
    assert!(keys.contains(&KeyCode::ArrowUp));
}

#[test]
fn bindings_for_unknown_command_returns_empty() {
    let registry = ShortcutRegistry::default();
    assert!(registry.bindings_for("nonexistent").is_empty());
}

#[test]
fn discrete_commands_excludes_continuous() {
    let mut registry = ShortcutRegistry::default();

    // Discrete command.
    registry.register(test_entry(
        "file.save",
        vec![KeyBinding::new(KeyCode::KeyS, Modifiers::CMD)],
    ));

    // Continuous command.
    registry.register(CommandEntry {
        id: CommandId("camera.pan_up"),
        name: "Pan Up".to_string(),
        description: String::new(),
        bindings: vec![KeyBinding::new(KeyCode::KeyW, Modifiers::NONE)],
        category: CommandCategory::Camera,
        continuous: true,
    });

    let discrete = registry.discrete_commands();
    assert_eq!(discrete.len(), 1);
    assert_eq!(discrete[0].id.0, "file.save");
}

#[test]
fn key_binding_display_string() {
    let cmd_s = KeyBinding::new(KeyCode::KeyS, Modifiers::CMD);
    assert_eq!(cmd_s.display_string(), "\u{2318}+S"); // ⌘+S

    let cmd_shift_s = KeyBinding::new(KeyCode::KeyS, Modifiers::CMD_SHIFT);
    assert_eq!(cmd_shift_s.display_string(), "\u{2318}+\u{21e7}+S"); // ⌘+⇧+S

    let plain_w = KeyBinding::new(KeyCode::KeyW, Modifiers::NONE);
    assert_eq!(plain_w.display_string(), "W");

    let escape = KeyBinding::new(KeyCode::Escape, Modifiers::NONE);
    assert_eq!(escape.display_string(), "Esc");
}

#[test]
fn command_id_display() {
    let id = CommandId("file.save");
    assert_eq!(format!("{id}"), "file.save");
}

// ---------------------------------------------------------------------------
// Fuzzy search / palette filtering tests
// ---------------------------------------------------------------------------

use super::systems::filtered_commands;

fn named_entry(id: &'static str, name: &str) -> CommandEntry {
    CommandEntry {
        id: CommandId(id),
        name: name.to_string(),
        description: String::new(),
        bindings: Vec::new(),
        category: CommandCategory::Edit,
        continuous: false,
    }
}

#[test]
fn filtered_commands_empty_query_returns_all_discrete() {
    let mut registry = ShortcutRegistry::default();
    registry.register(named_entry("a", "Alpha"));
    registry.register(named_entry("b", "Beta"));
    registry.register(CommandEntry {
        id: CommandId("c"),
        name: "Continuous".to_string(),
        description: String::new(),
        bindings: Vec::new(),
        category: CommandCategory::Camera,
        continuous: true,
    });

    let results = filtered_commands(&registry, "");
    assert_eq!(results.len(), 2, "should exclude continuous commands");
}

#[test]
fn filtered_commands_fuzzy_match() {
    let mut registry = ShortcutRegistry::default();
    registry.register(named_entry("file.save", "Save"));
    registry.register(named_entry("file.save_as", "Save As"));
    registry.register(named_entry("file.open", "Open"));
    registry.register(named_entry("camera.center", "Center View"));

    let results = filtered_commands(&registry, "sav");
    assert_eq!(results.len(), 2, "should match Save and Save As");
    assert!(results.iter().any(|e| e.id.0 == "file.save"));
    assert!(results.iter().any(|e| e.id.0 == "file.save_as"));
}

#[test]
fn filtered_commands_no_match() {
    let mut registry = ShortcutRegistry::default();
    registry.register(named_entry("file.save", "Save"));

    let results = filtered_commands(&registry, "xyz");
    assert!(results.is_empty());
}

#[test]
fn palette_state_defaults() {
    use hexorder_contracts::shortcuts::CommandPaletteState;
    let state = CommandPaletteState::default();
    assert!(!state.open);
    assert!(state.query.is_empty());
    assert_eq!(state.selected_index, 0);
}

// ---------------------------------------------------------------------------
// match_shortcuts system tests
// ---------------------------------------------------------------------------

use bevy::prelude::*;
use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandPaletteState};

#[derive(Resource)]
struct LastFiredCommand(CommandId);

fn capture_command(trigger: On<CommandExecutedEvent>, mut cmds: Commands) {
    cmds.insert_resource(LastFiredCommand(trigger.event().command_id.clone()));
}

/// Build a minimal app with match_shortcuts system.
fn shortcut_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<CommandPaletteState>();
    let mut registry = ShortcutRegistry::default();
    registry.register(test_entry(
        "file.save",
        vec![KeyBinding::new(KeyCode::KeyS, Modifiers::CMD)],
    ));
    registry.register(CommandEntry {
        id: CommandId("camera.pan_up"),
        name: "Pan Up".to_string(),
        description: String::new(),
        bindings: vec![KeyBinding::new(KeyCode::KeyW, Modifiers::NONE)],
        category: CommandCategory::Camera,
        continuous: true,
    });
    app.insert_resource(registry);
    app.add_systems(Update, super::systems::match_shortcuts);
    app.add_observer(capture_command);
    app.update();
    app
}

#[test]
fn match_shortcuts_fires_event_on_discrete_match() {
    let mut app = shortcut_app();

    // Press Cmd+S.
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::SuperLeft);
        keys.press(KeyCode::KeyS);
    }
    app.update();

    let cmd = app.world().get_resource::<LastFiredCommand>();
    assert!(cmd.is_some(), "should fire command event");
    assert_eq!(cmd.expect("exists").0.0, "file.save");
}

#[test]
fn match_shortcuts_skips_continuous_commands() {
    let mut app = shortcut_app();

    // Press W (pan_up is continuous).
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::KeyW);
    }
    app.update();

    let cmd = app.world().get_resource::<LastFiredCommand>();
    assert!(
        cmd.is_none(),
        "should not fire event for continuous commands"
    );
}

#[test]
fn match_shortcuts_skips_when_palette_open() {
    let mut app = shortcut_app();

    // Open palette.
    app.world_mut().resource_mut::<CommandPaletteState>().open = true;

    // Press Cmd+S.
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::SuperLeft);
        keys.press(KeyCode::KeyS);
    }
    app.update();

    let cmd = app.world().get_resource::<LastFiredCommand>();
    assert!(cmd.is_none(), "should not fire when palette is open");
}

#[test]
fn match_shortcuts_ignores_modifier_keys_alone() {
    let mut app = shortcut_app();

    // Press only a modifier key.
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::SuperLeft);
    }
    app.update();

    let cmd = app.world().get_resource::<LastFiredCommand>();
    assert!(
        cmd.is_none(),
        "pressing only a modifier should not fire any command"
    );
}

// ---------------------------------------------------------------------------
// intercept_palette_toggle system tests
// ---------------------------------------------------------------------------

fn palette_toggle_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<CommandPaletteState>();
    app.add_systems(Update, super::systems::intercept_palette_toggle);
    app.update();
    app
}

#[test]
fn cmd_k_toggles_palette_open() {
    let mut app = palette_toggle_app();

    // Press Cmd+K.
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::SuperLeft);
        keys.press(KeyCode::KeyK);
    }
    app.update();

    assert!(
        app.world().resource::<CommandPaletteState>().open,
        "Cmd+K should open palette"
    );
}

#[test]
fn cmd_k_closes_open_palette() {
    let mut app = palette_toggle_app();

    app.world_mut().resource_mut::<CommandPaletteState>().open = true;
    app.world_mut().resource_mut::<CommandPaletteState>().query = "search".to_string();

    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::SuperLeft);
        keys.press(KeyCode::KeyK);
    }
    app.update();

    let palette = app.world().resource::<CommandPaletteState>();
    assert!(!palette.open, "Cmd+K should close palette");
    assert!(palette.query.is_empty(), "query should be cleared");
}

#[test]
fn escape_closes_palette() {
    let mut app = palette_toggle_app();

    app.world_mut().resource_mut::<CommandPaletteState>().open = true;

    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::Escape);
    }
    app.update();

    assert!(
        !app.world().resource::<CommandPaletteState>().open,
        "Escape should close palette"
    );
}

#[test]
fn escape_noop_when_palette_closed() {
    let mut app = palette_toggle_app();

    // Palette already closed.
    assert!(!app.world().resource::<CommandPaletteState>().open);

    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::Escape);
    }
    app.update();

    assert!(
        !app.world().resource::<CommandPaletteState>().open,
        "should stay closed"
    );
}
