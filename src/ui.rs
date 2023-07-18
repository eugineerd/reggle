use bevy::prelude::*;

use crate::common::GameAssets;
use crate::{load_assets, GameStats};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui.after(load_assets))
            .add_systems(Update, update_score_system);
    }
}

#[derive(Component)]
struct ScoreUi;

fn setup_ui(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands
        .spawn(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "0".to_string(),
                    style: TextStyle {
                        font: game_assets.normal_font.clone(),
                        font_size: 42.0,
                        ..Default::default()
                    },
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
