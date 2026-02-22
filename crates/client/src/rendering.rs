use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use schizoid_shared::components::*;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(Startup, setup_arena);
        app.add_systems(Update, sync_transforms);
        app.add_systems(Update, spawn_ship_visuals);
        app.add_systems(Update, spawn_enemy_visuals);
        app.add_systems(Update, update_hud);
        app.add_systems(Update, blink_invulnerable);
    }
}

/// Marker: this entity already has visuals spawned
#[derive(Component)]
struct HasVisuals;

/// Marker for the wave counter text
#[derive(Component)]
struct WaveText;

/// Marker for connection status text
#[derive(Component)]
struct StatusText;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.04, 0.04, 0.04)),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: 0.3,
            ..default()
        },
    ));
}

fn setup_arena(
    mut commands: Commands,
    bounds: Res<ArenaBounds>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let wall_thickness = 2.0;
    let wall_color = Color::srgb(1.5, 1.5, 1.5); // slight glow
    let mat = materials.add(wall_color);

    // Top wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(bounds.half_width * 2.0, wall_thickness))),
        MeshMaterial2d(mat.clone()),
        Transform::from_xyz(0.0, bounds.half_height, 0.0),
    ));
    // Bottom wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(bounds.half_width * 2.0, wall_thickness))),
        MeshMaterial2d(mat.clone()),
        Transform::from_xyz(0.0, -bounds.half_height, 0.0),
    ));
    // Left wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(wall_thickness, bounds.half_height * 2.0))),
        MeshMaterial2d(mat.clone()),
        Transform::from_xyz(-bounds.half_width, 0.0, 0.0),
    ));
    // Right wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(wall_thickness, bounds.half_height * 2.0))),
        MeshMaterial2d(mat),
        Transform::from_xyz(bounds.half_width, 0.0, 0.0),
    ));

    // HUD - Wave counter
    commands.spawn((
        WaveText,
        Text::new("Wave 0"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(2.0, 2.0, 2.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Percent(45.0),
            ..default()
        },
    ));

    // Connection status
    commands.spawn((
        StatusText,
        Text::new("Waiting..."),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Percent(40.0),
            ..default()
        },
    ));
}

/// Sync Position component to Transform for rendering
fn sync_transforms(mut query: Query<(&Position, &mut Transform), Changed<Position>>) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation.x = pos.0.x;
        transform.translation.y = pos.0.y;
    }
}

/// Spawn visuals for new ship entities
#[allow(clippy::type_complexity)]
fn spawn_ship_visuals(
    ships: Query<(Entity, &TeamColor, &Radius, &Position), (With<Ship>, Without<HasVisuals>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, color, radius, pos) in ships.iter() {
        commands.entity(entity).insert((
            HasVisuals,
            Mesh2d(meshes.add(Circle::new(radius.0))),
            MeshMaterial2d(materials.add(color.emissive())),
            Transform::from_xyz(pos.0.x, pos.0.y, 1.0),
        ));
    }
}

/// Spawn visuals for new enemy entities
#[allow(clippy::type_complexity)]
fn spawn_enemy_visuals(
    enemies: Query<
        (Entity, &TeamColor, &EnemyType, &Radius, &Position),
        (Without<Ship>, Without<HasVisuals>),
    >,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, color, enemy_type, radius, pos) in enemies.iter() {
        let mesh = match enemy_type {
            EnemyType::Drifter => meshes.add(Circle::new(radius.0)),
            EnemyType::Chaser => meshes.add(RegularPolygon::new(radius.0, 3)),
            EnemyType::Orbiter => meshes.add(Annulus::new(radius.0 * 0.6, radius.0)),
        };

        commands.entity(entity).insert((
            HasVisuals,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(color.emissive_dim())),
            Transform::from_xyz(pos.0.x, pos.0.y, 0.5),
        ));
    }
}

/// Update wave counter HUD
fn update_hud(wave: Res<WaveState>, mut wave_text: Query<&mut Text, With<WaveText>>) {
    if wave.is_changed() {
        for mut text in wave_text.iter_mut() {
            *text = Text::new(format!("Wave {}", wave.current_wave));
        }
    }
}

/// Blink ships that are invulnerable
fn blink_invulnerable(mut ships: Query<(&Health, &mut Visibility), With<Ship>>, time: Res<Time>) {
    for (health, mut vis) in ships.iter_mut() {
        if health.invulnerable_timer > 0.0 {
            // Blink every 0.1 seconds
            let blink = ((time.elapsed_secs() * 10.0) as u32).is_multiple_of(2);
            *vis = if blink {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        } else if health.alive {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}
