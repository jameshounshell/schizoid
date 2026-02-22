use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::*;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Reflect)]
pub struct PlayerInput {
    pub direction: Vec2,
}

impl MapEntities for PlayerInput {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(lightyear::prelude::input::native::InputPlugin::<PlayerInput>::default());

        app.register_component::<TeamColor>();
        app.register_component::<Ship>();
        app.register_component::<OwnedBy>();
        app.register_component::<Radius>();
        app.register_component::<Position>();
        app.register_component::<Health>();
        app.register_component::<EnemyType>();
        app.register_component::<Velocity>();
        app.register_component::<OrbitData>();
    }
}
