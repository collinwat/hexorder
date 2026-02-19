//! Shared Undo/Redo types. See `docs/contracts/undo-redo.md`.
//!
//! Defines the `UndoableCommand` trait for reversible actions, the `UndoStack`
//! resource for managing undo/redo history, and `SetPropertyCommand` for
//! property value changes.

use std::fmt;

use bevy::prelude::*;

use super::game_system::{EntityData, PropertyValue, TypeId};

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
    pub(crate) pending_undo: bool,
    /// Flag set by observer, consumed by exclusive system.
    pub(crate) pending_redo: bool,
}

impl fmt::Debug for UndoStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UndoStack")
            .field("undo_depth", &self.undo_stack.len())
            .field("redo_depth", &self.redo_stack.len())
            .field("max_depth", &self.max_depth)
            .field("pending_undo", &self.pending_undo)
            .field("pending_redo", &self.pending_redo)
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
    pub(crate) fn pop_undo(&mut self) -> Option<Box<dyn UndoableCommand>> {
        self.undo_stack.pop()
    }

    /// After undoing, push the command onto the redo stack.
    pub(crate) fn push_redo(&mut self, cmd: Box<dyn UndoableCommand>) {
        self.redo_stack.push(cmd);
    }

    /// Pop the top command from the redo stack. Called by the exclusive system.
    pub(crate) fn pop_redo(&mut self) -> Option<Box<dyn UndoableCommand>> {
        self.redo_stack.pop()
    }

    /// After redoing, push the command back onto the undo stack.
    pub(crate) fn push_undo(&mut self, cmd: Box<dyn UndoableCommand>) {
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(cmd);
    }

    /// Reset both stacks (e.g., on project load).
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
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
}
