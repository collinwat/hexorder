use crate::contracts::settings::SettingsRegistry;
use crate::settings::config::{PartialEditorSettings, PartialSettings, merge};

fn empty() -> PartialSettings {
    PartialSettings::default()
}

fn defaults() -> PartialSettings {
    PartialSettings::defaults()
}

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
