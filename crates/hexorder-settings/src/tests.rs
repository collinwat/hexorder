use bevy::prelude::*;

use crate::SettingsLayers;
use crate::config::{
    PartialEditorSettings, PartialSettings, brand_theme_definition, config_dir, config_path,
    load_settings_from_path, load_themes, load_themes_from_dir, load_user_settings, merge,
};
use hexorder_contracts::persistence::{AppScreen, Workspace};
use hexorder_contracts::settings::{
    SettingsChanged, SettingsRegistry, ThemeDefinition, ThemeLibrary,
};

fn empty() -> PartialSettings {
    PartialSettings::default()
}

fn defaults() -> PartialSettings {
    PartialSettings::defaults()
}

// ---------------------------------------------------------------------------
// Merge tests (existing)
// ---------------------------------------------------------------------------

#[test]
fn merge_defaults_only() {
    let result = merge(&defaults(), &empty(), &empty());
    assert!((result.editor.font_size - 15.0).abs() < f32::EPSILON);
    assert!(result.editor.workspace_preset.is_empty());
    assert_eq!(result.active_theme, "brand");
}

#[test]
fn user_layer_overrides_defaults() {
    let user = PartialSettings {
        editor: PartialEditorSettings {
            font_size: Some(18.0),
            ..Default::default()
        },
        theme: Some("solarized".to_string()),
    };
    let result = merge(&defaults(), &user, &empty());
    assert!((result.editor.font_size - 18.0).abs() < f32::EPSILON);
    assert_eq!(result.active_theme, "solarized");
    // workspace_preset falls through to default
    assert!(result.editor.workspace_preset.is_empty());
}

#[test]
fn project_layer_overrides_user() {
    let user = PartialSettings {
        editor: PartialEditorSettings {
            font_size: Some(18.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let project = PartialSettings {
        editor: PartialEditorSettings {
            font_size: Some(20.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let result = merge(&defaults(), &user, &project);
    assert!((result.editor.font_size - 20.0).abs() < f32::EPSILON);
}

#[test]
fn project_overrides_only_specified_fields() {
    let user = PartialSettings {
        editor: PartialEditorSettings {
            font_size: Some(18.0),
            workspace_preset: Some("unit_design".to_string()),
        },
        theme: Some("solarized".to_string()),
    };
    let project = PartialSettings {
        editor: PartialEditorSettings {
            font_size: Some(20.0),
            workspace_preset: None, // not overridden
        },
        theme: None, // not overridden
    };
    let result = merge(&defaults(), &user, &project);
    assert!((result.editor.font_size - 20.0).abs() < f32::EPSILON);
    assert_eq!(result.editor.workspace_preset, "unit_design"); // from user
    assert_eq!(result.active_theme, "solarized"); // from user
}

#[test]
fn all_empty_layers_use_hardcoded_fallbacks() {
    let result = merge(&empty(), &empty(), &empty());
    assert!((result.editor.font_size - 15.0).abs() < f32::EPSILON);
    assert!(result.editor.workspace_preset.is_empty());
    assert_eq!(result.active_theme, "brand");
}

#[test]
fn partial_settings_defaults_has_all_values() {
    let d = defaults();
    assert!(d.editor.font_size.is_some());
    assert!(d.editor.workspace_preset.is_some());
    assert!(d.theme.is_some());
}

#[test]
fn partial_settings_deserializes_from_toml() {
    let toml_str = r#"
        theme = "dark"

        [editor]
        font_size = 16.5
        workspace_preset = "rule_authoring"
    "#;
    let parsed: PartialSettings = toml::from_str(toml_str).expect("should parse");
    assert!((parsed.editor.font_size.expect("set") - 16.5).abs() < f32::EPSILON);
    assert_eq!(
        parsed.editor.workspace_preset.as_deref(),
        Some("rule_authoring")
    );
    assert_eq!(parsed.theme.as_deref(), Some("dark"));
}

#[test]
fn partial_settings_deserializes_empty_toml() {
    let parsed: PartialSettings = toml::from_str("").expect("should parse empty");
    assert!(parsed.editor.font_size.is_none());
    assert!(parsed.editor.workspace_preset.is_none());
    assert!(parsed.theme.is_none());
}

#[test]
fn partial_settings_deserializes_partial_toml() {
    let toml_str = r"
        [editor]
        font_size = 12.0
    ";
    let parsed: PartialSettings = toml::from_str(toml_str).expect("should parse");
    assert!((parsed.editor.font_size.expect("set") - 12.0).abs() < f32::EPSILON);
    assert!(parsed.editor.workspace_preset.is_none());
    assert!(parsed.theme.is_none());
}

#[test]
fn settings_registry_default() {
    let reg = SettingsRegistry::default();
    assert!((reg.editor.font_size - 15.0).abs() < f32::EPSILON);
    assert!(reg.editor.workspace_preset.is_empty());
    assert_eq!(reg.active_theme, "brand");
}

// ---------------------------------------------------------------------------
// Theme tests (existing)
// ---------------------------------------------------------------------------

#[test]
fn brand_theme_has_correct_name() {
    let theme = brand_theme_definition();
    assert_eq!(theme.name, "Brand");
}

#[test]
fn brand_theme_bg_deep_matches_constant() {
    let theme = brand_theme_definition();
    // BrandTheme::BG_DEEP = from_gray(10)
    assert_eq!(theme.bg_deep, [10, 10, 10]);
}

#[test]
fn brand_theme_accent_primary_matches_teal() {
    let theme = brand_theme_definition();
    // BrandTheme::ACCENT_TEAL = from_rgb(0, 92, 128)
    assert_eq!(theme.accent_primary, [0, 92, 128]);
}

#[test]
fn brand_theme_accent_secondary_matches_amber() {
    let theme = brand_theme_definition();
    // BrandTheme::ACCENT_AMBER = from_rgb(200, 150, 64)
    assert_eq!(theme.accent_secondary, [200, 150, 64]);
}

#[test]
fn theme_definition_deserializes_from_toml() {
    let toml_str = r#"
        name = "Test Theme"
        bg_deep = [0, 0, 0]
        bg_panel = [20, 20, 20]
        bg_surface = [30, 30, 30]
        widget_inactive = [40, 40, 40]
        widget_hovered = [50, 50, 50]
        widget_active = [60, 60, 60]
        accent_primary = [0, 100, 200]
        accent_secondary = [200, 100, 0]
        text_primary = [220, 220, 220]
        text_secondary = [120, 120, 120]
        border = [50, 50, 50]
        danger = [200, 0, 0]
        success = [0, 200, 0]
    "#;
    let theme: ThemeDefinition = toml::from_str(toml_str).expect("should parse theme TOML");
    assert_eq!(theme.name, "Test Theme");
    assert_eq!(theme.accent_primary, [0, 100, 200]);
    assert_eq!(theme.danger, [200, 0, 0]);
}

// ---------------------------------------------------------------------------
// Config dir / path tests
// ---------------------------------------------------------------------------

#[test]
fn config_dir_returns_non_empty_path() {
    let dir = config_dir();
    assert!(!dir.as_os_str().is_empty());
}

#[test]
fn config_path_includes_settings_toml() {
    let path = config_path();
    assert!(
        path.to_str().unwrap_or_default().contains("settings.toml"),
        "config path should contain settings.toml"
    );
}

// ---------------------------------------------------------------------------
// load_settings_from_path tests
// ---------------------------------------------------------------------------

#[test]
fn load_settings_from_path_not_found_returns_default() {
    let path = std::env::temp_dir()
        .join("hexorder_test_settings_nonexistent")
        .join("settings.toml");
    let result = load_settings_from_path(&path);
    // Should return default (all None).
    assert!(result.editor.font_size.is_none());
    assert!(result.editor.workspace_preset.is_none());
    assert!(result.theme.is_none());
}

#[test]
fn load_settings_from_path_valid_toml() {
    let dir = std::env::temp_dir().join("hexorder_test_settings_valid");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("settings.toml");

    std::fs::write(
        &path,
        r#"
            theme = "solarized"

            [editor]
            font_size = 18.5
            workspace_preset = "playtesting"
        "#,
    )
    .expect("write temp file");

    let result = load_settings_from_path(&path);
    assert!((result.editor.font_size.expect("set") - 18.5).abs() < f32::EPSILON);
    assert_eq!(
        result.editor.workspace_preset.as_deref(),
        Some("playtesting")
    );
    assert_eq!(result.theme.as_deref(), Some("solarized"));

    // Cleanup.
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_settings_from_path_invalid_toml_returns_default() {
    let dir = std::env::temp_dir().join("hexorder_test_settings_invalid");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("settings.toml");

    std::fs::write(&path, "not valid toml {{{{").expect("write temp file");

    let result = load_settings_from_path(&path);
    // Parse error → returns default.
    assert!(result.editor.font_size.is_none());
    assert!(result.theme.is_none());

    // Cleanup.
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_settings_from_path_is_directory_returns_default() {
    // Pointing to a directory instead of a file triggers the non-NotFound Err branch.
    let dir = std::env::temp_dir().join("hexorder_test_settings_isdir");
    std::fs::create_dir_all(&dir).expect("create temp dir");

    let result = load_settings_from_path(&dir);
    // read_to_string on a directory is an I/O error (not NotFound).
    assert!(result.editor.font_size.is_none());
    assert!(result.theme.is_none());

    // Cleanup.
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_user_settings_returns_partial_settings() {
    // Exercises the thin wrapper (NotFound path in CI, or real config locally).
    let result = load_user_settings();
    // Just verify it returns without panicking and is a valid PartialSettings.
    let _ = format!("{result:?}");
}

// ---------------------------------------------------------------------------
// load_themes_from_dir tests
// ---------------------------------------------------------------------------

#[test]
fn load_themes_from_dir_not_found_returns_brand_only() {
    let dir = std::env::temp_dir()
        .join("hexorder_test_themes_nonexistent")
        .join("themes");
    let result = load_themes_from_dir(&dir);
    assert_eq!(result.themes.len(), 1);
    assert_eq!(result.themes[0].name, "Brand");
}

#[test]
fn load_themes_from_dir_empty_dir() {
    let dir = std::env::temp_dir().join("hexorder_test_themes_empty");
    std::fs::create_dir_all(&dir).expect("create temp dir");

    let result = load_themes_from_dir(&dir);
    assert_eq!(result.themes.len(), 1, "only brand theme when dir is empty");
    assert_eq!(result.themes[0].name, "Brand");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_themes_from_dir_valid_theme() {
    let dir = std::env::temp_dir().join("hexorder_test_themes_valid");
    std::fs::create_dir_all(&dir).expect("create temp dir");

    let theme_toml = r#"
        name = "Solarized"
        bg_deep = [0, 43, 54]
        bg_panel = [7, 54, 66]
        bg_surface = [14, 66, 78]
        widget_inactive = [88, 110, 117]
        widget_hovered = [101, 123, 131]
        widget_active = [131, 148, 150]
        accent_primary = [38, 139, 210]
        accent_secondary = [181, 137, 0]
        text_primary = [253, 246, 227]
        text_secondary = [147, 161, 161]
        border = [88, 110, 117]
        danger = [220, 50, 47]
        success = [133, 153, 0]
    "#;
    std::fs::write(dir.join("solarized.toml"), theme_toml).expect("write theme");

    let result = load_themes_from_dir(&dir);
    assert_eq!(result.themes.len(), 2, "brand + solarized");
    assert_eq!(result.themes[0].name, "Brand");
    assert_eq!(result.themes[1].name, "Solarized");
    assert_eq!(result.themes[1].accent_primary, [38, 139, 210]);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_themes_from_dir_invalid_theme_skipped() {
    let dir = std::env::temp_dir().join("hexorder_test_themes_invalid");
    std::fs::create_dir_all(&dir).expect("create temp dir");

    std::fs::write(dir.join("broken.toml"), "not valid {{{{").expect("write bad theme");

    let result = load_themes_from_dir(&dir);
    // Invalid TOML is skipped; only brand theme remains.
    assert_eq!(result.themes.len(), 1);
    assert_eq!(result.themes[0].name, "Brand");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_themes_from_dir_non_toml_skipped() {
    let dir = std::env::temp_dir().join("hexorder_test_themes_nontoml");
    std::fs::create_dir_all(&dir).expect("create temp dir");

    std::fs::write(dir.join("readme.txt"), "not a theme file").expect("write txt");

    let result = load_themes_from_dir(&dir);
    assert_eq!(result.themes.len(), 1, "non-.toml files are skipped");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_themes_from_dir_mixed_files() {
    let dir = std::env::temp_dir().join("hexorder_test_themes_mixed");
    std::fs::create_dir_all(&dir).expect("create temp dir");

    // Valid theme.
    let valid_toml = r#"
        name = "Dark"
        bg_deep = [0, 0, 0]
        bg_panel = [20, 20, 20]
        bg_surface = [30, 30, 30]
        widget_inactive = [40, 40, 40]
        widget_hovered = [50, 50, 50]
        widget_active = [60, 60, 60]
        accent_primary = [0, 128, 255]
        accent_secondary = [255, 180, 0]
        text_primary = [230, 230, 230]
        text_secondary = [160, 160, 160]
        border = [60, 60, 60]
        danger = [220, 50, 50]
        success = [50, 180, 50]
    "#;
    std::fs::write(dir.join("dark.toml"), valid_toml).expect("write valid");

    // Invalid theme.
    std::fs::write(dir.join("broken.toml"), "{{invalid}}").expect("write invalid");

    // Non-TOML file.
    std::fs::write(dir.join("notes.md"), "# Notes").expect("write non-toml");

    let result = load_themes_from_dir(&dir);
    // Brand theme + valid dark theme. Broken and .md are skipped.
    assert_eq!(result.themes.len(), 2);
    assert_eq!(result.themes[0].name, "Brand");
    // The valid theme should be present (exact index depends on read_dir order).
    assert!(
        result.themes.iter().any(|t| t.name == "Dark"),
        "should contain Dark theme"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_themes_from_dir_file_not_directory_returns_brand_only() {
    // Pointing read_dir at a file (not a directory) triggers the non-NotFound Err branch.
    let dir = std::env::temp_dir().join("hexorder_test_themes_notadir");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let file_path = dir.join("some_file");
    std::fs::write(&file_path, "not a directory").expect("write file");

    let result = load_themes_from_dir(&file_path);
    // read_dir on a file is an error (not NotFound) → returns brand theme only.
    assert_eq!(result.themes.len(), 1);
    assert_eq!(result.themes[0].name, "Brand");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_themes_from_dir_unreadable_toml_file() {
    // A .toml file that can't be read (permissions) triggers the read_to_string Err branch.
    let dir = std::env::temp_dir().join("hexorder_test_themes_unreadable");
    std::fs::create_dir_all(&dir).expect("create temp dir");

    let toml_path = dir.join("locked.toml");
    std::fs::write(&toml_path, "name = \"Locked\"").expect("write toml");

    // Remove read permission.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o000);
        std::fs::set_permissions(&toml_path, perms).expect("set permissions");
    }

    let result = load_themes_from_dir(&dir);

    // Restore permissions before cleanup.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o644);
        let _ = std::fs::set_permissions(&toml_path, perms);
    }

    // The unreadable file is skipped; only brand theme remains.
    assert_eq!(result.themes.len(), 1);
    assert_eq!(result.themes[0].name, "Brand");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_themes_returns_at_least_brand() {
    // Exercises the thin wrapper.
    let result = load_themes();
    assert!(!result.themes.is_empty());
    assert_eq!(result.themes[0].name, "Brand");
}

// ---------------------------------------------------------------------------
// SettingsPlugin tests (mod.rs coverage)
// ---------------------------------------------------------------------------

#[test]
fn settings_plugin_inserts_registry() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<AppScreen>();
    app.add_plugins(crate::SettingsPlugin);
    app.update();

    assert!(
        app.world().get_resource::<SettingsRegistry>().is_some(),
        "SettingsRegistry should be inserted"
    );
}

#[test]
fn settings_plugin_inserts_layers() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<AppScreen>();
    app.add_plugins(crate::SettingsPlugin);
    app.update();

    assert!(
        app.world().get_resource::<SettingsLayers>().is_some(),
        "SettingsLayers should be inserted"
    );
}

#[test]
fn settings_plugin_inserts_theme_library() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<AppScreen>();
    app.add_plugins(crate::SettingsPlugin);
    app.update();

    let lib = app.world().resource::<ThemeLibrary>();
    assert!(
        !lib.themes.is_empty(),
        "theme library should have brand theme"
    );
    assert_eq!(lib.themes[0].name, "Brand");
}

#[test]
fn settings_plugin_registry_has_default_values() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<AppScreen>();
    app.add_plugins(crate::SettingsPlugin);
    app.update();

    let reg = app.world().resource::<SettingsRegistry>();
    // Defaults layer should be merged in; font_size = 15.0 from defaults.
    assert!((reg.editor.font_size - 15.0).abs() < f32::EPSILON);
    assert_eq!(reg.active_theme, "brand");
}

// ---------------------------------------------------------------------------
// Systems tests (systems.rs coverage)
// ---------------------------------------------------------------------------

fn test_app_with_settings() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<AppScreen>();

    // Insert the three settings layers.
    let defs = PartialSettings::defaults();
    let user = PartialSettings::default();
    let project = PartialSettings::default();
    let registry = merge(&defs, &user, &project);

    app.insert_resource(SettingsLayers {
        defaults: defs,
        user,
        project,
    });
    app.insert_resource(registry);

    app
}

#[test]
fn apply_project_layer_updates_font_size() {
    let mut app = test_app_with_settings();
    app.insert_resource(Workspace {
        font_size_base: 20.0,
        workspace_preset: String::new(),
        ..Default::default()
    });

    app.add_systems(Update, super::systems::apply_project_layer);
    app.update();

    let reg = app.world().resource::<SettingsRegistry>();
    assert!(
        (reg.editor.font_size - 20.0).abs() < f32::EPSILON,
        "font_size should be updated to workspace value"
    );
}

#[test]
fn apply_project_layer_with_preset() {
    let mut app = test_app_with_settings();
    app.insert_resource(Workspace {
        font_size_base: 15.0,
        workspace_preset: "map_editing".to_string(),
        ..Default::default()
    });

    app.add_systems(Update, super::systems::apply_project_layer);
    app.update();

    let reg = app.world().resource::<SettingsRegistry>();
    assert_eq!(reg.editor.workspace_preset, "map_editing");
}

#[test]
fn apply_project_layer_empty_preset_inherits_from_lower_layer() {
    let mut app = test_app_with_settings();
    app.insert_resource(Workspace {
        font_size_base: 15.0,
        workspace_preset: String::new(),
        ..Default::default()
    });

    app.add_systems(Update, super::systems::apply_project_layer);
    app.update();

    let layers = app.world().resource::<SettingsLayers>();
    // Empty workspace_preset → project layer sets None, falling through.
    assert!(layers.project.editor.workspace_preset.is_none());
}

#[test]
fn apply_project_layer_triggers_settings_changed() {
    let mut app = test_app_with_settings();
    app.insert_resource(Workspace::default());

    let triggered = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = triggered.clone();
    app.add_observer(move |_trigger: On<SettingsChanged>| {
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    app.add_systems(Update, super::systems::apply_project_layer);
    app.update();

    assert!(
        triggered.load(std::sync::atomic::Ordering::SeqCst),
        "SettingsChanged should be triggered"
    );
}

#[test]
fn clear_project_layer_resets_to_user_and_defaults() {
    let mut app = test_app_with_settings();
    app.insert_resource(Workspace {
        font_size_base: 22.0,
        workspace_preset: "playtesting".to_string(),
        ..Default::default()
    });

    // Apply project layer first.
    app.add_systems(
        Update,
        (
            super::systems::apply_project_layer,
            super::systems::clear_project_layer,
        )
            .chain(),
    );
    app.update();

    // After clear, registry should revert to defaults (no project overrides).
    let reg = app.world().resource::<SettingsRegistry>();
    assert!(
        (reg.editor.font_size - 15.0).abs() < f32::EPSILON,
        "font_size should revert to default after clear"
    );
    assert!(
        reg.editor.workspace_preset.is_empty(),
        "workspace_preset should revert to default after clear"
    );
}

#[test]
fn clear_project_layer_triggers_settings_changed() {
    let mut app = test_app_with_settings();

    let triggered = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = triggered.clone();
    app.add_observer(move |_trigger: On<SettingsChanged>| {
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    app.add_systems(Update, super::systems::clear_project_layer);
    app.update();

    assert!(
        triggered.load(std::sync::atomic::Ordering::SeqCst),
        "SettingsChanged should be triggered on clear"
    );
}

#[test]
fn clear_project_layer_empties_project_layer() {
    let mut app = test_app_with_settings();

    // Manually set project layer to non-default.
    {
        let mut layers = app.world_mut().resource_mut::<SettingsLayers>();
        layers.project = PartialSettings {
            editor: PartialEditorSettings {
                font_size: Some(22.0),
                workspace_preset: Some("test".to_string()),
            },
            theme: Some("dark".to_string()),
        };
    }

    app.add_systems(Update, super::systems::clear_project_layer);
    app.update();

    let layers = app.world().resource::<SettingsLayers>();
    assert!(layers.project.editor.font_size.is_none());
    assert!(layers.project.editor.workspace_preset.is_none());
    assert!(layers.project.theme.is_none());
}
