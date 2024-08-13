use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

#[derive(Bundle, Default)]
pub struct PanOrbitCameraBundle {
    pub camera: Camera3dBundle,
    pub state: PanOrbitState,
    pub settings: PanOrbitSettings,
}

#[derive(Component)]
pub struct PanOrbitState {
    pub center: Vec3,
    pub radius: f32,
    pub upside_down: bool,
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Component)]
pub struct PanOrbitSettings {
    /// World units per pixel of mouse motion
    pub pan_sensitivity: f32,
    /// Radians per pixel of mouse motion
    pub orbit_sensitivity: f32,
    /// Exponent per pixel of mouse motion
    pub zoom_sensitivity: f32,
    /// Keybind for panning
    pub pan_key: Option<KeyCode>,
    /// Keybind for orbiting
    pub orbit_key: Option<KeyCode>,
    /// Keybind for zooming,
    pub zoom_key: Option<KeyCode>,
    /// Action binding for the scroll wheel
    pub scroll_action: Option<PanOrbitAction>,
    /// For non-smoothed scrolling
    pub scroll_line_sensitivity: f32,
    /// For smooth scrolling
    pub scroll_pixel_sensitivity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanOrbitAction {
    Pan,
    Orbit,
    Zoom,
}

pub fn pan_orbit_camera(
    kbd: Res<ButtonInput<KeyCode>>,
    mut evr_motion: EventReader<MouseMotion>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut q_camera: Query<(
        &PanOrbitSettings,
        &mut PanOrbitState,
        &mut Transform,
    )>,
) {
    // Accumulate total mouse motion and scroll for all pending events
    let mut total_motion: Vec2 = evr_motion.read()
        .map(|ev| ev.delta).sum();

    // Reverse Y
    // Bevy's default is Y-Up, but events are in window/ui coords (Y-Down)
    total_motion.y = -total_motion.y;

    let mut total_scroll_lines = Vec2::ZERO;
    let mut total_scroll_pixels = Vec2::ZERO;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                total_scroll_lines.x += ev.x;
                total_scroll_lines.y -= ev.y;
            }
            MouseScrollUnit::Pixel => {
                total_scroll_pixels.x += ev.x;
                total_scroll_pixels.y -= ev.y;
            }
        }
    }

    for (settings, mut state, mut transform) in &mut q_camera {
        // Apply motion and scroll to component configuration

        // PanAction: Pan
        let mut total_pan = Vec2::ZERO;
        if settings.pan_key.map(|key| kbd.pressed(key)).unwrap_or(false) {
            total_pan -= total_motion * settings.pan_sensitivity;
        }

        if settings.scroll_action == Some(PanOrbitAction::Pan) {
            total_pan -= total_scroll_lines
                * settings.scroll_line_sensitivity * settings.pan_sensitivity;
            total_pan -= total_scroll_pixels
                * settings.scroll_pixel_sensitivity * settings.pan_sensitivity;
        }

        // PanAction: Orbit
        let mut total_orbit = Vec2::ZERO;
        if settings.orbit_key.map(|key| kbd.pressed(key)).unwrap_or(false) {
            total_pan -= total_motion * settings.zoom_sensitivity;
        }
        if settings.scroll_action == Some(PanOrbitAction::Orbit) {
            total_orbit -= total_scroll_lines
                * settings.scroll_line_sensitivity * settings.orbit_sensitivity;
            total_orbit -= total_scroll_pixels
                * settings.scroll_pixel_sensitivity * settings.orbit_sensitivity;
        }

        // PanAction: Zoom
        let mut total_zoom = Vec2::ZERO;
        if settings.zoom_key.map(|key| kbd.pressed(key)).unwrap_or(false) {
            total_pan -= total_motion * settings.zoom_sensitivity;
        }
        if settings.scroll_action == Some(PanOrbitAction::Zoom) {
            total_zoom -= total_scroll_lines
                * settings.scroll_line_sensitivity * settings.zoom_sensitivity;
            total_zoom -= total_scroll_pixels
                * settings.scroll_pixel_sensitivity * settings.zoom_sensitivity;
        }

        // When beggining to orbit, check if we are starting upside-down
        if settings.orbit_key.map(|key| kbd.just_pressed(key)).unwrap_or(false) {
            state.upside_down = state.pitch < -FRAC_PI_2 || state.pitch > FRAC_PI_2;
        }

        // If we are upside down, reverse the X orbiting
        if state.upside_down {
            total_orbit.x = -total_orbit.x;
        }

        let mut any = false;

        // To Zoom, we multiply our radius
        if total_zoom != Vec2::ZERO {
            any = true;
            // N: In order for zoom to feel intuitive,
            // movement needs to be exponential (via multiplication)
            // rather than linear (via addition)
            state.radius *= (-total_zoom.y).exp();
        }

        // To Orbit, we modify pitch and yaw
        if total_orbit != Vec2::ZERO {
            any = true;
            state.yaw = total_orbit.x;
            state.pitch = total_orbit.y;
            if state.yaw > PI {
                state.yaw -= TAU; // 2 * PI
            }
            if state.yaw < -PI {
                state.yaw += TAU;
            }
            if state.pitch > PI {
                state.pitch -= TAU;
            }
            if state.pitch < -PI {
                state.pitch += TAU;
            }
        };

        // To Pan, gather UP and RIGHT direction
        // vectors from the camera's transform, and use them
        // to move center point. Multiply by the radius to make
        // the pan adapt to current zoom.
        if total_pan != Vec2::ZERO {
            any = true;
            let radius = state.radius;
            state.center += transform.right() * total_pan.x * radius;
            state.center += transform.up() * total_pan.y * radius;
        }

        // Finally, comput the new camera Transform
        // If pan-orbit controller is just-added or updated, initialize
        if any || state.is_added() {
            transform.rotation =
                Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
            transform.translation = state.center + transform.back() * state.radius;
        }
    }
}

impl Default for PanOrbitState {
    fn default() -> Self {
        PanOrbitState {
            center: Vec3::ZERO,
            radius: 1.0,
            upside_down: false,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

impl Default for PanOrbitSettings {
    fn default() -> Self {
        PanOrbitSettings {
            pan_sensitivity: 0.001, // 1000 pixels per world unit
            orbit_sensitivity: 0.1f32.to_radians(), // 0.1 degree per pixel
            zoom_sensitivity: 0.01,
            pan_key: Some(KeyCode::ControlLeft),
            orbit_key: Some(KeyCode::AltLeft),
            zoom_key: Some(KeyCode::ShiftLeft),
            scroll_action: Some(PanOrbitAction::Zoom),
            scroll_line_sensitivity: 16.0,
            scroll_pixel_sensitivity: 1.0,
        }
    }
}
