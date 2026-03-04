//! Shared system set definitions for cross-plugin scheduling.
//!
//! Plugin crates use these sets to declare ordering constraints without
//! depending on each other. The main binary wires set ordering in `main.rs`.

use bevy::prelude::*;

/// Top-level execution phases within a single frame.
///
/// Plugins schedule their systems into the appropriate phase. The main
/// binary enforces phase ordering: Input → Simulation → Render.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum HexorderPhase {
    /// Process user input (keyboard, mouse, UI interactions).
    Input,
    /// Run simulation logic (game system, rules, scripting).
    Simulation,
    /// Post-simulation bookkeeping (persistence, undo/redo).
    PostSimulation,
    /// Visual updates (camera, mesh sync, material sync).
    Render,
}

/// Fine-grained simulation sub-sets within [`HexorderPhase::Simulation`].
///
/// Allows ordering between simulation subsystems without direct coupling.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    /// Hex grid spatial operations.
    Grid,
    /// Game system definition and entity type registry.
    GameSystem,
    /// Ontology (property definitions, categories).
    Ontology,
    /// Cell (board position) data assignment and sync.
    Cell,
    /// Unit placement, movement, selection.
    Unit,
    /// Rules engine evaluation.
    Rules,
    /// Lua scripting execution.
    Scripting,
    /// Procedural map generation.
    MapGen,
}
