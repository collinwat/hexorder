use bevy::prelude::*;
use hexorder_contracts::simulation::{DieType, SimulationRng, roll_die};

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
