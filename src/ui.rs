use bevy::prelude::*;

use crate::GameStats;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_ui)
            .add_system(update_score_system);
    }
}

#[derive(Component)]
struct ScoreUi;

fn setup_ui(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "0".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..default()
            },
            ..Default::default()
        })
        .insert(ScoreUi);
}

fn update_score_system(game_state: Res<GameStats>, mut score_ui: Query<&mut Text, With<ScoreUi>>) {
    if let Ok(mut text) = score_ui.get_single_mut() {
        text.sections[0].value = game_state.player_score.to_string()
    }
}
