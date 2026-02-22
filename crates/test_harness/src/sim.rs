use bevy::prelude::*;
use lightyear::prelude::input::native::ActionState;
use schizoid_shared::components::*;
use schizoid_shared::protocol::PlayerInput;
use schizoid_shared::systems;

/// Test simulation harness for game logic.
/// Wraps a Bevy App with MinimalPlugins for headless testing.
pub struct GameSim {
    app: App,
}

impl GameSim {
    /// Create a new local simulation (no networking).
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Initialize resources without networking plugins
        app.init_resource::<WaveState>();
        app.init_resource::<ArenaBounds>();

        // Register component types for reflection
        app.register_type::<TeamColor>();
        app.register_type::<Ship>();
        app.register_type::<Position>();
        app.register_type::<Radius>();
        app.register_type::<Health>();
        app.register_type::<EnemyType>();
        app.register_type::<Velocity>();
        app.register_type::<OrbitData>();

        // Register systems in Update schedule (MinimalPlugins doesn't include FixedUpdate)
        app.add_systems(
            Update,
            (
                systems::ship_movement,
                systems::enemy_movement,
                systems::chaser_ai,
                systems::collision_system,
                systems::respawn_system,
            ),
        );

        Self { app }
    }

    /// Spawn a player ship at a position.
    pub fn spawn_player(&mut self, color: TeamColor, pos: Vec2) -> Entity {
        self.app
            .world_mut()
            .spawn((
                Ship,
                color,
                Position(pos),
                Velocity(Vec2::ZERO),
                Radius(SHIP_RADIUS),
                Health::default(),
                // For tests without networking, we need to provide ActionState manually
                ActionState(PlayerInput {
                    direction: Vec2::ZERO,
                }),
            ))
            .id()
    }

    /// Spawn a drifter enemy.
    pub fn spawn_drifter(&mut self, color: TeamColor, pos: Vec2, vel: Vec2) -> Entity {
        self.app
            .world_mut()
            .spawn((
                color,
                EnemyType::Drifter,
                Position(pos),
                Velocity(vel),
                Radius(ENEMY_RADIUS),
            ))
            .id()
    }

    /// Spawn a chaser enemy.
    pub fn spawn_chaser(&mut self, color: TeamColor, pos: Vec2) -> Entity {
        self.app
            .world_mut()
            .spawn((
                color,
                EnemyType::Chaser,
                Position(pos),
                Velocity(Vec2::ZERO),
                Radius(ENEMY_RADIUS),
            ))
            .id()
    }

    /// Spawn an orbiter enemy.
    pub fn spawn_orbiter(&mut self, color: TeamColor, center: Vec2, radius: f32) -> Entity {
        self.app
            .world_mut()
            .spawn((
                color,
                EnemyType::Orbiter,
                Position(center + Vec2::new(radius, 0.0)),
                Velocity(Vec2::ZERO),
                Radius(ENEMY_RADIUS),
                OrbitData {
                    center,
                    radius,
                    angle: 0.0,
                    speed: ORBITER_SPEED,
                },
            ))
            .id()
    }

    /// Spawn an enemy by type.
    pub fn spawn_enemy(&mut self, etype: EnemyType, color: TeamColor, pos: Vec2) -> Entity {
        match etype {
            EnemyType::Drifter => self.spawn_drifter(color, pos, Vec2::new(DRIFTER_SPEED, 0.0)),
            EnemyType::Chaser => self.spawn_chaser(color, pos),
            EnemyType::Orbiter => self.spawn_orbiter(color, pos, ORBITER_RADIUS),
        }
    }

    /// Set a player's movement direction.
    pub fn set_input(&mut self, entity: Entity, direction: Vec2) {
        if let Some(mut action_state) = self
            .app
            .world_mut()
            .get_mut::<ActionState<PlayerInput>>(entity)
        {
            action_state.0 = PlayerInput { direction };
        }
    }

    /// Step the simulation forward by N ticks.
    pub fn step(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.app.update();
        }
    }

    /// Get entity's position.
    pub fn position(&self, entity: Entity) -> Vec2 {
        self.app
            .world()
            .get::<Position>(entity)
            .expect("entity has no Position")
            .0
    }

    /// Check if entity is alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        if let Some(health) = self.app.world().get::<Health>(entity) {
            health.alive
        } else {
            // Enemies don't have Health — they're alive if they exist
            self.app.world().get_entity(entity).is_ok()
        }
    }

    /// Check if entity is dead (or despawned).
    pub fn is_dead(&self, entity: Entity) -> bool {
        !self.is_alive(entity)
    }

    /// Get distance between two entities.
    pub fn distance(&self, a: Entity, b: Entity) -> f32 {
        let pos_a = self.position(a);
        let pos_b = self.position(b);
        pos_a.distance(pos_b)
    }

    /// Check if an entity still exists in the world.
    pub fn entity_exists(&self, entity: Entity) -> bool {
        self.app.world().get_entity(entity).is_ok()
    }

    /// Get velocity of an entity.
    pub fn velocity(&self, entity: Entity) -> Vec2 {
        self.app
            .world()
            .get::<Velocity>(entity)
            .expect("entity has no Velocity")
            .0
    }

    /// Get the current wave state.
    pub fn wave_state(&self) -> WaveState {
        self.app.world().resource::<WaveState>().clone()
    }

    /// Count remaining enemies.
    pub fn enemy_count(&mut self) -> usize {
        self.app
            .world_mut()
            .query::<&EnemyType>()
            .iter(self.app.world())
            .count()
    }
}

impl Default for GameSim {
    fn default() -> Self {
        Self::new()
    }
}
