use std::collections::HashMap;

use bevy::ecs::{entity::Entity, system::Resource};
use renet::ClientId;

#[derive(Resource)]
pub struct MyClientId(pub ClientId);

#[derive(Resource)]
pub struct PlayerEntities(pub HashMap<ClientId, Entity>);

#[derive(Resource)]
// Définir BOARD comme une ressource
pub  struct Board {
    pub data: [[char; 15]; 15], // ou tout autre type
}