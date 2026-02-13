//! Feature-local components and resources for `editor_ui`.
//!
//! Contract types (`EditorTool`) live in `crate::contracts::editor_ui`.
//! This module holds types that are internal to the `editor_ui` feature plugin.

use bevy::prelude::*;

/// Persistent UI state for the editor panels.
#[derive(Resource, Debug)]
pub struct EditorState {
    /// Name for a new entity type being created.
    pub new_type_name: String,
    /// Color for a new entity type (RGB, 0.0-1.0).
    pub new_type_color: [f32; 3],
    /// Selected role index for new entity type (0 = `BoardPosition`, 1 = `Token`).
    /// Currently the role is determined by which section ("Cell Types" / "Unit Types")
    /// the user creates a type in. This field is reserved for a future unified creation panel.
    #[allow(dead_code)]
    pub new_type_role_index: usize,
    /// Name for a new property being added to an entity type.
    pub new_prop_name: String,
    /// Selected property type index (0=Bool, 1=Int, 2=Float, 3=String, 4=Color, 5=Enum).
    pub new_prop_type_index: usize,
    /// Comma-separated enum options when adding an Enum property.
    pub new_enum_options: String,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            new_type_name: String::new(),
            new_type_color: [0.5, 0.5, 0.5],
            new_type_role_index: 0,
            new_prop_name: String::new(),
            new_prop_type_index: 0,
            new_enum_options: String::new(),
        }
    }
}
