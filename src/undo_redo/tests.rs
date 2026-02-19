use bevy::prelude::*;

use crate::contracts::game_system::{EntityData, PropertyValue, TypeId};
use crate::contracts::shortcuts::{CommandExecutedEvent, CommandId, ShortcutRegistry};
use crate::contracts::undo_redo::{SetPropertyCommand, UndoStack};

/// Build a minimal app with the `UndoRedoPlugin` for testing.
/// Inserts `ShortcutRegistry` manually (normally provided by `ShortcutsPlugin`).
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ShortcutRegistry>();
    app.add_plugins(super::UndoRedoPlugin);
    app
}

#[test]
fn undo_redo_plugin_inserts_stack_resource() {
    let app = test_app();
    assert!(app.world().get_resource::<UndoStack>().is_some());
}

#[test]
fn process_undo_reverses_command() {
    let mut app = test_app();
    app.update();

    let prop_id = TypeId::new();
    let entity = app
        .world_mut()
        .spawn(EntityData {
            entity_type_id: TypeId::new(),
            properties: [(prop_id, PropertyValue::Int(10))].into(),
        })
        .id();

    // Record a property change (forward mutation already applied).
    app.world_mut()
        .resource_mut::<UndoStack>()
        .record(Box::new(SetPropertyCommand {
            entity,
            property_id: prop_id,
            old_value: PropertyValue::Int(10),
            new_value: PropertyValue::Int(20),
            label: "Set to 20".to_string(),
        }));

    // Request undo.
    app.world_mut().resource_mut::<UndoStack>().request_undo();
    app.update(); // Exclusive system processes the undo.

    let data = app
        .world()
        .entity(entity)
        .get::<EntityData>()
        .expect("entity should have EntityData");
    assert_eq!(
        data.properties.get(&prop_id),
        Some(&PropertyValue::Int(10)),
        "Property should be reverted to old value"
    );
}

#[test]
fn process_redo_reapplies_command() {
    let mut app = test_app();
    app.update();

    let prop_id = TypeId::new();
    let entity = app
        .world_mut()
        .spawn(EntityData {
            entity_type_id: TypeId::new(),
            properties: [(prop_id, PropertyValue::Int(10))].into(),
        })
        .id();

    app.world_mut()
        .resource_mut::<UndoStack>()
        .record(Box::new(SetPropertyCommand {
            entity,
            property_id: prop_id,
            old_value: PropertyValue::Int(10),
            new_value: PropertyValue::Int(20),
            label: "Set to 20".to_string(),
        }));

    // Undo, then redo.
    app.world_mut().resource_mut::<UndoStack>().request_undo();
    app.update();
    app.world_mut().resource_mut::<UndoStack>().request_redo();
    app.update();

    let data = app
        .world()
        .entity(entity)
        .get::<EntityData>()
        .expect("entity should have EntityData");
    assert_eq!(
        data.properties.get(&prop_id),
        Some(&PropertyValue::Int(20)),
        "Property should be re-applied to new value"
    );
}

#[test]
fn undo_via_shortcut_command() {
    let mut app = test_app();
    app.update();

    let prop_id = TypeId::new();
    let entity = app
        .world_mut()
        .spawn(EntityData {
            entity_type_id: TypeId::new(),
            properties: [(prop_id, PropertyValue::Int(10))].into(),
        })
        .id();

    app.world_mut()
        .resource_mut::<UndoStack>()
        .record(Box::new(SetPropertyCommand {
            entity,
            property_id: prop_id,
            old_value: PropertyValue::Int(10),
            new_value: PropertyValue::Int(20),
            label: "Set to 20".to_string(),
        }));

    // Fire edit.undo via CommandExecutedEvent (simulating Cmd+Z).
    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.undo"),
    });
    app.update();

    let data = app
        .world()
        .entity(entity)
        .get::<EntityData>()
        .expect("entity should have EntityData");
    assert_eq!(
        data.properties.get(&prop_id),
        Some(&PropertyValue::Int(10)),
        "Undo via shortcut should revert property"
    );
}

#[test]
fn redo_via_shortcut_command() {
    let mut app = test_app();
    app.update();

    let prop_id = TypeId::new();
    let entity = app
        .world_mut()
        .spawn(EntityData {
            entity_type_id: TypeId::new(),
            properties: [(prop_id, PropertyValue::Int(10))].into(),
        })
        .id();

    app.world_mut()
        .resource_mut::<UndoStack>()
        .record(Box::new(SetPropertyCommand {
            entity,
            property_id: prop_id,
            old_value: PropertyValue::Int(10),
            new_value: PropertyValue::Int(20),
            label: "Set to 20".to_string(),
        }));

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.undo"),
    });
    app.update();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.redo"),
    });
    app.update();

    let data = app
        .world()
        .entity(entity)
        .get::<EntityData>()
        .expect("entity should have EntityData");
    assert_eq!(
        data.properties.get(&prop_id),
        Some(&PropertyValue::Int(20)),
        "Redo via shortcut should re-apply property"
    );
}

#[test]
fn new_record_after_undo_clears_redo() {
    let mut app = test_app();
    app.update();

    let prop_id = TypeId::new();
    let entity = app
        .world_mut()
        .spawn(EntityData {
            entity_type_id: TypeId::new(),
            properties: [(prop_id, PropertyValue::Int(10))].into(),
        })
        .id();

    // Record action 1.
    app.world_mut()
        .resource_mut::<UndoStack>()
        .record(Box::new(SetPropertyCommand {
            entity,
            property_id: prop_id,
            old_value: PropertyValue::Int(10),
            new_value: PropertyValue::Int(20),
            label: "Set to 20".to_string(),
        }));

    // Undo action 1.
    app.world_mut().resource_mut::<UndoStack>().request_undo();
    app.update();

    // Record action 2 (should clear redo).
    app.world_mut()
        .resource_mut::<UndoStack>()
        .record(Box::new(SetPropertyCommand {
            entity,
            property_id: prop_id,
            old_value: PropertyValue::Int(10),
            new_value: PropertyValue::Int(30),
            label: "Set to 30".to_string(),
        }));

    assert!(
        !app.world().resource::<UndoStack>().can_redo(),
        "Redo stack should be cleared after new record"
    );
}
