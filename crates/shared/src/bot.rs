use bevy::prelude::*;

use crate::components::*;

/// Threats inside this radius of the travel path are penalized.
pub const BOT_DANGER_RADIUS: f32 = 200.0;
/// Quadratic threat penalty scale — tuned so a threat at point-blank
/// (~2000 penalty) always beats target attraction (hundreds).
pub const BOT_AVOID_K: f32 = 0.05;
/// How far ahead (in px) a candidate move is projected when scoring.
pub const BOT_LOOKAHEAD: f32 = 180.0;
/// Number of candidate directions sampled around the circle.
const BOT_SAMPLES: u32 = 16;

/// Pure steering function for an AI-controlled ship.
///
/// Samples candidate directions and scores each future position:
/// staying far from opposite-color enemies (lethal) along the whole
/// travel path, while closing distance to the nearest same-color enemy
/// (killable). Futures are clamped to arena bounds exactly like
/// ship_movement clamps, so wall-sliding is scored correctly and the
/// bot can't corner itself.
pub fn compute_bot_direction(
    ship_pos: Vec2,
    ship_color: TeamColor,
    enemies: &[(Vec2, TeamColor)],
    bounds: &ArenaBounds,
) -> Vec2 {
    if enemies.is_empty() {
        return Vec2::ZERO;
    }

    // Shrink lookahead as the nearest killable enemy gets close, otherwise
    // every candidate move overshoots the target and staying put scores
    // best — the bot stalls just outside collision range.
    let nearest_target_dist = enemies
        .iter()
        .filter(|(_, c)| *c == ship_color)
        .map(|(p, _)| p.distance(ship_pos))
        .min_by(|a, b| a.partial_cmp(b).unwrap());
    let lookahead = match nearest_target_dist {
        Some(d) => d.clamp(40.0, BOT_LOOKAHEAD),
        None => BOT_LOOKAHEAD,
    };

    let mut best_dir = Vec2::ZERO;
    let mut best_score = score_move(ship_pos, ship_pos, ship_color, enemies);

    for i in 0..BOT_SAMPLES {
        let angle = i as f32 / BOT_SAMPLES as f32 * std::f32::consts::TAU;
        let dir = Vec2::new(angle.cos(), angle.sin());
        let mut future = ship_pos + dir * lookahead;
        future.x = future.x.clamp(
            -bounds.half_width + SHIP_RADIUS,
            bounds.half_width - SHIP_RADIUS,
        );
        future.y = future.y.clamp(
            -bounds.half_height + SHIP_RADIUS,
            bounds.half_height - SHIP_RADIUS,
        );

        let score = score_move(ship_pos, future, ship_color, enemies);
        if score > best_score + 1e-3 {
            best_score = score;
            best_dir = dir;
        }
    }

    best_dir
}

/// Score a move from `from` to `to`: quadratically penalize lethal enemies
/// near the destination and the path midpoint, reward closing on the
/// nearest killable enemy. (The start point is deliberately NOT scored —
/// it's identical for every candidate, so including it flattens the
/// gradient and the bot freezes instead of fleeing.)
fn score_move(from: Vec2, to: Vec2, ship_color: TeamColor, enemies: &[(Vec2, TeamColor)]) -> f32 {
    let mut score = 0.0;
    let mid = (from + to) * 0.5;

    for (enemy_pos, enemy_color) in enemies {
        if *enemy_color != ship_color {
            let d_end = enemy_pos.distance(to);
            let enc_end = (BOT_DANGER_RADIUS - d_end).max(0.0);
            score -= enc_end * enc_end * BOT_AVOID_K;

            let d_mid = enemy_pos.distance(mid);
            let enc_mid = (BOT_DANGER_RADIUS - d_mid).max(0.0);
            score -= enc_mid * enc_mid * BOT_AVOID_K * 0.5;
        }
    }

    let nearest_target = enemies
        .iter()
        .filter(|(_, c)| *c == ship_color)
        .map(|(p, _)| p.distance(to))
        .min_by(|a, b| a.partial_cmp(b).unwrap());
    if let Some(d) = nearest_target {
        score -= d;
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> ArenaBounds {
        ArenaBounds::default()
    }

    #[test]
    fn seeks_same_color_enemy() {
        let dir = compute_bot_direction(
            Vec2::ZERO,
            TeamColor::Red,
            &[(Vec2::new(100.0, 0.0), TeamColor::Red)],
            &bounds(),
        );
        assert!(dir.x > 0.9, "should move toward red enemy, got {dir:?}");
    }

    #[test]
    fn flees_opposite_color_enemy() {
        let dir = compute_bot_direction(
            Vec2::ZERO,
            TeamColor::Red,
            &[(Vec2::new(50.0, 0.0), TeamColor::Blue)],
            &bounds(),
        );
        assert!(dir.x < -0.5, "should flee blue enemy, got {dir:?}");
    }

    #[test]
    fn avoidance_beats_attraction() {
        // Same-color target directly behind a close lethal enemy: the bot
        // must not path straight through the blue one.
        let dir = compute_bot_direction(
            Vec2::ZERO,
            TeamColor::Red,
            &[
                (Vec2::new(40.0, 0.0), TeamColor::Blue),
                (Vec2::new(120.0, 0.0), TeamColor::Red),
            ],
            &bounds(),
        );
        assert!(
            dir.x < 0.5,
            "should not charge through the blue enemy, got {dir:?}"
        );
    }

    #[test]
    fn escapes_a_corner() {
        let b = bounds();
        // Pinned in the top-right corner with a threat approaching the
        // diagonal escape route — bot must pick a wall-sliding direction.
        let corner = Vec2::new(b.half_width - 30.0, b.half_height - 30.0);
        let threat = corner + Vec2::new(-100.0, -100.0);
        let dir = compute_bot_direction(corner, TeamColor::Red, &[(threat, TeamColor::Blue)], &b);
        assert!(
            dir != Vec2::ZERO,
            "cornered bot must keep moving, got {dir:?}"
        );
    }

    #[test]
    fn idles_with_no_enemies() {
        let dir = compute_bot_direction(Vec2::ZERO, TeamColor::Red, &[], &bounds());
        assert_eq!(dir, Vec2::ZERO);
    }
}
