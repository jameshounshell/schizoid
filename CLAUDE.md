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

### Xbox Controller + Steam Input Conflict (2026-02-23)
**Problem:** xpadneo (proper analog) and Steam Input fight over the controller.
- `hid_microsoft` (no xpadneo): 8-direction digital sticks, but Steam works via hidraw
- `xpadneo`: smooth analog, but Steam also grabs hidraw — both try to own the controller
- EVIOCGRAB approach **failed**: it blocks gilrs (same process, different fd) AND doesn't affect Steam (which uses hidraw, not evdev)
- The evdev `event22` (xpadneo emulated Xbox 360) is what gilrs reads; `event17` is the raw HID device

**Next approach options (needs fresh thinking):**
1. **Simplest:** Disable "Enable Steam Input for Xbox controllers" in Steam settings — let xpadneo + gilrs handle it natively
2. **SDL2-based input** instead of gilrs — SDL2 handles Steam coexistence natively
3. **Add as non-Steam game** — Steam manages the controller handoff when launching through Steam
4. Test if disabling Steam Input for Xbox globally fixes both analog quality AND Steam interference

**Moonlander workaround (working):** udev rule strips `ID_INPUT_JOYSTICK` from ZSA Moonlander so gilrs/Bevy ignores it

### Current State
- WASD keyboard input: **working**
- Xbox controller input: **partially working** (detected by gilrs, left stick moves ship, but 8-way feel + Steam conflict)
- Client-server networking: **working** (UDP + netcode)
- Client-side prediction: **working** (shared systems in FixedUpdate)
- Bloom rendering: **working** (Bevy built-in post-processing)
- 10 game logic tests: **passing**

### What's Next (from design doc MVP scope)
- Second player support (two clients, red + blue ships)
- Color-matching collision (same color = kill enemy, opposite = kill ship)
- Death/respawn visuals (blinking during invulnerability)
- Wave counter UI
- Server verification API (user requested POST endpoint for programmatic testing)
