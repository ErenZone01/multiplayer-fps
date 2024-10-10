use bevy::{
    ecs::{
        event::EventReader,
        system::{Res, ResMut},
    },
    log::info,
    prelude::{Color, Local},
};
use multiplayer_demo::{send_board, PlayerAttributes, ServerMessage};
use rand::{seq::IteratorRandom, Rng};
use renet::{DefaultChannel, RenetServer, ServerEvent};

use crate::{
    resources::{IsDeathOnce, IsTakingBalise, IsTakingMap, PlayerLobby},
    SERVER_ADDR,
};

pub fn setup_system() {
    info!("Server started on {}", SERVER_ADDR);
}

pub fn send_message_system(mut server: ResMut<RenetServer>, player_lobby: Res<PlayerLobby>, is_death_once : ResMut<IsDeathOnce>) {
    let chanel = DefaultChannel::Unreliable;
    let lobby: std::collections::HashMap<renet::ClientId, PlayerAttributes> =
        player_lobby.0.clone();
    let event = multiplayer_demo::ServerMessage::LobbySync(lobby);
    let message = bincode::serialize(&event).unwrap();
    //print_lobby(&player_lobby);
    server.broadcast_message(chanel, message);
    if player_lobby.0.len() == 1 && is_death_once.death {
        for (key, _) in player_lobby.0.clone(){
            let gameover = multiplayer_demo::ServerMessage::GameOver(key);
            let msg = bincode::serialize(&gameover).unwrap();
            server.send_message(key, DefaultChannel::ReliableOrdered, msg );
        }
    }
}

pub fn receive_message_system(
    mut server: ResMut<RenetServer>,
    mut player_lobby: ResMut<PlayerLobby>,
    mut is_death_once: ResMut<IsDeathOnce>,
) {
    for client_id in server.clients_id() {
        //mettre a jours la position des joueurs
        let message = server.receive_message(client_id, DefaultChannel::Unreliable);
        if let Some(message) = message {
            let player: PlayerAttributes = bincode::deserialize(&message).unwrap();
            player_lobby.0.insert(client_id, player);
        }
        // let message2 = server.receive_message(client_id, DefaultChannel::ReliableOrdered);
        // if let Some(_) = message2 {
        //     // Envoi des messages
        //     let msg = bincode::serialize(&ServerMessage::GameOver(client_id)).unwrap_or_default();
        //     server.broadcast_message_except(client_id, DefaultChannel::ReliableOrdered, msg);
        // }
        // Traitement des messages fiables (y compris la mort des joueurs)
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered)
        {
            if let Ok(server_message) = bincode::deserialize::<ServerMessage>(&message) {
                match server_message {
                    ServerMessage::PlayerDeath(dead_client_id) => {
                        if let Some(_) = player_lobby.0.get_mut(&dead_client_id) {
                            // Informer tous les clients de la mort du joueur
                            let death_message = ServerMessage::PlayerDeath(dead_client_id);
                            let broadcast_message = bincode::serialize(&death_message).unwrap();
                            server.broadcast_message(
                                DefaultChannel::ReliableOrdered,
                                broadcast_message,
                            );
                            player_lobby.0.remove(&dead_client_id);
                            is_death_once.death = true;
                            // if player_lobby.0.remove(&dead_client_id).is_some() {
                            //     // Le joueur a été retiré du lobby
                            //     let message = ServerMessage::PlayerLeave(dead_client_id);
                            //     let broadcast_message = bincode::serialize(&message).unwrap();
                            //     server.broadcast_message_except(
                            //         dead_client_id,
                            //         DefaultChannel::ReliableOrdered,
                            //         broadcast_message,
                            //     );
                            // }
                            info!(
                                "*Player {} died and was removed from the game*",
                                dead_client_id
                            );
                        }
                    }
                    // Ajoutez ici d'autres types de messages si nécessaire
                    _ => {}
                }
            }
        }
    }
}

pub fn handle_events_system(
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
    mut player_lobby: ResMut<PlayerLobby>,
    mut existing_colors: Local<Vec<Color>>, // Stocke les couleurs déjà utilisées
    mut is_taking_map: ResMut<IsTakingMap>, // Changement ici
    mut is_taking_balise: ResMut<IsTakingBalise>, // Changement ici
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Client {client_id} connected");
                let mut rng = rand::thread_rng();
                let min_color_distance = 0.5;

                // Initialiser les valeurs par défaut
                let random_map = is_taking_map.map.unwrap_or_else(|| {
                    let map_value = rng.gen_range(0..=2);
                    is_taking_map.map = Some(map_value); // Assurez-vous de mettre à jour ici
                    map_value
                });

                let random_balise = is_taking_balise.balise.unwrap_or_else(|| {
                    let balise_value = choose_position_balise(random_map);
                    is_taking_balise.balise = Some(balise_value); // Assurez-vous de mettre à jour ici
                    balise_value
                });

                // Génération de couleur
                let random_color = loop {
                    let color = Color::rgb(
                        rng.gen_range(0.0..1.0),
                        rng.gen_range(0.0..1.0),
                        rng.gen_range(0.0..0.5),
                    );
                    if existing_colors.iter().all(|existing_color| {
                        color_distance(&color, existing_color) > min_color_distance
                    }) {
                        existing_colors.push(color.clone());
                        break color;
                    }
                };

                player_lobby.0.insert(
                    *client_id,
                    PlayerAttributes {
                        position: [0.0, 0.0, 0.0],
                        color: random_color,
                    },
                );

                // Envoi des messages
                let player_join_message =
                    bincode::serialize(&ServerMessage::PlayerJoin((*client_id, random_color)))
                        .unwrap_or_default();
                let map_message =
                    bincode::serialize(&ServerMessage::Map(random_map)).unwrap_or_default();
                let balise_message = bincode::serialize(&ServerMessage::PosBalise(random_balise))
                    .unwrap_or_default();

                server.broadcast_message_except(
                    *client_id,
                    DefaultChannel::ReliableOrdered,
                    player_join_message,
                );
                server.send_message(*client_id, DefaultChannel::ReliableOrdered, balise_message);
                server.send_message(*client_id, DefaultChannel::ReliableOrdered, map_message);
                info!("Tous les messages ont été envoyés.");
            }

            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Client {client_id} disconnected: {reason}");
                player_lobby.0.remove(client_id);

                let player_leave_message =
                    bincode::serialize(&ServerMessage::PlayerLeave(*client_id)).unwrap_or_default();
                server.broadcast_message(DefaultChannel::ReliableOrdered, player_leave_message);
            }
        }
    }
}

// Fonction pour calculer la distance entre deux positions (utile pour éloigner la balise des joueurs)
fn distance(p1: (usize, usize), p2: (usize, usize)) -> f32 {
    let dx = p1.0 as f32 - p2.0 as f32;
    let dz = p1.1 as f32 - p2.1 as f32;
    (dx * dx + dz * dz).sqrt()
}

fn choose_position_balise(random_map: usize) -> (usize, usize) {
    // Génération aléatoire de la balise (hors des murs et éloignée des joueurs)
    let board = match random_map {
        0 => send_board(),
        1 => send_board(),
        2 => send_board(),
        _ => send_board(),
    };
    let valid_positions: Vec<(usize, usize)> = board
        .iter()
        .enumerate()
        .flat_map(|(z, line)| {
            line.iter().enumerate().filter_map(
                move |(x, c)| {
                    if *c == '0' {
                        Some((x, z))
                    } else {
                        None
                    }
                },
            )
        })
        .collect();
    // Choisir une position aléatoire pour la balise, en s'assurant qu'elle est éloignée du joueur
    let beacon_position = valid_positions
        .iter()
        .filter(|&&(x, z)| distance((0, 0), (x, z)) > 3.0)
        .choose(&mut rand::thread_rng())
        .expect("Aucune position valide pour la balise");
    return *beacon_position;
}

// Fonction pour calculer la distance euclidienne entre deux couleurs
fn color_distance(c1: &Color, c2: &Color) -> f32 {
    let rgba1 = c1.as_rgba_f32(); // Tableau [f32; 4]
    let rgba2 = c2.as_rgba_f32(); // Tableau [f32; 4]

    // Calcul de la distance euclidienne sur les trois premières composantes (r, g, b)
    ((rgba1[0] - rgba2[0]).powi(2) + (rgba1[1] - rgba2[1]).powi(2) + (rgba1[2] - rgba2[2]).powi(2))
        .sqrt()
}
