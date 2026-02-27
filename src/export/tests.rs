//! Unit tests for the export plugin.

use super::*;
use bevy::ecs::observer::On;
use bevy::prelude::World;
use hexorder_contracts::editor_ui::{ToastEvent, ToastKind};

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
// Counter Sheet — format_property_value extended variants
// ---------------------------------------------------------------------------

#[test]
fn format_property_value_displays_nonempty_string() {
    use counter_sheet::format_property_value;
    use hexorder_contracts::game_system::PropertyValue;

    assert_eq!(
        format_property_value(&PropertyValue::String("hello".into())),
        Some("hello".to_string())
    );
}

#[test]
fn format_property_value_displays_enum_variant() {
    use counter_sheet::format_property_value;
    use hexorder_contracts::game_system::PropertyValue;

    assert_eq!(
        format_property_value(&PropertyValue::Enum("Heavy".into())),
        Some("Heavy".to_string())
    );
}

#[test]
fn format_property_value_skips_empty_enum() {
    use counter_sheet::format_property_value;
    use hexorder_contracts::game_system::PropertyValue;

    assert!(format_property_value(&PropertyValue::Enum(String::new())).is_none());
}

#[test]
fn format_property_value_displays_float_range() {
    use counter_sheet::format_property_value;
    use hexorder_contracts::game_system::PropertyValue;

    assert_eq!(
        format_property_value(&PropertyValue::FloatRange(1.5)),
        Some("1.5".to_string())
    );
}

#[test]
fn format_property_value_skips_color() {
    use counter_sheet::format_property_value;
    use hexorder_contracts::game_system::PropertyValue;

    assert!(format_property_value(&PropertyValue::Color(bevy::color::Color::WHITE)).is_none());
}

#[test]
fn format_property_value_skips_entity_ref_some() {
    use counter_sheet::format_property_value;
    use hexorder_contracts::game_system::{PropertyValue, TypeId};

    assert!(format_property_value(&PropertyValue::EntityRef(Some(TypeId::new()))).is_none());
}

#[test]
fn format_property_value_skips_map() {
    use counter_sheet::format_property_value;
    use hexorder_contracts::game_system::PropertyValue;

    assert!(format_property_value(&PropertyValue::Map(vec![])).is_none());
}

#[test]
fn format_property_value_skips_struct() {
    use counter_sheet::format_property_value;
    use hexorder_contracts::game_system::PropertyValue;

    assert!(
        format_property_value(&PropertyValue::Struct(std::collections::HashMap::new())).is_none()
    );
}

// ---------------------------------------------------------------------------
// Counter Sheet — token instances with properties
// ---------------------------------------------------------------------------

#[test]
fn counter_sheet_renders_token_instances_with_properties() {
    use counter_sheet::PrintAndPlayExporter;
    use hexorder_contracts::game_system::{
        EntityRole, EntityType, PropertyDefinition, PropertyType, PropertyValue, TypeId,
    };

    let type_id = TypeId::new();
    let prop_id = TypeId::new();
    let mut instance_props = std::collections::HashMap::new();
    instance_props.insert(prop_id, PropertyValue::Int(10));

    let data = ExportData {
        entity_types: vec![EntityType {
            id: type_id,
            name: "Tank".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.4, 0.4, 0.4),
            properties: vec![PropertyDefinition {
                id: prop_id,
                name: "Attack".to_string(),
                property_type: PropertyType::Int,
                default_value: PropertyValue::Int(5),
            }],
        }],
        board_entities: vec![],
        token_entities: vec![(
            HexPosition::new(0, 0),
            EntityData {
                entity_type_id: type_id,
                properties: instance_props,
            },
        )],
        grid_config: GridSnapshot {
            map_radius: 3,
            pointy_top: true,
        },
    };

    let exporter = PrintAndPlayExporter::default();
    let output = exporter.export(&data).expect("export should succeed");
    assert!(output.files[0].data.starts_with(b"%PDF"));
}

#[test]
fn counter_sheet_renders_multiple_property_values() {
    use counter_sheet::PrintAndPlayExporter;
    use hexorder_contracts::game_system::{
        EntityRole, EntityType, PropertyDefinition, PropertyType, PropertyValue, TypeId,
    };

    let type_id = TypeId::new();
    let data = ExportData {
        entity_types: vec![EntityType {
            id: type_id,
            name: "Infantry".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.2, 0.3, 0.8),
            properties: vec![
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "ATK".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(4),
                },
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "DEF".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(3),
                },
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "MOV".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(2),
                },
            ],
        }],
        board_entities: vec![],
        token_entities: vec![],
        grid_config: GridSnapshot {
            map_radius: 3,
            pointy_top: true,
        },
    };

    let exporter = PrintAndPlayExporter::default();
    let output = exporter.export(&data).expect("export should succeed");
    assert!(output.files[0].data.starts_with(b"%PDF"));
}

#[test]
fn counter_sheet_renders_counter_with_dark_background() {
    use counter_sheet::PrintAndPlayExporter;
    use hexorder_contracts::game_system::{EntityRole, EntityType, TypeId};

    let type_id = TypeId::new();
    let data = ExportData {
        entity_types: vec![EntityType {
            id: type_id,
            name: "Shadow".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.05, 0.05, 0.1),
            properties: vec![],
        }],
        board_entities: vec![],
        token_entities: vec![],
        grid_config: GridSnapshot {
            map_radius: 3,
            pointy_top: true,
        },
    };

    let exporter = PrintAndPlayExporter::default();
    let output = exporter.export(&data).expect("export should succeed");
    assert!(output.files[0].data.starts_with(b"%PDF"));
}

#[test]
fn counter_sheet_token_with_orphan_type_skipped() {
    use counter_sheet::PrintAndPlayExporter;
    use hexorder_contracts::game_system::{EntityRole, EntityType, TypeId};

    let known_type = TypeId::new();
    let orphan_type = TypeId::new();

    let data = ExportData {
        entity_types: vec![EntityType {
            id: known_type,
            name: "Ranger".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.3, 0.6, 0.2),
            properties: vec![],
        }],
        board_entities: vec![],
        token_entities: vec![
            (
                HexPosition::new(0, 0),
                EntityData {
                    entity_type_id: known_type,
                    properties: std::collections::HashMap::new(),
                },
            ),
            (
                HexPosition::new(1, 0),
                EntityData {
                    entity_type_id: orphan_type,
                    properties: std::collections::HashMap::new(),
                },
            ),
        ],
        grid_config: GridSnapshot {
            map_radius: 3,
            pointy_top: true,
        },
    };

    let exporter = PrintAndPlayExporter::default();
    let output = exporter.export(&data).expect("export should succeed");
    assert!(output.files[0].data.starts_with(b"%PDF"));
}

#[test]
fn counter_sheet_long_name_truncated() {
    use counter_sheet::PrintAndPlayExporter;
    use hexorder_contracts::game_system::{EntityRole, EntityType, TypeId};

    let type_id = TypeId::new();
    let long_name = "A Very Long Entity Type Name That Exceeds Counter Width";
    let data = ExportData {
        entity_types: vec![EntityType {
            id: type_id,
            name: long_name.to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![],
        }],
        board_entities: vec![],
        token_entities: vec![],
        grid_config: GridSnapshot {
            map_radius: 3,
            pointy_top: true,
        },
    };

    let exporter = PrintAndPlayExporter {
        counter_size: counter_sheet::CounterSize::Half,
    };
    let output = exporter.export(&data).expect("export should succeed");
    assert!(output.files[0].data.starts_with(b"%PDF"));
}

#[test]
fn counter_sheet_many_counters_spans_multiple_pages() {
    use counter_sheet::PrintAndPlayExporter;
    use hexorder_contracts::game_system::{EntityRole, EntityType, TypeId};

    let type_id = TypeId::new();
    let tokens: Vec<_> = (0..200)
        .map(|i| {
            (
                HexPosition::new(i, 0),
                EntityData {
                    entity_type_id: type_id,
                    properties: std::collections::HashMap::new(),
                },
            )
        })
        .collect();

    let data = ExportData {
        entity_types: vec![EntityType {
            id: type_id,
            name: "Soldier".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.3, 0.5, 0.3),
            properties: vec![],
        }],
        board_entities: vec![],
        token_entities: tokens,
        grid_config: GridSnapshot {
            map_radius: 10,
            pointy_top: true,
        },
    };

    let exporter = PrintAndPlayExporter::default();
    let output = exporter.export(&data).expect("export should succeed");
    assert!(output.files[0].data.starts_with(b"%PDF"));
    assert!(output.files[0].data.len() > 500);
}

// ---------------------------------------------------------------------------
// Export systems — run_export
// ---------------------------------------------------------------------------

#[test]
fn run_export_triggers_success_toast() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);

    let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let received_clone = received.clone();
    app.add_observer(move |trigger: On<ToastEvent>| {
        received_clone
            .lock()
            .expect("lock")
            .push(trigger.event().clone());
    });

    let temp_dir = std::env::temp_dir().join(format!(
        "hexorder-export-run-success-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");

    let data = test_export_data();
    let dir = temp_dir.clone();
    app.world_mut().commands().queue(move |world: &mut World| {
        systems::run_export(&data, &dir, world);
    });
    app.update();

    let toasts = received.lock().expect("lock");
    assert!(!toasts.is_empty(), "should have triggered a toast");
    assert_eq!(toasts[0].kind, ToastKind::Success);
    assert!(toasts[0].message.contains("Exported"));

    drop(toasts);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn run_export_triggers_info_toast_when_nothing_to_export() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);

    let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let received_clone = received.clone();
    app.add_observer(move |trigger: On<ToastEvent>| {
        received_clone
            .lock()
            .expect("lock")
            .push(trigger.event().clone());
    });

    let temp_dir =
        std::env::temp_dir().join(format!("hexorder-export-run-empty-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");

    let data = ExportData {
        entity_types: vec![],
        board_entities: vec![],
        token_entities: vec![],
        grid_config: GridSnapshot {
            map_radius: 0,
            pointy_top: true,
        },
    };

    let dir = temp_dir.clone();
    app.world_mut().commands().queue(move |world: &mut World| {
        systems::run_export(&data, &dir, world);
    });
    app.update();

    let toasts = received.lock().expect("lock");
    assert!(!toasts.is_empty(), "should have triggered a toast");
    assert_eq!(toasts[0].kind, ToastKind::Info);
    assert!(toasts[0].message.contains("Nothing to export"));

    drop(toasts);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn run_export_triggers_error_toast_on_write_failure() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);

    let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let received_clone = received.clone();
    app.add_observer(move |trigger: On<ToastEvent>| {
        received_clone
            .lock()
            .expect("lock")
            .push(trigger.event().clone());
    });

    let bad_dir = std::path::PathBuf::from("/nonexistent/hexorder-export-test");
    let data = test_export_data();

    app.world_mut().commands().queue(move |world: &mut World| {
        systems::run_export(&data, &bad_dir, world);
    });
    app.update();

    let toasts = received.lock().expect("lock");
    assert!(!toasts.is_empty(), "should have triggered a toast");
    assert_eq!(toasts[0].kind, ToastKind::Error);
    assert!(toasts[0].message.contains("Export errors"));
}

// ---------------------------------------------------------------------------
// Export systems — PendingExport Debug
// ---------------------------------------------------------------------------

#[test]
fn pending_export_debug_impl() {
    let future = Box::pin(async { None });
    let pending = systems::PendingExport {
        data: test_export_data(),
        future: std::sync::Mutex::new(future),
    };
    let debug_str = format!("{pending:?}");
    assert!(debug_str.contains("PendingExport"));
    assert!(debug_str.contains("<Future>"));
}

// ---------------------------------------------------------------------------
// Export mod — plugin registration
// ---------------------------------------------------------------------------

#[test]
fn export_plugin_registers_shortcut() {
    use hexorder_contracts::shortcuts::ShortcutRegistry;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<ShortcutRegistry>();
    app.add_plugins(super::ExportPlugin);
    app.update();

    let registry = app.world().resource::<ShortcutRegistry>();
    assert!(
        !registry.bindings_for("file.export_pnp").is_empty(),
        "ExportPlugin should register the file.export_pnp command"
    );
}

// ---------------------------------------------------------------------------
// Export mod — ExportError From<io::Error>
// ---------------------------------------------------------------------------

#[test]
fn export_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let export_err: ExportError = io_err.into();
    let msg = format!("{export_err}");
    assert!(msg.contains("access denied"));
}

// ---------------------------------------------------------------------------
// Export systems — handle_export_command guard clauses
// ---------------------------------------------------------------------------

#[test]
fn handle_export_command_ignores_other_commands() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_observer(systems::handle_export_command);

    // Trigger a different command — should early-return at line 52-53.
    app.world_mut().commands().trigger(CommandExecutedEvent {
        command_id: CommandId("some.other.command"),
    });
    app.update();
    app.update();

    assert!(
        app.world()
            .get_resource::<systems::PendingExport>()
            .is_none(),
        "PendingExport should not be created for non-export commands"
    );
}

#[test]
fn handle_export_command_noop_when_pending_export_exists() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_observer(systems::handle_export_command);

    // Insert HexGridConfig so we pass the grid guard.
    app.insert_resource(hexorder_contracts::hex_grid::HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 3,
    });

    // Insert an existing PendingExport — guard should prevent creating another.
    let future = Box::pin(async { None });
    app.insert_resource(systems::PendingExport {
        data: test_export_data(),
        future: std::sync::Mutex::new(future),
    });

    app.world_mut().commands().trigger(CommandExecutedEvent {
        command_id: CommandId("file.export_pnp"),
    });
    app.update();
    app.update();

    // PendingExport should still be present (guard prevented replacement).
    assert!(
        app.world()
            .get_resource::<systems::PendingExport>()
            .is_some(),
        "existing PendingExport should not be replaced"
    );
}

#[test]
fn handle_export_command_noop_without_grid_config() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_observer(systems::handle_export_command);

    // Trigger export command WITHOUT HexGridConfig — should early-return.
    app.world_mut().commands().trigger(CommandExecutedEvent {
        command_id: CommandId("file.export_pnp"),
    });
    app.update();
    app.update();

    assert!(
        app.world()
            .get_resource::<systems::PendingExport>()
            .is_none(),
        "PendingExport should not be created without HexGridConfig"
    );
}

// ---------------------------------------------------------------------------
// Export systems — poll_pending_export with pending future
// ---------------------------------------------------------------------------

#[test]
fn poll_pending_export_stays_when_future_pending() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_systems(bevy::app::Update, systems::poll_pending_export);

    // Create a future that never completes.
    let future = Box::pin(std::future::pending::<Option<std::path::PathBuf>>());
    app.insert_resource(systems::PendingExport {
        data: test_export_data(),
        future: std::sync::Mutex::new(future),
    });

    app.update();

    assert!(
        app.world()
            .get_resource::<systems::PendingExport>()
            .is_some(),
        "PendingExport should remain when future is still pending"
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
