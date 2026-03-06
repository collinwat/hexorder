//! Systems and factory functions for the `game_system` plugin.
//!
//! Factory functions delegate to `hexorder_contracts::defaults`.

use hexorder_contracts::defaults;
use hexorder_contracts::game_system::{EntityTypeRegistry, EnumRegistry, GameSystem};
use hexorder_contracts::mechanics::{CombatResultsTable, TurnStructure};

/// Creates a new `GameSystem` resource with a fresh UUID and default version.
pub fn create_game_system() -> GameSystem {
    defaults::create_game_system()
}

/// Creates the default `EntityTypeRegistry` populated with starter entity types.
pub fn create_entity_type_registry() -> EntityTypeRegistry {
    defaults::create_entity_type_registry()
}

/// Creates the default `EnumRegistry` with starter enum definitions.
pub fn create_enum_registry() -> EnumRegistry {
    defaults::create_enum_registry()
}

/// Creates a default 5-phase turn structure for new game systems.
pub fn create_default_turn_structure() -> TurnStructure {
    defaults::create_default_turn_structure()
}

/// Creates a default CRT with standard odds-ratio columns and 6 rows (1d6).
pub fn create_default_crt() -> CombatResultsTable {
    defaults::create_default_crt()
}
