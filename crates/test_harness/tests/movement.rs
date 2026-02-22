use bevy::math::Vec2;
use schizoid_shared::components::*;
use schizoid_test_harness::sim::GameSim;

#[test]
fn ship_moves_right() {
    let mut sim = GameSim::new();
    let ship = sim.spawn_player(TeamColor::Red, Vec2::ZERO);

    sim.set_input(ship, Vec2::new(1.0, 0.0));
    sim.step(60); // ~1 second

    let pos = sim.position(ship);
    assert!(pos.x > 0.0, "Ship should have moved right, got {:?}", pos);
    assert!(pos.y.abs() < 0.1, "Ship should not have moved vertically");
}

#[test]
fn ship_stays_in_bounds() {
    let mut sim = GameSim::new();
    let ship = sim.spawn_player(TeamColor::Blue, Vec2::new(590.0, 0.0));

    sim.set_input(ship, Vec2::new(1.0, 0.0)); // move right toward boundary
    sim.step(120); // 2 seconds

    let pos = sim.position(ship);
    assert!(pos.x < 600.0, "Ship should be clamped to arena bounds");
}

#[test]
fn ship_diagonal_normalized() {
    let mut sim = GameSim::new();
    let ship = sim.spawn_player(TeamColor::Red, Vec2::ZERO);

    sim.set_input(ship, Vec2::new(1.0, 1.0)); // diagonal
    sim.step(60);

    let pos = sim.position(ship);
    // Diagonal movement should be normalized — should go ~same total distance
    let dist = pos.length();
    // At 300 units/sec for 1 sec, should be ~300 units total
    assert!(
        dist > 200.0 && dist < 400.0,
        "Diagonal should be normalized, got dist {}",
        dist
    );
}
