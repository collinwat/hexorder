//! Feature-local components and resources for editor_ui.
//!
//! Contract types (EditorTool) live in `crate::contracts::editor_ui`.
//! This module holds types that are internal to the editor_ui feature plugin.

use bevy::prelude::*;

/// Persistent UI state for the editor panels.
#[derive(Resource, Debug)]
pub struct EditorState {
    // -- Cell type creation fields --
    /// Name for a new cell type being created.
    pub new_type_name: String,
    /// Color for a new cell type (RGB, 0.0-1.0).
    pub new_type_color: [f32; 3],
    /// Name for a new property being added to a cell type.
    pub new_prop_name: String,
    /// Selected property type index (0=Bool, 1=Int, 2=Float, 3=String, 4=Color, 5=Enum).
    pub new_prop_type_index: usize,
    /// Comma-separated enum options when adding an Enum property.
    pub new_enum_options: String,

    // -- Unit type creation fields --
    /// Name for a new unit type being created.
    pub new_unit_type_name: String,
    /// Color for a new unit type (RGB, 0.0-1.0).
    pub new_unit_type_color: [f32; 3],
    /// Name for a new property being added to a unit type.
    pub new_unit_prop_name: String,
    /// Selected property type index for unit properties.
    pub new_unit_prop_type_index: usize,
    /// Comma-separated enum options for unit Enum properties.
    pub new_unit_enum_options: String,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            new_type_name: String::new(),
            new_type_color: [0.5, 0.5, 0.5],
            new_prop_name: String::new(),
            new_prop_type_index: 0,
            new_enum_options: String::new(),
            new_unit_type_name: String::new(),
            new_unit_type_color: [0.5, 0.5, 0.5],
            new_unit_prop_name: String::new(),
            new_unit_prop_type_index: 0,
            new_unit_enum_options: String::new(),
        }
    }
}
