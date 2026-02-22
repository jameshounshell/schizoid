use bevy::prelude::*;
use lightyear::prelude::input::native::ActionState;

use crate::components::*;
use crate::protocol::PlayerInput;

pub fn ship_movement(
    mut query: Query<(&mut Position, &ActionState<PlayerInput>), With<Ship>>,
    bounds: Res<ArenaBounds>,
) {
    for (mut pos, input) in query.iter_mut() {
        let dir = input.0.direction;
        if dir.length_squared() > 0.0 {
            let movement = dir.normalize_or_zero() * SHIP_SPEED * (1.0 / 60.0);
            pos.0 += movement;
            pos.0.x = pos.0.x.clamp(
                -bounds.half_width + SHIP_RADIUS,
                bounds.half_width - SHIP_RADIUS,
            );
            pos.0.y = pos.0.y.clamp(
                -bounds.half_height + SHIP_RADIUS,
                bounds.half_height - SHIP_RADIUS,
            );
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn enemy_movement(
    mut drifters: Query<
        (&mut Position, &mut Velocity),
        (With<EnemyType>, Without<OrbitData>, Without<Ship>),
    >,
    mut orbiters: Query<(&mut Position, &mut OrbitData), (With<EnemyType>, Without<Ship>)>,
    bounds: Res<ArenaBounds>,
) {
    let dt = 1.0 / 60.0;

    for (mut pos, mut vel) in drifters.iter_mut() {
        pos.0 += vel.0 * dt;

        if pos.0.x.abs() > bounds.half_width - ENEMY_RADIUS {
            vel.0.x = -vel.0.x;
            pos.0.x = pos.0.x.clamp(
                -bounds.half_width + ENEMY_RADIUS,
                bounds.half_width - ENEMY_RADIUS,
            );
        }
        if pos.0.y.abs() > bounds.half_height - ENEMY_RADIUS {
            vel.0.y = -vel.0.y;
            pos.0.y = pos.0.y.clamp(
                -bounds.half_height + ENEMY_RADIUS,
                bounds.half_height - ENEMY_RADIUS,
            );
        }
    }

    for (mut pos, mut orbit) in orbiters.iter_mut() {
        orbit.angle += orbit.speed * dt;
        pos.0 = orbit.center + Vec2::new(orbit.angle.cos(), orbit.angle.sin()) * orbit.radius;
    }
}

pub fn chaser_ai(
    mut chasers: Query<(&mut Velocity, &Position, &TeamColor, &EnemyType), Without<Ship>>,
    ships: Query<(&Position, &TeamColor, &Health), With<Ship>>,
) {
    for (mut vel, chaser_pos, chaser_color, enemy_type) in chasers.iter_mut() {
        if *enemy_type != EnemyType::Chaser {
            continue;
        }

        let target = ships
            .iter()
            .filter(|(_, color, health)| **color != *chaser_color && health.alive)
            .min_by(|(pos_a, _, _), (pos_b, _, _)| {
                let dist_a = pos_a.0.distance_squared(chaser_pos.0);
                let dist_b = pos_b.0.distance_squared(chaser_pos.0);
                dist_a.partial_cmp(&dist_b).unwrap()
            });

        if let Some((target_pos, _, _)) = target {
            let dir = (target_pos.0 - chaser_pos.0).normalize_or_zero();
            vel.0 = dir * CHASER_SPEED;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn collision_system(
    mut ships: Query<(&Position, &TeamColor, &mut Health, &Radius), With<Ship>>,
    enemies: Query<(Entity, &Position, &TeamColor, &Radius), (With<EnemyType>, Without<Ship>)>,
    mut commands: Commands,
) {
    for (ship_pos, ship_color, mut ship_health, ship_radius) in ships.iter_mut() {
        if !ship_health.alive {
            continue;
        }

        for (enemy_entity, enemy_pos, enemy_color, enemy_radius) in enemies.iter() {
            let dist = ship_pos.0.distance(enemy_pos.0);
            let min_dist = ship_radius.0 + enemy_radius.0;

            if dist < min_dist {
                if *ship_color == *enemy_color {
                    commands.entity(enemy_entity).despawn();
                } else if ship_health.invulnerable_timer <= 0.0 {
                    ship_health.alive = false;
                    ship_health.respawn_timer = RESPAWN_TIME;
                }
            }
        }
    }
}

pub fn respawn_system(mut ships: Query<(&mut Health, &mut Position), With<Ship>>) {
    let dt = 1.0 / 60.0;

    for (mut health, mut pos) in ships.iter_mut() {
        if !health.alive {
            health.respawn_timer -= dt;
            if health.respawn_timer <= 0.0 {
                health.alive = true;
                health.invulnerable_timer = INVULNERABLE_TIME;
                pos.0 = Vec2::ZERO;
            }
        }

        if health.invulnerable_timer > 0.0 {
            health.invulnerable_timer -= dt;
        }
    }
}
