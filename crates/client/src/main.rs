use bevy::prelude::*;
use clap::Parser;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use schizoid_shared::{SharedPlugin, SERVER_PORT, TICK_DURATION};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

mod input;
mod rendering;

#[derive(Parser, Debug)]
#[command(name = "schizoid-client")]
struct Args {
    /// Server address to connect to
    #[arg(short, long, default_value = "127.0.0.1")]
    connect: String,

    /// Server port
    #[arg(short, long, default_value_t = SERVER_PORT)]
    port: u16,
}

#[derive(Resource)]
struct ServerAddr(SocketAddr);

fn main() {
    let args = Args::parse();
    let server_addr = SocketAddr::new(
        args.connect
            .parse::<IpAddr>()
            .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        args.port,
    );

    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Schizoid".to_string(),
            resolution: (1280, 800).into(),
            ..default()
        }),
        ..default()
    }));

    // Lightyear client
    app.add_plugins(lightyear::prelude::client::ClientPlugins {
        tick_duration: TICK_DURATION,
    });

    // Game
    app.add_plugins(SharedPlugin);
    app.add_plugins(input::InputPlugin);
    app.add_plugins(rendering::RenderingPlugin);

    // Client-side prediction: run shared game systems on predicted entities
    app.add_systems(
        FixedUpdate,
        (
            schizoid_shared::systems::ship_movement,
            schizoid_shared::systems::enemy_movement,
            schizoid_shared::systems::chaser_ai,
            schizoid_shared::systems::collision_system,
            schizoid_shared::systems::respawn_system,
        )
            .chain(),
    );

    // Client connection setup
    app.insert_resource(ServerAddr(server_addr));
    app.add_systems(Startup, setup_connection);

    info!("Starting client, connecting to {}", server_addr);
    app.run();
}

fn setup_connection(mut commands: Commands, server_addr: Res<ServerAddr>) {
    let client_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        0, // Let OS assign port
    );

    let auth = lightyear::prelude::Authentication::Manual {
        server_addr: server_addr.0,
        client_id: rand::random::<u64>(),
        private_key: [0u8; 32], // Default key matches server default
        protocol_id: 0,         // Default protocol matches server default
    };

    let client = commands
        .spawn((
            Client::default(),
            LocalAddr(client_addr),
            PeerAddr(server_addr.0),
            Link::new(None),
            ReplicationReceiver::default(),
            PredictionManager::default(),
            NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
            UdpIo::default(),
        ))
        .id();

    commands.trigger(Connect { entity: client });

    info!("Connecting to server at {}", server_addr.0);
}
