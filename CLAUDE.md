# Schizoid

Twin-stick co-op shooter built with Rust, Bevy 0.18, and lightyear 0.26.

## Development

Requires `nix develop` for system dependencies (wayland, vulkan, etc.).

```bash
nix develop           # Enter dev shell
task build            # Build all crates
task test             # Run all tests
task lint             # fmt + clippy
task ci               # Full CI check
task server           # Start server on port 5555
task client           # Connect to localhost:5555
```

## Verification Rules

**Runtime verification is mandatory.** Compilation and tests passing is NOT sufficient.

- After any change to networking, rendering, or input: **run the server and client binaries** and verify they work at runtime
- After any change to game logic: **run both the test suite AND the binaries**
- Never declare a feature "done" based only on `cargo check` / `cargo test` passing
- When dispatching workers for implementation, include runtime verification commands in their instructions

### Runtime Verification Checklist

1. `task server` starts without panic
2. `task client` opens a window without panic
3. Server logs show client connection
4. Ship appears on screen
5. WASD moves the ship
6. Enemies are visible and moving

## Architecture

```
crates/
  shared/         # Game logic, components, protocol (runs on client + server)
  client/         # Rendering (neon bloom), input, prediction
  server/         # Headless authoritative simulation
  test_harness/   # GameSim test DSL for headless testing
```

### Client-Side Prediction

The client runs the same game systems as the server in `FixedUpdate` so that player input is applied immediately (predicted). Lightyear handles rollback if the server disagrees.

**Both client and server must run:** `ship_movement`, `enemy_movement`, `chaser_ai`, `collision_system`, `respawn_system`.

### Lightyear Patterns

- Server entity needs `Start` trigger after spawning to begin listening
- Client entity needs `Connect` trigger after spawning to initiate connection
- Use `Authentication::Manual` with matching `private_key` and `protocol_id` on both sides
- `PostProcessPlugin` is included in Bevy 0.18 `DefaultPlugins` - don't add it separately
- **Replication timing:** Entity spawn (adds `Predicted`/`Interpolated`) happens in a first pass; component inserts (`Ship`, `Position`, etc.) happen in a second pass. Use tuple observers like `On<Add, (Ship, Predicted)>` — NOT `On<Add, Predicted>` with a query check for `Ship`, because `Ship` won't exist yet when `Predicted` fires.
- Gamepad input: iterate ALL gamepads and use the first with active stick input (non-gamepad HID devices like keyboards can register as gamepads via gilrs)

## Resume on Next Session

### Bot Demo Mode (2026-06-12)

Two-player game runs unattended: `task demo` (or `task demo DURATION=90`) starts
the server + two `--bot` clients. Bots are full network clients — same input
path as a human, just AI-fed. Screenshots land in `.tmp/demo-p*.png`, logs in
`.tmp/demo-*.log`. Verify visually by reading the PNGs.

**Client flags:** `--bot` (AI ship), `--screenshot <path>` + `--screenshot-at <s>`,
`--exit-after <s>` (unattended teardown).

**Bot AI** (`shared/src/bot.rs`): samples 16 directions, scores endpoint+midpoint
threat encroachment (quadratic) vs nearest-target distance, adaptive lookahead.
Futures are wall-clamped exactly like ship_movement so the bot can't corner itself.

### Lightyear Gotchas Learned This Session
- **Enemies were never replicated** until 2026-06-12. They now get
  `Replicate::to_clients(All)` + `PredictionTarget::to_clients(All)` in
  `wave_manager` (spawn_wave returns the entities; the server attaches networking).
- **Tuple observers fire per component:** `On<Add, (Ship, Predicted)>` fires when
  EITHER lands — predicted enemies and the interpolated remote ship triggered it
  too, giving them InputMarkers and breaking `single_mut()`. Guard the observer
  body with a query check for the full component set.
- **Never plain-`despawn()` predicted entities** in shared systems. Client-predicted
  kills the server didn't confirm left enemies alive server-side but invisible
  client-side — waves stalled forever. Use `prediction_despawn()`
  (lightyear::prelude::PredictionDespawnCommandsExt): on clients it disables +
  lets rollback revive; on the server it despawns for real.
- **Resources don't replicate.** Wave number rides on the replicated `WaveInfo`
  component (server spawns one entity at startup); `WaveState` stays server-local.
- Bevy's default font has no em-dash glyph — ASCII only in UI text.

### Controller Input — SHELVED (2026-06-12)
Xbox controller (xpadneo vs Steam Input hidraw conflict) parked by user decision.
Keyboard WASD still works. History in git: CLAUDE.md @ b71a6b1.

### Current State
- Two-client co-op (red + blue ships): **working** (bot-driven demo verified)
- Bot AI players: **working** (hunt same-color, flee opposite-color, kite chasers)
- Enemy replication + client prediction: **working**
- Color-matching collision: **working** (same = kill enemy, opposite = die)
- Death/respawn + invulnerability blink: implemented
- Wave counter UI: **working** (replicated WaveInfo)
- Tests: 17 passing (10 logic + 5 bot unit + 2 bot integration)

### What's Next (from design doc MVP scope)
- Verify wave progression across multiple waves in longer runs
- Death/respawn visuals polish (respawn at safe location, not center)
- Server verification API (user requested POST endpoint for programmatic testing)
- Chaser/Orbiter visual distinctiveness check (triangle/ring shapes at bloom)
- Revisit human input later (keyboard works; controller shelved)
