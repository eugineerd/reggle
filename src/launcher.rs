use bevy::prelude::*;
use bevy_rapier2d::prelude::Velocity;

use crate::common::{GameState, InGameState};
use crate::LAUNCHER_BASE_POWER;
use crate::{
    ball::BallBundle,
    common::GameAssets,
    input::{GameAction, GameInput},
};

pub struct LauncherPlugin;

impl Plugin for LauncherPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup_ball_launcher)
            .add_systems(
                Update,
                (
                    launcher_control_system,
                    ball_launcher_system.run_if(in_state(InGameState::Launcher)),
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Component, Reflect)]
pub struct Launcher {
    pub direction: Vec2,
    pub power: f32,
}

impl Launcher {
    pub fn get_impulse(&self) -> Vec2 {
        self.direction * self.power
    }
}

fn setup_ball_launcher(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                ..Default::default()
            },
            texture: game_assets.launcher_image.clone(),
            ..Default::default()
        })
        .insert(Name::new("Launcher"))
        .insert(Transform::from_xyz(0.0, 150.0, 1.0))
        .insert(Launcher {
            direction: Vec2::ZERO,
            power: LAUNCHER_BASE_POWER,
        });
}

fn launcher_control_system(
    input_state: Res<GameInput>,
    mut launcher: Query<(&mut Transform, &mut Launcher)>,
) {
    let (mut tr, mut launcher) = launcher.single_mut();
    if input_state.active(GameAction::MoveLauncher) {
        tr.translation = input_state.cursor_position.extend(0.0);
    }
    let diff = input_state.cursor_position - tr.translation.truncate();
    let direction = diff.normalize_or_zero();
    launcher.direction = direction;
    tr.rotation = Quat::from_rotation_arc_2d(Vec2::new(0.0, 1.0), direction);
}

fn ball_launcher_system(
    mut commands: Commands,
    input_state: Res<GameInput>,
    game_assets: Res<GameAssets>,
    launcher: Query<(&Transform, &Launcher)>,
) {
    if input_state.just_active(GameAction::Shoot) {
        commands.insert_resource(NextState(Some(InGameState::Ball)));
        let (tr, launcher) = launcher.single();
        commands
            .spawn(BallBundle::new(tr.translation, &game_assets))
            .insert(Velocity {
                linvel: launcher.get_impulse(),
                ..Default::default()
            });
    }
}
