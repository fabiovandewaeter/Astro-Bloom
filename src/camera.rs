use crate::{CAMERA_SPEED, TILE_SIZE, ZOOM_IN_SPEED, ZOOM_OUT_SPEED};
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};

#[derive(Resource)]
pub struct UpsCounter {
    pub ticks: u32,
    pub last_second: f64,
    pub ups: u32,
}

pub fn display_fps_ups_system(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut counter: ResMut<UpsCounter>,
) {
    let now = time.elapsed_secs_f64();
    if now - counter.last_second >= 1.0 {
        // Calcule l’UPS
        counter.ups = counter.ticks;
        counter.ticks = 0;
        counter.last_second = now;

        // Récupère le FPS depuis le plugin
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_avg) = fps.smoothed() {
                println!("FPS: {:.0} | UPS: {}", fps_avg, counter.ups);
            }
        }
    }
}

#[derive(Component)]
pub struct FixedBackground;

// Système pour maintenir le background à la position de la caméra
pub fn update_fixed_background(
    camera_query: Query<&Transform, (With<Camera>, Without<FixedBackground>)>,
    mut background_query: Query<&mut Transform, (With<FixedBackground>, Without<Camera>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    for mut background_transform in background_query.iter_mut() {
        // Le background suit la caméra pour rester fixe à l'écran
        background_transform.translation.x = camera_transform.translation.x;
        background_transform.translation.y = camera_transform.translation.y;
        // Garde le Z fixe pour rester en arrière-plan
        background_transform.translation.z = -10.0;
    }
}

#[derive(Resource, Default)]
pub struct CameraDragState {
    pub is_dragging: bool,
}

pub fn handle_camera_inputs_system(
    mut camera_query: Query<(&mut Transform, &mut Projection), With<Camera>>,
    input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut drag_state: ResMut<CameraDragState>,
    mut input_mouse_wheel: EventReader<MouseWheel>,
    time: Res<Time>,
) {
    let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    // Gestion du clic souris pour le drag
    if mouse_button.just_pressed(MouseButton::Left) {
        drag_state.is_dragging = true;
    }
    if mouse_button.just_released(MouseButton::Left) {
        drag_state.is_dragging = false;
    }

    // Récupérer le niveau de zoom actuel
    let zoom_scale = if let Projection::Orthographic(projection2d) = &*projection {
        projection2d.scale
    } else {
        1.0 // Valeur par défaut si ce n'est pas une projection orthographique
    };

    // Déplacement avec la souris (drag)
    if drag_state.is_dragging {
        for event in mouse_motion.read() {
            // Inverser le delta pour un mouvement naturel
            // (bouger la souris vers la droite déplace la caméra vers la gauche)
            camera_transform.translation.x -= event.delta.x * zoom_scale;
            camera_transform.translation.y += event.delta.y * zoom_scale;
        }
    }

    // free Camera movement controls
    let mut direction = Vec3::ZERO;
    if input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    // normalizes to have constant diagonal speed
    if direction != Vec3::ZERO {
        direction = direction.normalize();
        let speed_in_pixels =
            CAMERA_SPEED * TILE_SIZE.x as f32 * zoom_scale.powf(0.7) * time.delta_secs();
        camera_transform.translation += direction * speed_in_pixels;
    }

    // zoom controls
    if let Projection::Orthographic(projection2d) = &mut *projection {
        for mouse_wheel_event in input_mouse_wheel.read() {
            use bevy::math::ops::powf;
            match mouse_wheel_event.unit {
                MouseScrollUnit::Line => {
                    if mouse_wheel_event.y > 0.0 {
                        projection2d.scale *= powf(ZOOM_IN_SPEED, time.delta_secs());
                    } else if mouse_wheel_event.y < 0.0 {
                        projection2d.scale *= powf(ZOOM_OUT_SPEED, time.delta_secs());
                    }
                }
                MouseScrollUnit::Pixel => {
                    if mouse_wheel_event.y > 0.0 {
                        projection2d.scale *= powf(ZOOM_IN_SPEED, time.delta_secs());
                    } else if mouse_wheel_event.y < 0.0 {
                        projection2d.scale *= powf(ZOOM_OUT_SPEED, time.delta_secs());
                    }
                }
            }
        }
    }
}
