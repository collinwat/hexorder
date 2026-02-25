//! Unit tests for the export plugin.

use super::*;

/// A mock export target for testing the trait interface.
struct MockExporter {
    should_fail: bool,
}

#[allow(clippy::unnecessary_literal_bound)]
impl ExportTarget for MockExporter {
    fn name(&self) -> &str {
        "Mock Exporter"
    }

    fn extension(&self) -> &str {
        "mock"
    }

    fn export(&self, data: &ExportData) -> Result<ExportOutput, ExportError> {
        if self.should_fail {
            return Err(ExportError::GenerationFailed(
                "intentional test failure".to_string(),
            ));
        }

        let summary = format!(
            "types={} tiles={} tokens={} radius={}",
            data.entity_types.len(),
            data.board_entities.len(),
            data.token_entities.len(),
            data.grid_config.map_radius,
        );

        Ok(ExportOutput {
            files: vec![ExportFile {
                name: "test-output".to_string(),
                extension: "mock".to_string(),
                data: summary.into_bytes(),
            }],
        })
    }
}

#[test]
fn mock_exporter_produces_output() {
    let exporter = MockExporter { should_fail: false };
    let data = test_export_data();

    let output = exporter.export(&data).expect("export should succeed");
    assert_eq!(output.files.len(), 1);
    assert_eq!(output.files[0].name, "test-output");
    assert_eq!(output.files[0].extension, "mock");

    let content = String::from_utf8(output.files[0].data.clone()).expect("valid utf8");
    assert!(content.contains("types=2"));
    assert!(content.contains("tiles=1"));
    assert!(content.contains("tokens=1"));
    assert!(content.contains("radius=5"));
}

#[test]
fn mock_exporter_returns_error_on_failure() {
    let exporter = MockExporter { should_fail: true };
    let data = test_export_data();

    let result = exporter.export(&data);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("intentional test failure"));
}

#[test]
fn export_error_display_formats_correctly() {
    let empty = ExportError::EmptyGameSystem;
    assert_eq!(
        format!("{empty}"),
        "Nothing to export — game system is empty"
    );

    let generation_err = ExportError::GenerationFailed("bad data".to_string());
    assert_eq!(format!("{generation_err}"), "Export failed: bad data");

    let io = ExportError::IoError(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "file missing",
    ));
    assert!(format!("{io}").contains("file missing"));
}

#[test]
fn collect_export_data_captures_all_fields() {
    use hexorder_contracts::game_system::{EntityRole, EntityType, EntityTypeRegistry, TypeId};

    let registry = EntityTypeRegistry {
        types: vec![
            EntityType {
                id: TypeId::new(),
                name: "Plains".to_string(),
                role: EntityRole::BoardPosition,
                color: bevy::color::Color::srgb(0.5, 0.8, 0.3),
                properties: vec![],
            },
            EntityType {
                id: TypeId::new(),
                name: "Infantry".to_string(),
                role: EntityRole::Token,
                color: bevy::color::Color::srgb(0.2, 0.3, 0.8),
                properties: vec![],
            },
        ],
    };

    let grid_config = hexorder_contracts::hex_grid::HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 3,
    };

    let tiles = vec![(
        HexPosition::new(0, 0),
        EntityData {
            entity_type_id: registry.types[0].id,
            properties: std::collections::HashMap::new(),
        },
    )];

    let tokens = vec![(
        HexPosition::new(1, 0),
        EntityData {
            entity_type_id: registry.types[1].id,
            properties: std::collections::HashMap::new(),
        },
    )];

    let data = collect_export_data(&registry, &grid_config, &tiles, &tokens);

    assert_eq!(data.entity_types.len(), 2);
    assert_eq!(data.board_entities.len(), 1);
    assert_eq!(data.token_entities.len(), 1);
    assert_eq!(data.grid_config.map_radius, 3);
    assert!(data.grid_config.pointy_top);
}

#[test]
fn collect_export_data_handles_empty_state() {
    let registry = EntityTypeRegistry::default();
    let grid_config = hexorder_contracts::hex_grid::HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 0,
    };

    let data = collect_export_data(&registry, &grid_config, &[], &[]);

    assert!(data.entity_types.is_empty());
    assert!(data.board_entities.is_empty());
    assert!(data.token_entities.is_empty());
    assert_eq!(data.grid_config.map_radius, 0);
}

#[test]
fn export_target_trait_is_object_safe() {
    // Verify ExportTarget can be used as a trait object (dyn dispatch).
    let exporter: Box<dyn ExportTarget> = Box::new(MockExporter { should_fail: false });
    assert_eq!(exporter.name(), "Mock Exporter");
    assert_eq!(exporter.extension(), "mock");
}

#[test]
fn grid_snapshot_captures_flat_top_orientation() {
    let grid_config = hexorder_contracts::hex_grid::HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Flat,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 10,
    };

    let data = collect_export_data(&EntityTypeRegistry::default(), &grid_config, &[], &[]);

    assert!(!data.grid_config.pointy_top);
    assert_eq!(data.grid_config.map_radius, 10);
}

// ---------------------------------------------------------------------------
// Counter Sheet Tests
// ---------------------------------------------------------------------------

#[test]
fn counter_sheet_exports_pdf_bytes() {
    use counter_sheet::PrintAndPlayExporter;

    let exporter = PrintAndPlayExporter::default();
    let data = test_export_data();

    let output = exporter.export(&data).expect("export should succeed");
    assert_eq!(output.files.len(), 1);
    assert_eq!(output.files[0].name, "counter-sheet");
    assert_eq!(output.files[0].extension, "pdf");

    // PDF files start with %PDF header.
    assert!(
        output.files[0].data.starts_with(b"%PDF"),
        "output should be a valid PDF"
    );
    // Sanity check: non-trivial size.
    assert!(
        output.files[0].data.len() > 100,
        "PDF should have meaningful content"
    );
}

#[test]
fn counter_sheet_fails_on_empty_game_system() {
    use counter_sheet::PrintAndPlayExporter;

    let exporter = PrintAndPlayExporter::default();
    let data = ExportData {
        entity_types: vec![],
        board_entities: vec![],
        token_entities: vec![],
        grid_config: GridSnapshot {
            map_radius: 5,
            pointy_top: true,
        },
    };

    let result = exporter.export(&data);
    assert!(result.is_err());
}

#[test]
fn counter_sheet_generates_from_type_definitions_when_no_instances() {
    use counter_sheet::PrintAndPlayExporter;
    use hexorder_contracts::game_system::{
        EntityRole, EntityType, PropertyDefinition, PropertyType, PropertyValue, TypeId,
    };

    let exporter = PrintAndPlayExporter::default();
    let data = ExportData {
        entity_types: vec![EntityType {
            id: TypeId::new(),
            name: "Panzer".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "Attack".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(6),
                },
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "Defense".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(5),
                },
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "Movement".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(4),
                },
            ],
        }],
        board_entities: vec![],
        token_entities: vec![],
        grid_config: GridSnapshot {
            map_radius: 5,
            pointy_top: true,
        },
    };

    let output = exporter.export(&data).expect("export should succeed");
    assert!(output.files[0].data.starts_with(b"%PDF"));
}

#[test]
fn counter_sheet_all_sizes_produce_valid_pdf() {
    use counter_sheet::{CounterSize, PrintAndPlayExporter};

    let data = test_export_data();

    for size in [
        CounterSize::Half,
        CounterSize::FiveEighths,
        CounterSize::ThreeQuarters,
    ] {
        let exporter = PrintAndPlayExporter { counter_size: size };
        let output = exporter
            .export(&data)
            .unwrap_or_else(|_| panic!("export should succeed for size {size:?}"));
        assert!(
            output.files[0].data.starts_with(b"%PDF"),
            "size {size:?} should produce valid PDF"
        );
    }
}

#[test]
fn format_property_value_displays_numeric_types() {
    use counter_sheet::format_property_value;
    use hexorder_contracts::game_system::PropertyValue;

    assert_eq!(
        format_property_value(&PropertyValue::Int(42)),
        Some("42".to_string())
    );
    assert_eq!(
        format_property_value(&PropertyValue::IntRange(7)),
        Some("7".to_string())
    );
    assert_eq!(
        format_property_value(&PropertyValue::Float(2.75)),
        Some("2.8".to_string())
    );
    assert_eq!(
        format_property_value(&PropertyValue::Bool(true)),
        Some("Y".to_string())
    );
    assert_eq!(
        format_property_value(&PropertyValue::Bool(false)),
        Some("N".to_string())
    );
}

#[test]
fn format_property_value_skips_non_displayable() {
    use counter_sheet::format_property_value;
    use hexorder_contracts::game_system::PropertyValue;

    assert!(format_property_value(&PropertyValue::List(vec![])).is_none());
    assert!(format_property_value(&PropertyValue::String(String::new())).is_none());
    assert!(format_property_value(&PropertyValue::EntityRef(None)).is_none());
}

// ---------------------------------------------------------------------------
// Hex Map Tests
// ---------------------------------------------------------------------------

#[test]
fn hex_map_exports_pdf_bytes() {
    use hex_map::HexMapExporter;

    let exporter = HexMapExporter::default();
    let data = test_export_data();

    let output = exporter.export(&data).expect("export should succeed");
    assert_eq!(output.files.len(), 1);
    assert_eq!(output.files[0].name, "hex-map");
    assert_eq!(output.files[0].extension, "pdf");

    // PDF files start with %PDF header.
    assert!(
        output.files[0].data.starts_with(b"%PDF"),
        "output should be a valid PDF"
    );
    assert!(
        output.files[0].data.len() > 100,
        "PDF should have meaningful content"
    );
}

#[test]
fn hex_map_fails_on_empty_state() {
    use hex_map::HexMapExporter;

    let exporter = HexMapExporter::default();
    let data = ExportData {
        entity_types: vec![],
        board_entities: vec![],
        token_entities: vec![],
        grid_config: GridSnapshot {
            map_radius: 0,
            pointy_top: true,
        },
    };

    let result = exporter.export(&data);
    assert!(result.is_err());
}

#[test]
fn hex_map_renders_flat_top_orientation() {
    use hex_map::HexMapExporter;

    let exporter = HexMapExporter::default();
    let mut data = test_export_data();
    data.grid_config.pointy_top = false;

    let output = exporter
        .export(&data)
        .expect("flat-top export should succeed");
    assert!(output.files[0].data.starts_with(b"%PDF"));
}

#[test]
fn hex_map_all_counter_sizes_produce_valid_pdf() {
    use counter_sheet::CounterSize;
    use hex_map::HexMapExporter;

    // Use radius 2 so even the largest counter size fits on one page.
    let mut data = test_export_data();
    data.grid_config.map_radius = 2;

    for size in [
        CounterSize::Half,
        CounterSize::FiveEighths,
        CounterSize::ThreeQuarters,
    ] {
        let exporter = HexMapExporter { counter_size: size };
        let output = exporter
            .export(&data)
            .unwrap_or_else(|_| panic!("export should succeed for size {size:?}"));
        assert!(
            output.files[0].data.starts_with(b"%PDF"),
            "size {size:?} should produce valid PDF"
        );
    }
}

#[test]
fn hex_map_colors_board_entities() {
    use hex_map::HexMapExporter;
    use hexorder_contracts::game_system::{EntityRole, EntityType, TypeId};

    let board_type_id = TypeId::new();
    let data = ExportData {
        entity_types: vec![EntityType {
            id: board_type_id,
            name: "Forest".to_string(),
            role: EntityRole::BoardPosition,
            color: bevy::color::Color::srgb(0.1, 0.5, 0.1),
            properties: vec![],
        }],
        board_entities: vec![
            (
                HexPosition::new(0, 0),
                EntityData {
                    entity_type_id: board_type_id,
                    properties: std::collections::HashMap::new(),
                },
            ),
            (
                HexPosition::new(1, 0),
                EntityData {
                    entity_type_id: board_type_id,
                    properties: std::collections::HashMap::new(),
                },
            ),
        ],
        token_entities: vec![],
        grid_config: GridSnapshot {
            map_radius: 2,
            pointy_top: true,
        },
    };

    let exporter = HexMapExporter::default();
    let output = exporter
        .export(&data)
        .expect("export with board entities should succeed");
    assert!(output.files[0].data.starts_with(b"%PDF"));
}

#[test]
fn hex_map_auto_scales_oversized_grid() {
    use counter_sheet::CounterSize;
    use hex_map::HexMapExporter;

    let exporter = HexMapExporter {
        counter_size: CounterSize::ThreeQuarters,
    };
    let mut data = test_export_data();
    data.grid_config.map_radius = 10;

    let result = exporter.export(&data);
    assert!(result.is_ok(), "large grid should auto-scale to fit page");
}

// ---------------------------------------------------------------------------
// Polling System Tests
// ---------------------------------------------------------------------------

#[test]
fn poll_noop_when_no_pending_export() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_systems(bevy::app::Update, systems::poll_pending_export);
    app.update(); // Should not panic.
}

#[test]
fn poll_removes_resource_and_writes_files_on_completion() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_systems(bevy::app::Update, systems::poll_pending_export);

    let temp_dir =
        std::env::temp_dir().join(format!("hexorder-export-test-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");

    let dir_for_task = temp_dir.clone();
    let future = Box::pin(async move { Some(dir_for_task) });

    app.insert_resource(systems::PendingExport {
        data: test_export_data(),
        future: std::sync::Mutex::new(future),
    });

    // First update: polling system completes task, runs export.
    app.update();

    assert!(
        app.world()
            .get_resource::<systems::PendingExport>()
            .is_none(),
        "PendingExport should be removed after completion"
    );

    // Verify export files were written.
    let entries: Vec<_> = std::fs::read_dir(&temp_dir)
        .expect("read temp dir")
        .filter_map(Result::ok)
        .collect();
    assert!(
        !entries.is_empty(),
        "export should have written files to the output directory"
    );

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn poll_removes_resource_when_user_cancels() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_systems(bevy::app::Update, systems::poll_pending_export);

    let future = Box::pin(async { None });

    app.insert_resource(systems::PendingExport {
        data: test_export_data(),
        future: std::sync::Mutex::new(future),
    });

    app.update();

    assert!(
        app.world()
            .get_resource::<systems::PendingExport>()
            .is_none(),
        "PendingExport should be removed even when user cancels"
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal `ExportData` for tests.
fn test_export_data() -> ExportData {
    use hexorder_contracts::game_system::{EntityRole, EntityType, TypeId};

    let board_type_id = TypeId::new();
    let token_type_id = TypeId::new();

    ExportData {
        entity_types: vec![
            EntityType {
                id: board_type_id,
                name: "Plains".to_string(),
                role: EntityRole::BoardPosition,
                color: bevy::color::Color::srgb(0.5, 0.8, 0.3),
                properties: vec![],
            },
            EntityType {
                id: token_type_id,
                name: "Infantry".to_string(),
                role: EntityRole::Token,
                color: bevy::color::Color::srgb(0.2, 0.3, 0.8),
                properties: vec![],
            },
        ],
        board_entities: vec![(
            HexPosition::new(0, 0),
            EntityData {
                entity_type_id: board_type_id,
                properties: std::collections::HashMap::new(),
            },
        )],
        token_entities: vec![(
            HexPosition::new(1, 0),
            EntityData {
                entity_type_id: token_type_id,
                properties: std::collections::HashMap::new(),
            },
        )],
        grid_config: GridSnapshot {
            map_radius: 5,
            pointy_top: true,
        },
    }
}
