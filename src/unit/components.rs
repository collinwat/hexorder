//! Feature-local components and resources for the unit plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::contracts::game_system::UnitTypeId;

/// Pre-created material handles for each unit type, keyed by `UnitTypeId`.
#[derive(Resource, Debug)]
pub struct UnitMaterials {
    pub materials: HashMap<UnitTypeId, Handle<StandardMaterial>>,
}

impl UnitMaterials {
    pub fn get(&self, id: UnitTypeId) -> Option<&Handle<StandardMaterial>> {
        self.materials.get(&id)
    }
}

/// Shared mesh handle for all unit tokens (a cylinder).
#[derive(Resource, Debug)]
pub struct UnitMesh {
    pub handle: Handle<Mesh>,
}
