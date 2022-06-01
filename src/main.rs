#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

const PLAYER_BALL_RADIUS: f32 = 8.0;
const PEG_RADIUS: f32 = 10.0;
const PIXELS_PER_METER: f32 = 100.0;

mod ball;
mod common;
mod debug;
mod input_state;
mod launcher;
mod peg;
mod trajectory;
mod ui;

use common::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(ShapePlugin)
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -9.81 * 20.0),
            ..Default::default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(debug::DebugPlugin)
        .add_plugin(input_state::InputStatePlugin)
        .add_plugin(ball::BallPlugin)
        .add_plugin(peg::PegPlugin)
        .add_plugin(launcher::LauncherPlugin)
        .add_plugin(trajectory::TrajectoryPlugin)
        .add_plugin(ui::UiPlugin)
        .insert_resource(GameAssets::default())
        .insert_resource(GameState::default())
        .add_startup_system(load_assets)
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_level)
        .run();
}

fn load_assets(asset_server: Res<AssetServer>, mut assets: ResMut<GameAssets>) {
    assets.peg_hit_sound = vec![
        asset_server.load("sfx/peg/impactGlass_medium_000.ogg"),
        asset_server.load("sfx/peg/impactGlass_medium_001.ogg"),
        asset_server.load("sfx/peg/impactGlass_medium_002.ogg"),
        asset_server.load("sfx/peg/impactGlass_medium_003.ogg"),
        asset_server.load("sfx/peg/impactGlass_medium_004.ogg"),
    ];

    assets.peg_image = asset_server.load("sprites/peg.png");
    assets.peg_hit_image = asset_server.load("sprites/peg_hit.png");

    assets.ball_hit_sound = vec![
        asset_server.load("sfx/ball/impactSoft_heavy_001.ogg"),
        asset_server.load("sfx/ball/impactSoft_heavy_002.ogg"),
        asset_server.load("sfx/ball/impactSoft_heavy_003.ogg"),
        asset_server.load("sfx/ball/impactSoft_heavy_004.ogg"),
    ];

    assets.ball_image = asset_server.load("sprites/ball.png");
    assets.launcher_image = asset_server.load("sprites/launcher.png");
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn setup_level(mut commands: Commands) {
    // commands
    //     .spawn()
    //     .insert(Collider::cuboid(500.0, 20.0))
    //     .insert(Transform::from_xyz(0.0, -300.0, 0.0));
    commands
        .spawn()
        .insert(Collider::cuboid(500.0, 20.0))
        .insert(Transform::from_xyz(0.0, 300.0, 0.0))
        .insert(GlobalTransform::default());
    commands
        .spawn()
        .insert(Collider::cuboid(20.0, 500.0))
        .insert(Transform::from_xyz(-400.0, 0.0, 0.0))
        .insert(GlobalTransform::default());
    commands
        .spawn()
        .insert(Collider::cuboid(20.0, 500.0))
        .insert(Transform::from_xyz(400.0, 0.0, 0.0))
        .insert(GlobalTransform::default());
}
