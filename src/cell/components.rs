//! Plugin-local components and resources for the cell plugin.
//!
//! Contract types (`EntityData`, `EntityTypeRegistry`, `ActiveBoardType`)
//! live in `crate::contracts::game_system`. This module holds types that are
//! internal to the cell plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::contracts::game_system::TypeId;

/// Stores pre-created material handles for each `BoardPosition` entity type.
/// Keyed by `TypeId` for dynamic lookup.
#[derive(Resource, Debug)]
pub struct CellMaterials {
    pub materials: HashMap<TypeId, Handle<StandardMaterial>>,
}

impl CellMaterials {
    /// Look up the material handle for a given entity type ID.
    pub fn get(&self, id: TypeId) -> Option<&Handle<StandardMaterial>> {
        self.materials.get(&id)
    }
}
