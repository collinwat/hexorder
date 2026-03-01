use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};

mod macros;

mod camera;
mod cell;
mod editor_ui;
mod export;
mod game_system;
mod hex_grid;
mod map_gen;
mod mechanic_reference;
mod ontology;
mod persistence;
mod rules_engine;
mod scripting;
mod settings;
mod shortcuts;
mod undo_redo;
mod unit;

use hexorder_contracts::persistence::AppScreen;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.04)))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hexorder".to_string(),
                window_theme: Some(bevy::window::WindowTheme::Dark),
                // Start hidden to prevent OS-default white flash before
                // the GPU renders its first frame with our dark ClearColor.
                visible: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppScreen>()
        .add_plugins(shortcuts::ShortcutsPlugin)
        .add_plugins(hex_grid::HexGridPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(game_system::GameSystemPlugin)
        .add_plugins(ontology::OntologyPlugin)
        .add_plugins(cell::CellPlugin)
        .add_plugins(unit::UnitPlugin)
        .add_plugins(rules_engine::RulesEnginePlugin)
        .add_plugins(scripting::ScriptingPlugin)
        .add_plugins(persistence::PersistencePlugin)
        .add_plugins(undo_redo::UndoRedoPlugin)
        .add_plugins(map_gen::MapGenPlugin)
        .add_plugins(mechanic_reference::MechanicReferencePlugin)
        .add_plugins(export::ExportPlugin)
        .add_plugins(settings::SettingsPlugin)
        .add_plugins(editor_ui::EditorUiPlugin)
        .add_systems(Update, reveal_window)
        .run();
}

/// Reveal the hidden window after 3 frames, once the GPU has rendered
/// dark content. Runs once via `Local<bool>` guard.
fn reveal_window(
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    mut frames: Local<u32>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    *frames += 1;
    if *frames >= 3 {
        if let Ok(mut window) = windows.single_mut() {
            window.visible = true;
        }
        *done = true;
    }
}

/// Unit tests for the `reveal_window` system.
#[cfg(test)]
mod reveal_window_tests {
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;

    /// When no `PrimaryWindow` entity exists, `reveal_window` must not panic.
    /// This exercises the `Err` branch of `windows.single_mut()` (line 76-78).
    #[test]
    fn no_primary_window_does_not_panic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, super::reveal_window);

        // Run enough updates to pass the 3-frame threshold.
        for _ in 0..5 {
            app.update();
        }

        // No PrimaryWindow entity exists — the system should have silently
        // skipped the visibility toggle without panicking.
        let mut query = app
            .world_mut()
            .query_filtered::<&Window, With<PrimaryWindow>>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    /// When a `PrimaryWindow` entity exists, `reveal_window` sets it visible
    /// after 3 frames. This exercises the `Ok` branch of `windows.single_mut()`.
    #[test]
    fn reveals_window_after_three_frames() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, super::reveal_window);

        // Spawn a hidden window with PrimaryWindow marker.
        app.world_mut().spawn((
            Window {
                visible: false,
                ..default()
            },
            PrimaryWindow,
        ));

        // After 2 frames, window should still be hidden.
        app.update();
        app.update();
        {
            let mut query = app
                .world_mut()
                .query_filtered::<&Window, With<PrimaryWindow>>();
            let window = query
                .single(app.world())
                .expect("PrimaryWindow should exist");
            assert!(
                !window.visible,
                "Window should still be hidden after 2 frames"
            );
        }

        // 3rd frame triggers the reveal.
        app.update();
        {
            let mut query = app
                .world_mut()
                .query_filtered::<&Window, With<PrimaryWindow>>();
            let window = query
                .single(app.world())
                .expect("PrimaryWindow should exist");
            assert!(window.visible, "Window should be visible after 3 frames");
        }
    }

    /// After the window is revealed, additional frames should not change anything.
    #[test]
    fn reveal_window_is_idempotent_after_done() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, super::reveal_window);

        app.world_mut().spawn((
            Window {
                visible: false,
                ..default()
            },
            PrimaryWindow,
        ));

        // Frames 1-3 reveal the window; frame 4 should be a no-op.
        for _ in 0..4 {
            app.update();
        }

        let mut query = app
            .world_mut()
            .query_filtered::<&Window, With<PrimaryWindow>>();
        let window = query.single(app.world()).expect("window should exist");
        assert!(
            window.visible,
            "Window should still be visible after extra frame"
        );
    }
}

/// Architecture enforcement tests.
/// These verify structural rules that apply across the entire project.
#[cfg(test)]
mod architecture_tests {
    use std::fs;
    use std::path::Path;

    /// Scans all plugin `mod.rs` files under `src_dir` and returns violations
    /// where sub-modules are declared `pub mod`. Plugin internals must be
    /// private; shared types go through `crates/hexorder-contracts/`.
    fn find_pub_mod_violations(src_dir: &Path) -> Vec<String> {
        let mut violations = Vec::new();

        for entry in fs::read_dir(src_dir).expect("failed to read src/") {
            let entry = entry.expect("failed to read dir entry");
            let path = entry.path();

            // Skip non-directories and special directories.
            if !path.is_dir() {
                continue;
            }
            let dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            // contracts/ is intentionally public — skip it.
            if dir_name == "contracts" {
                continue;
            }

            let mod_file = path.join("mod.rs");
            if !mod_file.exists() {
                continue;
            }

            let content = fs::read_to_string(&mod_file).expect("failed to read mod.rs");

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                // Check for `pub mod <name>;` declarations (not inline modules).
                if trimmed.starts_with("pub mod ") && trimmed.ends_with(';') {
                    violations.push(format!(
                        "{}:{}: `{}` — plugin sub-modules must be private (use `mod` not `pub mod`). \
                         Shared types belong in crates/hexorder-contracts/.",
                        mod_file.display(),
                        line_num + 1,
                        trimmed,
                    ));
                }
            }
        }

        violations
    }

    #[test]
    fn plugin_modules_are_private() {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let violations = find_pub_mod_violations(&src_dir);

        assert!(
            violations.is_empty(),
            "Contract boundary violations found:\n{}",
            violations.join("\n"),
        );
    }

    #[test]
    fn find_pub_mod_violations_detects_bad_module() {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");

        // Plugin with a pub mod violation.
        let plugin_dir = tmp.path().join("fake_plugin");
        fs::create_dir(&plugin_dir).expect("failed to create plugin dir");
        fs::write(
            plugin_dir.join("mod.rs"),
            "mod private_ok;\npub mod leaked;\n",
        )
        .expect("failed to write mod.rs");

        // "contracts" directory should be skipped (not flagged).
        let contracts_dir = tmp.path().join("contracts");
        fs::create_dir(&contracts_dir).expect("failed to create contracts dir");
        fs::write(contracts_dir.join("mod.rs"), "pub mod types;\n")
            .expect("failed to write contracts mod.rs");

        // Directory without mod.rs should be skipped silently.
        let no_mod_dir = tmp.path().join("no_mod_plugin");
        fs::create_dir(&no_mod_dir).expect("failed to create no_mod dir");

        let violations = find_pub_mod_violations(tmp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("pub mod leaked;"));
    }

    /// Walks the project for `.md` files and fails if any filename contains
    /// underscores. Markdown filenames must use hyphens as word separators.
    fn walk_for_underscore_md(dir: &Path, violations: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            // Skip hidden dirs, target, node_modules.
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }

            if path.is_dir() {
                walk_for_underscore_md(&path, violations);
            } else if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                && name.contains('_')
            {
                violations.push(format!("  {}", path.display()));
            }
        }
    }

    #[test]
    fn markdown_filenames_use_hyphens() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut violations = Vec::new();

        walk_for_underscore_md(root, &mut violations);

        assert!(
            violations.is_empty(),
            "Markdown filenames must use hyphens, not underscores:\n{}",
            violations.join("\n"),
        );
    }

    #[test]
    fn walk_for_underscore_md_detects_bad_filename() {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        fs::write(tmp.path().join("bad_name.md"), "# Bad").expect("failed to write file");
        fs::write(tmp.path().join("good-name.md"), "# Good").expect("failed to write file");

        let mut violations = Vec::new();
        walk_for_underscore_md(tmp.path(), &mut violations);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("bad_name.md"));
    }

    // --- Approved brand palette constants ---

    const APPROVED_GRAYS: &[u8] = &[
        10,  // #0a0a0a — deep background (BG_DEEP)
        25,  // #191919 — panel fill (BG_PANEL)
        30,  // widget noninteractive (WIDGET_NONINTERACTIVE)
        35,  // #232323 — surface / faint bg (BG_SURFACE)
        40,  // widget inactive (WIDGET_INACTIVE)
        55,  // widget hovered (WIDGET_HOVERED)
        60,  // #3c3c3c — border (BORDER_SUBTLE)
        70,  // widget active (WIDGET_ACTIVE)
        80,  // #505050 — disabled text (TEXT_DISABLED)
        120, // tertiary text (TEXT_TERTIARY)
        128, // #808080 — secondary text (TEXT_SECONDARY)
        224, // #e0e0e0 — primary text (TEXT_PRIMARY)
    ];

    const APPROVED_RGB: &[(u8, u8, u8)] = &[
        (0, 92, 128),   // #005c80 — teal accent (ACCENT_TEAL)
        (200, 80, 80),  // #c85050 — danger red (DANGER)
        (200, 150, 64), // #c89640 — amber/gold accent (ACCENT_AMBER)
        (80, 152, 80),  // #509850 — success green (SUCCESS)
    ];

    const APPROVED_NAMED: &[&str] = &[];

    /// Scans `.rs` files in `editor_dir` for color literals and returns
    /// violations where a color is not in the approved brand palette.
    ///
    /// Exempt patterns:
    /// - Color conversion utilities (functions that transform values, not define them)
    /// - Dynamic colors constructed from variables (e.g., user-picked colors)
    /// - Test modules
    fn find_brand_palette_violations(editor_dir: &Path) -> Vec<String> {
        let mut violations = Vec::new();

        for entry in fs::read_dir(editor_dir).expect("failed to read editor_ui/") {
            let entry = entry.expect("failed to read dir entry");
            let path = entry.path();

            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            if ext != "rs" {
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            // Skip test files.
            if filename == "tests.rs" {
                continue;
            }

            let content = fs::read_to_string(&path).expect("failed to read editor_ui file");

            // Track whether we are inside a color conversion utility function.
            let mut in_conversion_fn = false;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                // Detect start/end of conversion utility functions.
                if trimmed.starts_with("fn bevy_color_to_egui")
                    || trimmed.starts_with("fn egui_color_to_bevy")
                    || trimmed.starts_with("fn rgb_to_color32")
                    || trimmed.starts_with("fn color32_to_rgb")
                {
                    in_conversion_fn = true;
                    continue;
                }
                // Simple heuristic: conversion functions end at a closing brace
                // at column 0. This is reliable for our code style.
                if in_conversion_fn && trimmed == "}" && line.starts_with('}') {
                    in_conversion_fn = false;
                    continue;
                }
                if in_conversion_fn {
                    continue;
                }

                // --- Check from_gray(N) ---
                if let Some(start) = trimmed.find("from_gray(") {
                    let after = &trimmed[start + "from_gray(".len()..];
                    if let Some(end) = after.find(')') {
                        let num_str = &after[..end];
                        if let Ok(val) = num_str.trim().parse::<u8>()
                            && !APPROVED_GRAYS.contains(&val)
                        {
                            violations.push(format!(
                                "{}:{}: `from_gray({})` is not in the brand palette. \
                                     See docs/brand.md for approved colors.",
                                path.display(),
                                line_num + 1,
                                val,
                            ));
                        }
                    }
                }

                // --- Check from_rgb(R, G, B) ---
                if let Some(start) = trimmed.find("from_rgb(") {
                    let after = &trimmed[start + "from_rgb(".len()..];
                    if let Some(end) = after.find(')') {
                        let parts: Vec<&str> = after[..end].split(',').map(str::trim).collect();
                        if parts.len() == 3 {
                            let parsed: Vec<Option<u8>> =
                                parts.iter().map(|s| s.parse::<u8>().ok()).collect();
                            if let (Some(r), Some(g), Some(b)) = (parsed[0], parsed[1], parsed[2])
                                && !APPROVED_RGB.contains(&(r, g, b))
                            {
                                violations.push(format!(
                                    "{}:{}: `from_rgb({}, {}, {})` is not in the brand palette. \
                                         See docs/brand.md for approved colors.",
                                    path.display(),
                                    line_num + 1,
                                    r,
                                    g,
                                    b,
                                ));
                            }
                            // If parsing fails (variables, not literals), skip — it's dynamic.
                        }
                    }
                }

                // --- Check from_rgba_unmultiplied with literal args ---
                // (conversion utilities are already skipped above)
                if let Some(start) = trimmed.find("from_rgba") {
                    // Only flag if all args are numeric literals.
                    let after = &trimmed[start..];
                    if let Some(paren) = after.find('(')
                        && let Some(end) = after[paren..].find(')')
                    {
                        let args_str = &after[paren + 1..paren + end];
                        let parts: Vec<&str> = args_str.split(',').map(str::trim).collect();
                        let all_numeric =
                            parts.len() >= 3 && parts[..3].iter().all(|s| s.parse::<u8>().is_ok());
                        if all_numeric {
                            let r: u8 = parts[0].parse().unwrap_or(0);
                            let g: u8 = parts[1].parse().unwrap_or(0);
                            let b: u8 = parts[2].parse().unwrap_or(0);
                            if !APPROVED_RGB.contains(&(r, g, b)) {
                                violations.push(format!(
                                        "{}:{}: `from_rgba*({}, {}, {}, ...)` is not in the brand palette. \
                                         See docs/brand.md for approved colors.",
                                        path.display(),
                                        line_num + 1,
                                        r, g, b,
                                    ));
                            }
                        }
                    }
                }

                // --- Check named Color32 constants (e.g., Color32::RED) ---
                // Only flag Color32::<NAME> patterns that aren't in the allowlist.
                if let Some(start) = trimmed.find("Color32::") {
                    let after = &trimmed[start + "Color32::".len()..];
                    // Extract the identifier (uppercase letters/underscores).
                    let name_end = after
                        .find(|c: char| !c.is_ascii_uppercase() && c != '_')
                        .unwrap_or(after.len());
                    let name = &after[..name_end];
                    // Skip constructors (from_gray, from_rgb, etc.) — handled above.
                    if !name.is_empty()
                        && !name.starts_with("from_")
                        && name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                    {
                        // Ignore lowercase-starting (method calls like .r(), .g()).
                        let full = format!("Color32::{name}");
                        if !APPROVED_NAMED.contains(&full.as_str()) {
                            violations.push(format!(
                                "{}:{}: `{}` is not in the brand palette. \
                                 See docs/brand.md for approved colors.",
                                path.display(),
                                line_num + 1,
                                full,
                            ));
                        }
                    }
                }
            }
        }

        violations
    }

    /// Validates the real `editor_ui` directory has no brand palette violations.
    #[test]
    fn editor_ui_colors_match_brand_palette() {
        let editor_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("editor_ui");

        let violations = find_brand_palette_violations(&editor_dir);

        assert!(
            violations.is_empty(),
            "Brand palette violations found in editor_ui/:\n{}",
            violations.join("\n"),
        );
    }

    #[test]
    fn brand_palette_detects_unapproved_gray() {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        fs::write(
            tmp.path().join("bad_gray.rs"),
            "let c = Color32::from_gray(99);\n",
        )
        .expect("failed to write file");

        let violations = find_brand_palette_violations(tmp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("from_gray(99)"));
    }

    #[test]
    fn brand_palette_detects_unapproved_rgb() {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        fs::write(
            tmp.path().join("bad_rgb.rs"),
            "let c = Color32::from_rgb(255, 0, 0);\n",
        )
        .expect("failed to write file");

        let violations = find_brand_palette_violations(tmp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("from_rgb(255, 0, 0)"));
    }

    #[test]
    fn brand_palette_detects_unapproved_rgba() {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        fs::write(
            tmp.path().join("bad_rgba.rs"),
            "let c = Color32::from_rgba_unmultiplied(255, 0, 0, 255);\n",
        )
        .expect("failed to write file");

        let violations = find_brand_palette_violations(tmp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("from_rgba*(255, 0, 0, ...)"));
    }

    #[test]
    fn brand_palette_detects_unapproved_named_constant() {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        fs::write(tmp.path().join("bad_named.rs"), "let c = Color32::RED;\n")
            .expect("failed to write file");

        let violations = find_brand_palette_violations(tmp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("Color32::RED"));
    }

    #[test]
    fn brand_palette_allows_approved_colors() {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        fs::write(
            tmp.path().join("good_colors.rs"),
            "let a = Color32::from_gray(25);\n\
             let b = Color32::from_rgb(0, 92, 128);\n",
        )
        .expect("failed to write file");

        let violations = find_brand_palette_violations(tmp.path());
        assert!(
            violations.is_empty(),
            "Approved colors should pass: {violations:?}"
        );
    }

    #[test]
    fn brand_palette_skips_test_files() {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        // tests.rs should be exempt even with bad colors.
        fs::write(tmp.path().join("tests.rs"), "let c = Color32::RED;\n")
            .expect("failed to write file");

        let violations = find_brand_palette_violations(tmp.path());
        assert!(
            violations.is_empty(),
            "tests.rs should be exempt: {violations:?}"
        );
    }

    #[test]
    fn brand_palette_skips_conversion_functions() {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        fs::write(
            tmp.path().join("conversions.rs"),
            "fn bevy_color_to_egui(color: Color) -> Color32 {\n\
             \x20   Color32::from_rgb(255, 0, 0)\n\
             }\n",
        )
        .expect("failed to write file");

        let violations = find_brand_palette_violations(tmp.path());
        assert!(
            violations.is_empty(),
            "Conversion functions should be exempt: {violations:?}"
        );
    }

    #[test]
    fn brand_palette_skips_non_rs_files() {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        fs::write(
            tmp.path().join("readme.md"),
            "Color32::RED and from_gray(99)\n",
        )
        .expect("failed to write file");

        let violations = find_brand_palette_violations(tmp.path());
        assert!(
            violations.is_empty(),
            "Non-.rs files should be skipped: {violations:?}"
        );
    }
}

/// Cross-plugin integration tests.
///
/// Unit tests manually insert resources, so they never catch cross-plugin
/// ordering issues. These tests assemble real plugins in a headless Bevy
/// app and verify they cooperate correctly.
///
/// Plugins that require rendering (`HexGridPlugin`, `CameraPlugin`, `EditorUiPlugin`)
/// are excluded — their dependencies (egui context, window) are unavailable in
/// headless mode. We manually provide the resources and entities they would create.
#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;

    use hexorder_contracts::editor_ui::EditorTool;
    use hexorder_contracts::game_system::{
        ActiveBoardType, ActiveTokenType, EntityData, EntityRole, EntityTypeRegistry, GameSystem,
        SelectedUnit, UnitInstance,
    };
    use hexorder_contracts::hex_grid::{
        HexGridConfig, HexPosition, HexSelectedEvent, HexTile, TileBaseMaterial,
    };

    /// Build a headless app with `GameSystemPlugin` + `CellPlugin` + `UnitPlugin`.
    /// Manually provides `EditorTool`, `HexGridConfig` (normally from EditorUiPlugin/HexGridPlugin)
    /// and asset stores (normally from `DefaultPlugins`).
    /// Starts in `AppScreen::Editor` so gated systems run immediately.
    fn headless_app() -> App {
        use hexorder_contracts::persistence::AppScreen;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.insert_state(AppScreen::Editor);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(EditorTool::default());
        app.insert_resource(HexGridConfig {
            layout: hexx::HexLayout {
                orientation: hexx::HexOrientation::Pointy,
                scale: bevy::math::Vec2::splat(1.0),
                origin: bevy::math::Vec2::ZERO,
            },
            map_radius: 5,
        });
        app.add_plugins(crate::game_system::GameSystemPlugin);
        app.init_resource::<hexorder_contracts::undo_redo::UndoStack>();
        app.add_plugins(crate::cell::CellPlugin);
        app.add_plugins(crate::unit::UnitPlugin);
        app
    }

    /// Spawn a minimal hex tile entity (simulates what `HexGridPlugin` does).
    fn spawn_test_tile(app: &mut App, q: i32, r: i32) -> Entity {
        let material = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Mesh::from(Cuboid::new(1.0, 0.1, 1.0)));

        app.world_mut()
            .spawn((
                HexTile,
                HexPosition::new(q, r),
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::default(),
            ))
            .id()
    }

    /// Resources inserted in `build()` must be available before the first update.
    /// This catches the deferred-vs-immediate class of bugs.
    #[test]
    fn game_system_resources_available_immediately() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(crate::game_system::GameSystemPlugin);

        // Resources should exist BEFORE the first update (inserted in build()).
        assert!(
            app.world().get_resource::<GameSystem>().is_some(),
            "GameSystem should exist before first update"
        );
        assert!(
            app.world().get_resource::<EntityTypeRegistry>().is_some(),
            "EntityTypeRegistry should exist before first update"
        );
        assert!(
            app.world().get_resource::<ActiveBoardType>().is_some(),
            "ActiveBoardType should exist before first update"
        );
    }

    /// `GameSystemPlugin` + `CellPlugin` must start without panicking.
    /// This is the exact test that would have caught the original crash.
    #[test]
    fn game_system_and_cell_startup_succeeds() {
        let mut app = headless_app();
        app.update(); // Startup runs
        app.update(); // First Update runs
    }

    /// Tiles spawned between Startup and Update get default `EntityData`
    /// from the cell plugin's `assign_default_cell_data` system.
    #[test]
    fn cell_assigns_default_data_to_new_tiles() {
        let mut app = headless_app();
        app.update(); // Startup: setup_cell_materials

        // Spawn tiles (simulating what HexGridPlugin does in Startup).
        spawn_test_tile(&mut app, 0, 0);
        spawn_test_tile(&mut app, 1, 0);
        spawn_test_tile(&mut app, 0, 1);

        app.update(); // Update: assign_default_cell_data runs

        let registry = app.world().resource::<EntityTypeRegistry>();
        let first_id = registry
            .first_by_role(EntityRole::BoardPosition)
            .expect("registry should have BoardPosition types")
            .id;

        let mut query = app
            .world_mut()
            .query_filtered::<&EntityData, With<HexTile>>();
        let entity_data: Vec<_> = query.iter(app.world()).collect();

        assert_eq!(
            entity_data.len(),
            3,
            "All tiles should have EntityData after update"
        );
        for ed in &entity_data {
            assert_eq!(
                ed.entity_type_id, first_id,
                "Default entity type should be the first BoardPosition in registry"
            );
        }
    }

    /// Unit resources inserted in `build()` must be available before the first update.
    #[test]
    fn game_system_unit_resources_available_immediately() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(crate::game_system::GameSystemPlugin);

        assert!(
            app.world().get_resource::<EntityTypeRegistry>().is_some(),
            "EntityTypeRegistry should exist before first update"
        );
        assert!(
            app.world().get_resource::<ActiveTokenType>().is_some(),
            "ActiveTokenType should exist before first update"
        );
        assert!(
            app.world().get_resource::<SelectedUnit>().is_some(),
            "SelectedUnit should exist before first update"
        );
    }

    /// `GameSystemPlugin` + `UnitPlugin` must start without panicking.
    #[test]
    fn game_system_and_unit_startup_succeeds() {
        let mut app = headless_app();
        app.update(); // Startup runs
        app.update(); // First Update runs

        // Verify token entity types are registered.
        let registry = app.world().resource::<EntityTypeRegistry>();
        assert!(
            !registry.types_by_role(EntityRole::Token).is_empty(),
            "Token entity types should be registered"
        );
    }

    /// Placing a unit via `HexSelectedEvent` in Place mode creates an entity.
    #[test]
    fn unit_placement_creates_entity_on_grid() {
        let mut app = headless_app();
        app.update(); // Startup: setup_unit_visuals

        // Switch to Place mode.
        *app.world_mut().resource_mut::<EditorTool>() = EditorTool::Place;

        let active_id = app
            .world()
            .resource::<ActiveTokenType>()
            .entity_type_id
            .expect("ActiveTokenType should have a type selected");

        // Trigger placement at (0, 0).
        app.world_mut().trigger(HexSelectedEvent {
            position: HexPosition::new(0, 0),
        });

        app.update(); // Process any deferred commands

        // Find the unit.
        let mut query = app
            .world_mut()
            .query_filtered::<(&EntityData, &HexPosition), With<UnitInstance>>();
        let units: Vec<_> = query.iter(app.world()).collect();

        assert_eq!(units.len(), 1, "Exactly one unit should be placed");
        assert_eq!(units[0].0.entity_type_id, active_id);
        assert_eq!(*units[0].1, HexPosition::new(0, 0));
    }

    /// Moving a unit via `HexSelectedEvent` in Select mode updates its position.
    #[test]
    fn unit_movement_updates_position() {
        let mut app = headless_app();
        app.update(); // Startup

        // Place a unit first.
        *app.world_mut().resource_mut::<EditorTool>() = EditorTool::Place;
        app.world_mut().trigger(HexSelectedEvent {
            position: HexPosition::new(0, 0),
        });
        app.update();

        // Find the unit entity.
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<UnitInstance>>();
        let unit_entity = query.iter(app.world()).next().expect("Unit should exist");

        // Switch to Select mode and select the unit.
        *app.world_mut().resource_mut::<EditorTool>() = EditorTool::Select;
        app.world_mut().resource_mut::<SelectedUnit>().entity = Some(unit_entity);

        // Click a different position to trigger movement.
        app.world_mut().trigger(HexSelectedEvent {
            position: HexPosition::new(1, 0),
        });
        app.update();

        // Verify the unit moved.
        let pos = app
            .world()
            .entity(unit_entity)
            .get::<HexPosition>()
            .expect("Unit should have HexPosition");
        assert_eq!(
            *pos,
            HexPosition::new(1, 0),
            "Unit should have moved to (1, 0)"
        );
    }

    /// The full assign -> `sync_materials` -> `sync_visuals` chain works across
    /// plugin boundaries: tiles get `EntityData` and their material is updated.
    #[test]
    fn cell_visual_sync_after_data_assignment() {
        let mut app = headless_app();
        app.update(); // Startup

        let original_material = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgb(0.99, 0.99, 0.99),
                ..default()
            });
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Mesh::from(Cuboid::new(1.0, 0.1, 1.0)));

        let tile = app
            .world_mut()
            .spawn((
                HexTile,
                HexPosition::new(0, 0),
                Mesh3d(mesh),
                MeshMaterial3d(original_material.clone()),
                TileBaseMaterial(original_material.clone()),
                Transform::default(),
            ))
            .id();

        app.update(); // assign_default_cell_data + sync chain

        let tile_material = app
            .world()
            .entity(tile)
            .get::<MeshMaterial3d<StandardMaterial>>()
            .expect("Tile should have material");

        assert_ne!(
            tile_material.0, original_material,
            "Tile material should have been updated by cell visual sync"
        );
    }
}
