//! Scripting plugin: embedded Lua (`LuaJIT`) for game rule definitions
//! and integration test automation.

use bevy::prelude::*;

use hexorder_contracts::persistence::AppScreen;
use hexorder_sdk::{HexorderPlugin, PluginId};

#[allow(dead_code)]
mod lua_api;
mod systems;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppScreen::Editor), systems::init_lua);
    }
}

impl HexorderPlugin for ScriptingPlugin {
    fn id(&self) -> PluginId {
        PluginId("hexorder-scripting")
    }

    fn plugin_name(&self) -> &'static str {
        "Scripting"
    }

    fn build(&self, app: &mut App) {
        Plugin::build(self, app);
    }
}
