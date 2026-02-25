//! Settings plugin.
//!
//! Provides a three-layer settings infrastructure merging compiled defaults,
//! user config (TOML), and project overrides into a typed `SettingsRegistry`.

use bevy::prelude::*;

use crate::contracts::persistence::AppScreen;
use crate::contracts::settings::SettingsReady;

mod config;
mod systems;

#[cfg(test)]
mod tests;

/// Internal resource holding the three settings layers for re-merge.
/// Plugin-private — not exposed through contracts.
#[derive(Resource, Debug)]
pub(crate) struct SettingsLayers {
    pub(crate) defaults: config::PartialSettings,
    pub(crate) user: config::PartialSettings,
    pub(crate) project: config::PartialSettings,
}

/// Plugin that manages layered settings.
#[derive(Debug)]
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        // 1. Build layers.
        let defaults = config::PartialSettings::defaults();
        let user = config::load_user_settings();
        let project = config::PartialSettings::default();

        // 2. Merge and insert.
        let registry = config::merge(&defaults, &user, &project);
        app.insert_resource(registry);
        app.insert_resource(SettingsLayers {
            defaults,
            user,
            project,
        });

        // 3. Project layer lifecycle.
        app.add_systems(
            OnEnter(AppScreen::Editor),
            systems::apply_project_layer.in_set(SettingsReady),
        );
        app.add_systems(OnExit(AppScreen::Editor), systems::clear_project_layer);
    }
}
