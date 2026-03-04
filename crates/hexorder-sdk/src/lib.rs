//! Hexorder SDK — internal plugin infrastructure.
//!
//! Provides the `HexorderPlugin` trait that all extracted plugin crates
//! implement, plus an adapter to bridge it into Bevy's `Plugin` trait.

use bevy::prelude::*;

/// Unique identifier for a Hexorder plugin.
///
/// Uses the plugin's crate name by convention (e.g., `"hexorder-map-gen"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(pub &'static str);

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

/// Trait for Hexorder plugin crates.
///
/// Every extracted plugin crate implements this trait. It extends Bevy's
/// plugin concept with metadata for the plugin registry. The `build`
/// method wires the plugin's systems, resources, and events into the app.
pub trait HexorderPlugin: Send + Sync + 'static {
    /// Unique identifier for this plugin.
    fn id(&self) -> PluginId;

    /// Human-readable display name.
    fn plugin_name(&self) -> &'static str;

    /// Wire this plugin's systems, resources, and events into the app.
    fn build(&self, app: &mut App);
}

/// Adapter that wraps a [`HexorderPlugin`] into a Bevy [`Plugin`].
///
/// Use this in `main.rs` to register extracted plugin crates:
///
/// ```ignore
/// app.add_plugins(PluginAdapter(hexorder_map_gen::MapGenPlugin));
/// ```
#[derive(Debug)]
pub struct PluginAdapter<T: HexorderPlugin>(pub T);

impl<T: HexorderPlugin + std::fmt::Debug> Plugin for PluginAdapter<T> {
    fn build(&self, app: &mut App) {
        self.0.build(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestPlugin;

    impl HexorderPlugin for TestPlugin {
        fn id(&self) -> PluginId {
            PluginId("test-plugin")
        }
        fn plugin_name(&self) -> &'static str {
            "Test Plugin"
        }
        fn build(&self, _app: &mut App) {}
    }

    #[test]
    fn plugin_id_display() {
        let id = PluginId("hexorder-map-gen");
        assert_eq!(id.to_string(), "hexorder-map-gen");
    }

    #[test]
    fn plugin_adapter_works_as_bevy_plugin() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PluginAdapter(TestPlugin));
        app.update();
    }

    #[test]
    fn hexorder_plugin_metadata() {
        let plugin = TestPlugin;
        assert_eq!(plugin.id(), PluginId("test-plugin"));
        assert_eq!(plugin.plugin_name(), "Test Plugin");
    }
}
