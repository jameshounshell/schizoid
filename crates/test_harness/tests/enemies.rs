use bevy::math::Vec2;
use schizoid_shared::components::*;
use schizoid_test_harness::sim::GameSim;

#[test]
fn drifter_bounces_off_walls() {
    let mut sim = GameSim::new();
    // Place drifter near right wall moving right
    let drifter = sim.spawn_drifter(
        TeamColor::Red,
        Vec2::new(580.0, 0.0),
        Vec2::new(DRIFTER_SPEED, 0.0),
    );

    sim.step(30); // Should hit wall and bounce

    let vel = sim.velocity(drifter);
    assert!(
        vel.x < 0.0,
        "Drifter should have bounced, velocity x should be negative: {:?}",
        vel
    );
}

#[test]
fn chaser_moves_toward_opposite_ship() {
    let mut sim = GameSim::new();
    let blue_ship = sim.spawn_player(TeamColor::Blue, Vec2::new(200.0, 0.0));
    let red_chaser = sim.spawn_chaser(TeamColor::Red, Vec2::ZERO);

    let initial_dist = sim.distance(red_chaser, blue_ship);
    sim.step(60);
    let final_dist = sim.distance(red_chaser, blue_ship);

    assert!(
        final_dist < initial_dist,
        "Red chaser should move toward blue ship. Initial: {}, Final: {}",
        initial_dist,
        final_dist
    );
}

#[test]
fn chaser_ignores_same_color_ship() {
    let mut sim = GameSim::new();
    let red_ship = sim.spawn_player(TeamColor::Red, Vec2::new(100.0, 0.0));
    let blue_ship = sim.spawn_player(TeamColor::Blue, Vec2::new(-100.0, 0.0));
    let red_chaser = sim.spawn_chaser(TeamColor::Red, Vec2::ZERO);

    sim.step(60);

    // Chaser should move toward blue (opposite color), not red (same color)
    let dist_to_blue = sim.distance(red_chaser, blue_ship);
    let dist_to_red = sim.distance(red_chaser, red_ship);

    assert!(
        dist_to_blue < dist_to_red,
        "Red chaser should be closer to blue ship. To blue: {}, To red: {}",
        dist_to_blue,
        dist_to_red
    );
}

#[test]
fn orbiter_stays_near_center() {
    let mut sim = GameSim::new();
    let center = Vec2::new(100.0, 100.0);
    let orbiter = sim.spawn_orbiter(TeamColor::Blue, center, 80.0);

    sim.step(120); // 2 seconds

    let pos = sim.position(orbiter);
    let dist_from_center = pos.distance(center);

    assert!(
        (dist_from_center - 80.0).abs() < 5.0,
        "Orbiter should stay ~80 units from center. Actual: {}",
        dist_from_center
    );
}
