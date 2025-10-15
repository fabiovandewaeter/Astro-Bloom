use bevy::prelude::*;

// marque une planète
#[derive(Component)]
pub struct Planet {
    pub name: String,
}

// le bâtiment
#[derive(Component)]
pub struct Building {
    pub kind: BuildingType,
}

// simple enum
#[derive(Clone, Copy)]
pub enum BuildingType {
    Mine,
    Farm,
    Habitat,
}
