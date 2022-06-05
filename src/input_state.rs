use bevy::{prelude::*, utils::HashSet};

pub struct InputStatePlugin;

impl Plugin for InputStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputState::default())
            .add_system_to_stage(CoreStage::PreUpdate, input_state_system);
    }
}

#[derive(Default)]
pub struct InputState {
    pub cursor_position: Vec2,
    lock_input: bool,
    active_actions: HashSet<GameAction>,
    just_active_actions: HashSet<GameAction>,
}

impl InputState {
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
    SpawnPegs,
    MoveLauncher,
}

fn input_state_system(
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut cursor_event_reader: EventReader<CursorMoved>,
    windows: Res<Windows>,
    mut input_state: ResMut<InputState>,
) {
    input_state.active_actions.clear();
    input_state.just_active_actions.clear();

    if let Some(e) = cursor_event_reader.iter().last() {
        if let Some(window) = windows.get(e.id) {
            let game_x = e.position.x - window.width() / 2.0;
            let game_y = e.position.y - window.height() / 2.0;
            input_state.cursor_position = Vec2::new(game_x, game_y)
        }
    }

    if keys.just_pressed(KeyCode::Space) {
        input_state.lock_input = !input_state.lock_input
    }

    if input_state.lock_input {
        return;
    }

    if keys.pressed(KeyCode::LControl) {
        input_state.active_actions.insert(GameAction::MoveLauncher);
    }
    if keys.just_pressed(KeyCode::LControl) {
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

    if mouse_buttons.pressed(MouseButton::Right) {
        input_state.active_actions.insert(GameAction::SpawnPegs);
    }
    if mouse_buttons.just_pressed(MouseButton::Right) {
        input_state
            .just_active_actions
            .insert(GameAction::SpawnPegs);
    }
}
