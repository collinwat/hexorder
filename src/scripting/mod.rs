//! Scripting plugin: embedded Lua (`LuaJIT`) for game rule definitions
//! and integration test automation.

use bevy::prelude::*;

use crate::contracts::persistence::AppScreen;

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
