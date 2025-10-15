use bevy::prelude::*;

const NODE_RADIUS: f32 = 10.0;
const NODE_SELECTED_COLOR: Color = Color::srgb(1.0, 1.0, 0.0); // YELLOW
const LINK_COLOR: Color = Color::srgba(0.5, 0.5, 0.8, 0.5);
const LINK_SELECTED_COLOR: Color = Color::srgb(1.0, 1.0, 0.0); // YELLOW

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<GalaxyGraph>()
            .add_systems(PostStartup, setup)
            .add_systems(
                Update,
                (
                    draw_links_system,
                    node_interaction_system,
                    update_visuals_system,
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
}

/// Composant marqueur pour le nœud actuellement sélectionné.
#[derive(Component)]
struct SelectedNode;

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
    // Création des nœuds (étoiles)
    let node_positions = vec![
        (Vec2::new(-200.0, 150.0), "Sol", Color::srgb(1.0, 0.27, 0.0)), // ORANGE_RED
        (Vec2::new(50.0, 250.0), "Sirius", Color::srgb(0.0, 1.0, 1.0)), // CYAN
        (
            Vec2::new(300.0, 100.0),
            "Alpha Centauri",
            Color::srgb(1.0, 1.0, 1.0),
        ), // WHITE
        (
            Vec2::new(100.0, -200.0),
            "Vega",
            Color::srgb(0.1, 0.1, 0.44),
        ), // MIDNIGHT_BLUE
        (
            Vec2::new(-250.0, -150.0),
            "Proxima",
            Color::srgb(1.0, 0.0, 0.0),
        ), // RED
    ];

    let mut node_entities = Vec::new();

    for (pos, name, color) in node_positions {
        let entity = commands
            .spawn((
                Mesh2d(meshes.add(Circle::new(NODE_RADIUS))),
                MeshMaterial2d(materials.add(ColorMaterial::from(color))),
                Transform::from_translation(pos.extend(0.0)),
                StarNode {
                    name: name.to_string(),
                    base_color: color,
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

            // Change la couleur si un des nœuds connectés est sélectionné
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

fn node_interaction_system(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    node_query: Query<(Entity, &GlobalTransform, &StarNode)>,
    selected_query: Query<Entity, With<SelectedNode>>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.single() else {
            return;
        };
        let Ok((camera, camera_transform)) = camera_query.single() else {
            return;
        };

        // Convertit la position du curseur en coordonnées du monde
        if let Some(world_pos) = window
            .cursor_position()
            .and_then(|cursor| Some(camera.viewport_to_world(camera_transform, cursor)))
            .map(|ray| ray.unwrap().origin.truncate())
        {
            // D'abord, on désélectionne le nœud précédent
            for entity in selected_query.iter() {
                commands.entity(entity).remove::<SelectedNode>();
            }

            // Trouve le nœud le plus proche du clic, s'il est assez proche
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

            // Si un nœud a été cliqué, on le marque comme sélectionné
            if let Some(entity) = clicked_entity {
                if let Some(star_node) = clicked_star_node {
                    println!("Node clicked!: {:?}", star_node.name);
                    commands.entity(entity).insert(SelectedNode);
                }
            }
        }
    }
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
