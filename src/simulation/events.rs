use bevy::prelude::*;
use hexorder_contracts::game_system::TypeId;
use hexorder_contracts::simulation::{RollRecord, TableResolution};

/// Fired when a die is rolled (for UI display, logging).
#[derive(Event, Debug, Clone)]
pub struct DieRolled {
    pub record: RollRecord,
}

/// Fired when a table resolution completes.
#[derive(Event, Debug, Clone)]
pub struct TableResolved {
    pub table_id: TypeId,
    pub resolution: TableResolution,
    pub roll: Option<RollRecord>,
}
