//! Camera plugin components.

use bevy::prelude::*;

/// Marker component for the main top-down camera.
/// Attached to the camera entity so systems can query it.
#[derive(Component, Debug)]
pub struct TopDownCamera;

/// Resource holding the camera's desired (target) state.
/// Systems modify these target values; a separate smoothing system
/// interpolates the actual camera transform and projection toward them.
#[derive(Resource, Debug)]
pub struct CameraState {
    /// Target position on the XZ plane (Y is fixed).
    pub target_position: Vec2,
    /// Target orthographic scale (zoom level).
    pub target_scale: f32,
    /// Current smoothed scale (to allow smooth interpolation).
    pub current_scale: f32,
    /// Minimum allowed orthographic scale (most zoomed in).
    pub min_scale: f32,
    /// Maximum allowed orthographic scale (most zoomed out).
    pub max_scale: f32,
    /// Pan speed in world units per second at scale 1.0.
    pub pan_speed: f32,
    /// Zoom speed multiplier per scroll tick.
    pub zoom_speed: f32,
    /// How far panning can go from the origin (world units).
    pub pan_bounds: f32,
    /// Smoothing factor for interpolation (higher = snappier, 0..=1 range after dt multiply).
    pub smoothing: f32,
    /// Whether the user is currently dragging with middle mouse button.
    pub is_dragging: bool,
    /// When true, a reset-view (fit + center) will be applied on the next
    /// frame where `ViewportMargins` are populated. Set by
    /// `configure_bounds_from_grid`; cleared by `apply_pending_reset`.
    pub pending_reset: bool,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            target_position: Vec2::ZERO,
            // Scale values are for ScalingMode::WindowSize(1.0):
            // scale S means 1 pixel covers S world units.
            // 0.03 shows ~30 world units in a 1000px viewport.
            target_scale: 0.03,
            current_scale: 0.03,
            min_scale: 0.005,
            max_scale: 0.15,
            pan_speed: 500.0,
            zoom_speed: 0.1,
            pan_bounds: 50.0,
            smoothing: 10.0,
            is_dragging: false,
            pending_reset: false,
        }
    }
}
