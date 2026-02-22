//! Unit tests for the `mechanic_reference` plugin.

use bevy::prelude::*;

use super::MechanicReferencePlugin;
use super::components::{MechanicCatalog, MechanicCategory, MechanicEntry, TemplateAvailability};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(MechanicReferencePlugin);
    app
}

// ---------------------------------------------------------------------------
// MechanicCategory tests
// ---------------------------------------------------------------------------

#[test]
fn mechanic_category_has_six_areas() {
    let categories = MechanicCategory::all();
    assert_eq!(categories.len(), 6, "Engelstein taxonomy has 6 areas");
}

#[test]
fn mechanic_category_display_names_are_nonempty() {
    for category in MechanicCategory::all() {
        let name = category.display_name();
        assert!(!name.is_empty(), "Category {category:?} has empty name");
    }
}

#[test]
fn mechanic_category_descriptions_are_nonempty() {
    for category in MechanicCategory::all() {
        let desc = category.description();
        assert!(
            !desc.is_empty(),
            "Category {category:?} has empty description"
        );
    }
}

// ---------------------------------------------------------------------------
// MechanicEntry tests
// ---------------------------------------------------------------------------

#[test]
fn mechanic_entry_requires_name_and_category() {
    let entry = MechanicEntry {
        name: "Hex Grid Systems".to_string(),
        category: MechanicCategory::CoreUniversal,
        description: "The game map is overlaid with a hexagonal grid.".to_string(),
        example_games: vec!["Panzerblitz".to_string()],
        design_considerations: "Must support coordinate systems.".to_string(),
        template: TemplateAvailability::None,
    };
    assert_eq!(entry.name, "Hex Grid Systems");
    assert_eq!(entry.category, MechanicCategory::CoreUniversal);
}

#[test]
fn mechanic_entry_supports_template_availability() {
    let with_template = MechanicEntry {
        name: "Combat Resolution".to_string(),
        category: MechanicCategory::CoreUniversal,
        description: "CRT-based combat.".to_string(),
        example_games: vec![],
        design_considerations: String::new(),
        template: TemplateAvailability::Available {
            template_id: "crt_combat".to_string(),
            preview: "Creates a standard odds-based CRT.".to_string(),
        },
    };
    assert!(
        matches!(
            with_template.template,
            TemplateAvailability::Available { .. }
        ),
        "Entry should have a template"
    );

    let without_template = MechanicEntry {
        name: "Card-Driven Combat".to_string(),
        category: MechanicCategory::BespokeUnusual,
        description: "Cards replace dice.".to_string(),
        example_games: vec![],
        design_considerations: String::new(),
        template: TemplateAvailability::None,
    };
    assert!(
        matches!(without_template.template, TemplateAvailability::None),
        "Entry should have no template"
    );
}

// ---------------------------------------------------------------------------
// MechanicCatalog resource tests
// ---------------------------------------------------------------------------

#[test]
fn mechanic_catalog_default_is_empty() {
    let catalog = MechanicCatalog::default();
    assert!(catalog.entries.is_empty());
}

#[test]
fn mechanic_catalog_entries_by_category() {
    let mut catalog = MechanicCatalog::default();
    catalog.entries.push(MechanicEntry {
        name: "Hex Grid".to_string(),
        category: MechanicCategory::CoreUniversal,
        description: "Grid system.".to_string(),
        example_games: vec![],
        design_considerations: String::new(),
        template: TemplateAvailability::None,
    });
    catalog.entries.push(MechanicEntry {
        name: "Supply".to_string(),
        category: MechanicCategory::AdvancedCommon,
        description: "Logistics.".to_string(),
        example_games: vec![],
        design_considerations: String::new(),
        template: TemplateAvailability::None,
    });
    catalog.entries.push(MechanicEntry {
        name: "Movement".to_string(),
        category: MechanicCategory::CoreUniversal,
        description: "MP-based movement.".to_string(),
        example_games: vec![],
        design_considerations: String::new(),
        template: TemplateAvailability::None,
    });

    let core = catalog.entries_by_category(MechanicCategory::CoreUniversal);
    assert_eq!(core.len(), 2);
    assert!(
        core.iter()
            .all(|e| e.category == MechanicCategory::CoreUniversal)
    );

    let advanced = catalog.entries_by_category(MechanicCategory::AdvancedCommon);
    assert_eq!(advanced.len(), 1);

    let bespoke = catalog.entries_by_category(MechanicCategory::BespokeUnusual);
    assert!(bespoke.is_empty());
}

#[test]
fn mechanic_catalog_entries_with_templates() {
    let mut catalog = MechanicCatalog::default();
    catalog.entries.push(MechanicEntry {
        name: "CRT Combat".to_string(),
        category: MechanicCategory::CoreUniversal,
        description: "Combat.".to_string(),
        example_games: vec![],
        design_considerations: String::new(),
        template: TemplateAvailability::Available {
            template_id: "crt".to_string(),
            preview: "Standard CRT.".to_string(),
        },
    });
    catalog.entries.push(MechanicEntry {
        name: "Card-Driven".to_string(),
        category: MechanicCategory::BespokeUnusual,
        description: "Cards.".to_string(),
        example_games: vec![],
        design_considerations: String::new(),
        template: TemplateAvailability::None,
    });

    let with_templates = catalog.entries_with_templates();
    assert_eq!(with_templates.len(), 1);
    assert_eq!(with_templates[0].name, "CRT Combat");
}

// ---------------------------------------------------------------------------
// Plugin registration tests
// ---------------------------------------------------------------------------

#[test]
fn plugin_registers_catalog_resource() {
    let app = test_app();
    assert!(
        app.world().get_resource::<MechanicCatalog>().is_some(),
        "MechanicCatalog resource should exist after plugin registration"
    );
}

// ---------------------------------------------------------------------------
// Catalog content tests (Scope 2)
// ---------------------------------------------------------------------------

#[test]
fn catalog_has_entries_in_all_six_categories() {
    let app = test_app();
    let catalog = app
        .world()
        .get_resource::<MechanicCatalog>()
        .expect("catalog should exist");

    for category in MechanicCategory::all() {
        let entries = catalog.entries_by_category(*category);
        assert!(
            !entries.is_empty(),
            "Category {:?} ({}) should have at least one entry",
            category,
            category.display_name()
        );
    }
}

#[test]
fn catalog_core_universal_has_expected_count() {
    let app = test_app();
    let catalog = app.world().resource::<MechanicCatalog>();
    let core = catalog.entries_by_category(MechanicCategory::CoreUniversal);
    assert_eq!(core.len(), 9, "Area 1 has 9 mechanics (1.1 through 1.9)");
}

#[test]
fn catalog_advanced_common_has_expected_count() {
    let app = test_app();
    let catalog = app.world().resource::<MechanicCatalog>();
    let advanced = catalog.entries_by_category(MechanicCategory::AdvancedCommon);
    assert_eq!(
        advanced.len(),
        10,
        "Area 2 has 10 mechanics (2.1 through 2.10)"
    );
}

#[test]
fn catalog_total_entry_count() {
    let app = test_app();
    let catalog = app.world().resource::<MechanicCatalog>();
    assert_eq!(catalog.entries.len(), 56, "Survey has 56 total mechanics");
}

#[test]
fn catalog_entries_have_nonempty_fields() {
    let app = test_app();
    let catalog = app.world().resource::<MechanicCatalog>();

    for entry in &catalog.entries {
        assert!(!entry.name.is_empty(), "Entry has empty name");
        assert!(
            !entry.description.is_empty(),
            "Entry '{}' has empty description",
            entry.name
        );
        assert!(
            !entry.design_considerations.is_empty(),
            "Entry '{}' has empty design_considerations",
            entry.name
        );
    }
}

#[test]
fn catalog_known_entries_are_present() {
    let app = test_app();
    let catalog = app.world().resource::<MechanicCatalog>();

    let names: Vec<&str> = catalog.entries.iter().map(|e| e.name.as_str()).collect();
    assert!(
        names.contains(&"Hex Grid Systems"),
        "Missing Hex Grid Systems"
    );
    assert!(
        names.contains(&"Movement Systems"),
        "Missing Movement Systems"
    );
    assert!(
        names.contains(&"Combat Resolution Systems"),
        "Missing Combat Resolution"
    );
    assert!(
        names.contains(&"Zones of Control"),
        "Missing Zones of Control"
    );
    assert!(
        names.contains(&"Supply and Logistics"),
        "Missing Supply and Logistics"
    );
    assert!(
        names.contains(&"Chit-Pull Activation Systems"),
        "Missing Chit-Pull"
    );
}

#[test]
fn catalog_entries_have_example_games() {
    let app = test_app();
    let catalog = app.world().resource::<MechanicCatalog>();

    // Most entries should have at least one example game
    let with_examples = catalog
        .entries
        .iter()
        .filter(|e| !e.example_games.is_empty())
        .count();
    assert!(
        with_examples > 40,
        "Most entries should have example games, got {with_examples}"
    );
}
