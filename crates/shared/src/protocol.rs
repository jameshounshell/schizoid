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

        app.register_component::<TeamColor>().add_prediction();

        app.register_component::<Ship>().add_prediction();

        app.register_component::<OwnedBy>().add_prediction();

        app.register_component::<Radius>().add_prediction();

        app.register_component::<Position>().add_prediction();

        app.register_component::<Health>().add_prediction();

        app.register_component::<EnemyType>().add_prediction();

        app.register_component::<Velocity>().add_prediction();

        app.register_component::<OrbitData>().add_prediction();
    }
}
