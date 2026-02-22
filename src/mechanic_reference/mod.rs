#[allow(dead_code)]
mod components;
mod systems;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct MechanicReferencePlugin;

impl bevy::prelude::Plugin for MechanicReferencePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(systems::create_catalog());
    }
}
