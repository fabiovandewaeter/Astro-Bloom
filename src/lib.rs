use bevy::math::UVec2;

pub mod camera;

pub const UPS_TARGET: f64 = 30.0;
pub const ZOOM_IN_SPEED: f32 = 0.25 / 400000000.0;
pub const ZOOM_OUT_SPEED: f32 = 4.0 * 400000000.0;
pub const CAMERA_SPEED: f32 = 37.5;

pub const TILE_SIZE: UVec2 = UVec2 { x: 16, y: 16 };
