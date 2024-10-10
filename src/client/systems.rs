use std::{
    io::{self, Write},
    net::UdpSocket,
    time::SystemTime,
};

use crate::{
    components::{
        Beacon, ButtonTag, FpsText, InitialImageTag, MiniMap, MiniMapCell, MiniPlayer, MyPlayer,
        PlayerEntity, Projectile, TextTag,
    },
    events::{LobbySyncEvent, PlayerSpawnEvent},
    resources::{
        AppState, Board, ButtonClicked, ColorOtherPlayer, ConnectionInfo, IsWin, PlayerDeathEvent,
        PlayerEntities, PositionBalise,
    },
    MyClientId,
};
use bevy::{
    asset::{AssetServer, Assets},
    core_pipeline::core_3d::Camera3dBundle,
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::{
        event::{EventReader, EventWriter},
        system::{Commands, Query, Res, ResMut},
    },
    input::{
        keyboard::{KeyCode, KeyboardInput},
        mouse::MouseMotion,
    },
    log::{error, info},
    math::{
        primitives::{Cuboid, Plane3d, Sphere},
        Vec3,
    },
    pbr::{MaterialMeshBundle, StandardMaterial},
    prelude::{
        default, BuildChildren, ButtonBundle, Camera, Camera2dBundle, Changed, Entity, ImageBundle,
        NextState, NodeBundle, ParamSet, TextBundle, With,
    },
    render::{
        color::{self, Color},
        mesh::Mesh,
    },
    text::{Text, TextStyle},
    time::Time,
    transform::components::Transform,
    ui::{
        AlignItems, BackgroundColor, Interaction, JustifyContent, PositionType, Style, UiImage,
        UiRect, Val,
    },
};
use bevy::{input::ButtonState, prelude::DespawnRecursiveExt};
use multiplayer_demo::{send_board, PlayerAttributes, ServerMessage};
use renet::{
    transport::{ClientAuthentication, NetcodeClientTransport},
    ClientId, ConnectionConfig, DefaultChannel, RenetClient,
};

pub fn send_message_system(mut client: ResMut<RenetClient>, query: Query<(&MyPlayer, &Transform)>) {
    if client.is_disconnected() {
        panic!("Client is disconnected from the server");
    }
    let (_, transform) = query.single();

    // Envoie uniquement la position du joueur local au serveur
    let player_sync = PlayerAttributes {
        position: transform.translation.into(),
        color: color::Color::YELLOW,
    };
    let message = bincode::serialize(&player_sync).unwrap();
    client.send_message(DefaultChannel::Unreliable, message);
}

pub fn receive_message_system(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
    mut lobby_sync_events: EventWriter<LobbySyncEvent>,
    mut death_event: ResMut<PlayerDeathEvent>,
    my_client_id: Res<MyClientId>,
    mut player_entities: ResMut<PlayerEntities>,
    mut next_state: ResMut<NextState<AppState>>,
    mut win: ResMut<IsWin>,
) {
    // Boucle pour traiter les messages fiables
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        match bincode::deserialize::<multiplayer_demo::ServerMessage>(&message) {
            Ok(server_message) => match server_message {
                multiplayer_demo::ServerMessage::PlayerJoin((client_id, colors)) => {
                    info!("Client connected: {}", client_id);
                    spawn_events.send(PlayerSpawnEvent(client_id));
                    commands.insert_resource(ColorOtherPlayer { color: colors });
                }
                multiplayer_demo::ServerMessage::PlayerLeave(client_id) => {
                    info!("leave : Client disconnected: {}", client_id);
                }
                multiplayer_demo::ServerMessage::Map(map_id) => {
                    // Logique pour charger la carte
                    info!("Received map_id: {}", map_id);

                    let board_data = match map_id {
                        0 => send_board(),
                        1 => send_board(),
                        2 => send_board(),
                        _ => send_board(), // Gestion par défaut si map_id non reconnu
                    };

                    // Insérer la map dans les ressources
                    commands.insert_resource(Board { data: board_data });
                    //commands.insert_resource(ColorOtherPlayer{color : color_player});
                    // Passer à l'état Playing une fois que la map est reçue
                    next_state.set(AppState::Menu);
                    info!("Map data has been inserted and state set to Playing.");
                }
                multiplayer_demo::ServerMessage::PosBalise(pos_balise) => {
                    // Logique pour recevoir la position de la balise
                    info!("Received balise position: {:?}", pos_balise);

                    // Insérer la position de la balise dans les ressources
                    commands.insert_resource(PositionBalise { pos: pos_balise });
                }
                multiplayer_demo::ServerMessage::GameOver(_) => {
                    next_state.set(AppState::GameOver);
                }
                multiplayer_demo::ServerMessage::PlayerDeath(client_id) => {
                    if client_id == my_client_id.0 {
                        info!("c'est moi qui suis mort");
                        death_event.0 = true;
                    } else {
                        info!("j'ai recu la mort du joueur : {:?} ", client_id);
                        despawn_player(&mut commands, &mut player_entities, client_id, &mut win);
                    }
                }
                _ => {
                    // Pour les messages non gérés
                    info!("Unhandled message: {:?}", server_message);
                }
            },
            Err(e) => {
                // Gestion de l'erreur de désérialisation
                error!("Failed to deserialize reliable message: {:?}", e);
            }
        }
    }

    // Boucle pour traiter les messages non fiables
    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        match bincode::deserialize::<multiplayer_demo::ServerMessage>(&message) {
            Ok(server_message) => match server_message {
                multiplayer_demo::ServerMessage::LobbySync(map) => {
                    lobby_sync_events.send(LobbySyncEvent(map));
                }
                _ => {
                    info!("Unhandled unreliable message: {:?}", server_message);
                }
            },
            Err(e) => {
                // Gestion de l'erreur de désérialisation pour les messages non fiables
                error!("Failed to deserialize unreliable message: {:?}", e);
            }
        }
    }
}

fn despawn_player(
    commands: &mut Commands,
    player_entities: &mut ResMut<PlayerEntities>,
    client_id: ClientId,
    win: &mut IsWin,
) {
    if player_entities.0.contains_key(&client_id) {
        for (key, value) in player_entities.0.clone() {
            if key == client_id {
                info!("j'ai supprimer le client_id");
                commands.entity(value).despawn_recursive();
                win.0 = true;
            }
        }
    }
}

pub fn update_player_movement_system(
    map: Res<Board>,
    mut keyboard_events: EventReader<KeyboardInput>,
    mut query: ParamSet<(
        Query<(&mut Transform, &MyPlayer)>, // Paramètre 1 : Joueur
        Query<&Transform, With<Camera>>,    // Paramètre 2 : Caméra
    )>,
) {
    // Taille du plateau
    let board_width = map.data[0].len() as f32;
    let board_height = map.data.len() as f32;

    // Récupération de la rotation de la caméra
    let binding = query.p1();
    let camera_transform = binding.single();
    let forward = camera_transform.forward(); // Direction avant
    let right = camera_transform.right(); // Direction droite

    // Récupération des informations du joueur
    let mut binding = query.p0();
    let (mut transform, _) = binding.single_mut();

    // Fixer la hauteur à une constante pour rester au sol
    let ground_level_y = 1.0;

    for event in keyboard_events.read() {
        let mut delta_position = Vec3::ZERO;

        // Ajuster les mouvements en fonction des touches pressées
        match event.key_code {
            KeyCode::KeyW => delta_position += forward * 0.1, // Avancer
            KeyCode::KeyS => delta_position -= forward * 0.1, // Reculer
            KeyCode::KeyA => delta_position -= right * 0.1,   // Aller à gauche
            KeyCode::KeyD => delta_position += right * 0.1,   // Aller à droite
            _ => {}
        }

        // Calculer la nouvelle position
        let mut new_position = transform.translation + delta_position;

        // S'assurer que le joueur reste à la hauteur du sol
        new_position.y = ground_level_y;

        // Limiter les mouvements du joueur aux dimensions du plateau
        new_position.x = new_position.x.clamp(-board_width / 2.0, board_width / 2.0);
        new_position.z = new_position
            .z
            .clamp(-board_height / 2.0, board_height / 2.0);

        // Vérifier les collisions avant de mettre à jour la position
        if !check_collision(map.data.clone(), new_position) {
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
    position_balise: Res<PositionBalise>,
    client: ResMut<RenetClient>,
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
) {
    if client.is_disconnected() {
        panic!("disconnected : Client is not connected to the server");
    }

    // Création de la caméra et du joueur
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(3.0, 1.0, 0.0).looking_at(Vec3::NEG_Z, Vec3::Y),
                ..default()
            },
            MyPlayer,
        ))
        .with_children(|command| {
            // Joueur représenté par une sphère
            command.spawn(MaterialMeshBundle {
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.0, 1.0, 0.0),
                    ..default()
                }),
                mesh: meshes.add(Sphere::new(0.2)),
                transform: Transform::from_xyz(3.0, 1.0, 0.0),
                ..default()
            });

            // // Création du fusil devant la caméra pour un FPS
            // command.spawn(MaterialMeshBundle {
            //     material: materials.add(StandardMaterial {
            //         base_color: Color::rgb(0.0, 0.0, 0.0), // Couleur grise pour le fusil
            //         ..default()
            //     }),
            //     // Le fusil est un cuboid allongé
            //     mesh: meshes.add(Cuboid::new(0.2, 0.2, 0.6)),
            //     // Le fusil est positionné légèrement devant et en bas de la caméra, typique des jeux FPS
            //     transform: Transform::from_xyz(0.5, -0.5, -1.0), // Position devant la caméra
            //     ..default()
            // });
        });

    // Création du sol
    let board_width = board.data[0].len() as f32;
    let board_height = board.data.len() as f32;

    commands.spawn(MaterialMeshBundle {
        material: materials.add(StandardMaterial::default()),
        mesh: meshes.add(Plane3d::default()),
        transform: Transform::from_scale(Vec3::new(board_width, 1.0, board_height)),
        ..default()
    });

    // Charger la texture de brique
    let brick_texture_handle = asset_server.load("textures/brick_textures.png");

    // Génération des murs avec Cuboid
    for (z, line) in board.data.iter().enumerate() {
        for (x, c) in line.iter().enumerate() {
            if *c == '0' {
                continue;
            } else {
                commands.spawn(MaterialMeshBundle {
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(brick_texture_handle.clone()), // Appliquer la texture de brique
                        ..default()
                    }),
                    mesh: meshes.add(Cuboid::new(1.0, 3.0, 1.0)), // Conserver l'utilisation de Cuboid
                    transform: Transform::from_xyz(-(x as f32) + 7.0, 0.0, z as f32 - 7.0),
                    ..default()
                });
            }
        }
    }

    // Création de la balise
    let _ = commands
        .spawn(MaterialMeshBundle {
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 0.0, 0.0), // Couleur rouge pour la balise
                ..default()
            }),
            mesh: meshes.add(Sphere::new(0.3)), // Une petite sphère représente la balise
            transform: Transform::from_xyz(
                -(position_balise.pos.0 as f32) + 7.0,
                0.5,
                position_balise.pos.1 as f32 - 7.0,
            ),
            ..default()
        })
        .insert(Beacon)
        .id(); // Insertion du composant `Beacon`

    // Passer à l'état Playing une fois que la map est reçue
    next_state.set(AppState::Playing);
    info!("setup create");
}

pub fn check_victory_system(
    player_entities: Res<PlayerEntities>,
    mut next_state: ResMut<NextState<AppState>>,
    is_win: Res<IsWin>,
) {
    // let beacon_transform = query_beacon.single();

    // for player_transform in query_players.iter() {
    //     let distance = player_transform
    //         .translation
    //         .distance(beacon_transform.translation);

    //     if distance < 1.0 {
    //         println!("Victoire ! Le joueur a trouvé la balise !");
    //         next_state.set(AppState::GameOver)
    //         // Ajoute ici la logique pour gérer la victoire (exemple : fin de la partie)
    //     }
    // }
    info!("le nombre de player : {:?}", player_entities.0.len());
    if is_win.0 == true && player_entities.0.len() == 0 {
        next_state.set(AppState::GameOver);
    }
}
pub fn setup_game_over(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::BLACK.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "You win",
                TextStyle {
                    font_size: 50.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

pub fn setup_game_over_lose(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::BLACK.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "You lose",
                TextStyle {
                    font_size: 50.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
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
    mut player_entities: ResMut<PlayerEntities>,
    my_client_id: Res<MyClientId>,
    color: Res<ColorOtherPlayer>,
) {
    for event in spawn_events.read() {
        let client_id = event.0;
        if client_id == my_client_id.0 || player_entities.0.contains_key(&client_id) {
            continue;
        }
        // info!(
        //     "Handling player spawn event: {:?} color : {:?}",
        //     event.0, color
        // );

        // Créer le joueur avec la couleur générée
        let player = commands.spawn((
            MaterialMeshBundle {
                material: materials.add(StandardMaterial {
                    base_color: color.color, // Applique la couleur générée pour le joueur
                    ..default()
                }),
                mesh: meshes.add(Cuboid::new(0.5, 0.5, 0.5)), // Cube pour représenter le joueur
                transform: Transform::from_xyz(0.0, 1.0, 0.0), // Position initiale du joueur
                ..default()
            },
            PlayerEntity(client_id),
        ));
        if let Some(entity) = player_entities.0.get_mut(&client_id) {
            // info!("je sais pas ce qui ce passe");
            *entity = player.id();
        } else if !player_entities.0.contains_key(&client_id) {
            //info!("j'ai inserer un joueur");
            player_entities.0.insert(client_id, player.id());
        }
    }
}

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
            info!(
                "Spawning player loby {}: {:?}",
                client_id, player_sync.position
            );
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
// fn rand_position_player(board: &Res<Board>) -> Vec3 {
//     let mut rng = rand::thread_rng();
//     loop {
//         let z = rng.gen_range(0..board.data.len());
//         let x = rng.gen_range(0..board.data[0].len());
//         if board.data[z][x] == '0' {
//             return Vec3::new(x as f32 - 7.0, 1.0, z as f32 - 7.0);
//         }
//     }
// }

fn check_collision(map: Vec<Vec<char>>, position: Vec3) -> bool {
    let x = (-position.x + 7.0).round() as usize;
    let z = (position.z + 7.0).round() as usize;

    // if x >= map[0].len() || z >= map.len() {
    //     return true; // Out of bounds, consider it a collision
    // }

    //println!("Checking collision at x = {}, z = {}", x, z);
    map[z][x] == '1'
}

// Système pour configurer l'UI initiale
pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // Affichage de l'image initiale et du bouton "PLAY"
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            // Image initiale
            parent.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..Default::default()
                    },
                    image: UiImage::new(asset_server.load("images/image.png")),
                    ..Default::default()
                },
                InitialImageTag,
            ));

            // Node pour centrer le bouton
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    // Bouton Play
                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(150.0),
                                    height: Val::Px(65.0),
                                    margin: UiRect::all(Val::Px(20.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    position_type: PositionType::Absolute,
                                    ..Default::default()
                                },
                                background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                                ..Default::default()
                            },
                            ButtonTag,
                        ))
                        .with_children(|parent| {
                            // Texte du bouton
                            parent.spawn((
                                TextBundle::from_section(
                                    "PLAY",
                                    TextStyle {
                                        font: asset_server
                                            .load("fonts/FiraSansExtraCondensed-Black.ttf"),
                                        font_size: 40.0,
                                        color: Color::WHITE,
                                    },
                                ),
                                TextTag, // Tag pour identifier le texte
                            ));
                        });
                });
        });
}

// Système pour gérer le clic sur le bouton "PLAY"
pub fn handle_button_click(
    mut commands: Commands,
    mut interaction_query: Query<(&Interaction, Entity), (Changed<Interaction>, With<ButtonTag>)>,
    image_query: Query<Entity, With<InitialImageTag>>,
    text_query: Query<(Entity, &TextTag)>,
    mut button_clicked: ResMut<ButtonClicked>,
    mut next_state: ResMut<NextState<AppState>>,
    camera_query: Query<Entity, With<Camera>>, // Ajout de cette Query pour cibler la caméra
) {
    for (interaction, button_entity) in &mut interaction_query {
        if *interaction == Interaction::Pressed && !button_clicked.0 {
            // Supprimer le bouton "PLAY" et l'image initiale
            commands.entity(button_entity).despawn();
            for entity in image_query.iter() {
                commands.entity(entity).despawn();
            }

            for (text_entity, _) in text_query.iter() {
                commands.entity(text_entity).despawn();
            }

            // Supprimer la caméra
            for camera_entity in camera_query.iter() {
                commands.entity(camera_entity).despawn();
            }
            // ici met toutes fonctions pour le commencement du jeu

            button_clicked.0 = true; // Mettre à jour l'état pour indiquer que le bouton a été cliqué
            next_state.set(AppState::Setup);
        }
    }
}

pub fn get_connection_info() -> ConnectionInfo {
    let server_addr = loop {
        let mut input = String::new();
        print!("Entrez l'adresse IP du serveur (ex: 127.0.0.1:5000) ou appuyez sur Entrée directement pour jouer en local : ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();

        let trimmed = input.trim();
        if trimmed.is_empty() {
            println!("Vous jouerez localement.");
            break "127.0.0.1:5000".parse().unwrap(); // Adresse locale par défaut
        }

        match trimmed.parse() {
            Ok(addr) => break addr,
            Err(_) => {
                println!("Adresse IP invalide. Veuillez réessayer.");
                continue;
            }
        };
    };

    let username = loop {
        let mut input = String::new();
        print!("Entrez votre pseudo (ne depassant pas 255 caractères): ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();

        let trimmed = input.trim();
        if trimmed.is_empty() {
            println!("Le pseudo ne peut pas être vide. Veuillez réessayer.");
            continue;
        }
        if trimmed.len() > 255 {
            println!(
                "Le nom d'utilisateur est trop long (max 255 caractères). Veuillez réessayer."
            );
            continue;
        }
        break trimmed.to_string();
    };
    ConnectionInfo::new(server_addr, username)
}

pub fn connect_to_server(
    mut commands: Commands,
    connection_info: Res<ConnectionInfo>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // renet client
    let client = RenetClient::new(ConnectionConfig::default());
    commands.insert_resource(client);

    let client_id = rand::random::<u64>();
    commands.insert_resource(MyClientId(ClientId::from_raw(client_id)));

    let mut user_data = [0u8; 256];
    let username_bytes = connection_info.username.as_bytes();
    user_data[..username_bytes.len().min(255)]
        .copy_from_slice(&username_bytes[..username_bytes.len().min(255)]);

    let authentication = ClientAuthentication::Unsecure {
        server_addr: connection_info.server_addr,
        client_id,
        user_data: Some(user_data),
        protocol_id: 0,
    };
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    //initialise les ressources
    commands.insert_resource(transport);
    next_state.set(AppState::WaitingForMap);
}

use bevy::diagnostic::DiagnosticsStore;

// Système de mise à jour des FPS
pub fn update_fps(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    // Récupère les FPS via DiagnosticsStore
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.average())
    {
        for mut text in query.iter_mut() {
            text.sections[0].value = format!("FPS: {:.0}", fps); // Met à jour le texte avec les FPS
        }
    }
}

// Système d'initialisation pour configurer l'affichage des FPS
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Ici, pas besoin d'ajouter de caméra, ta caméra 3D existante sera utilisée pour tout.

    // Ajoute une caméra 2D pour le texte, avec un ordre explicite pour éviter les ambiguïtés
    commands.spawn((Camera2dBundle {
        camera: Camera {
            order: 2, // Priorité explicite de la caméra
            ..default()
        },
        ..default()
    },));

    // Création du texte des FPS (cela sera rendu dans le coin supérieur droit)
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "FPS:".to_string(), // Texte initial
                TextStyle {
                    font: asset_server.load("fonts/FiraSansExtraCondensed-Black.ttf"), // Police de caractères
                    font_size: 30.0,     // Taille du texte
                    color: Color::WHITE, // Couleur du texte
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                margin: UiRect {
                    top: Val::Px(10.0),   // Position en haut
                    left: Val::Auto,      // Ceci permet de pousser vers la droite
                    right: Val::Px(10.0), // Alignement à droite
                    bottom: Val::Auto,
                },
                ..default()
            },
            ..default()
        },
        FpsText, // Composant pour identifier ce texte
    ));
}

pub fn shoot_system(
    mut commands: Commands,
    mut keyboard_input_events: EventReader<KeyboardInput>,
    query: Query<(Entity, &Transform), With<Camera>>, // Récupérer à la fois l'entité et sa transformation
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in keyboard_input_events.read() {
        if event.state == ButtonState::Pressed && event.key_code == KeyCode::Space {
            let (camera_entity, camera_transform) = query.single(); // Récupérer l'entité de la caméra

            let forward = camera_transform.forward().into();
            let position = camera_transform.translation + forward * 0.1;

            commands.spawn((
                MaterialMeshBundle {
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(1.0, 0.0, 0.0),
                        ..default()
                    }),
                    mesh: meshes.add(Sphere::new(0.1)),
                    transform: Transform::from_translation(position),
                    ..default()
                },
                Projectile {
                    direction: forward,
                    shooter: camera_entity,
                }, // Associer l'entité du tireur
            ));
        }
    }
}

pub fn update_projectile_system(time: Res<Time>, mut query: Query<(&mut Transform, &Projectile)>) {
    for (mut transform, projectile) in query.iter_mut() {
        let speed = 8.0; // Vitesse du projectile
                         // Utiliser la direction du projectile pour le déplacement
        transform.translation += projectile.direction * speed * time.delta_seconds();
    }
}

pub fn handle_collision_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Projectile)>,
    board: Res<Board>,
) {
    for (entity, transform, _) in query.iter() {
        let x = (-transform.translation.x + 7.0).round() as usize;
        let z = (transform.translation.z + 7.0).round() as usize;

        if x >= board.data[0].len() || z >= board.data.len() {
            // Le projectile est sorti des limites du plateau, le supprimer
            commands.entity(entity).despawn();
        } else if board.data[z][x] == '1' {
            // Le projectile a touché un mur, le supprimer
            commands.entity(entity).despawn();
        }
    }
}
pub fn check_projectile_collision_system(
    mut commands: Commands,
    query_projectiles: Query<(Entity, &Transform, &Projectile)>,
    query_players: Query<(Entity, &Transform, &PlayerEntity)>,
    next_state: ResMut<NextState<AppState>>,
    mut client: ResMut<RenetClient>,
) {
    let _ = next_state;
    for (projectile_entity, projectile_transform, projectile) in query_projectiles.iter() {
        for (player_entity, player_transform, player) in query_players.iter() {
            // Ne pas vérifier la collision si le joueur a tiré le projectile
            if projectile.shooter != player_entity {
                // Comparez avec l'ID du joueur
                let distance = player_transform
                    .translation
                    .distance(projectile_transform.translation);

                if distance < 1.0 {
                    // Le projectile a touché un joueur
                    // println!("Un joueur a été touché par un projectile !");

                    // Supprimez le projectile de la scène
                    commands.entity(projectile_entity).despawn();
                    // commands.entity(player_entity).despawn();
                    // Envoyer un message au serveur pour informer de la mort du joueur
                    let message =
                        bincode::serialize(&ServerMessage::PlayerDeath(player.0)).unwrap();
                    client.send_message(DefaultChannel::ReliableOrdered, message);

                    // Changez l'état du jeu en GameOver
                    println!("la balle a touché le joueur {} ", player.0);
                    //return; // Sortir de la boucle après avoir terminé le jeu
                }
            }
        }
    }
}

pub fn handle_local_player_death(
    mut next_state: ResMut<NextState<AppState>>,
    mut death_event: ResMut<PlayerDeathEvent>,
    mut client: ResMut<RenetClient>
) {
    if death_event.0 {
        next_state.set(AppState::Lose);
        death_event.0 = false;
        client.disconnect();
    }
}
