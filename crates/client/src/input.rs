use bevy::prelude::*;
use lightyear::prelude::client::input::InputSystems;
use lightyear::prelude::input::native::{ActionState, InputMarker};
use lightyear::prelude::Predicted;
use schizoid_shared::components::Ship;
use schizoid_shared::protocol::PlayerInput;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            buffer_input.in_set(InputSystems::WriteClientInputs),
        );
        app.add_observer(handle_predicted_spawn);
    }
}

fn buffer_input(
    mut query: Query<&mut ActionState<PlayerInput>, With<InputMarker<PlayerInput>>>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
) {
    if let Ok(mut action_state) = query.single_mut() {
        let mut dir = Vec2::ZERO;

        // Keyboard WASD + arrows
        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
            dir.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
            dir.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            dir.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            dir.x += 1.0;
        }

        // Gamepad left stick — use whichever gamepad has actual stick input
        for gamepad in gamepads.iter() {
            let stick = gamepad.left_stick();
            if stick.length_squared() > 0.04 {
                dir = stick;
                break;
            }
        }

        // Normalize to prevent diagonal speed boost
        if dir.length_squared() > 1.0 {
            dir = dir.normalize();
        }

        action_state.0 = PlayerInput { direction: dir };
    }
}

fn handle_predicted_spawn(trigger: On<Add, (Ship, Predicted)>, mut commands: Commands) {
    let entity = trigger.entity;
    commands
        .entity(entity)
        .insert(InputMarker::<PlayerInput>::default());
    info!("Predicted ship spawned, added input marker to {:?}", entity);
}
