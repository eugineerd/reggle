use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::Velocity;

use crate::{
    ball::{Ball, BallBundle},
    common::GameAssets,
    input_state::{GameAction, InputState},
    load_assets,
};

pub struct LauncherPlugin;

impl Plugin for LauncherPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_ball_launcher.after(load_assets))
            .add_system(launcher_control_system)
            .add_system(ball_launcher_system);
    }
}

#[derive(Component, Inspectable)]
pub struct Launcher {
    pub direction: Vec2,
    pub draw_trajectory: bool,
    pub power: f32,
}

impl Launcher {
    pub fn get_impulse(&self) -> Vec2 {
        self.direction * self.power
    }
}

fn setup_ball_launcher(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands
        .spawn_bundle(SpriteBundle {
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
            draw_trajectory: true,
            power: 200.0,
        });
}

fn launcher_control_system(
    input_state: Res<InputState>,
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
    input_state: Res<InputState>,
    game_assets: Res<GameAssets>,
    mut launcher: Query<(&Transform, &mut Launcher)>,
    balls: Query<&Ball>,
) {
    let (tr, mut launcher) = launcher.single_mut();
    if !balls.is_empty() {
        launcher.draw_trajectory = false;
        return;
    }
    launcher.draw_trajectory = true;
    if !input_state.just_active(GameAction::Shoot) {
        return;
    }
    commands
        .spawn_bundle(BallBundle::new(tr.translation, &game_assets))
        .insert(Velocity {
            linvel: launcher.get_impulse(),
            ..Default::default()
        });
}
