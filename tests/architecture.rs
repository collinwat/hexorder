//! Architecture enforcement tests.
//!
//! These verify structural rules that apply across the entire project.
//! They run as integration tests so they can scan the workspace layout.

use std::fs;
use std::path::Path;

/// Scans all plugin `mod.rs` files under `src_dir` and returns violations
/// where sub-modules are declared `pub mod`. Plugin internals must be
/// private; shared types go through `crates/hexorder-contracts/`.
fn find_pub_mod_violations(src_dir: &Path) -> Vec<String> {
    let mut violations = Vec::new();

    let Ok(entries) = fs::read_dir(src_dir) else {
        return violations;
    };

    for entry in entries {
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

/// Scans `src/` for plugins with `pub mod` violations.
///
/// Also scans `crates/hexorder-*/src/` for extracted plugin crates.
#[test]
fn plugin_modules_are_private() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // Check in-tree plugins.
    let src_dir = root.join("src");
    let mut violations = find_pub_mod_violations(&src_dir);

    // Check extracted plugin crates.
    let crates_dir = root.join("crates");
    if crates_dir.exists()
        && let Ok(entries) = fs::read_dir(&crates_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            // Skip contracts and sdk — they are intentionally public.
            if name == "hexorder-contracts" || name == "hexorder-sdk" {
                continue;
            }
            if path.is_dir() {
                let crate_src = path.join("src");
                if crate_src.exists() {
                    violations.extend(find_pub_mod_violations(&crate_src));
                }
            }
        }
    }

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

const APPROVED_NAMED: &[&str] = &["Color32::TRANSPARENT"];

/// Scans `.rs` files in `editor_dir` for color literals and returns
/// violations where a color is not in the approved brand palette.
///
/// Exempt patterns:
/// - Color conversion utilities (functions that transform values, not define them)
/// - Dynamic colors constructed from variables (e.g., user-picked colors)
/// - Test modules
fn find_brand_palette_violations(editor_dir: &Path) -> Vec<String> {
    let mut violations = Vec::new();

    let Ok(entries) = fs::read_dir(editor_dir) else {
        return violations;
    };

    for entry in entries {
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
                                r,
                                g,
                                b,
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
