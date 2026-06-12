use bevy::prelude::*;
use schizoid_shared::bot::compute_bot_direction;
use schizoid_shared::components::*;
use schizoid_test_harness::sim::GameSim;

/// Drive a ship with the bot each tick, mirroring what the client does
/// in buffer_input.
fn bot_tick(sim: &mut GameSim, ship: Entity, color: TeamColor, enemies: &[Entity]) {
    let ship_pos = sim.position(ship);
    let enemy_list: Vec<(Vec2, TeamColor)> = enemies
        .iter()
        .filter(|e| sim.entity_exists(**e))
        .map(|e| {
            let pos = sim.position(*e);
            let c = sim.team_color(*e);
            (pos, c)
        })
        .collect();
    let dir = compute_bot_direction(ship_pos, color, &enemy_list, &ArenaBounds::default());
    sim.set_input(ship, dir);
}

#[test]
fn bot_hunts_down_same_color_enemy() {
    let mut sim = GameSim::new();
    let red = sim.spawn_player(TeamColor::Red, Vec2::ZERO);
    // Stationary same-color drifter across the arena
    let prey = sim.spawn_drifter(TeamColor::Red, Vec2::new(300.0, 100.0), Vec2::ZERO);

    for _ in 0..600 {
        if !sim.entity_exists(prey) {
            break;
        }
        bot_tick(&mut sim, red, TeamColor::Red, &[prey]);
        sim.step(1);
    }

    assert!(
        !sim.entity_exists(prey),
        "bot should have killed the same-color enemy within 10s"
    );
    assert!(sim.is_alive(red), "bot should survive the hunt");
}

#[test]
fn bot_survives_opposite_color_chaser() {
    let mut sim = GameSim::new();
    let red = sim.spawn_player(TeamColor::Red, Vec2::ZERO);
    // Blue chaser hunts the red ship; bot must kite it. Ship speed (300)
    // outruns chaser speed (150), so a working bot never gets caught.
    let threat = sim.spawn_chaser(TeamColor::Blue, Vec2::new(250.0, 0.0));

    for _ in 0..600 {
        bot_tick(&mut sim, red, TeamColor::Red, &[threat]);
        sim.step(1);
        assert!(
            sim.is_alive(red),
            "bot died to a slower opposite-color chaser"
        );
    }
}
