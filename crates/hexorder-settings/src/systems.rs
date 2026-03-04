//! Systems for the settings plugin.

use bevy::prelude::*;

use hexorder_contracts::persistence::Workspace;
use hexorder_contracts::settings::{SettingsChanged, SettingsRegistry};

use super::SettingsLayers;
use super::config::{PartialEditorSettings, PartialSettings, merge};

/// On entering the editor, read the Workspace resource and apply the project
/// layer to the settings registry.
pub(crate) fn apply_project_layer(
    workspace: Res<Workspace>,
    mut layers: ResMut<SettingsLayers>,
    mut registry: ResMut<SettingsRegistry>,
    mut commands: Commands,
) {
    layers.project = PartialSettings {
        editor: PartialEditorSettings {
            font_size: Some(workspace.font_size_base),
            workspace_preset: if workspace.workspace_preset.is_empty() {
                None
            } else {
                Some(workspace.workspace_preset.clone())
            },
        },
        theme: None, // project-level theme override not yet supported
    };

    *registry = merge(&layers.defaults, &layers.user, &layers.project);
    commands.trigger(SettingsChanged);
    info!(
        "Settings: applied project layer (font_size={})",
        registry.editor.font_size
    );
}

/// On exiting the editor, clear the project layer and re-merge.
pub(crate) fn clear_project_layer(
    mut layers: ResMut<SettingsLayers>,
    mut registry: ResMut<SettingsRegistry>,
    mut commands: Commands,
) {
    layers.project = PartialSettings::default();
    *registry = merge(&layers.defaults, &layers.user, &layers.project);
    commands.trigger(SettingsChanged);
    info!("Settings: cleared project layer");
}
