# Schizoid-Inspired Twin-Stick Co-op Shooter — Game Design

*Date: 2026-02-22*
*Status: Approved*

## Overview

A twin-stick cooperative action game inspired by [Schizoid](https://en.wikipedia.org/wiki/Schizoid_(video_game)). Two players control colored ships (red and blue) in an arena full of colored enemies. Touch a same-color enemy to destroy it; touch an opposite-color enemy and you die. Players must cooperate to cover each other.

**Goals:**
- Learn Rust through a real project
- Ship a fun, playable game with online co-op
- Play with a friend over the internet (~80ms latency)

**Stack:** Rust + Bevy 0.18 + lightyear 0.26

**Platforms:** Desktop (Linux + Windows)

## Core Mechanic

Every entity has a `TeamColor` (Red or Blue). The single rule:

- `ship.color == enemy.color` → enemy destroyed
- `ship.color != enemy.color` → ship destroyed

This creates forced cooperation: the red player must protect the blue player from red enemies, and vice versa.

## MVP Scope (v1)

### Included
- 1 arena (bounded rectangular playfield)
- 2 player ships (red + blue) with omnidirectional movement
- 3 enemy types (Drifter, Chaser, Orbiter)
- Wave-based spawning (endless, escalating difficulty)
- Online co-op (each player controls one ship)
- Death → 2s respawn with 1s invulnerability
- Neon/glow visual style (geometric shapes + bloom)
- Comprehensive test suite with `GameSim` DSL

### Excluded from v1 (future features)
- Power-ups (electric rope, shield, speed boost)
- Level/arena variety
- Solo "Uber" twin-stick mode (one player, both ships)
- Scoring/leaderboards
- Matchmaking/lobby system
- Scripting API for test scenarios

## Architecture

### Workspace Structure

```
schizoid/
├── Cargo.toml               # workspace root
├── crates/
│   ├── shared/              # Game logic, protocol, components
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── components.rs    # Ship, Enemy, TeamColor, Health
│   │   │   ├── protocol.rs      # lightyear protocol registration
│   │   │   ├── systems.rs       # movement, collision, spawning
│   │   │   └── enemies.rs       # enemy AI behaviors
│   │   └── Cargo.toml
│   ├── client/              # Rendering, input, prediction
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── rendering.rs     # bloom, neon effects, camera
│   │   │   ├── input.rs         # gamepad/keyboard mapping
│   │   │   └── ui.rs            # HUD, menus
│   │   └── Cargo.toml
│   ├── server/              # Authoritative simulation
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   └── server.rs        # replication, authority
│   │   └── Cargo.toml
│   └── test_harness/        # GameSim DSL + integration tests
│       ├── src/
│       │   ├── lib.rs           # GameSim struct
│       │   └── sim.rs           # helpers, assertions
│       ├── tests/
│       │   ├── movement.rs
│       │   ├── collision.rs
│       │   ├── enemies.rs
│       │   └── networking.rs
│       └── Cargo.toml
└── assets/                  # minimal (fonts, maybe sounds later)
```

### Crate Responsibilities

| Crate | Depends On | Responsibility |
|-------|-----------|----------------|
| `shared` | bevy, lightyear, serde | All game logic, components, protocol. Runs on both client and server. |
| `client` | shared, bevy (full) | Rendering, input handling, prediction, interpolation, UI |
| `server` | shared, bevy (minimal) | Authoritative simulation, replication, wave management |
| `test_harness` | shared, bevy (minimal), lightyear (test_utils) | GameSim DSL, integration tests, network simulation |

### Key Principle

All game logic lives in `shared`. The client and server are thin wrappers that configure lightyear roles and add rendering/input or headless operation respectively.

## Networking

### lightyear Configuration

| Setting | Value | Rationale |
|---------|-------|-----------|
| Tick rate | 60 Hz | Smooth action game feel |
| Replication interval | Every 2 ticks (30 Hz) | Bandwidth savings, interpolation covers gaps |
| Transport | UDP (native) | Lowest latency for desktop |
| Input buffer | 6 frames | Covers ~100ms jitter |

### Prediction & Authority Model

| Aspect | Owner | Notes |
|--------|-------|-------|
| Own ship movement | Client-predicted | Immediate response, server reconciles |
| Other player's ship | Interpolated | Smooth remote entity display |
| Enemy positions | Server-authoritative, interpolated on client | Server runs AI, clients interpolate |
| Collision outcomes | Server-authoritative | Who dies, who scores — server decides |
| Wave progression | Server-authoritative | When waves start/end |
| Game state | Server-authoritative | Playing, wave complete, game over |

### Connection Flow

1. Client connects to server `IP:port` (CLI argument or config)
2. Server spawns both ship entities, replicates to clients
3. Clients receive entity mapping, begin predicting their own ship
4. Server starts spawning enemies when both players are synced
5. No matchmaking — direct connect (you know your co-op partner)

### Data Flow

```
Client                          Server
  │                               │
  ├─ read gamepad/keyboard        │
  ├─ create ActionState<Input>    │
  ├─ predict own ship locally ──► receive inputs at correct tick
  │                               ├─ run authoritative simulation
  │                               ├─ detect collisions (lag-compensated)
  ◄── receive state updates ◄──── replicate components
  ├─ reconcile prediction         │
  ├─ interpolate other player     │
  └─ render (bloom, glow)         │
```

## Gameplay Systems

### Ship Movement
- Omnidirectional (left stick or WASD)
- Constant speed (tunable)
- Clamped to arena bounds (no wrapping)

### Enemy Types

| Type | Behavior | Difficulty | Shape |
|------|----------|------------|-------|
| Drifter | Constant velocity, bounces off walls | Easy | Circle |
| Chaser | Steers toward nearest opposite-color ship | Medium | Triangle |
| Orbiter | Circles a fixed point at set radius | Medium | Ring |

### Wave System
- Wave N spawns `3N + 2` enemies (5, 8, 11, 14...)
- Enemy mix shifts toward Chasers as waves increase
- Wave clears when all enemies destroyed
- 3-second breather between waves
- Endless mode — survive as long as possible

### Death & Respawn
- Ship destroyed → 2 second respawn timer
- Respawn at safe location (away from enemies)
- 1 second invulnerability on respawn (ship blinks)
- Both ships dead simultaneously → wave resets

### Collision Detection
- Circle-circle overlap (no physics engine needed)
- Ship radius and enemy radius are tunable constants
- Server performs authoritative collision with lag compensation

## Rendering

### Neon/Bloom Pipeline
- `Camera2d` with `BloomSettings` + HDR enabled
- Emissive colors (channel values > 1.0) trigger bloom glow
- Dark background (#0A0A0A)
- Thin white arena border

### Visual Design
- Ships: larger circles with bright glow
- Drifters: small glowing circles
- Chasers: small glowing triangles
- Orbiters: glowing rings
- Death: brief particle burst in entity's color
- Respawn: pulsing/blinking during invulnerability
- No sprite art — pure geometry + bloom

### UI (Minimal)
- Wave counter (top center)
- Connection status ("Connecting...", "Waiting for player")
- Death/respawn indicator per player

## Testing Architecture

### GameSim DSL

A Rust test harness wrapping lightyear's `ClientServerStepper` pattern:

```rust
#[test]
fn chaser_targets_opposite_color() {
    let mut sim = GameSim::new();
    let red = sim.spawn_player(TeamColor::Red, Vec2::ZERO);
    let blue = sim.spawn_player(TeamColor::Blue, Vec2::new(100.0, 0.0));
    let chaser = sim.spawn_enemy(EnemyType::Chaser, TeamColor::Red, Vec2::new(50.0, 0.0));

    sim.step(60); // 1 second at 60 tick

    // Red chaser should move toward Blue (opposite color)
    assert!(sim.distance(chaser, blue) < sim.distance(chaser, red));
}

#[test]
fn same_color_kill_over_network() {
    let mut sim = GameSim::networked(NetworkCondition::Good);
    sim.connect_players();
    let red = sim.spawn_player(TeamColor::Red, Vec2::ZERO);
    let enemy = sim.spawn_enemy(EnemyType::Drifter, TeamColor::Red, Vec2::new(5.0, 0.0));

    sim.move_player(red, Vec2::new(1.0, 0.0));
    sim.step(30);

    assert!(sim.is_dead(enemy));
    assert!(sim.is_alive(red));
}
```

### Test Modes

| Mode | What it tests | How |
|------|--------------|-----|
| `GameSim::new()` | Game logic only | Single `App`, `MinimalPlugins`, no networking |
| `GameSim::networked(condition)` | Full client-server | Separate `App`s, crossbeam channels, `LinkConditionerConfig` |
| `--features gui2d` | Visual debugging | Same scenarios rendered with `DefaultPlugins` + bloom |

### Network Condition Presets

| Preset | Latency | Jitter | Packet Loss |
|--------|---------|--------|-------------|
| Good | 40ms | 6ms | 0.2% |
| Average | 100ms | 15ms | 2% |
| Poor | 200ms | 30ms | 10% |

### What Gets Tested
- Movement and boundary clamping
- Color-matching collision (same = kill, opposite = death)
- Enemy AI behaviors (drift, chase, orbit)
- Wave spawning and progression
- Respawn mechanics and invulnerability
- Client prediction accuracy under various network conditions
- Rollback correctness after misprediction
- Simultaneous death handling

## Infrastructure

### Server Deployment (K8s)
- Server binary as a Kubernetes Deployment (1 replica)
- Exposed via NodePort or LoadBalancer on UDP port
- Headless (no rendering, minimal resources)
- Configuration via environment variables: `PORT`, `TICK_RATE`, `MAX_PLAYERS`

### Local Development

```bash
# Run all tests (headless, fast)
cargo test -p test_harness

# Visual debugging of a test scenario
cargo run -p test_harness --features gui2d --example chaser_behavior

# Local play (server + 2 clients)
cargo run -p server &
cargo run -p client -- --connect 127.0.0.1:5000 --player red
cargo run -p client -- --connect 127.0.0.1:5000 --player blue
```

### Cross-Compilation
- Linux: native build on NixOS desktop
- Windows: cross-compile with `cross` or `cargo-xwin`
- Distribute: single binary + assets folder

## Version Pinning

Pin to Bevy 0.18 + lightyear 0.26 throughout development. Do not chase upstream releases unless a specific feature is needed.

## Future Features (post-MVP)

1. **Electric rope power-up** — stretch a beam between ships that kills all colors
2. **More enemy types** — Splitter (divides on death), Shielder (immune front, vulnerable back)
3. **Level system** — different arena shapes, obstacle placement, enemy compositions
4. **Solo Uber mode** — one player controls both ships (one per analog stick)
5. **Scoring & leaderboards** — points per kill, wave survival records
6. **Lobby/matchmaking** — simple room system for finding games
7. **Scripting API** — Lua/Rhai for hot-reloadable scenarios and modding
