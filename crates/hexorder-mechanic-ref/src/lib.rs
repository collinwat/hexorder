//! Mechanic reference catalog plugin.
//!
//! Populates a browsable catalog of hex wargame mechanics organized by
//! the Engelstein taxonomy. Each entry includes descriptions, example
//! games, design considerations, and optional scaffolding templates.

use bevy::prelude::*;

use hexorder_sdk::{HexorderPlugin, PluginId};

#[allow(dead_code)]
mod components;
mod systems;

#[cfg(test)]
mod tests;

/// Plugin that provides the mechanic reference catalog.
#[derive(Debug)]
pub struct MechanicReferencePlugin;

impl Plugin for MechanicReferencePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(systems::create_catalog());
    }
}

impl HexorderPlugin for MechanicReferencePlugin {
    fn id(&self) -> PluginId {
        PluginId("hexorder-mechanic-ref")
    }

    fn plugin_name(&self) -> &'static str {
        "Mechanic Reference"
    }

    fn build(&self, app: &mut App) {
        Plugin::build(self, app);
    }
}
