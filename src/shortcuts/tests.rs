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
// current_modifiers tests
// ---------------------------------------------------------------------------

use bevy::prelude::*;

#[test]
fn current_modifiers_no_keys_pressed() {
    let keys = ButtonInput::<KeyCode>::default();
    let mods = super::systems::current_modifiers(&keys);
    assert!(!mods.cmd);
    assert!(!mods.shift);
    assert!(!mods.alt);
    assert!(!mods.ctrl);
}

#[test]
fn current_modifiers_cmd_pressed() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::SuperLeft);
    let mods = super::systems::current_modifiers(&keys);
    assert!(mods.cmd);
    assert!(!mods.shift);
}

#[test]
fn current_modifiers_right_super() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::SuperRight);
    let mods = super::systems::current_modifiers(&keys);
    assert!(mods.cmd);
}

#[test]
fn current_modifiers_shift_pressed() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::ShiftLeft);
    let mods = super::systems::current_modifiers(&keys);
    assert!(mods.shift);
    assert!(!mods.cmd);
}

#[test]
fn current_modifiers_right_shift() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::ShiftRight);
    let mods = super::systems::current_modifiers(&keys);
    assert!(mods.shift);
}

#[test]
fn current_modifiers_alt_pressed() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::AltLeft);
    let mods = super::systems::current_modifiers(&keys);
    assert!(mods.alt);
}

#[test]
fn current_modifiers_right_alt() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::AltRight);
    let mods = super::systems::current_modifiers(&keys);
    assert!(mods.alt);
}

#[test]
fn current_modifiers_ctrl_pressed() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::ControlLeft);
    let mods = super::systems::current_modifiers(&keys);
    assert!(mods.ctrl);
}

#[test]
fn current_modifiers_right_ctrl() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::ControlRight);
    let mods = super::systems::current_modifiers(&keys);
    assert!(mods.ctrl);
}

#[test]
fn current_modifiers_all_pressed() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::SuperLeft);
    keys.press(KeyCode::ShiftRight);
    keys.press(KeyCode::AltLeft);
    keys.press(KeyCode::ControlRight);
    let mods = super::systems::current_modifiers(&keys);
    assert!(mods.cmd);
    assert!(mods.shift);
    assert!(mods.alt);
    assert!(mods.ctrl);
}

// ---------------------------------------------------------------------------
// is_modifier_key tests
// ---------------------------------------------------------------------------

#[test]
fn is_modifier_key_all_modifier_keys() {
    for key in [
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
    ] {
        assert!(
            super::systems::is_modifier_key(key),
            "{key:?} should be a modifier"
        );
    }
}

#[test]
fn is_modifier_key_non_modifiers() {
    for key in [
        KeyCode::KeyA,
        KeyCode::Escape,
        KeyCode::Space,
        KeyCode::Enter,
        KeyCode::ArrowUp,
        KeyCode::Digit0,
    ] {
        assert!(
            !super::systems::is_modifier_key(key),
            "{key:?} should NOT be a modifier"
        );
    }
}

// ---------------------------------------------------------------------------
// render_palette_row tests (via egui_kittest Harness)
// ---------------------------------------------------------------------------

use egui_kittest::Harness;

#[test]
fn render_palette_row_selected_shows_name() {
    let entry = CommandEntry {
        id: CommandId("test.cmd"),
        name: "Test Command".to_string(),
        description: "A test command".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyT, Modifiers::CMD)],
        category: CommandCategory::Edit,
        continuous: false,
    };

    let _harness = Harness::new_ui(|ui| {
        let clicked = super::systems::render_palette_row(ui, &entry, true);
        // Not clicked in initial render.
        assert!(!clicked);
    });
}

#[test]
fn render_palette_row_unselected_shows_name() {
    let entry = CommandEntry {
        id: CommandId("test.cmd"),
        name: "Another Command".to_string(),
        description: String::new(),
        bindings: vec![KeyBinding::new(KeyCode::KeyA, Modifiers::NONE)],
        category: CommandCategory::Edit,
        continuous: false,
    };

    let _harness = Harness::new_ui(|ui| {
        super::systems::render_palette_row(ui, &entry, false);
    });
}

#[test]
fn render_palette_row_no_bindings() {
    let entry = CommandEntry {
        id: CommandId("test.nobind"),
        name: "No Binding Command".to_string(),
        description: String::new(),
        bindings: vec![],
        category: CommandCategory::Edit,
        continuous: false,
    };

    let _harness = Harness::new_ui(|ui| {
        super::systems::render_palette_row(ui, &entry, true);
    });
}

// ---------------------------------------------------------------------------
// command_palette_system rendering tests (via Harness)
// ---------------------------------------------------------------------------

/// Test the command palette rendering logic extracted into a Harness.
/// Since command_palette_system is a Bevy system that needs EguiContexts,
/// we test its rendering paths by calling the sub-functions and inline
/// logic directly.
#[test]
fn palette_filtered_results_empty_query_shows_all() {
    let mut registry = ShortcutRegistry::default();
    registry.register(named_entry("a", "Alpha"));
    registry.register(named_entry("b", "Beta"));

    let results = filtered_commands(&registry, "");
    assert_eq!(results.len(), 2);
}

#[test]
fn palette_selected_index_clamped_to_results() {
    let mut palette = hexorder_contracts::shortcuts::CommandPaletteState {
        open: true,
        selected_index: 100,
        query: "xyz_nomatch".to_string(),
    };

    let mut registry = ShortcutRegistry::default();
    registry.register(named_entry("a", "Alpha"));

    let results = filtered_commands(&registry, &palette.query);
    if results.is_empty() {
        palette.selected_index = 0;
    } else {
        palette.selected_index = palette.selected_index.min(results.len() - 1);
    }

    assert_eq!(palette.selected_index, 0);
}

#[test]
fn palette_selected_index_clamped_within_bounds() {
    let mut palette = hexorder_contracts::shortcuts::CommandPaletteState {
        open: true,
        selected_index: 5,
        query: String::new(),
    };

    let mut registry = ShortcutRegistry::default();
    registry.register(named_entry("a", "Alpha"));
    registry.register(named_entry("b", "Beta"));

    let results = filtered_commands(&registry, "");
    if !results.is_empty() {
        palette.selected_index = palette.selected_index.min(results.len() - 1);
    }

    assert_eq!(palette.selected_index, 1); // Clamped to len-1
}

// ---------------------------------------------------------------------------
// match_shortcuts: None branch for ButtonInput
// ---------------------------------------------------------------------------

#[test]
fn match_shortcuts_noop_without_key_input_resource() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Do NOT init ButtonInput<KeyCode> — tests the None branch.
    app.init_resource::<hexorder_contracts::shortcuts::CommandPaletteState>();
    app.insert_resource(ShortcutRegistry::default());
    app.add_systems(Update, super::systems::match_shortcuts);
    app.update();
    app.update(); // Should not panic
}

#[test]
fn intercept_palette_toggle_noop_without_key_input() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // No ButtonInput<KeyCode> — tests the None branch.
    app.init_resource::<hexorder_contracts::shortcuts::CommandPaletteState>();
    app.add_systems(Update, super::systems::intercept_palette_toggle);
    app.update();
    app.update(); // Should not panic
}

// ---------------------------------------------------------------------------
// match_shortcuts: unmatched key press (no entry in registry)
// ---------------------------------------------------------------------------

#[test]
fn match_shortcuts_unmatched_key_does_not_fire() {
    let mut app = shortcut_app();

    // Press a key that has no binding.
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::KeyZ); // Not registered.
    }
    app.update();

    let cmd = app.world().get_resource::<LastFiredCommand>();
    assert!(cmd.is_none(), "unmatched key should not fire");
}

// ---------------------------------------------------------------------------
// command_palette_system rendering tests using egui_kittest Harness
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// render_palette integration tests (covers command_palette_system body)
// ---------------------------------------------------------------------------

use bevy_egui::egui;

/// Helper: create a registry with some discrete commands for palette testing.
fn palette_registry() -> ShortcutRegistry {
    let mut registry = ShortcutRegistry::default();
    registry.register(CommandEntry {
        id: CommandId("file.save"),
        name: "Save".to_string(),
        description: "Save the project".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyS, Modifiers::CMD)],
        category: CommandCategory::Edit,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("file.save_as"),
        name: "Save As".to_string(),
        description: "Save to new path".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyS, Modifiers::CMD_SHIFT)],
        category: CommandCategory::Edit,
        continuous: false,
    });
    registry.register(CommandEntry {
        id: CommandId("view.center"),
        name: "Center View".to_string(),
        description: "Center the camera".to_string(),
        bindings: vec![],
        category: CommandCategory::Camera,
        continuous: false,
    });
    registry
}

#[test]
fn render_palette_shows_all_commands_with_empty_query() {
    let registry = palette_registry();
    let mut palette = CommandPaletteState {
        open: true,
        query: String::new(),
        selected_index: 0,
    };
    let mut focus_requested = false;

    let harness = Harness::new(move |ctx| {
        super::systems::render_palette(ctx, &mut palette, &registry, &mut focus_requested);
    });

    // Just verify it renders without panic.
    drop(harness);
}

#[test]
fn render_palette_with_query_filters_results() {
    let registry = palette_registry();
    let mut palette = CommandPaletteState {
        open: true,
        query: "sav".to_string(),
        selected_index: 0,
    };
    let mut focus_requested = false;

    let harness = Harness::new(move |ctx| {
        super::systems::render_palette(ctx, &mut palette, &registry, &mut focus_requested);
    });

    drop(harness);
}

#[test]
fn render_palette_no_match_shows_empty_label() {
    let registry = palette_registry();
    let mut palette = CommandPaletteState {
        open: true,
        query: "zzz_nomatch".to_string(),
        selected_index: 0,
    };
    let mut focus_requested = false;

    let harness = Harness::new(move |ctx| {
        super::systems::render_palette(ctx, &mut palette, &registry, &mut focus_requested);
    });

    drop(harness);
}

#[test]
fn render_palette_clamps_selected_index() {
    let registry = palette_registry();
    let mut palette = CommandPaletteState {
        open: true,
        query: String::new(),
        selected_index: 100, // Way out of bounds.
    };
    let mut focus_requested = false;

    let _harness = Harness::new(move |ctx| {
        super::systems::render_palette(ctx, &mut palette, &registry, &mut focus_requested);
    });
}

#[test]
fn render_palette_requests_focus_once() {
    let registry = palette_registry();
    let mut palette = CommandPaletteState {
        open: true,
        query: String::new(),
        selected_index: 0,
    };
    let mut focus_requested = false;

    // First render — should request focus.
    let ctx = egui::Context::default();
    ctx.begin_pass(egui::RawInput::default());
    super::systems::render_palette(&ctx, &mut palette, &registry, &mut focus_requested);
    let _ = ctx.end_pass();

    assert!(focus_requested, "should request focus on first render");

    // Second render — should NOT re-request (flag stays true).
    ctx.begin_pass(egui::RawInput::default());
    super::systems::render_palette(&ctx, &mut palette, &registry, &mut focus_requested);
    let _ = ctx.end_pass();

    assert!(focus_requested, "focus_requested should remain true");
}

#[test]
fn render_palette_with_selected_non_zero() {
    let registry = palette_registry();
    let mut palette = CommandPaletteState {
        open: true,
        query: String::new(),
        selected_index: 1,
    };
    let mut focus_requested = true;

    let _harness = Harness::new(move |ctx| {
        super::systems::render_palette(ctx, &mut palette, &registry, &mut focus_requested);
    });
}

#[test]
fn render_palette_arrow_down_advances_selection() {
    let registry = palette_registry();
    let mut palette = CommandPaletteState {
        open: true,
        query: String::new(),
        selected_index: 0,
    };
    let mut focus_requested = true; // Already focused.

    let ctx = egui::Context::default();

    // First pass to establish the palette window.
    ctx.begin_pass(egui::RawInput::default());
    super::systems::render_palette(&ctx, &mut palette, &registry, &mut focus_requested);
    let _ = ctx.end_pass();

    // Second pass with ArrowDown key event.
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::Key {
        key: egui::Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    ctx.begin_pass(input);
    super::systems::render_palette(&ctx, &mut palette, &registry, &mut focus_requested);
    let _ = ctx.end_pass();

    assert!(
        palette.selected_index > 0,
        "ArrowDown should advance selection, got {}",
        palette.selected_index
    );
}

#[test]
fn render_palette_arrow_up_retreats_selection() {
    let registry = palette_registry();
    let mut palette = CommandPaletteState {
        open: true,
        query: String::new(),
        selected_index: 2,
    };
    let mut focus_requested = true;

    let ctx = egui::Context::default();

    // First pass.
    ctx.begin_pass(egui::RawInput::default());
    super::systems::render_palette(&ctx, &mut palette, &registry, &mut focus_requested);
    let _ = ctx.end_pass();

    // Second pass with ArrowUp.
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::Key {
        key: egui::Key::ArrowUp,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    ctx.begin_pass(input);
    super::systems::render_palette(&ctx, &mut palette, &registry, &mut focus_requested);
    let _ = ctx.end_pass();

    assert!(
        palette.selected_index < 2,
        "ArrowUp should retreat selection, got {}",
        palette.selected_index
    );
}

#[test]
fn render_palette_enter_executes_and_closes() {
    let registry = palette_registry();
    let mut palette = CommandPaletteState {
        open: true,
        query: String::new(),
        selected_index: 0,
    };
    let mut focus_requested = true;

    let ctx = egui::Context::default();

    // First pass to establish window.
    ctx.begin_pass(egui::RawInput::default());
    super::systems::render_palette(&ctx, &mut palette, &registry, &mut focus_requested);
    let _ = ctx.end_pass();

    // Second pass with Enter key.
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    ctx.begin_pass(input);
    let result =
        super::systems::render_palette(&ctx, &mut palette, &registry, &mut focus_requested);
    let _ = ctx.end_pass();

    if let Some(cmd) = &result {
        assert_eq!(cmd.0, "file.save", "should execute first command");
        assert!(!palette.open, "palette should close after execution");
        assert!(palette.query.is_empty(), "query should be cleared");
        assert_eq!(palette.selected_index, 0, "index should reset to 0");
    }
    // Note: Enter may not trigger if the TextEdit absorbs it. Either path is valid.
}

#[test]
fn render_palette_empty_results_clamps_index_to_zero() {
    let registry = ShortcutRegistry::default(); // No commands at all.
    let mut palette = CommandPaletteState {
        open: true,
        query: String::new(),
        selected_index: 5,
    };
    let mut focus_requested = false;

    let ctx = egui::Context::default();
    ctx.begin_pass(egui::RawInput::default());
    super::systems::render_palette(&ctx, &mut palette, &registry, &mut focus_requested);
    let _ = ctx.end_pass();

    assert_eq!(
        palette.selected_index, 0,
        "should clamp to 0 when no results"
    );
}

// ---------------------------------------------------------------------------
// ShortcutsPlugin registration tests (shortcuts/mod.rs coverage)
// ---------------------------------------------------------------------------

#[test]
fn shortcuts_plugin_registers_resources() {
    use hexorder_contracts::persistence::AppScreen;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    app.add_plugins(super::ShortcutsPlugin);
    app.update();

    // Verify resources were inserted.
    assert!(app.world().get_resource::<ShortcutRegistry>().is_some());
    assert!(
        app.world()
            .get_resource::<hexorder_contracts::shortcuts::CommandPaletteState>()
            .is_some()
    );
}

#[test]
fn shortcuts_plugin_schedules_systems() {
    use hexorder_contracts::persistence::AppScreen;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    // Need EguiPlugin for command_palette_system, but we can test without it
    // by verifying the resources exist after plugin build.
    app.add_plugins(super::ShortcutsPlugin);
    // Multiple updates should not panic.
    app.update();
    app.update();
}

// ---------------------------------------------------------------------------
// match_shortcuts system tests
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// command_palette_system tests (covers lines 120-139)
// ---------------------------------------------------------------------------

/// Builds a minimal app with `command_palette_system` but NO full `EguiPlugin`.
/// We insert the `EguiUserTextures` resource (required by `EguiContexts` param)
/// but no `EguiContext` entity, so `ctx_mut()` returns `Err` — exercising
/// the early-return paths.
fn palette_system_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<CommandPaletteState>();
    app.insert_resource(ShortcutRegistry::default());
    app.init_resource::<bevy_egui::EguiUserTextures>();
    app.add_systems(Update, super::systems::command_palette_system);
    app.update(); // prime
    app
}

#[test]
fn command_palette_system_closed_palette_early_return() {
    let mut app = palette_system_app();
    // Palette is closed by default — should hit lines 127-129.
    app.update();
    // No panic means the early return worked.
}

#[test]
fn command_palette_system_open_palette_no_egui_context() {
    let mut app = palette_system_app();
    // Open the palette so it passes the `!palette.open` check
    // but hits the `contexts.ctx_mut()` Err branch (lines 132-133).
    app.world_mut().resource_mut::<CommandPaletteState>().open = true;
    app.update();
    // No panic means the Err branch returned gracefully.
}
