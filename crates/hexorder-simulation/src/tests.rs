use bevy::prelude::*;
use hexorder_contracts::simulation::{
    ColumnType, DieType, ResolutionTable, SimulationRng, TableColumn, TableResult, TableRow,
    reset_rng, resolve_table, roll_die,
};

/// `SimulationPlugin` inserts `SimulationRng` resource.
#[test]
fn simulation_rng_resource_available() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(super::SimulationPlugin);

    assert!(
        app.world().get_resource::<SimulationRng>().is_some(),
        "SimulationRng should exist after plugin build"
    );
}

/// `DieRolled` event fires when a die is rolled through the plugin.
#[test]
fn die_rolled_event_fires() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(super::SimulationPlugin);
    app.update();

    // Roll a die via the resource.
    let mut rng = app.world_mut().resource_mut::<SimulationRng>();
    let _result = roll_die(&mut rng, DieType::D6, "test");

    assert_eq!(rng.roll_count(), 1);
}

/// End-to-end: seed RNG → roll → resolve table → verify deterministic.
#[test]
fn rng_table_resolution_deterministic() {
    let table = ResolutionTable {
        id: hexorder_contracts::game_system::TypeId::new(),
        name: "Test CRT".to_string(),
        columns: vec![
            TableColumn {
                label: "1:2".to_string(),
                column_type: ColumnType::Ratio,
                threshold: 0.5,
            },
            TableColumn {
                label: "1:1".to_string(),
                column_type: ColumnType::Ratio,
                threshold: 1.0,
            },
            TableColumn {
                label: "2:1".to_string(),
                column_type: ColumnType::Ratio,
                threshold: 2.0,
            },
        ],
        rows: vec![
            TableRow {
                label: "1-2".to_string(),
                value_min: 1,
                value_max: 2,
            },
            TableRow {
                label: "3-4".to_string(),
                value_min: 3,
                value_max: 4,
            },
            TableRow {
                label: "5-6".to_string(),
                value_min: 5,
                value_max: 6,
            },
        ],
        outcomes: vec![
            vec![
                TableResult::Text("AE".to_string()),
                TableResult::Text("NE".to_string()),
                TableResult::Text("DR".to_string()),
            ],
            vec![
                TableResult::Text("NE".to_string()),
                TableResult::Text("DR".to_string()),
                TableResult::Text("DE".to_string()),
            ],
            vec![
                TableResult::Text("EX".to_string()),
                TableResult::Text("SL".to_string()),
                TableResult::Text("DSL".to_string()),
            ],
        ],
    };

    // Run the same resolution twice with the same seed.
    let mut rng1 = SimulationRng::new(42);
    let roll1 = roll_die(&mut rng1, DieType::D6, "combat");
    let result1 = resolve_table(&table, 6.0, 2.0, roll1);

    let mut rng2 = SimulationRng::new(42);
    let roll2 = roll_die(&mut rng2, DieType::D6, "combat");
    let result2 = resolve_table(&table, 6.0, 2.0, roll2);

    assert_eq!(roll1, roll2, "Same seed should produce same roll");
    let r1 = result1.expect("result1 should resolve");
    let r2 = result2.expect("result2 should resolve");
    assert_eq!(r1.column_index, r2.column_index);
    assert_eq!(r1.row_index, r2.row_index);
    assert_eq!(r1.column_label, r2.column_label);
    assert_eq!(r1.row_label, r2.row_label);
}

/// Reset and re-roll produces the same sequence.
#[test]
fn reset_replays_same_sequence() {
    let mut rng = SimulationRng::new(42);
    let first_run: Vec<u32> = (0..10)
        .map(|_| roll_die(&mut rng, DieType::D6, ""))
        .collect();

    reset_rng(&mut rng, 42);
    let second_run: Vec<u32> = (0..10)
        .map(|_| roll_die(&mut rng, DieType::D6, ""))
        .collect();

    assert_eq!(first_run, second_run);
}
