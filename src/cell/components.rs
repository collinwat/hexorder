//! Feature-local components and resources for the cell plugin.
//!
//! Contract types (CellData, CellTypeId, CellTypeRegistry, ActiveCellType)
//! live in `crate::contracts::game_system`. This module holds types that are
//! internal to the cell feature plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::contracts::game_system::CellTypeId;

/// Stores pre-created material handles for each cell type.
/// Keyed by CellTypeId for dynamic lookup.
#[derive(Resource, Debug)]
pub struct CellMaterials {
    pub materials: HashMap<CellTypeId, Handle<StandardMaterial>>,
}

impl CellMaterials {
    /// Look up the material handle for a given cell type ID.
    pub fn get(&self, id: CellTypeId) -> Option<&Handle<StandardMaterial>> {
        self.materials.get(&id)
    }
}
