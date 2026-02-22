use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum TeamColor {
    Red,
    Blue,
}

impl TeamColor {
    pub fn opposite(self) -> Self {
        match self {
            TeamColor::Red => TeamColor::Blue,
            TeamColor::Blue => TeamColor::Red,
        }
    }

    pub fn emissive(self) -> Color {
        match self {
            TeamColor::Red => Color::srgb(5.0, 0.2, 0.2),
            TeamColor::Blue => Color::srgb(0.2, 0.5, 5.0),
        }
    }

    pub fn emissive_dim(self) -> Color {
        match self {
            TeamColor::Red => Color::srgb(3.0, 0.1, 0.1),
            TeamColor::Blue => Color::srgb(0.1, 0.3, 3.0),
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct Ship;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect, Deref, DerefMut)]
pub struct Position(pub Vec2);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct Radius(pub f32);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct Health {
    pub alive: bool,
    pub respawn_timer: f32,
    pub invulnerable_timer: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            alive: true,
            respawn_timer: 0.0,
            invulnerable_timer: 0.0,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum EnemyType {
    Drifter,
    Chaser,
    Orbiter,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct OrbitData {
    pub center: Vec2,
    pub radius: f32,
    pub angle: f32,
    pub speed: f32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct OwnedBy(pub u64);

#[derive(Resource, Clone, Debug, Default)]
pub struct WaveState {
    pub current_wave: u32,
    pub enemies_remaining: u32,
    pub breather_timer: f32,
    pub active: bool,
}

#[derive(Resource, Clone, Debug)]
pub struct ArenaBounds {
    pub half_width: f32,
    pub half_height: f32,
}

impl Default for ArenaBounds {
    fn default() -> Self {
        Self {
            half_width: 600.0,
            half_height: 400.0,
        }
    }
}

pub const SHIP_SPEED: f32 = 300.0;
pub const SHIP_RADIUS: f32 = 15.0;
pub const ENEMY_RADIUS: f32 = 10.0;
pub const DRIFTER_SPEED: f32 = 120.0;
pub const CHASER_SPEED: f32 = 150.0;
pub const ORBITER_SPEED: f32 = 2.0;
pub const ORBITER_RADIUS: f32 = 80.0;
pub const RESPAWN_TIME: f32 = 2.0;
pub const INVULNERABLE_TIME: f32 = 1.0;
pub const WAVE_BREATHER: f32 = 3.0;
