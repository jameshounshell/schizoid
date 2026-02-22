# Schizoid

A twin-stick cooperative action game inspired by [Schizoid](https://en.wikipedia.org/wiki/Schizoid_(video_game)). Two players control colored ships (red and blue) in an arena full of colored enemies. Touch a same-color enemy to destroy it. Touch an opposite-color enemy and you die.

Built with Rust, [Bevy](https://bevyengine.org/) 0.18, and [lightyear](https://github.com/cBournhonesque/lightyear) 0.26 for online co-op.

## Development

Requires [Nix](https://nixos.org/download/) with flakes enabled.

```bash
# Enter dev shell (provides all system dependencies)
nix develop

# Build everything
task build

# Run all tests
task test

# Run lints (fmt + clippy)
task lint

# Full CI check
task ci
```

## Running

```bash
# Start the server
task server

# Start a client (connects to localhost:5555 by default)
task client

# Connect to a remote server
task client -- --connect 192.168.1.100 --port 5555
```

## Project Structure

```
crates/
  shared/         # Game logic, components, protocol (runs on client + server)
  client/         # Rendering (neon bloom), input, prediction
  server/         # Headless authoritative simulation
  test_harness/   # GameSim test DSL for headless testing
```

## Design

See [docs/plans/](docs/plans/) for the game design and implementation plan.
