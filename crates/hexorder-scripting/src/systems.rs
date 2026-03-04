//! Systems for the `scripting` plugin.

use bevy::prelude::*;
use mlua::Lua;

/// Wrapper around `mlua::Lua` stored as a `NonSend` resource.
/// `Lua` is `!Send` so it cannot be a normal Bevy `Resource`.
#[allow(dead_code)]
pub(crate) struct LuaState {
    pub(crate) lua: Lua,
}

impl std::fmt::Debug for LuaState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LuaState").finish_non_exhaustive()
    }
}

/// Initialize the Lua VM and register the `hexorder` module.
pub fn init_lua(world: &mut World) {
    let lua = Lua::new();

    // Register the hexorder module table
    lua.globals()
        .set(
            "hexorder",
            super::lua_api::create_hexorder_module(&lua)
                .expect("failed to create hexorder Lua module"),
        )
        .expect("failed to set hexorder global");

    world.insert_non_send_resource(LuaState { lua });
}
