use std::collections::HashMap;

use bevy::ecs::system::Resource;
use multiplayer_demo::PlayerAttributes;
use renet::ClientId;

#[derive(Resource, Clone)]
pub struct PlayerLobby(pub HashMap<ClientId, PlayerAttributes>);

#[derive(Resource)]
pub  struct  IsTakingMap{ pub map : Option<usize>}

#[derive(Resource)]
pub  struct  IsDeathOnce{ pub death : bool}
