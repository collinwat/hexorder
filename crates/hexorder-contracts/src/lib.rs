//! Shared contract types between Hexorder plugins.
//!
//! Each module corresponds to a contract spec in `docs/contracts/`.
//!
//! Contract types are defined ahead of their consumers. Allow `dead_code`
//! since not all types are used by all plugins at every point in development.

#[allow(dead_code)]
pub mod editor_ui;
#[allow(dead_code)]
pub mod game_system;
#[allow(dead_code)]
pub mod hex_grid;
#[allow(dead_code)]
pub mod map_gen;
#[allow(dead_code)]
pub mod mechanic_reference;
#[allow(dead_code)]
pub mod mechanics;
#[allow(dead_code)]
pub mod ontology;
#[allow(dead_code)]
pub mod persistence;
#[allow(dead_code)]
pub mod settings;
#[allow(dead_code)]
pub mod shortcuts;
#[allow(dead_code)]
pub mod storage;
#[allow(dead_code)]
pub mod undo_redo;
#[allow(dead_code)]
pub mod validation;
