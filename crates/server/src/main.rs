use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use clap::Parser;
use schizoid_shared::{SharedPlugin, SERVER_PORT, TICK_DURATION};

mod game;

#[derive(Parser, Debug)]
#[command(name = "schizoid-server")]
struct Args {
    #[arg(short, long, default_value_t = SERVER_PORT)]
    port: u16,
}

fn main() {
    let args = Args::parse();

    let mut app = App::new();

    // Headless: no rendering, no window
    app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(TICK_DURATION)));
    app.add_plugins(bevy::log::LogPlugin::default());

    // Lightyear server
    app.add_plugins(lightyear::prelude::server::ServerPlugins {
        tick_duration: TICK_DURATION,
    });

    // Game
    app.add_plugins(SharedPlugin);
    app.add_plugins(game::ServerGamePlugin { port: args.port });

    info!("Starting server on port {}", args.port);
    app.run();
}
