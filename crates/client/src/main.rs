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

    /// Drive the ship with the built-in bot AI instead of keyboard/gamepad
    #[arg(long)]
    bot: bool,

    /// Save a screenshot to this path (for unattended verification)
    #[arg(long)]
    screenshot: Option<String>,

    /// Seconds to wait before taking the screenshot
    #[arg(long, default_value_t = 8.0)]
    screenshot_at: f32,

    /// Exit automatically after this many seconds (unattended runs)
    #[arg(long)]
    exit_after: Option<f32>,
}

#[derive(Resource)]
struct ServerAddr(SocketAddr);

#[derive(Resource)]
struct AutoRunConfig {
    screenshot: Option<String>,
    screenshot_at: f32,
    exit_after: Option<f32>,
}

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
    app.insert_resource(input::BotMode(args.bot));
    app.insert_resource(AutoRunConfig {
        screenshot: args.screenshot,
        screenshot_at: args.screenshot_at,
        exit_after: args.exit_after,
    });
    app.add_systems(Startup, setup_connection);
    app.add_systems(Update, auto_run);

    info!("Starting client, connecting to {}", server_addr);
    app.run();
}

/// Unattended-run helpers: timed screenshot for visual verification and
/// timed exit so demo runs tear themselves down.
fn auto_run(
    mut commands: Commands,
    cfg: Res<AutoRunConfig>,
    time: Res<Time>,
    mut screenshot_taken: Local<bool>,
    mut exit: MessageWriter<AppExit>,
) {
    let elapsed = time.elapsed_secs();

    if let Some(path) = &cfg.screenshot {
        if !*screenshot_taken && elapsed >= cfg.screenshot_at {
            *screenshot_taken = true;
            info!("Taking screenshot -> {}", path);
            commands
                .spawn(bevy::render::view::window::screenshot::Screenshot::primary_window())
                .observe(bevy::render::view::window::screenshot::save_to_disk(
                    path.clone(),
                ));
        }
    }

    if let Some(deadline) = cfg.exit_after {
        if elapsed >= deadline {
            info!("exit_after {} reached, exiting", deadline);
            exit.write(AppExit::Success);
        }
    }
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
