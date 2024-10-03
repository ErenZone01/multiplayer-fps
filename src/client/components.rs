use bevy::ecs::component::Component;
use renet::ClientId;

#[derive(Component)]
pub struct PlayerEntity(pub ClientId);

#[derive(Component)]
pub struct MyPlayer;

#[derive(Component)]
pub struct MiniMap;

#[derive(Component)]
pub struct MiniMapCell;

#[derive(Component)]
pub struct MiniPlayer;