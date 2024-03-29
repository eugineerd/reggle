use crate::common::MainCamera;
use bevy::{prelude::*, utils::HashSet};

pub struct GameInputPlugin;

impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameInput::default())
            .add_systems(PreUpdate, input_state_system);
    }
}

#[derive(Default, Resource)]
pub struct GameInput {
    pub cursor_position: Vec2,
    lock_input: bool,
    active_actions: HashSet<GameAction>,
    just_active_actions: HashSet<GameAction>,
}

impl GameInput {
    pub fn active(&self, action: GameAction) -> bool {
        self.active_actions.contains(&action)
    }

    pub fn just_active(&self, action: GameAction) -> bool {
        self.just_active_actions.contains(&action)
    }
}

// Should this be in common?
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum GameAction {
    Shoot,
    MoveLauncher,
}

fn input_state_system(
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut cursor_event_reader: EventReader<CursorMoved>,
    main_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut input_state: ResMut<GameInput>,
) {
    input_state.active_actions.clear();
    input_state.just_active_actions.clear();

    if let Some(e) = cursor_event_reader.iter().last() {
        if let Ok((camera, cam_tr)) = main_camera.get_single() {
            if let Some(pos) = camera.viewport_to_world_2d(cam_tr, e.position) {
                input_state.cursor_position = pos;
            };
        }
    }

    if keys.just_pressed(KeyCode::Space) {
        input_state.lock_input = !input_state.lock_input
    }

    if input_state.lock_input {
        return;
    }

    if keys.pressed(KeyCode::ControlLeft) {
        input_state.active_actions.insert(GameAction::MoveLauncher);
    }
    if keys.just_pressed(KeyCode::ControlLeft) {
        input_state
            .just_active_actions
            .insert(GameAction::MoveLauncher);
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        input_state.active_actions.insert(GameAction::Shoot);
    }
    if mouse_buttons.just_pressed(MouseButton::Left) {
        input_state.just_active_actions.insert(GameAction::Shoot);
    }
}
