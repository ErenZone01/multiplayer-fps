use std::{
    collections::HashMap,
    net::{SocketAddrV4, UdpSocket},
    time::SystemTime,
};

use bevy::{
    app::{App, Startup, Update},
    log::info,
    prelude::IntoSystemConfigs,
    render::{camera::ClearColor, color::Color},
    DefaultPlugins,
};
use bevy_renet::{transport::NetcodeClientPlugin, RenetClientPlugin};
use rand::Rng;
use renet::{
    transport::{ClientAuthentication, NetcodeClientTransport},
    ClientId, ConnectionConfig, RenetClient,
};
use multiplayer_demo::BOARD;
//use systems::check_connection;
use crate::{
    resources::{MyClientId, PlayerEntities,Board},
    systems::{
        handle_lobby_sync_event_system, handle_player_spawn_event_system, receive_message_system,
        rotation_player, send_message_system, setup_system,spawn_map_2d, update_player_movement_system,
        mini_map_sync_event_system
    },
};

mod components;
mod events;
mod resources;
mod systems;

fn main() {
    let mut app = App::new();
    let mut colors : Vec<&str> = Vec::new();
    colors.push("FF00FF");
    colors.push("#bbCEcB");
    colors.push("FF00FF7F");
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..3);
    app.insert_resource(ClearColor(Color::hex(colors[index]).unwrap()));
    // base plugins
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(NetcodeClientPlugin);
    app.add_plugins(DefaultPlugins);

    // renet client
    let client = RenetClient::new(ConnectionConfig::default());
    app.insert_resource(client);

    let client_id = rand::random::<u64>();
    app.insert_resource(MyClientId(ClientId::from_raw(client_id)));
    app.insert_resource(PlayerEntities(HashMap::new()));

    app.insert_resource(Board{data:BOARD});
    let authentication = ClientAuthentication::Unsecure {
        server_addr: std::net::SocketAddr::V4(SocketAddrV4::new(
            std::net::Ipv4Addr::new(127, 0, 0, 1),
            5000,
        )),
        client_id,
        user_data: None,
        protocol_id: 0,
    };
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    app.insert_resource(transport);

    // game events
    app.add_event::<events::PlayerSpawnEvent>();
    app.add_event::<events::PlayerDespawnEvent>();
    app.add_event::<events::PlayerMoveEvent>();
    app.add_event::<events::LobbySyncEvent>();

    
    // game systems
    app.add_systems(Update, (send_message_system,receive_message_system));
    app.add_systems(Update, handle_player_spawn_event_system);
    app.add_systems(Update, ((rotation_player, update_player_movement_system),mini_map_sync_event_system).chain());
    app.add_systems(Update, handle_lobby_sync_event_system);
    app.add_systems(Startup, (setup_system,spawn_map_2d).chain());

    info!("Client {} started", client_id);

    app.run();
}
