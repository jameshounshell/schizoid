use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use schizoid_shared::components::*;
use schizoid_shared::enemies::spawn_wave;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub struct ServerGamePlugin {
    pub port: u16,
}

impl Plugin for ServerGamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerConfig { port: self.port });

        app.add_systems(Startup, setup_server);
        app.add_observer(handle_new_client);
        app.add_observer(handle_connected);
        app.add_systems(
            FixedUpdate,
            (
                schizoid_shared::systems::ship_movement,
                schizoid_shared::systems::enemy_movement,
                schizoid_shared::systems::chaser_ai,
                schizoid_shared::systems::collision_system,
                schizoid_shared::systems::respawn_system,
                wave_manager,
            )
                .chain(),
        );
    }
}

#[derive(Resource)]
struct ServerConfig {
    port: u16,
}

#[derive(Resource, Default)]
struct ConnectedClients {
    count: usize,
}

fn setup_server(mut commands: Commands, config: Res<ServerConfig>) {
    commands.init_resource::<ConnectedClients>();

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);

    commands.spawn((
        Server::default(),
        NetcodeServer::new(NetcodeConfig::default()),
        LocalAddr(addr),
        ServerUdpIo::default(),
    ));

    info!("Server listening on port {}", config.port);
}

fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    info!("New client link created: {:?}", trigger.entity);
    commands
        .entity(trigger.entity)
        .insert(ReplicationSender::new(
            std::time::Duration::from_millis(100),
            SendUpdatesMode::SinceLastAck,
            false,
        ));
}

fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
    mut connected: ResMut<ConnectedClients>,
) {
    let client_id = query.get(trigger.entity).unwrap().0;
    info!("Client connected: {}", client_id);

    connected.count += 1;

    // Alternate colors for each new connection
    let color = if connected.count.is_multiple_of(2) {
        TeamColor::Red
    } else {
        TeamColor::Blue
    };

    // Spawn player ship
    commands.spawn((
        Ship,
        color,
        Position(Vec2::ZERO),
        Velocity(Vec2::ZERO),
        Radius(SHIP_RADIUS),
        Health::default(),
        OwnedBy(client_id.to_bits()),
        Replicate::to_clients(NetworkTarget::All),
        PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
        InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
        ControlledBy {
            owner: trigger.entity,
            lifetime: Default::default(),
        },
    ));

    info!(
        "Spawned ship for client {} with color {:?}",
        client_id, color
    );
}

fn wave_manager(
    mut wave: ResMut<WaveState>,
    enemies: Query<Entity, With<EnemyType>>,
    bounds: Res<ArenaBounds>,
    mut commands: Commands,
) {
    // Count remaining enemies
    wave.enemies_remaining = enemies.iter().count() as u32;

    if wave.active && wave.enemies_remaining == 0 {
        // Wave cleared, start breather
        info!("Wave {} cleared, starting breather", wave.current_wave);
        wave.active = false;
        wave.breather_timer = WAVE_BREATHER;
    }

    if !wave.active {
        wave.breather_timer -= 1.0 / 60.0;
        if wave.breather_timer <= 0.0 {
            wave.current_wave += 1;
            info!("Starting wave {}", wave.current_wave);
            spawn_wave(&mut commands, wave.current_wave, &bounds);
            wave.active = true;
        }
    }
}
