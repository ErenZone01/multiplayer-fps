use std::{
    collections::HashMap,
    net::{SocketAddrV4, UdpSocket},
    time::SystemTime,
};

use bevy::{
    app::{App, Update},
    log::info,
    prelude::{in_state, IntoSystemConfigs, OnEnter},
    render::{
        camera::ClearColor,
        color::{self, Color},
    },
    DefaultPlugins,
};
use bevy_renet::{transport::NetcodeClientPlugin, RenetClientPlugin};
use renet::{
    transport::{ClientAuthentication, NetcodeClientTransport},
    ClientId, ConnectionConfig, RenetClient,
};
use resources::{AppState, ButtonClicked, ColorOtherPlayer};
use systems::{check_victory_system, get_connection_info, handle_button_click, setup_ui};
//use systems::check_connection;
use crate::{
    resources::{MyClientId, PlayerEntities},
    systems::{
        handle_lobby_sync_event_system, handle_player_spawn_event_system,
        mini_map_sync_event_system, receive_message_system, rotation_player, send_message_system,
        setup_system, spawn_map_2d, update_player_movement_system,
    },
};

mod components;
mod events;
mod resources;
mod systems;

fn main() {
    //demander les informations du client
    let connection_info = get_connection_info();

    let mut app = App::new();
    // Insérez ConnectionInfo comme ressource
    app.insert_resource(connection_info);
    // Add AppState
    app.init_state::<AppState>();

    app.insert_resource(ClearColor(Color::hex("#bbCEcB").unwrap()));
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

    //app.insert_resource(Board { data: send_board() });

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

    //initialise les ressources
    app.insert_resource(transport);
    app.insert_resource(ColorOtherPlayer {
        color: color::Color::WHITE,
    });
    app.insert_resource(ButtonClicked::default());

    // // game systems
    app.add_systems(Update, receive_message_system);
    // Appeler setup_ui pour afficher l'interface de démarrage
    app.add_systems(OnEnter(AppState::Menu), setup_ui);
    app.add_systems(Update, handle_button_click.run_if(in_state(AppState::Menu)));
    app.add_systems(
        OnEnter(AppState::Setup),
        (setup_system, spawn_map_2d).chain(),
    );
    app.add_systems(
        Update,
        (
            handle_player_spawn_event_system,
            send_message_system,
            handle_lobby_sync_event_system,
            (rotation_player, update_player_movement_system),
            mini_map_sync_event_system,
            check_victory_system,
        )
            .chain()
            .run_if(in_state(AppState::Playing)),
    );

    // game events
    app.add_event::<events::PlayerSpawnEvent>();
    app.add_event::<events::PlayerDespawnEvent>();
    app.add_event::<events::PlayerMoveEvent>();
    app.add_event::<events::LobbySyncEvent>();

    info!("Client {} started", client_id);

    app.run();
}
