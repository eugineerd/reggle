use bevy::prelude::*;

#[derive(Default)]
pub struct InputState {
    pub cursor_position: Vec2,
    pub spawn_ball_action: bool,
    pub spawn_platform_action: bool,
}

pub struct InputStatePlugin;

impl Plugin for InputStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputState::default())
            .add_system_to_stage(CoreStage::PreUpdate, input_state_system);
    }
}

fn input_state_system(
    _keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut cursor_event_reader: EventReader<CursorMoved>,
    windows: Res<Windows>,
    mut input_state: ResMut<InputState>,
) {
    if let Some(e) = cursor_event_reader.iter().last() {
        if let Some(window) = windows.get(e.id) {
            let game_x = e.position.x - window.width() / 2.0;
            let game_y = e.position.y - window.height() / 2.0;
            input_state.cursor_position = Vec2::new(game_x, game_y)
        }
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        input_state.spawn_ball_action = true;
    } else {
        input_state.spawn_ball_action = false;
    }

    if mouse_buttons.just_pressed(MouseButton::Right) {
        input_state.spawn_platform_action = true;
    } else {
        input_state.spawn_platform_action = false;
    }
}
