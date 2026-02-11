//! Unit tests for the editor_ui feature plugin.

use bevy::prelude::*;

use crate::contracts::editor_ui::EditorTool;

use super::components::EditorState;

#[test]
fn editor_tool_defaults_to_select() {
    let tool = EditorTool::default();
    assert_eq!(tool, EditorTool::Select);
}

#[test]
fn editor_tool_variants_are_distinct() {
    assert_ne!(EditorTool::Select, EditorTool::Paint);
    assert_ne!(EditorTool::Select, EditorTool::Place);
    assert_ne!(EditorTool::Paint, EditorTool::Place);
}

#[test]
fn editor_tool_resource_inserts_correctly() {
    let mut app = App::new();
    app.insert_resource(EditorTool::default());
    app.update();

    let tool = app.world().resource::<EditorTool>();
    assert_eq!(*tool, EditorTool::Select);
}

#[test]
fn editor_state_defaults() {
    let state = EditorState::default();
    // Cell fields
    assert!(state.new_type_name.is_empty());
    assert_eq!(state.new_type_color, [0.5, 0.5, 0.5]);
    assert!(state.new_prop_name.is_empty());
    assert_eq!(state.new_prop_type_index, 0);
    assert!(state.new_enum_options.is_empty());
    // Unit fields
    assert!(state.new_unit_type_name.is_empty());
    assert_eq!(state.new_unit_type_color, [0.5, 0.5, 0.5]);
    assert!(state.new_unit_prop_name.is_empty());
    assert_eq!(state.new_unit_prop_type_index, 0);
    assert!(state.new_unit_enum_options.is_empty());
}

#[test]
fn editor_state_resource_inserts_correctly() {
    let mut app = App::new();
    app.insert_resource(EditorState::default());
    app.update();

    let state = app.world().resource::<EditorState>();
    assert!(state.new_type_name.is_empty());
    assert_eq!(state.new_prop_type_index, 0);
}
