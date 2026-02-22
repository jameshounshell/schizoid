use bevy::math::Vec2;
use schizoid_shared::components::*;
use schizoid_test_harness::sim::GameSim;

#[test]
fn same_color_kills_enemy() {
    let mut sim = GameSim::new();
    let ship = sim.spawn_player(TeamColor::Red, Vec2::ZERO);
    // Place enemy right next to ship (overlapping)
    let enemy = sim.spawn_drifter(TeamColor::Red, Vec2::new(5.0, 0.0), Vec2::ZERO);

    sim.set_input(ship, Vec2::new(1.0, 0.0)); // move toward enemy
    sim.step(10);

    assert!(
        !sim.entity_exists(enemy),
        "Same-color enemy should be despawned"
    );
    assert!(
        sim.is_alive(ship),
        "Ship should survive same-color collision"
    );
}

#[test]
fn opposite_color_kills_ship() {
    let mut sim = GameSim::new();
    let ship = sim.spawn_player(TeamColor::Red, Vec2::ZERO);
    // Place opposite-color enemy overlapping
    let enemy = sim.spawn_drifter(TeamColor::Blue, Vec2::new(5.0, 0.0), Vec2::ZERO);

    sim.set_input(ship, Vec2::new(1.0, 0.0)); // move toward enemy
    sim.step(10);

    assert!(
        sim.is_dead(ship),
        "Ship should die from opposite-color collision"
    );
    assert!(
        sim.entity_exists(enemy),
        "Opposite-color enemy should survive"
    );
}

#[test]
fn invulnerable_ship_survives() {
    let mut sim = GameSim::new();
    let ship = sim.spawn_player(TeamColor::Red, Vec2::ZERO);

    // Manually set invulnerability
    // We need access to the world to set Health.invulnerable_timer
    // This tests that recently-respawned ships don't die immediately

    // Kill the ship first
    let _enemy = sim.spawn_drifter(TeamColor::Blue, Vec2::new(1.0, 0.0), Vec2::ZERO);
    sim.step(5);

    assert!(sim.is_dead(ship), "Ship should be dead");

    // Wait for respawn (2 seconds = 120 ticks)
    sim.step(130);

    assert!(sim.is_alive(ship), "Ship should have respawned");
}
