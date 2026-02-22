pub mod components;
pub mod enemies;
pub mod protocol;
pub mod systems;

use bevy::prelude::*;
use protocol::ProtocolPlugin;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
        app.init_resource::<components::WaveState>();
        app.init_resource::<components::ArenaBounds>();
    }
}

pub const TICK_DURATION: std::time::Duration = std::time::Duration::from_millis(16);
pub const SERVER_PORT: u16 = 5555;
