use bevy::prelude::*;
use rand::Rng;

use crate::components::*;

pub fn spawn_wave(commands: &mut Commands, wave: u32, bounds: &ArenaBounds) {
    let count = (wave * 3 + 2) as usize;
    let mut rng = rand::rng();

    for _ in 0..count {
        let color = if rng.random_bool(0.5) {
            TeamColor::Red
        } else {
            TeamColor::Blue
        };

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
        0 => Vec2::new(
            rng.random_range(-bounds.half_width..bounds.half_width),
            bounds.half_height - 20.0,
        ),
        1 => Vec2::new(
            rng.random_range(-bounds.half_width..bounds.half_width),
            -bounds.half_height + 20.0,
        ),
        2 => Vec2::new(
            bounds.half_width - 20.0,
            rng.random_range(-bounds.half_height..bounds.half_height),
        ),
        _ => Vec2::new(
            -bounds.half_width + 20.0,
            rng.random_range(-bounds.half_height..bounds.half_height),
        ),
    }
}
