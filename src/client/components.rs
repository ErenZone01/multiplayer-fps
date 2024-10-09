use bevy::{ecs::component::Component, math::Vec3, prelude::Entity};
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

// Définition de la composante `Beacon`
#[derive(Component)]
pub struct Beacon;

// Tags pour identifier les éléments spécifiques
#[derive(Component)]
pub struct ButtonTag;

#[derive(Component)]
pub struct InitialImageTag;

#[derive(Component)]
pub struct TextTag; // Tag pour le texte

// #[derive(Component)]
// pub struct FpsDisplay;

// Composant pour identifier le texte des FPS
#[derive(Component)]
pub struct FpsText;



#[derive(Component)]
pub struct Projectile {
    pub direction: Vec3,
    pub shooter: Entity, // Ajoutez cet identifiant de joueur
}