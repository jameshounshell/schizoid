use bevy::prelude::*;
use clap::Parser;
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

    // TODO: spawn client connection entity and connect to server
    // For now, the client starts but doesn't connect

    info!("Starting client, connecting to {}", server_addr);
    app.run();
}
