#[allow(dead_code)]
mod components;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct MechanicReferencePlugin;

impl bevy::prelude::Plugin for MechanicReferencePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<components::MechanicCatalog>();
    }
}
