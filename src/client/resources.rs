use std::{collections::HashMap, net::SocketAddr};

use bevy::{
    ecs::system::Resource,
    prelude::{Color, Entity, States},
};
use renet::ClientId;

#[derive(Resource)]
pub struct MyClientId(pub ClientId);

// #[derive(Resource)]
// pub struct PlayerEntities(pub HashMap<ClientId, Entity>);

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
    Menu,
    ConnectToServer,
    GameOver,
    Lose
}

impl Default for AppState {
    fn default() -> Self {
        AppState::ConnectToServer
    }
}

#[derive(Resource, Default)]
pub struct ButtonClicked(pub bool);

#[derive(Resource, Clone)]
pub struct ConnectionInfo {
    pub server_addr: SocketAddr,
    pub username: String,
}

#[derive(Resource)]
pub struct PlayerDeathEvent(pub bool);

impl ConnectionInfo {
    pub fn new(server_addr: SocketAddr, username: String) -> Self {
        Self {
            server_addr,
            username,
        }
    }
}

#[allow(dead_code)]
#[derive(Resource)]
pub struct PlayerEntities(pub HashMap<ClientId, Entity>);