
use crate::{
    components::{MiniMap, MiniMapCell, MiniPlayer, MyPlayer, PlayerEntity},
    events::{LobbySyncEvent, PlayerDespawnEvent, PlayerSpawnEvent},
    resources::Board,
    MyClientId,
};
use bevy::{
    asset::Assets,
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        event::{EventReader, EventWriter},
        system::{Commands, Query, Res, ResMut},
    },
    input::{
        keyboard::{KeyCode, KeyboardInput},
        mouse::MouseMotion,
    },
    log::info,
    math::{
        primitives::{Cuboid, Plane3d, Sphere},
        Quat, Vec3,
    },
    pbr::{MaterialMeshBundle, StandardMaterial},
    prelude::{default, BuildChildren, DetectChanges, NodeBundle, With},
    reflect::Reflect,
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
    ui::{BackgroundColor, Style, Val},
};
use multiplayer_demo::PlayerAttributes;
use rand::Rng;
use renet::{DefaultChannel, RenetClient};

// pub fn send_message_system(mut client: ResMut<RenetClient>, query: Query<(&MyPlayer, &Transform)>) {
//     if client.is_disconnected() {
//         panic!("<++++++++++++++++++++++++++++Client is disconnected to the server++++++++++++++++++++++++++++++++++++++++++>");
//     }
//     let (_, transform) = query.single();
//     let player_sync = PlayerAttributes {
//         position: transform.translation.into(),
//     };
//     let message = bincode::serialize(&player_sync).unwrap();
//     client.send_message(DefaultChannel::Unreliable, message);
// }

pub fn send_message_system(
    mut client: ResMut<RenetClient>,
    query: Query<(&MyPlayer, &Transform)>,
) {
    if client.is_disconnected() {
        panic!("Client is disconnected from the server");
    }
    let (_, transform) = query.single();
    
    // Envoie uniquement la position du joueur local au serveur
    let player_sync = PlayerAttributes {
        position: transform.translation.into(),
    };
    let message = bincode::serialize(&player_sync).unwrap();
    client.send_message(DefaultChannel::Unreliable, message);
}


pub fn receive_message_system(
    mut client: ResMut<RenetClient>,
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
    mut despawn_events: EventWriter<PlayerDespawnEvent>,
    mut lobby_sync_events: EventWriter<LobbySyncEvent>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        let server_message = bincode::deserialize(&message).unwrap();

        match server_message {
            multiplayer_demo::ServerMessage::PlayerJoin(client_id) => {
                info!("Client connected: {}", client_id);
                spawn_events.send(PlayerSpawnEvent(client_id));
            }
            multiplayer_demo::ServerMessage::PlayerLeave(client_id) => {
                info!("Client disconnected: {}", client_id);
                despawn_events.send(PlayerDespawnEvent(client_id));
                // client a supprimer ici cote frontend
            }
            _ => {
                info!("Unhandled message: {:?}", server_message);
            }
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let message = bincode::deserialize(&message).unwrap();

        match message {
            multiplayer_demo::ServerMessage::LobbySync(map) => {
                lobby_sync_events.send(LobbySyncEvent(map));
            }
            _ => {
                info!("Unhandled message: {:?}", message);
            }
        }
    }
}

pub fn update_player_movement_system(
    mut map: Res<Board>,
    mut keyboard_events: EventReader<KeyboardInput>,
    mut query: Query<(&mut Transform, &MyPlayer)>,
) {
    let (mut transform, _) = query.single_mut();

    for event in keyboard_events.read() {
        let mut delta_position = Vec3::new(0.0, 0.0, 0.0);

        match event.key_code {
            KeyCode::KeyW => delta_position.z += 0.1,
            KeyCode::KeyS => delta_position.z -= 0.1,
            KeyCode::KeyA => delta_position.x -= 0.1,
            KeyCode::KeyD => delta_position.x += 0.1,
            _ => {}
        }
        let new_position = transform.translation + delta_position;
        if !check_collision(map.data, new_position) {
            transform.translation = new_position;
        }
    }
}

// La fonction setup_system est responsable de la configuration initiale de la scène dans Bevy. Elle crée :

// Une caméra 3D : Placée à une certaine position pour voir la scène.
// Un sol : Un grand plan 3D servant de sol dans la scène.
// Un objet joueur : Une sphère verte placée légèrement au-dessus du sol et marquée comme étant un joueur.

pub fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    board: Res<Board>,
    client: ResMut<RenetClient>,
) {
    if client.is_disconnected() {
        panic!("disconnected : Client is not connected to the server");
    }
    // let position=rand_position_player(&board);
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(3.0, 1.0, 0.0).looking_at(Vec3::NEG_Z, Vec3::Y),
                ..default()
            },
            MyPlayer,
        ))
        .with_children(|command| {
            command.spawn(MaterialMeshBundle {
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.0, 1.0, 0.0),
                    ..default()
                }),
                mesh: meshes.add(Sphere::new(0.2)),
                transform: Transform::from_xyz(3.0, 1.0, 0.0),
                ..default()
            });
        });
    // world start
    commands.spawn(MaterialMeshBundle {
        material: materials.add(StandardMaterial::default()),
        mesh: meshes.add(Plane3d::default()),
        transform: Transform::from_scale(Vec3::splat(7.0)),
        ..default()
    });
    for (z, line) in board.data.iter().enumerate() {
        for (x, c) in line.iter().enumerate() {
            if *c == '0' {
                continue;
            } else {
                commands.spawn(MaterialMeshBundle {
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.0, 0.0, 1.0),
                        ..default()
                    }),
                    // mesh: meshes.add(Plane3d::default()),
                    mesh: meshes.add(Cuboid::new(1.0, 3.0, 1.0)),
                    transform: Transform::from_xyz(-(x as f32) + 7.0, 0.0, z as f32 - 7.0),
                    ..default()
                });
            }
        }
    }
    // world end
}

pub fn spawn_map_2d(
    mut commands: Commands,
    board: Res<Board>,
    query: Query<&Transform, With<MyPlayer>>,
) {
    let size_map = 100.0; // size_map d'une cellule dans l'affichage
    let cell = size_map / board.data[0].len() as f32;
    let transform = query.single().translation;
    // j'ai un probleme avec les coordonnes
    // elle ne reflete pas la position reel du joueur
    let xp = (-transform[0].round() + 7.0) * cell;
    let zp = (transform[2].round() + 7.0) * cell;
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(size_map),
                    height: Val::Px(size_map),
                    position_type: bevy::ui::PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    ..Default::default()
                },
                // transform: Transform {
                //     rotation: Quat::from_rotation_z(std::f32::consts::PI), // Rotation de 180 degrés
                //     ..Default::default()
                // },
                background_color: BackgroundColor::DEFAULT,
                ..Default::default()
            },
            MiniMap,
        ))
        .with_children(|parent| {
            for row in 0..board.data.len() {
                for col in 0..board.data[0].len() {
                    if board.data[row][col] == '0' {
                        continue;
                    }
                    parent.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Px(cell),
                                height: Val::Px(cell),
                                position_type: bevy::ui::PositionType::Absolute,
                                top: Val::Px(row as f32 * cell),
                                left: Val::Px(col as f32 * cell),
                                ..Default::default()
                            },
                            background_color: BackgroundColor(Color::GRAY),
                            ..Default::default()
                        },
                        MiniMapCell,
                    ));
                }
            }
            parent.spawn((
                NodeBundle {
                    style: Style {
                        position_type: bevy::ui::PositionType::Absolute,
                        width: Val::Px(cell),
                        height: Val::Px(cell),
                        top: Val::Px(zp),
                        left: Val::Px(xp),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::RED),
                    ..default()
                },
                MiniPlayer,
            ));
        });
}

pub fn handle_player_spawn_event_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_events: EventReader<PlayerSpawnEvent>,
) {
    for event in spawn_events.read() {
        info!("Handling player spawn event: {:?}", event.0);
        let client_id = event.0;

        commands.spawn((
            MaterialMeshBundle {
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(1.0, 0.0, 0.0),
                    ..default()
                }),
                mesh: meshes.add(Cuboid::new(0.2, 0.2, 0.2)),
                ..default()
            },
            PlayerEntity(client_id),
        ));
    }
}

// pub fn handle_lobby_sync_event_system(
//     mut spawn_events: EventWriter<PlayerSpawnEvent>,
//     mut sync_events: EventReader<LobbySyncEvent>,
//     mut query: Query<(&PlayerEntity, &mut Transform)>,
//     my_clinet_id: Res<MyClientId>,
// ) {
//     let event_option = sync_events.read().last();
//     if event_option.is_none() {
//         return;
//     }
//     let event = event_option.unwrap();

//     for (client_id, player_sync) in event.0.iter() {
//         if *client_id == my_clinet_id.0 {
//             continue;
//         }

//         let mut found = false;
//         for (player_entity, mut transform) in query.iter_mut() {
//             if *client_id == player_entity.0 {
//                 let new_position = player_sync.position;
//                 transform.translation = new_position.into();
//                 found = true;
//             }
//         }

//         if !found {
//             info!("Spawning player {}: {:?}", client_id, player_sync.position);
//             spawn_events.send(PlayerSpawnEvent(*client_id));
//         }
//     }
// }

pub fn handle_lobby_sync_event_system(
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
    mut sync_events: EventReader<LobbySyncEvent>,
    mut query: Query<(&PlayerEntity, &mut Transform)>,
    my_client_id: Res<MyClientId>, // ID du joueur local
) {
    let event_option = sync_events.read().last();
    if event_option.is_none() {
        return;
    }
    let event = event_option.unwrap();

    for (client_id, player_sync) in event.0.iter() {
        // Ne pas mettre à jour la position du joueur local
        if *client_id == my_client_id.0 {
            continue;
        }

        // Mettre à jour la position des autres joueurs
        let mut found = false;
        for (player_entity, mut transform) in query.iter_mut() {
            if *client_id == player_entity.0 {
                // Met à jour uniquement la position du joueur distant
                let new_position = player_sync.position;
                transform.translation = new_position.into();
                found = true;
            }
        }

        // Si le joueur n'existe pas encore, le spawn
        if !found {
            info!("Spawning player {}: {:?}", client_id, player_sync.position);
            spawn_events.send(PlayerSpawnEvent(*client_id));
        }
    }
}


pub fn mini_map_sync_event_system(
    player_3_d: Query<&mut Transform, With<MyPlayer>>,
    board: Res<Board>,
    mut mini_player: Query<&mut Style, With<MiniPlayer>>,
) {
    let cell = 100.0 / board.data[0].len() as f32;
    let transform = player_3_d.single();
    let xp = (-transform.translation[0].round() + 7.0) * cell;
    let zp = (transform.translation[2].round() + 7.0) * cell;
    let mut style = mini_player.single_mut();

    style.left = Val::Px(xp);
    style.top = Val::Px(zp);
}

pub fn rotation_player(
    mut query: Query<&mut Transform, With<MyPlayer>>,
    mut mouse_event: EventReader<MouseMotion>,
) {
    let mut transform = query.single_mut();
    for event in mouse_event.read() {
        let x = event.delta.x;
        let mut y = event.delta.y;
        // la supperpositin de rotion_y et de rotate_x est tres important
        //  le changer change aussi le comportemnet de la rotion du camera
        // du joueur
        transform.rotate_y(-x * 0.003);
        let t = transform.forward().y;
        if t < (-0.7) {
            y = -0.7;
        } else if t > 0.7 {
            y = 0.7
        }
        transform.rotate_local_x(-y * 0.002);
    }
}

fn rand_position_player(board: &Res<Board>) -> Vec3 {
    let mut rng = rand::thread_rng();
    loop {
        let z = rng.gen_range(0..board.data.len());
        let x = rng.gen_range(0..board.data[0].len());
        if board.data[z][x] == '0' {
            return Vec3::new(x as f32 - 7.0, 1.0, z as f32 - 7.0);
        }
    }
}

fn check_collision(map: [[char; 15]; 15], position: Vec3) -> bool {
    let x = (-position.x + 7.0).round() as usize;
    let z = (position.z + 7.0).round() as usize;
    
    if x >= 15 || z >= 15 {
        return true; // Out of bounds, consider it a collision
    }
    
    //println!("Checking collision at x = {}, z = {}", x, z);
    map[z][x] == '1'
}

// fn check_collision(map: [[char; 15]; 15], position: Vec3) -> bool {
//     let (x, z) = (
//         (-position.x + 7.1).powi(2).sqrt() as usize,
//         (position.z + 7.1).powi(2).sqrt() as usize,
//     );
//     println!("new_x ==> {} and new_z ==> {} and bool ==>", x, z);
//     map[x][z] == '1'
// }

// pub fn check_connection(client: Res<RenetClient>) {
//     if client.is_connected() {
//         println!("Client is connected to the server");
//     } else {
//         println!(" checkconnection : Client is not connected to the server");
//     }
// }
