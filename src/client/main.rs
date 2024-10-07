use bevy::{
    app::{App, Startup, Update},
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{in_state, Condition, IntoSystemConfigs, OnEnter},
    render::{
        camera::ClearColor,
        color::{self, Color},
    },
    DefaultPlugins,
};
use bevy_renet::{transport::NetcodeClientPlugin, RenetClientPlugin};

use resources::{AppState, ButtonClicked, ColorOtherPlayer};
use systems::{
    check_victory_system, connect_to_server, get_connection_info, handle_button_click, send_message_game_over, setup, setup_game_over, setup_game_over_lose, setup_ui, update_fps
};
//use systems::check_connection;
use crate::{
    resources::MyClientId,
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
    app.add_plugins(FrameTimeDiagnosticsPlugin);

    // app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);

    app.insert_resource(ColorOtherPlayer {
        color: color::Color::WHITE,
    });
    app.insert_resource(ButtonClicked::default());
    // // game systems
    app.add_systems(OnEnter(AppState::ConnectToServer), connect_to_server);

    app.add_systems(
        Update,
        receive_message_system.run_if(
            in_state(AppState::WaitingForMap).or_else(
                in_state(AppState::Menu)
                    .or_else(in_state(AppState::Setup).or_else(in_state(AppState::Playing))),
            ),
        ),
    );


    app.add_systems(Startup, setup); // Configurez le texte FPS
    app.add_systems(Update, update_fps.run_if(in_state(AppState::Playing))); // Ajoutez le système d'affichage des FPS

    // Appeler setup_ui pour afficher l'interface de démarrage
    app.add_systems(OnEnter(AppState::Menu), setup_ui);
    app.add_systems(Update, handle_button_click.run_if(in_state(AppState::Menu)));
    app.add_systems(
        OnEnter(AppState::Setup),
        (setup_system, spawn_map_2d).chain(),
    );
   // app.add_systems(Update, update_fps_display.run_if(in_state(AppState::Playing)));
   

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
    app.add_systems(OnEnter(AppState::GameOver), setup_game_over);
    app.add_systems(OnEnter(AppState::Lose), setup_game_over_lose);

    app.add_systems(OnEnter(AppState::GameOver), send_message_game_over);

    // game events
    app.add_event::<events::PlayerSpawnEvent>();
    app.add_event::<events::PlayerDespawnEvent>();
    app.add_event::<events::PlayerMoveEvent>();
    app.add_event::<events::LobbySyncEvent>();

    // info!("Client {} started", client_id);

    app.run();
}
