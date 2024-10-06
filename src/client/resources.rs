use std::{collections::HashMap, net::SocketAddr};

use bevy::{
    ecs::{entity::Entity, system::Resource},
    prelude::{Color, States},
};
use renet::ClientId;

#[derive(Resource)]
pub struct MyClientId(pub ClientId);

#[derive(Resource)]
pub struct PlayerEntities(pub HashMap<ClientId, Entity>);

#[derive(Resource)]
// Définir BOARD comme une ressource

pub struct Board {
    pub data: Vec<Vec<char>>, // ou tout autre type
}

#[derive(Resource)]
pub struct PositionBalise {
    pub pos: (usize, usize), // ou tout autre type
}

#[derive(Resource, Debug)]
pub struct ColorOtherPlayer {
    pub color: Color, // ou tout autre type
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, States)]
pub enum AppState {
    WaitingForMap,
    Playing,
    Setup,
    Menu
}

impl Default for AppState {
    fn default() -> Self {
        AppState::WaitingForMap
    }
}

#[derive(Resource, Default)]
pub struct ButtonClicked(pub bool);

#[derive(Resource)]
pub struct ConnectionInfo {
    pub server_addr: SocketAddr,
    pub username: String,
}

impl ConnectionInfo {
    pub fn new(server_addr: SocketAddr, username: String) -> Self {
        Self {
            server_addr,
            username,
        }
    }
}