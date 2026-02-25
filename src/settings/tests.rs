use crate::settings::config::{
    PartialEditorSettings, PartialSettings, brand_theme_definition, merge,
};
use hexorder_contracts::settings::{SettingsRegistry, ThemeDefinition};

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

// ---------------------------------------------------------------------------
// Theme tests
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
