# Schizoid Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a twin-stick co-op shooter with online multiplayer using Bevy 0.18 + lightyear 0.26.

**Architecture:** Cargo workspace with 4 crates: shared (game logic + protocol), client (rendering + input), server (authoritative simulation), test_harness (GameSim DSL + integration tests). All game logic in shared, thin client/server wrappers.

**Tech Stack:** Rust 2024 edition, Bevy 0.18, lightyear 0.26.4, serde, clap

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/shared/Cargo.toml`
- Create: `crates/shared/src/lib.rs`
- Create: `crates/client/Cargo.toml`
- Create: `crates/client/src/main.rs`
- Create: `crates/server/Cargo.toml`
- Create: `crates/server/src/main.rs`
- Create: `crates/test_harness/Cargo.toml`
- Create: `crates/test_harness/src/lib.rs`
- Create: `.gitignore`
- Create: `rust-toolchain.toml`
- Create: `Taskfile.yml`
- Create: `.pre-commit-config.yaml`

**Step 1: Create workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = [
    "crates/shared",
    "crates/client",
    "crates/server",
    "crates/test_harness",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
rust-version = "1.88"

[workspace.dependencies]
bevy = "0.18"
lightyear = { version = "0.26", features = [
    "replication",
    "prediction",
    "interpolation",
    "input_native",
] }
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```

**Step 2: Create crate Cargo.tomls**

`crates/shared/Cargo.toml`:
```toml
[package]
name = "schizoid-shared"
version.workspace = true
edition.workspace = true

[dependencies]
bevy = { workspace = true }
lightyear = { workspace = true }
serde = { workspace = true }
```

`crates/client/Cargo.toml`:
```toml
[package]
name = "schizoid-client"
version.workspace = true
edition.workspace = true

[dependencies]
bevy = { workspace = true }
lightyear = { workspace = true, features = ["client", "netcode", "udp"] }
schizoid-shared = { path = "../shared" }
clap = { workspace = true }
```

`crates/server/Cargo.toml`:
```toml
[package]
name = "schizoid-server"
version.workspace = true
edition.workspace = true

[dependencies]
bevy = { workspace = true, default-features = false, features = ["bevy_log", "bevy_state"] }
lightyear = { workspace = true, features = ["server", "netcode", "udp"] }
schizoid-shared = { path = "../shared" }
clap = { workspace = true }
```

`crates/test_harness/Cargo.toml`:
```toml
[package]
name = "schizoid-test-harness"
version.workspace = true
edition.workspace = true

[dependencies]
bevy = { workspace = true, default-features = false }
lightyear = { workspace = true, features = ["client", "server", "netcode", "crossbeam"] }
schizoid-shared = { path = "../shared" }

[dev-dependencies]
```

**Step 3: Create stub source files**

`crates/shared/src/lib.rs`:
```rust
pub mod components;
pub mod protocol;
pub mod systems;
pub mod enemies;
```

`crates/client/src/main.rs`:
```rust
fn main() {
    println!("schizoid client");
}
```

`crates/server/src/main.rs`:
```rust
fn main() {
    println!("schizoid server");
}
```

`crates/test_harness/src/lib.rs`:
```rust
pub mod sim;
```

Create empty module files:
- `crates/shared/src/components.rs`
- `crates/shared/src/protocol.rs`
- `crates/shared/src/systems.rs`
- `crates/shared/src/enemies.rs`
- `crates/test_harness/src/sim.rs`

**Step 4: Create .gitignore**

```
/target
*.swp
*.swo
.DS_Store
```

**Step 5: Create rust-toolchain.toml**

```toml
[toolchain]
channel = "stable"
```

**Step 6: Create Taskfile.yml**

```yaml
version: '3'

tasks:
  build:
    desc: Build all crates
    cmds:
      - cargo build --workspace

  test:
    desc: Run all tests
    cmds:
      - cargo test --workspace

  test-harness:
    desc: Run test harness tests only
    cmds:
      - cargo test -p schizoid-test-harness

  check:
    desc: Check all crates compile
    cmds:
      - cargo check --workspace

  clippy:
    desc: Run clippy lints
    cmds:
      - cargo clippy --workspace -- -D warnings

  fmt:
    desc: Format all code
    cmds:
      - cargo fmt --all

  fmt-check:
    desc: Check formatting
    cmds:
      - cargo fmt --all -- --check

  server:
    desc: Run the game server
    cmds:
      - cargo run -p schizoid-server -- {{.CLI_ARGS}}

  client:
    desc: Run the game client
    cmds:
      - cargo run -p schizoid-client -- {{.CLI_ARGS}}

  lint:
    desc: Run all lints (fmt + clippy)
    cmds:
      - task: fmt-check
      - task: clippy

  ci:
    desc: Full CI check (fmt + clippy + test)
    cmds:
      - task: fmt-check
      - task: clippy
      - task: test
```

**Step 7: Create .pre-commit-config.yaml**

```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all -- --check
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --workspace -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-test
        name: cargo test
        entry: cargo test --workspace
        language: system
        types: [rust]
        pass_filenames: false
        stages: [pre-push]
```

**Step 8: Verify it compiles**

Run: `cargo check --workspace`
Expected: Compiles with no errors

**Step 9: Commit**

```bash
git add -A
git commit -m "feat: scaffold workspace with shared, client, server, test_harness crates"
git push
```

---

### Task 2: Components & Protocol

**Files:**
- Create: `crates/shared/src/components.rs`
- Create: `crates/shared/src/protocol.rs`
- Modify: `crates/shared/src/lib.rs`

**Step 1: Define components**

`crates/shared/src/components.rs`:
```rust
use bevy::math::Curve;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

/// Red or Blue team color — the core mechanic.
#[derive(
    Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect,
)]
pub enum TeamColor {
    Red,
    Blue,
}

impl TeamColor {
    /// Returns the opposite color.
    pub fn opposite(self) -> Self {
        match self {
            TeamColor::Red => TeamColor::Blue,
            TeamColor::Blue => TeamColor::Red,
        }
    }

    /// Returns the render color (emissive values for bloom).
    pub fn emissive(self) -> Color {
        match self {
            TeamColor::Red => Color::srgb(5.0, 0.2, 0.2),
            TeamColor::Blue => Color::srgb(0.2, 0.5, 5.0),
        }
    }

    /// Returns a dimmer version for enemies.
    pub fn emissive_dim(self) -> Color {
        match self {
            TeamColor::Red => Color::srgb(3.0, 0.1, 0.1),
            TeamColor::Blue => Color::srgb(0.1, 0.3, 3.0),
        }
    }
}

/// Marks an entity as a player ship.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct Ship;

/// Player position — replicated, predicted, and interpolated.
#[derive(
    Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect, Deref, DerefMut,
)]
pub struct Position(pub Vec2);

impl Ease for Position {
    fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self> {
        bevy::math::curve::FunctionCurve::new(
            bevy::math::curve::Interval::UNIT,
            move |t| Position(Vec2::lerp(start.0, end.0, t)),
        )
    }
}

/// Entity radius for collision detection.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct Radius(pub f32);

/// Whether entity is alive.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct Health {
    pub alive: bool,
    pub respawn_timer: f32,
    pub invulnerable_timer: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            alive: true,
            respawn_timer: 0.0,
            invulnerable_timer: 0.0,
        }
    }
}

/// Enemy type determines AI behavior.
#[derive(
    Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Reflect,
)]
pub enum EnemyType {
    Drifter,
    Chaser,
    Orbiter,
}

/// Velocity for moving entities.
#[derive(
    Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect, Deref, DerefMut,
)]
pub struct Velocity(pub Vec2);

/// Orbiter-specific: the center point and orbit radius.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct OrbitData {
    pub center: Vec2,
    pub radius: f32,
    pub angle: f32,
    pub speed: f32,
}

/// Which player owns this entity (for input association).
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct OwnedBy(pub PeerId);

/// Wave tracking.
#[derive(Resource, Clone, Debug, Default)]
pub struct WaveState {
    pub current_wave: u32,
    pub enemies_remaining: u32,
    pub breather_timer: f32,
    pub active: bool,
}

/// Arena bounds.
#[derive(Resource, Clone, Debug)]
pub struct ArenaBounds {
    pub half_width: f32,
    pub half_height: f32,
}

impl Default for ArenaBounds {
    fn default() -> Self {
        Self {
            half_width: 600.0,
            half_height: 400.0,
        }
    }
}

/// Game constants.
pub const SHIP_SPEED: f32 = 300.0;
pub const SHIP_RADIUS: f32 = 15.0;
pub const ENEMY_RADIUS: f32 = 10.0;
pub const DRIFTER_SPEED: f32 = 120.0;
pub const CHASER_SPEED: f32 = 150.0;
pub const ORBITER_SPEED: f32 = 2.0; // radians per second
pub const ORBITER_RADIUS: f32 = 80.0;
pub const RESPAWN_TIME: f32 = 2.0;
pub const INVULNERABLE_TIME: f32 = 1.0;
pub const WAVE_BREATHER: f32 = 3.0;
```

**Step 2: Define inputs and protocol**

`crates/shared/src/protocol.rs`:
```rust
use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::*;

/// Player input: movement direction as a normalized Vec2.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Reflect)]
pub struct PlayerInput {
    /// Movement direction (normalized or zero).
    pub direction: Vec2,
}

impl MapEntities for PlayerInput {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

/// Protocol plugin — registers all networked types.
#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Inputs
        app.add_plugins(
            lightyear::prelude::input::native::InputPlugin::<PlayerInput>::default(),
        );

        // Components
        app.register_component::<TeamColor>();
        app.register_component::<Ship>();
        app.register_component::<OwnedBy>();
        app.register_component::<Radius>();

        app.register_component::<Position>()
            .add_prediction()
            .add_linear_interpolation();

        app.register_component::<Health>()
            .add_prediction();

        app.register_component::<EnemyType>();

        app.register_component::<Velocity>()
            .add_prediction()
            .add_linear_interpolation();

        app.register_component::<OrbitData>();
    }
}

impl Ease for Velocity {
    fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self> {
        bevy::math::curve::FunctionCurve::new(
            bevy::math::curve::Interval::UNIT,
            move |t| Velocity(Vec2::lerp(start.0, end.0, t)),
        )
    }
}
```

**Step 3: Update lib.rs**

`crates/shared/src/lib.rs`:
```rust
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

pub const TICK_DURATION: std::time::Duration = std::time::Duration::from_millis(16); // ~60Hz
pub const SERVER_PORT: u16 = 5555;
```

**Step 4: Verify it compiles**

Run: `cargo check -p schizoid-shared`

**Step 5: Commit**

```bash
git add crates/shared/
git commit -m "feat(shared): define components, protocol, and inputs"
git push
```

---

### Task 3: Movement & Collision Systems

**Files:**
- Create: `crates/shared/src/systems.rs`

**Step 1: Implement shared systems**

`crates/shared/src/systems.rs`:
```rust
use bevy::prelude::*;

use crate::components::*;
use crate::protocol::PlayerInput;

/// Apply player input to ship position. Runs on both client (prediction) and server.
pub fn ship_movement(
    mut query: Query<(&mut Position, &lightyear::prelude::input::native::ActionState<PlayerInput>), With<Ship>>,
    bounds: Res<ArenaBounds>,
) {
    for (mut pos, input) in query.iter_mut() {
        let dir = input.0.direction;
        if dir.length_squared() > 0.0 {
            let movement = dir.normalize_or_zero() * SHIP_SPEED * (1.0 / 60.0);
            pos.0 += movement;
            // Clamp to arena bounds
            pos.0.x = pos.0.x.clamp(-bounds.half_width + SHIP_RADIUS, bounds.half_width - SHIP_RADIUS);
            pos.0.y = pos.0.y.clamp(-bounds.half_height + SHIP_RADIUS, bounds.half_height - SHIP_RADIUS);
        }
    }
}

/// Move enemies based on their type. Server-authoritative.
pub fn enemy_movement(
    mut drifters: Query<(&mut Position, &mut Velocity), (With<EnemyType>, Without<OrbitData>, Without<Ship>)>,
    mut orbiters: Query<(&mut Position, &mut OrbitData), (With<EnemyType>, Without<Ship>)>,
    bounds: Res<ArenaBounds>,
) {
    let dt = 1.0 / 60.0;

    // Drifters: move in direction, bounce off walls
    for (mut pos, mut vel) in drifters.iter_mut() {
        pos.0 += vel.0 * dt;

        // Bounce off walls
        if pos.0.x.abs() > bounds.half_width - ENEMY_RADIUS {
            vel.0.x = -vel.0.x;
            pos.0.x = pos.0.x.clamp(-bounds.half_width + ENEMY_RADIUS, bounds.half_width - ENEMY_RADIUS);
        }
        if pos.0.y.abs() > bounds.half_height - ENEMY_RADIUS {
            vel.0.y = -vel.0.y;
            pos.0.y = pos.0.y.clamp(-bounds.half_height + ENEMY_RADIUS, bounds.half_height - ENEMY_RADIUS);
        }
    }

    // Orbiters: circle their center point
    for (mut pos, mut orbit) in orbiters.iter_mut() {
        orbit.angle += orbit.speed * dt;
        pos.0 = orbit.center + Vec2::new(orbit.angle.cos(), orbit.angle.sin()) * orbit.radius;
    }
}

/// Chaser AI: steer toward the nearest opposite-color ship.
pub fn chaser_ai(
    mut chasers: Query<(&mut Velocity, &Position, &TeamColor, &EnemyType), Without<Ship>>,
    ships: Query<(&Position, &TeamColor, &Health), With<Ship>>,
) {
    for (mut vel, chaser_pos, chaser_color, enemy_type) in chasers.iter_mut() {
        if *enemy_type != EnemyType::Chaser {
            continue;
        }

        // Find nearest opposite-color ship that is alive
        let target = ships
            .iter()
            .filter(|(_, color, health)| **color != *chaser_color && health.alive)
            .min_by(|(pos_a, _, _), (pos_b, _, _)| {
                let dist_a = pos_a.0.distance_squared(chaser_pos.0);
                let dist_b = pos_b.0.distance_squared(chaser_pos.0);
                dist_a.partial_cmp(&dist_b).unwrap()
            });

        if let Some((target_pos, _, _)) = target {
            let dir = (target_pos.0 - chaser_pos.0).normalize_or_zero();
            vel.0 = dir * CHASER_SPEED;
        }
    }
}

/// Check collisions between ships and enemies.
/// Same color = enemy dies. Different color = ship dies (if not invulnerable).
pub fn collision_system(
    mut ships: Query<(&Position, &TeamColor, &mut Health, &Radius), With<Ship>>,
    mut enemies: Query<(Entity, &Position, &TeamColor, &Radius), (With<EnemyType>, Without<Ship>)>,
    mut commands: Commands,
) {
    for (ship_pos, ship_color, mut ship_health, ship_radius) in ships.iter_mut() {
        if !ship_health.alive {
            continue;
        }

        for (enemy_entity, enemy_pos, enemy_color, enemy_radius) in enemies.iter() {
            let dist = ship_pos.0.distance(enemy_pos.0);
            let min_dist = ship_radius.0 + enemy_radius.0;

            if dist < min_dist {
                if *ship_color == *enemy_color {
                    // Same color: enemy destroyed
                    commands.entity(enemy_entity).despawn();
                } else if ship_health.invulnerable_timer <= 0.0 {
                    // Different color: ship dies
                    ship_health.alive = false;
                    ship_health.respawn_timer = RESPAWN_TIME;
                }
            }
        }
    }
}

/// Handle respawn timers.
pub fn respawn_system(mut ships: Query<(&mut Health, &mut Position), With<Ship>>) {
    let dt = 1.0 / 60.0;

    for (mut health, mut pos) in ships.iter_mut() {
        if !health.alive {
            health.respawn_timer -= dt;
            if health.respawn_timer <= 0.0 {
                health.alive = true;
                health.invulnerable_timer = INVULNERABLE_TIME;
                // Respawn at center-ish (simple for MVP)
                pos.0 = Vec2::ZERO;
            }
        }

        if health.invulnerable_timer > 0.0 {
            health.invulnerable_timer -= dt;
        }
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p schizoid-shared`

**Step 3: Commit**

```bash
git add crates/shared/src/systems.rs
git commit -m "feat(shared): implement movement, collision, and respawn systems"
git push
```

---

### Task 4: Enemy AI Module

**Files:**
- Create: `crates/shared/src/enemies.rs`

`crates/shared/src/enemies.rs`:
```rust
use bevy::prelude::*;
use rand::Rng;

use crate::components::*;

/// Spawn a wave of enemies.
pub fn spawn_wave(commands: &mut Commands, wave: u32, bounds: &ArenaBounds) {
    let count = (wave * 3 + 2) as usize;
    let mut rng = rand::rng();

    for i in 0..count {
        let color = if rng.random_bool(0.5) {
            TeamColor::Red
        } else {
            TeamColor::Blue
        };

        // Distribute enemy types: more chasers as waves increase
        let chaser_chance = (wave as f32 * 0.1).min(0.6);
        let orbiter_chance = 0.2;
        let roll: f32 = rng.random();

        let enemy_type = if roll < chaser_chance {
            EnemyType::Chaser
        } else if roll < chaser_chance + orbiter_chance {
            EnemyType::Orbiter
        } else {
            EnemyType::Drifter
        };

        // Random position along edges (not too close to center where players spawn)
        let spawn_pos = random_edge_position(&mut rng, bounds);

        match enemy_type {
            EnemyType::Drifter => {
                let angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
                let vel = Vec2::new(angle.cos(), angle.sin()) * DRIFTER_SPEED;
                commands.spawn((
                    color,
                    enemy_type,
                    Position(spawn_pos),
                    Velocity(vel),
                    Radius(ENEMY_RADIUS),
                ));
            }
            EnemyType::Chaser => {
                commands.spawn((
                    color,
                    enemy_type,
                    Position(spawn_pos),
                    Velocity(Vec2::ZERO),
                    Radius(ENEMY_RADIUS),
                ));
            }
            EnemyType::Orbiter => {
                let orbit_center = Vec2::new(
                    rng.random_range(-bounds.half_width * 0.6..bounds.half_width * 0.6),
                    rng.random_range(-bounds.half_height * 0.6..bounds.half_height * 0.6),
                );
                commands.spawn((
                    color,
                    enemy_type,
                    Position(spawn_pos),
                    Velocity(Vec2::ZERO),
                    Radius(ENEMY_RADIUS),
                    OrbitData {
                        center: orbit_center,
                        radius: ORBITER_RADIUS,
                        angle: rng.random_range(0.0..std::f32::consts::TAU),
                        speed: ORBITER_SPEED * if rng.random_bool(0.5) { 1.0 } else { -1.0 },
                    },
                ));
            }
        }
    }
}

fn random_edge_position(rng: &mut impl Rng, bounds: &ArenaBounds) -> Vec2 {
    let side = rng.random_range(0..4);
    match side {
        0 => Vec2::new(rng.random_range(-bounds.half_width..bounds.half_width), bounds.half_height - 20.0),
        1 => Vec2::new(rng.random_range(-bounds.half_width..bounds.half_width), -bounds.half_height + 20.0),
        2 => Vec2::new(bounds.half_width - 20.0, rng.random_range(-bounds.half_height..bounds.half_height)),
        _ => Vec2::new(-bounds.half_width + 20.0, rng.random_range(-bounds.half_height..bounds.half_height)),
    }
}
```

Add `rand` to shared deps.

**Step 2: Verify and commit**

```bash
cargo check -p schizoid-shared
git add crates/shared/
git commit -m "feat(shared): add enemy spawning and wave generation"
git push
```

---

### Task 5: Server Implementation

**Files:**
- Create: `crates/server/src/main.rs`

The server:
- Runs headless (MinimalPlugins)
- Spawns player entities when clients connect
- Runs authoritative game simulation
- Manages wave progression

**Step 1: Implement server**

See design doc for server plugin pattern. Key systems:
- `handle_new_client` — add ReplicationSender
- `handle_connected` — spawn ship entity with Replicate + PredictionTarget + InterpolationTarget
- `movement` — apply inputs server-side
- `wave_manager` — spawn enemy waves, track remaining

**Step 2: Verify and commit**

---

### Task 6: Client Implementation

**Files:**
- Create: `crates/client/src/main.rs`
- Create: `crates/client/src/rendering.rs`
- Create: `crates/client/src/input.rs`

The client:
- Runs with DefaultPlugins + bloom
- Reads gamepad/keyboard input, buffers via lightyear
- Predicts own ship movement
- Interpolates remote entities
- Renders everything with neon bloom

**Step 1: Implement input buffering**

Buffer gamepad left_stick + WASD into `ActionState<PlayerInput>`.

**Step 2: Implement rendering**

Camera with `Bloom::default()`, spawn `Mesh2d` + `MeshMaterial2d` for ships/enemies.

**Step 3: Implement predicted/interpolated spawn handlers**

Add `InputMarker` on predicted spawn, set up visuals on both predicted and interpolated entities.

**Step 4: Verify and commit**

---

### Task 7: Test Harness (GameSim DSL)

**Files:**
- Create: `crates/test_harness/src/lib.rs`
- Create: `crates/test_harness/src/sim.rs`
- Create: `crates/test_harness/tests/movement.rs`
- Create: `crates/test_harness/tests/collision.rs`

Implement `GameSim` struct wrapping Bevy's `App` with `MinimalPlugins` for testing game logic in isolation.

**Step 1: Implement GameSim**

```rust
pub struct GameSim {
    app: App,
}

impl GameSim {
    pub fn new() -> Self { /* MinimalPlugins + SharedPlugin */ }
    pub fn spawn_player(&mut self, color: TeamColor, pos: Vec2) -> Entity { ... }
    pub fn spawn_enemy(&mut self, etype: EnemyType, color: TeamColor, pos: Vec2) -> Entity { ... }
    pub fn step(&mut self, ticks: u32) { ... }
    pub fn position(&self, entity: Entity) -> Vec2 { ... }
    pub fn is_alive(&self, entity: Entity) -> bool { ... }
    pub fn is_dead(&self, entity: Entity) -> bool { ... }
    pub fn distance(&self, a: Entity, b: Entity) -> f32 { ... }
}
```

**Step 2: Write tests and verify**

**Step 3: Commit**

---

### Task 8: Integration & Polish

- Wire up `Taskfile.yml` commands to actual binaries
- Install pre-commit hooks
- Verify `task ci` passes
- Create README.md with build/run instructions
- Final commit + push
