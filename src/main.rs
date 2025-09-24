use astro_bloom::{
    UPS_TARGET,
    camera::{
        FixedBackground, UpsCounter, display_fps_ups_system, handle_camera_inputs_system,
        update_fixed_background,
    },
};
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Overlord".to_string(),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(Time::<Fixed>::from_hz(UPS_TARGET))
        .insert_resource(TimeState::default())
        .insert_resource(UpsCounter {
            ticks: 0,
            last_second: 0.0,
            ups: 0,
        })
        .add_systems(Startup, (setup_system,).chain())
        .add_systems(
            Update,
            (
                handle_camera_inputs_system,
                update_fixed_background,
                display_fps_ups_system,
                control_time_system,
            ),
        )
        .add_systems(FixedUpdate, (update_logic_system,))
        .run();
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut orthographic_projection = OrthographicProjection::default_2d();
    orthographic_projection.scale *= 0.8;
    let projection = Projection::Orthographic(orthographic_projection);
    commands.spawn((Camera2d, Camera { ..default() }, projection));

    // Background sprite qui sera repositionné
    commands.spawn((
        Sprite {
            image: asset_server.load("background.png"),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
        FixedBackground,
    ));
}

pub fn update_logic_system(mut counter: ResMut<UpsCounter>) {
    counter.ticks += 1;
}

#[derive(Resource, Default)]
struct TimeState {
    is_paused: bool,
}

fn control_time_system(
    mut fixed_time: ResMut<Time<Fixed>>,
    input: Res<ButtonInput<KeyCode>>,
    mut time_state: ResMut<TimeState>,
) {
    // P pour Pause, pour alterner entre l'état de pause
    if input.just_pressed(KeyCode::Space) {
        if time_state.is_paused {
            println!("Temps de la simulation repris.");
            fixed_time.set_timestep_hz(UPS_TARGET);
            time_state.is_paused = false;
        } else {
            println!("Temps de la simulation mis en pause.");
            fixed_time.set_timestep_hz(0.0);
            time_state.is_paused = true;
        }
    }

    // Si le jeu est en pause, on ne gère pas les autres commandes de vitesse
    if time_state.is_paused {
        return;
    }

    // Accélérer (x2)
    if input.just_pressed(KeyCode::KeyY) {
        let current_hz = fixed_time.timestep().as_secs_f64().recip();
        let new_hz = current_hz * 2.0;
        println!("Temps de la simulation accéléré à {} Hz.", new_hz);
        fixed_time.set_timestep_hz(new_hz);
    }

    // Ralentir (/2)
    if input.just_pressed(KeyCode::KeyU) {
        let current_hz = fixed_time.timestep().as_secs_f64().recip();
        let new_hz = current_hz / 2.0;
        println!("Temps de la simulation ralenti à {} Hz.", new_hz);
        fixed_time.set_timestep_hz(new_hz);
    }

    // Normal (retour à la vitesse initiale)
    if input.just_pressed(KeyCode::KeyI) {
        println!("Temps de la simulation réinitialisé à {} Hz.", UPS_TARGET);
        fixed_time.set_timestep_hz(UPS_TARGET);
    }
}
