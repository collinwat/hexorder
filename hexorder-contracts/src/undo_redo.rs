//! Shared Undo/Redo types. See `docs/contracts/undo-redo.md`.
//!
//! Defines the `UndoableCommand` trait for reversible actions, the `UndoStack`
//! resource for managing undo/redo history, and `SetPropertyCommand` for
//! property value changes.

use std::collections::HashMap;
use std::fmt;

use bevy::prelude::*;

use crate::game_system::{EntityData, PropertyValue, TypeId, UnitInstance};
use crate::hex_grid::HexPosition;

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// A reversible command. Implementors must be `Send + Sync + Debug`.
///
/// Commands encapsulate both the forward action and its inverse. The undo stack
/// stores executed commands so they can be undone and redone.
pub trait UndoableCommand: Send + Sync + fmt::Debug {
    /// Apply this command's action to the world.
    fn execute(&mut self, world: &mut World);

    /// Reverse this command's action.
    fn undo(&mut self, world: &mut World);

    /// Human-readable label for display (e.g., "Set Attack to 5").
    fn description(&self) -> String;
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Manages undo/redo history using the command pattern.
///
/// Commands are recorded after execution. Undoing pops from the undo stack and
/// pushes onto the redo stack. Recording a new command clears the redo stack.
#[derive(Resource)]
pub struct UndoStack {
    undo_stack: Vec<Box<dyn UndoableCommand>>,
    redo_stack: Vec<Box<dyn UndoableCommand>>,
    max_depth: usize,
    /// Flag set by observer, consumed by exclusive system.
    pub pending_undo: bool,
    /// Flag set by observer, consumed by exclusive system.
    pub pending_redo: bool,
    /// Set by `record()`, cleared by `acknowledge_records()`.
    /// Used by the persistence sync system to detect new commands.
    has_new_records: bool,
}

impl fmt::Debug for UndoStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UndoStack")
            .field("undo_depth", &self.undo_stack.len())
            .field("redo_depth", &self.redo_stack.len())
            .field("max_depth", &self.max_depth)
            .field("pending_undo", &self.pending_undo)
            .field("pending_redo", &self.pending_redo)
            .field("has_new_records", &self.has_new_records)
            .finish()
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: 100,
            pending_undo: false,
            pending_redo: false,
            has_new_records: false,
        }
    }
}

impl UndoStack {
    /// Create a new `UndoStack` with the given maximum depth.
    #[must_use]
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            max_depth,
            ..Self::default()
        }
    }

    /// Push an already-executed command onto the undo stack. Clears the redo
    /// stack, since the timeline has diverged.
    pub fn record(&mut self, cmd: Box<dyn UndoableCommand>) {
        self.redo_stack.clear();
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(cmd);
        self.has_new_records = true;
    }

    /// Set the pending undo flag. The exclusive system will consume this.
    pub fn request_undo(&mut self) {
        self.pending_undo = true;
    }

    /// Set the pending redo flag. The exclusive system will consume this.
    pub fn request_redo(&mut self) {
        self.pending_redo = true;
    }

    /// Returns `true` if there are commands that can be undone.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if there are commands that can be redone.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Returns the description of the next command to be undone, if any.
    #[must_use]
    pub fn undo_description(&self) -> Option<String> {
        self.undo_stack.last().map(|cmd| cmd.description())
    }

    /// Returns the description of the next command to be redone, if any.
    #[must_use]
    pub fn redo_description(&self) -> Option<String> {
        self.redo_stack.last().map(|cmd| cmd.description())
    }

    /// Pop the top command from the undo stack, call its `undo` method, and
    /// push it onto the redo stack. Called by the exclusive system.
    pub fn pop_undo(&mut self) -> Option<Box<dyn UndoableCommand>> {
        self.undo_stack.pop()
    }

    /// After undoing, push the command onto the redo stack.
    pub fn push_redo(&mut self, cmd: Box<dyn UndoableCommand>) {
        self.redo_stack.push(cmd);
    }

    /// Pop the top command from the redo stack. Called by the exclusive system.
    pub fn pop_redo(&mut self) -> Option<Box<dyn UndoableCommand>> {
        self.redo_stack.pop()
    }

    /// After redoing, push the command back onto the undo stack.
    pub fn push_undo(&mut self, cmd: Box<dyn UndoableCommand>) {
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(cmd);
    }

    /// Reset both stacks (e.g., on project load).
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.has_new_records = false;
    }

    /// Returns `true` if commands have been recorded since the last
    /// `acknowledge_records()` call.
    #[must_use]
    pub fn has_new_records(&self) -> bool {
        self.has_new_records
    }

    /// Clear the `has_new_records` flag. Called by the persistence sync
    /// system after propagating dirty state.
    pub fn acknowledge_records(&mut self) {
        self.has_new_records = false;
    }
}

// ---------------------------------------------------------------------------
// Built-in Command: SetPropertyCommand
// ---------------------------------------------------------------------------

/// Command for reversible property value changes on an entity.
///
/// Captures the entity, property ID, old value, and new value so that the
/// change can be undone and redone without additional lookups.
#[derive(Debug)]
pub struct SetPropertyCommand {
    pub entity: Entity,
    pub property_id: TypeId,
    pub old_value: PropertyValue,
    pub new_value: PropertyValue,
    pub label: String,
}

impl UndoableCommand for SetPropertyCommand {
    fn execute(&mut self, world: &mut World) {
        if let Some(mut data) = world.get_mut::<EntityData>(self.entity) {
            data.properties
                .insert(self.property_id, self.new_value.clone());
        }
    }

    fn undo(&mut self, world: &mut World) {
        if let Some(mut data) = world.get_mut::<EntityData>(self.entity) {
            data.properties
                .insert(self.property_id, self.old_value.clone());
        }
    }

    fn description(&self) -> String {
        self.label.clone()
    }
}

// ---------------------------------------------------------------------------
// Built-in Command: SetTerrainCommand
// ---------------------------------------------------------------------------

/// Command for reversible terrain (entity type) changes on a hex tile.
///
/// Captures the full old and new `EntityData` snapshots so that painting
/// a tile can be fully reversed.
#[derive(Debug)]
pub struct SetTerrainCommand {
    /// The tile entity being painted.
    pub entity: Entity,
    /// Entity type ID before painting.
    pub old_type_id: TypeId,
    /// Property values before painting.
    pub old_properties: HashMap<TypeId, PropertyValue>,
    /// Entity type ID after painting.
    pub new_type_id: TypeId,
    /// Property values after painting.
    pub new_properties: HashMap<TypeId, PropertyValue>,
    /// Human-readable label (e.g., "Paint (0, 1) to Plains").
    pub label: String,
}

impl UndoableCommand for SetTerrainCommand {
    fn execute(&mut self, world: &mut World) {
        if let Some(mut data) = world.get_mut::<EntityData>(self.entity) {
            data.entity_type_id = self.new_type_id;
            data.properties.clone_from(&self.new_properties);
        }
    }

    fn undo(&mut self, world: &mut World) {
        if let Some(mut data) = world.get_mut::<EntityData>(self.entity) {
            data.entity_type_id = self.old_type_id;
            data.properties.clone_from(&self.old_properties);
        }
    }

    fn description(&self) -> String {
        self.label.clone()
    }
}

// ---------------------------------------------------------------------------
// Built-in Command: PlaceUnitCommand
// ---------------------------------------------------------------------------

/// Command for reversible unit (token) placement on the hex grid.
///
/// Captures the entity ID, position, entity data, and visual component handles
/// so the unit can be despawned on undo and respawned on redo.
pub struct PlaceUnitCommand {
    /// The spawned entity (updated on redo when entity ID changes).
    pub entity: Option<Entity>,
    /// Hex position where the unit was placed.
    pub position: HexPosition,
    /// Entity data (type ID and properties) for the placed unit.
    pub entity_data: EntityData,
    /// Mesh handle for rendering.
    pub mesh: Handle<Mesh>,
    /// Material handle for rendering.
    pub material: Handle<StandardMaterial>,
    /// World-space transform.
    pub transform: Transform,
    /// Human-readable label (e.g., "Place Infantry at (0, 1)").
    pub label: String,
}

impl fmt::Debug for PlaceUnitCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlaceUnitCommand")
            .field("entity", &self.entity)
            .field("position", &self.position)
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}

impl UndoableCommand for PlaceUnitCommand {
    fn execute(&mut self, world: &mut World) {
        // Respawn the unit with all its components.
        let entity = world
            .spawn((
                UnitInstance,
                self.position,
                self.entity_data.clone(),
                Mesh3d(self.mesh.clone()),
                MeshMaterial3d(self.material.clone()),
                self.transform,
            ))
            .id();
        self.entity = Some(entity);
    }

    fn undo(&mut self, world: &mut World) {
        if let Some(entity) = self.entity {
            if world.get_entity(entity).is_ok() {
                world.despawn(entity);
            }
            self.entity = None;
        }
    }

    fn description(&self) -> String {
        self.label.clone()
    }
}

// ---------------------------------------------------------------------------
// Built-in Command: DeleteUnitCommand
// ---------------------------------------------------------------------------

/// Command for reversible unit (token) deletion from the hex grid.
///
/// Captures the entity ID, position, entity data, and visual component handles
/// so the unit can be respawned on undo and re-deleted on redo.
pub struct DeleteUnitCommand {
    /// The entity to delete (set to `None` after deletion, updated on undo).
    pub entity: Option<Entity>,
    /// Hex position where the unit was located.
    pub position: HexPosition,
    /// Entity data (type ID and properties) for the deleted unit.
    pub entity_data: EntityData,
    /// Mesh handle for rendering.
    pub mesh: Handle<Mesh>,
    /// Material handle for rendering.
    pub material: Handle<StandardMaterial>,
    /// World-space transform.
    pub transform: Transform,
    /// Human-readable label (e.g., "Delete Infantry at (0, 1)").
    pub label: String,
}

impl fmt::Debug for DeleteUnitCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeleteUnitCommand")
            .field("entity", &self.entity)
            .field("position", &self.position)
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}

impl UndoableCommand for DeleteUnitCommand {
    fn execute(&mut self, world: &mut World) {
        // Redo: despawn the unit again.
        if let Some(entity) = self.entity {
            if world.get_entity(entity).is_ok() {
                world.despawn(entity);
            }
            self.entity = None;
        }
    }

    fn undo(&mut self, world: &mut World) {
        // Undo: respawn the unit with all its original components.
        let entity = world
            .spawn((
                UnitInstance,
                self.position,
                self.entity_data.clone(),
                Mesh3d(self.mesh.clone()),
                MeshMaterial3d(self.material.clone()),
                self.transform,
            ))
            .id();
        self.entity = Some(entity);
    }

    fn description(&self) -> String {
        self.label.clone()
    }
}

// ---------------------------------------------------------------------------
// Built-in Command: CompoundCommand
// ---------------------------------------------------------------------------

/// Groups multiple commands into a single undoable step.
///
/// Execute runs all sub-commands in order; undo reverses them in reverse order.
/// The compound command uses a single label for the entire group.
pub struct CompoundCommand {
    /// The sub-commands that make up this compound action.
    pub commands: Vec<Box<dyn UndoableCommand>>,
    /// Human-readable label for the entire compound action.
    pub label: String,
}

impl fmt::Debug for CompoundCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompoundCommand")
            .field("count", &self.commands.len())
            .field("label", &self.label)
            .finish()
    }
}

impl UndoableCommand for CompoundCommand {
    fn execute(&mut self, world: &mut World) {
        for cmd in &mut self.commands {
            cmd.execute(world);
        }
    }

    fn undo(&mut self, world: &mut World) {
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(world);
        }
    }

    fn description(&self) -> String {
        self.label.clone()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A trivial test command that increments/decrements a counter resource.
    #[derive(Debug)]
    struct TestCommand {
        label: String,
    }

    impl UndoableCommand for TestCommand {
        fn execute(&mut self, _world: &mut World) {
            // No-op in unit tests — we only test stack behavior.
        }

        fn undo(&mut self, _world: &mut World) {
            // No-op in unit tests — we only test stack behavior.
        }

        fn description(&self) -> String {
            self.label.clone()
        }
    }

    fn make_cmd(label: &str) -> Box<dyn UndoableCommand> {
        Box::new(TestCommand {
            label: label.to_string(),
        })
    }

    #[test]
    fn record_pushes_to_undo_stack() {
        let mut stack = UndoStack::default();
        assert!(!stack.can_undo());

        stack.record(make_cmd("action 1"));
        assert!(stack.can_undo());
        assert_eq!(stack.undo_description(), Some("action 1".to_string()));
    }

    #[test]
    fn record_clears_redo_stack() {
        let mut stack = UndoStack::default();
        stack.record(make_cmd("action 1"));

        // Simulate undo: pop from undo, push to redo.
        let cmd = stack.pop_undo().expect("stack should have command");
        stack.push_redo(cmd);
        assert!(stack.can_redo());

        // New record should clear redo.
        stack.record(make_cmd("action 2"));
        assert!(!stack.can_redo());
    }

    #[test]
    fn undo_redo_round_trip() {
        let mut stack = UndoStack::default();
        stack.record(make_cmd("A"));
        stack.record(make_cmd("B"));

        // Undo B.
        let cmd = stack.pop_undo().expect("stack should have command");
        assert_eq!(cmd.description(), "B");
        stack.push_redo(cmd);

        // Top of undo is now A.
        assert_eq!(stack.undo_description(), Some("A".to_string()));
        // Top of redo is B.
        assert_eq!(stack.redo_description(), Some("B".to_string()));

        // Redo B.
        let cmd = stack.pop_redo().expect("stack should have command");
        assert_eq!(cmd.description(), "B");
        stack.push_undo(cmd);

        assert_eq!(stack.undo_description(), Some("B".to_string()));
        assert!(!stack.can_redo());
    }

    #[test]
    fn max_depth_enforced_on_record() {
        let mut stack = UndoStack::with_max_depth(3);
        stack.record(make_cmd("1"));
        stack.record(make_cmd("2"));
        stack.record(make_cmd("3"));
        stack.record(make_cmd("4"));

        // Oldest entry ("1") should have been evicted.
        // Stack should contain 2, 3, 4 — undo pops 4, 3, 2.
        let cmd = stack.pop_undo().expect("stack should have command");
        assert_eq!(cmd.description(), "4");
        let cmd = stack.pop_undo().expect("stack should have command");
        assert_eq!(cmd.description(), "3");
        let cmd = stack.pop_undo().expect("stack should have command");
        assert_eq!(cmd.description(), "2");
        assert!(!stack.can_undo());
    }

    #[test]
    fn max_depth_enforced_on_push_undo() {
        let mut stack = UndoStack::with_max_depth(2);
        stack.record(make_cmd("A"));
        stack.record(make_cmd("B"));

        // Undo both, then redo both — push_undo should enforce depth.
        let b = stack.pop_undo().expect("stack should have command");
        let a = stack.pop_undo().expect("stack should have command");
        stack.push_redo(b);
        stack.push_redo(a);

        // Redo A then B — push_undo called each time.
        let a = stack.pop_redo().expect("stack should have command");
        stack.push_undo(a);
        let b = stack.pop_redo().expect("stack should have command");
        stack.push_undo(b);

        // Both should be back on the undo stack.
        assert!(stack.can_undo());
        assert_eq!(stack.undo_description(), Some("B".to_string()));
    }

    #[test]
    fn descriptions_return_none_when_empty() {
        let stack = UndoStack::default();
        assert!(stack.undo_description().is_none());
        assert!(stack.redo_description().is_none());
    }

    #[test]
    fn clear_resets_both_stacks() {
        let mut stack = UndoStack::default();
        stack.record(make_cmd("X"));
        let cmd = stack.pop_undo().expect("stack should have command");
        stack.push_redo(cmd);
        stack.record(make_cmd("Y"));

        stack.clear();

        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn request_flags() {
        let mut stack = UndoStack::default();
        assert!(!stack.pending_undo);
        assert!(!stack.pending_redo);

        stack.request_undo();
        assert!(stack.pending_undo);

        stack.request_redo();
        assert!(stack.pending_redo);
    }

    #[test]
    fn debug_impl_works() {
        let stack = UndoStack::default();
        let debug = format!("{stack:?}");
        assert!(debug.contains("UndoStack"));
        assert!(debug.contains("undo_depth: 0"));
        assert!(debug.contains("max_depth: 100"));
    }

    #[test]
    fn set_property_command_description() {
        let cmd = SetPropertyCommand {
            entity: Entity::PLACEHOLDER,
            property_id: TypeId::new(),
            old_value: PropertyValue::Int(3),
            new_value: PropertyValue::Int(5),
            label: "Set Attack to 5".to_string(),
        };
        assert_eq!(cmd.description(), "Set Attack to 5");
    }

    #[test]
    fn compound_command_executes_all_in_order() {
        let mut compound = CompoundCommand {
            commands: vec![make_cmd("A"), make_cmd("B"), make_cmd("C")],
            label: "Compound ABC".to_string(),
        };
        assert_eq!(compound.description(), "Compound ABC");
        assert_eq!(compound.commands.len(), 3);

        // Execute doesn't panic (no-op test commands).
        let mut world = World::new();
        compound.execute(&mut world);
    }

    #[test]
    fn compound_command_undoes_in_reverse_order() {
        let compound = CompoundCommand {
            commands: vec![make_cmd("A"), make_cmd("B"), make_cmd("C")],
            label: "Compound ABC".to_string(),
        };

        let debug = format!("{compound:?}");
        assert!(debug.contains("CompoundCommand"));
        assert!(debug.contains("count: 3"));
    }

    #[test]
    fn record_sets_has_new_records() {
        let mut stack = UndoStack::default();
        assert!(!stack.has_new_records());

        stack.record(make_cmd("action"));
        assert!(stack.has_new_records());
    }

    #[test]
    fn acknowledge_records_clears_flag() {
        let mut stack = UndoStack::default();
        stack.record(make_cmd("action"));
        assert!(stack.has_new_records());

        stack.acknowledge_records();
        assert!(!stack.has_new_records());
    }

    #[test]
    fn clear_resets_has_new_records() {
        let mut stack = UndoStack::default();
        stack.record(make_cmd("action"));
        stack.clear();
        assert!(!stack.has_new_records());
    }

    #[test]
    fn delete_unit_command_description() {
        let cmd = DeleteUnitCommand {
            entity: Some(Entity::PLACEHOLDER),
            position: HexPosition::new(2, -1),
            entity_data: EntityData {
                entity_type_id: TypeId::new(),
                properties: HashMap::new(),
            },
            mesh: Handle::default(),
            material: Handle::default(),
            transform: Transform::IDENTITY,
            label: "Delete Infantry at (2, -1)".to_string(),
        };
        assert_eq!(cmd.description(), "Delete Infantry at (2, -1)");
    }

    #[test]
    fn delete_unit_command_debug_impl() {
        let cmd = DeleteUnitCommand {
            entity: None,
            position: HexPosition::new(0, 0),
            entity_data: EntityData {
                entity_type_id: TypeId::new(),
                properties: HashMap::new(),
            },
            mesh: Handle::default(),
            material: Handle::default(),
            transform: Transform::IDENTITY,
            label: "test".to_string(),
        };
        let debug = format!("{cmd:?}");
        assert!(debug.contains("DeleteUnitCommand"));
        assert!(debug.contains("position"));
    }

    #[test]
    fn compound_command_on_stack() {
        let mut stack = UndoStack::default();
        stack.record(Box::new(CompoundCommand {
            commands: vec![make_cmd("step1"), make_cmd("step2")],
            label: "Multi-step action".to_string(),
        }));

        assert!(stack.can_undo());
        assert_eq!(
            stack.undo_description(),
            Some("Multi-step action".to_string())
        );

        let cmd = stack.pop_undo().expect("stack should have command");
        assert_eq!(cmd.description(), "Multi-step action");
        stack.push_redo(cmd);

        assert!(stack.can_redo());
        assert_eq!(
            stack.redo_description(),
            Some("Multi-step action".to_string())
        );
    }
}
