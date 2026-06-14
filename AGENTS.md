# AGENTS.md — Schizoid

Orientation for an agent picking up this project. Read this first, then
`CLAUDE.md` (deeper architecture notes + session resume log).

## What this is

A twin-stick co-op shooter. Two colored ships (red + blue) in an arena of
colored enemies. **The one rule:** touch a same-color enemy → it dies; touch
an opposite-color enemy → *you* die. Forced cooperation — each player must
clear the other's lethal enemies.

**Stack:** Rust, Bevy 0.18, lightyear 0.26 (client-server netcode, client-side
prediction + rollback). Pinned versions — do **not** chase upstream.

## Build & run — ALWAYS inside `nix develop`

Every cargo/task invocation needs the dev shell for system deps (wayland,
vulkan, etc.). The `task` targets that touch graphics already wrap `nix
develop -c`; raw `cargo` commands do not, so prefix them.

```bash
nix develop                  # enter dev shell (or prefix individual commands)
task build                   # cargo build --workspace
task test                    # cargo test --workspace  (18 tests)
task lint                    # fmt-check + clippy -D warnings
task ci                      # fmt-check + clippy + test
task demo                    # ⭐ server + two AI-bot clients, unattended
task demo DURATION=180       #    longer soak; teardown is automatic
task stop                    # kill any stray server/client processes
```

### The demo is how you see the game

There is **no human-input path you need** to verify gameplay. `task demo`
launches the server plus two `--bot` clients that play themselves. It writes:

- `.tmp/demo-server.log`, `.tmp/demo-client{1,2}.log` — logs
- `.tmp/demo-p1.png`, `.tmp/demo-p2.png` — screenshots (one per client)

Verify a run by grepping the logs and **reading the PNGs**:

```bash
grep -E "Starting wave|cleared" .tmp/demo-server.log     # wave progression
grep -c "cannot find entity" .tmp/demo-client*.log       # must be 0
grep -ci panic .tmp/demo-*.log                           # must be 0
```

A healthy 180s run clears ~9 waves with **zero** "cannot find entity" errors
and zero panics.

### Client flags (for unattended verification)

| Flag | Effect |
|------|--------|
| `--bot` | Drive the ship with the built-in AI instead of keyboard/gamepad |
| `--screenshot <path>` | Save a PNG to `<path>` |
| `--screenshot-at <s>` | Seconds to wait before the screenshot (default 8) |
| `--exit-after <s>` | Exit the process automatically (unattended teardown) |
| `--connect <ip>` / `--port <n>` | Server address (default `127.0.0.1:5555`) |

## ⚠️ Verification is mandatory — runtime, not just `cargo test`

Compilation + tests passing is **not** sufficient. This is the project's
hard rule (see `CLAUDE.md` → Verification Rules). After ANY change to
networking, rendering, input, or game logic:

1. `task test` (all 18 pass), AND
2. `task demo` — confirm in the logs + screenshots that ships spawn, enemies
   appear, waves progress, no replication errors, no panics.

Most bugs in this codebase only surface at runtime over the network (see the
lightyear gotchas below) — the test harness runs game logic headless without
the replication layer, so it cannot catch them.

## Architecture

```
crates/
  shared/         game logic + components + protocol — runs on client AND server
    components.rs    Ship, Enemy, TeamColor, Health, Position, WaveInfo, constants
    systems.rs       ship_movement, enemy_movement, chaser_ai, collision, respawn
    enemies.rs       spawn_wave (returns entities so caller attaches networking)
    bot.rs           AI steering (pure fn, unit-tested)
    protocol.rs      lightyear component registration
  client/         rendering (neon bloom), input/bot, prediction, screenshots
  server/         headless authoritative sim, wave manager, connection handling
  test_harness/   GameSim DSL — headless game-logic tests (no networking)
```

**Key principle:** all game logic lives in `shared`. Client and server are
thin wrappers. Both run the same systems in `FixedUpdate`
(`ship_movement`, `enemy_movement`, `chaser_ai`, `collision_system`,
`respawn_system`) so the client predicts and lightyear rolls back on
disagreement.

## lightyear 0.26 gotchas — learned the hard way, do not re-break

These cost real debugging time. They are load-bearing.

- **Replicate what clients must see.** Enemies were originally never
  replicated (`spawn_wave` added no `Replicate`). The server now attaches
  `Replicate::to_clients(All)` + `PredictionTarget::to_clients(All)` to each
  spawned enemy in `wave_manager`. If you add a new entity type clients render,
  it needs `Replicate`.
- **Never plain-`despawn()` a predicted entity in shared systems.** Use
  `commands.entity(e).prediction_despawn()` (from
  `lightyear::prelude::PredictionDespawnCommandsExt`). A client-predicted
  `despawn()` the server hasn't confirmed leaves the entity alive server-side
  but gone client-side → enemies invisible, waves stall forever. On the server
  `prediction_despawn` despawns for real; on the client it disables + lets
  rollback revive. This is in `collision_system`.
- **Tuple `On<Add, (A, B)>` observers fire per-component, not on the pair.**
  `handle_predicted_spawn` watches `(Ship, Predicted)` and fires when *either*
  lands — predicted enemies and the interpolated remote ship triggered it too.
  Guard the body with a query for the full component set before acting (it's
  why own-ship input markers don't leak onto enemies).
- **Resources don't replicate.** Wave number rides on a replicated `WaveInfo`
  *component* (server spawns one entity at startup). `WaveState` is a
  server-local resource for sim bookkeeping only.
- **Single-entity prediction model.** In 0.26 the `Predicted` marker is added
  directly to the replicated entity (no separate confirmed/predicted entity
  pair). Observe `On<Add, (Ship, Predicted)>`, not a Confirmed→Predicted hop.
- **Bevy default font has no em-dash glyph** — use ASCII `-` in UI text or it
  renders as tofu.

## The bot AI (`shared/src/bot.rs`)

`compute_bot_direction(ship_pos, ship_color, &enemies, &bounds) -> Vec2` is a
**pure function** (easy to unit-test, no ECS). It samples 16 directions,
projects a wall-clamped future for each, and scores: quadratic penalty for
opposite-color (lethal) enemies near the endpoint + path midpoint, reward for
closing on the nearest same-color (killable) enemy. Lookahead shrinks as the
nearest target gets close so the bot doesn't overshoot and stall.

The client's `buffer_input` calls it each tick when `--bot` is set, feeding the
result into `PlayerInput.direction` — the *same* input path a human uses.

## Conventions

- **Personal repo:** commit to `main` directly, **commit + push after every
  coherent change** (don't batch, don't wait to be asked).
- **Conventional commits** (`feat:`, `fix:`, `docs:`…). Explain *why*, note what
  was observed working at runtime.
- **Pre-commit hook** runs `cargo fmt` + `cargo clippy`. The fmt hook reformats
  and aborts the commit if files weren't already formatted — run
  `nix develop -c cargo fmt --all` *before* `git add`, or expect to re-commit.
  If the hook errors with a missing nix-store path (GC'd), run
  `pre-commit install -f` to regenerate it.
- **Preserve inline comments** — they carry cross-session context. The lightyear
  gotchas above are documented inline at their call sites; keep them.

## Current state (2026-06-12)

Working and verified at runtime:

- Two-client co-op (red + blue), bot-driven demo
- Bot AI players (hunt same-color, flee opposite-color, kite chasers, safe respawn)
- Enemy replication + client-side prediction
- Color-matching collision, death/respawn + invulnerability blink
- Wave counter HUD (replicated `WaveInfo`), 3 enemy types (Drifter/Chaser/Orbiter)
- 18 tests passing (10 game-logic + 5 bot unit + 2 bot integration + 1 respawn)

**Controller input is SHELVED** by user decision — xpadneo vs Steam Input fight
over the controller's hidraw. Keyboard WASD works. Do not reopen this unless
asked; history is in git (`CLAUDE.md` @ commit `b71a6b1`).

## Where to pick up (from the design doc MVP scope)

- **Server verification API** — user wants a POST endpoint for programmatic
  testing of game state (highest-interest next item)
- Wave reset when both ships die simultaneously
- Respawn-visual polish (currently respawns at the safest grid point, not center)
- Verify wave progression holds up in very long runs (10+ waves)
- Human input revisit (keyboard works today; controller shelved)

Excluded from MVP: power-ups, arena variety, solo "Uber" mode, scoring,
matchmaking. See `docs/plans/2026-02-22-game-design.md` for the full design.
