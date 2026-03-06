//! Feature-local components and resources for the unit plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use hexorder_contracts::game_system::TypeId;

/// Pre-created material handles for each Token entity type, keyed by `TypeId`.
#[derive(Resource, Debug)]
pub struct UnitMaterials {
    pub materials: HashMap<TypeId, Handle<StandardMaterial>>,
}

impl UnitMaterials {
    pub fn get(&self, id: TypeId) -> Option<&Handle<StandardMaterial>> {
        self.materials.get(&id)
    }
}

/// Shared mesh handle for all unit tokens (a cylinder).
#[derive(Resource, Debug)]
pub struct UnitMesh {
    pub handle: Handle<Mesh>,
}
