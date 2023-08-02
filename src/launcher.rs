use bevy::prelude::*;
use bevy_rapier2d::prelude::{RapierConfiguration, Velocity};

use crate::common::{GameState, InGameState};
use crate::LAUNCHER_BASE_POWER;
use crate::{
    assets::GameAssets,
    ball::BallBundle,
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

#[derive(Component, Reflect, Default)]
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
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                ..Default::default()
            },
            texture: game_assets.launcher.image.clone(),
            transform: Transform::from_xyz(0.0, 150.0, 1.0),
            ..Default::default()
        },
        Name::new("Launcher"),
        Launcher {
            direction: Vec2::ZERO,
            power: LAUNCHER_BASE_POWER,
        },
    ));
}

fn angle_to_hit_target(start_pos: Vec2, target_pos: Vec2, g: f32, v: f32) -> f32 {
    let target = target_pos - start_pos;
    let (x, y) = (target.x, target.y);
    let r = (v.powi(4) - g * (g * x * x + 2.0 * y * v * v)).sqrt();
    let mut r = ((v * v - r) / (g * x)).atan();
    if x < 0.0 {
        r = r - std::f32::consts::PI;
    }
    r
}

fn launcher_control_system(
    input_state: Res<GameInput>,
    rapier_config: Res<RapierConfiguration>,
    mut launcher: Query<(&mut Transform, &mut Launcher)>,
) {
    let (mut tr, mut launcher) = launcher.single_mut();
    if input_state.active(GameAction::MoveLauncher) {
        tr.translation = input_state.cursor_position.extend(0.0);
    }
    let target_angle = angle_to_hit_target(
        tr.translation.truncate(),
        input_state.cursor_position,
        rapier_config.gravity.length(),
        launcher.power,
    );
    if target_angle.is_nan() {
        return;
    }
    let direction = Vec2::from_angle(target_angle);
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
