use bevy::{input::mouse::MouseMotion, prelude::*};

const NODE_RADIUS: f32 = 10.0;
const NODE_SELECTED_COLOR: Color = Color::srgb(1.0, 1.0, 0.0); // YELLOW
const LINK_COLOR: Color = Color::srgba(0.5, 0.5, 0.8, 0.5);
const LINK_SELECTED_COLOR: Color = Color::srgb(1.0, 1.0, 0.0); // YELLOW

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<GalaxyGraph>()
            .init_resource::<MouseDragState>()
            .add_systems(PostStartup, setup)
            .add_systems(
                Update,
                (
                    draw_links_system,
                    node_interaction_system,
                    update_visuals_system,
                    close_menu_system,
                ),
            );
    }
}

// --- COMPOSANTS ET RESSOURCES ---

/// Composant pour marquer une entité comme un nœud d'étoile.
#[derive(Component, Clone)]
struct StarNode {
    name: String,
    base_color: Color,
    // Ajoute des infos supplémentaires
    star_type: String,
    distance: f32, // en années-lumière
    population: u64,
}

/// Composant marqueur pour le nœud actuellement sélectionné.
#[derive(Component)]
struct SelectedNode;

/// Composant marqueur pour le menu UI
#[derive(Component)]
struct NodeInfoMenu;

/// Ressource pour stocker les liens du graphe.
/// Un lien est une paire d'entités (Entity) qui ont un composant StarNode.
#[derive(Resource, Default)]
struct GalaxyGraph {
    links: Vec<(Entity, Entity)>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut galaxy_graph: ResMut<GalaxyGraph>,
) {
    // Création des nœuds (étoiles) avec plus d'infos
    let node_data = vec![
        (
            Vec2::new(-200.0, 150.0),
            "Sol",
            Color::srgb(1.0, 0.27, 0.0),
            "Naine jaune",
            0.0,
            8_000_000_000,
        ),
        (
            Vec2::new(50.0, 250.0),
            "Sirius",
            Color::srgb(0.0, 1.0, 1.0),
            "Étoile double",
            8.6,
            3_500_000,
        ),
        (
            Vec2::new(300.0, 100.0),
            "Alpha Centauri",
            Color::srgb(1.0, 1.0, 1.0),
            "Système triple",
            4.37,
            12_000_000,
        ),
        (
            Vec2::new(100.0, -200.0),
            "Vega",
            Color::srgb(0.1, 0.1, 0.44),
            "Naine bleue",
            25.0,
            900_000,
        ),
        (
            Vec2::new(-250.0, -150.0),
            "Proxima",
            Color::srgb(1.0, 0.0, 0.0),
            "Naine rouge",
            4.24,
            250_000,
        ),
    ];

    let mut node_entities = Vec::new();

    for (pos, name, color, star_type, distance, population) in node_data {
        let entity = commands
            .spawn((
                Mesh2d(meshes.add(Circle::new(NODE_RADIUS))),
                MeshMaterial2d(materials.add(ColorMaterial::from(color))),
                Transform::from_translation(pos.extend(0.0)),
                StarNode {
                    name: name.to_string(),
                    base_color: color,
                    star_type: star_type.to_string(),
                    distance,
                    population,
                },
            ))
            .id();
        node_entities.push(entity);
    }

    // Création des liens entre les nœuds
    galaxy_graph
        .links
        .push((node_entities[0], node_entities[1]));
    galaxy_graph
        .links
        .push((node_entities[1], node_entities[2]));
    galaxy_graph
        .links
        .push((node_entities[2], node_entities[3]));
    galaxy_graph
        .links
        .push((node_entities[3], node_entities[4]));
    galaxy_graph
        .links
        .push((node_entities[4], node_entities[0]));
    galaxy_graph
        .links
        .push((node_entities[0], node_entities[2]));
}

/// Système pour dessiner les liens en utilisant les Gizmos.
fn draw_links_system(
    mut gizmos: Gizmos,
    galaxy_graph: Res<GalaxyGraph>,
    transforms: Query<&Transform>,
    selected_query: Query<Entity, With<SelectedNode>>,
) {
    let selected_nodes: Vec<Entity> = selected_query.iter().collect();

    for &(start_entity, end_entity) in &galaxy_graph.links {
        if let (Ok(start_transform), Ok(end_transform)) =
            (transforms.get(start_entity), transforms.get(end_entity))
        {
            let start_pos = start_transform.translation.truncate();
            let end_pos = end_transform.translation.truncate();

            let is_selected =
                selected_nodes.contains(&start_entity) || selected_nodes.contains(&end_entity);
            let color = if is_selected {
                LINK_SELECTED_COLOR
            } else {
                LINK_COLOR
            };

            gizmos.line_2d(start_pos, end_pos, color);
        }
    }
}

#[derive(Resource, Default)]
struct MouseDragState {
    is_dragging: bool,
    start_pos: Option<Vec2>,
}

fn node_interaction_system(
    mut commands: Commands,
    mut mouse_drag: ResMut<MouseDragState>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    node_query: Query<(Entity, &GlobalTransform, &StarNode)>,
    selected_query: Query<Entity, With<SelectedNode>>,
    menu_query: Query<Entity, With<NodeInfoMenu>>,
) {
    // Détecte le début du clic gauche
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.single() else {
            return;
        };
        mouse_drag.start_pos = window.cursor_position();
        mouse_drag.is_dragging = false;
    }

    // Si la souris bouge pendant le clic, on considère que c'est un drag
    if mouse_buttons.pressed(MouseButton::Left) {
        for ev in mouse_motion_events.read() {
            if ev.delta.length() > 0.2 {
                mouse_drag.is_dragging = true;
                break;
            }
        }
    }

    // Quand le clic gauche est relâché
    if mouse_buttons.just_released(MouseButton::Left) {
        let Ok(window) = windows.single() else {
            return;
        };
        let Ok((camera, camera_transform)) = camera_query.single() else {
            return;
        };

        if let Some(world_pos) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
            .map(|ray| ray.origin.truncate())
        {
            // Vérifie si un nœud a été cliqué
            let mut clicked_entity = None;
            let mut clicked_star_node = None;
            for (node_entity, node_transform, star_node) in node_query.iter() {
                let distance = world_pos.distance(node_transform.translation().truncate());
                if distance < NODE_RADIUS {
                    clicked_entity = Some(node_entity);
                    clicked_star_node = Some(star_node);
                    break;
                }
            }

            // Si c’est un clic sur un nœud → sélectionne-le
            if let (Some(entity), Some(star_node)) = (clicked_entity, clicked_star_node) {
                for entity_selected in selected_query.iter() {
                    commands.entity(entity_selected).remove::<SelectedNode>();
                }
                for entity_menu in menu_query.iter() {
                    commands.entity(entity_menu).despawn_related::<Children>();
                }

                commands.entity(entity).insert(SelectedNode);
                spawn_info_menu(&mut commands, star_node);
            }
            // Si c’est un clic "vide" (pas de nœud) ET pas un drag → désélectionne
            else if !mouse_drag.is_dragging {
                for entity_selected in selected_query.iter() {
                    commands.entity(entity_selected).remove::<SelectedNode>();
                }
                for entity_menu in menu_query.iter() {
                    commands.entity(entity_menu).despawn_related::<Children>();
                }
            }
        }

        // Reset de l'état de drag
        mouse_drag.is_dragging = false;
        mouse_drag.start_pos = None;
    }
}

fn spawn_info_menu(commands: &mut Commands, star_node: &StarNode) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
            BorderColor::all(Color::srgb(0.8, 0.8, 0.2)),
            BorderRadius::all(Val::Px(8.0)),
            NodeInfoMenu,
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new(format!("★ {}", star_node.name)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.3)),
            ));

            // Type d'étoile
            parent.spawn((
                Text::new(format!("Type: {}", star_node.star_type)),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // Distance
            if star_node.distance > 0.0 {
                parent.spawn((
                    Text::new(format!("Distance: {:.2} al", star_node.distance)),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            }

            // Population
            parent.spawn((
                Text::new(format!(
                    "Population: {} hab.",
                    format_number(star_node.population)
                )),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // Bouton de fermeture
            parent.spawn((
                Text::new("\n[Clic droit pour fermer]"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.7, 0.8)),
            ));
        });
}

fn close_menu_system(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    menu_query: Query<Entity, With<NodeInfoMenu>>,
    selected_query: Query<Entity, With<SelectedNode>>,
) {
    if mouse_buttons.just_pressed(MouseButton::Right) {
        // Ferme le menu
        for entity in menu_query.iter() {
            commands.entity(entity).despawn_related::<Children>();
            commands.entity(entity).despawn();
        }

        // Désélectionne le nœud
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<SelectedNode>();
        }
    }
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(' ');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Met à jour la couleur des nœuds en fonction de leur état (sélectionné ou non).
fn update_visuals_system(
    query: Query<(
        &StarNode,
        &MeshMaterial2d<ColorMaterial>,
        Option<&SelectedNode>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (node, material_handle, selected) in query.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            if selected.is_some() {
                material.color = NODE_SELECTED_COLOR;
            } else {
                material.color = node.base_color;
            }
        }
    }
}
