#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_tweening::TweeningPlugin;
use iyes_loopless::prelude::*;

const PLAYER_BALL_RADIUS: f32 = 10.0;
const LAUNCHER_BASE_POWER: f32 = 300.0;
const PEG_RADIUS: f32 = 12.0;
const PIXELS_PER_METER: f32 = 100.0;
const SCREEN_HEIGHT: f32 = 1000.0;
const ARENA_SIZE: Vec2 = Vec2::new(1000.0, 800.0);
const ARENA_POS: Vec2 = Vec2::new(0.0, -100.0);

mod ball;
mod common;
mod debug;
mod input;
mod launcher;
mod peg;
mod trajectory;
mod ui;

use common::*;

fn main() {
    App::new()
        .add_loopless_state(GameState::InGame)
        .add_loopless_state(InGameState::Launcher)
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(TweeningPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -9.81 * 24.0),
            ..Default::default()
        })
        // .add_plugin(debug::DebugPlugin)
        .add_plugin(input::GameInputPlugin)
        .add_plugin(ball::BallPlugin)
        .add_plugin(peg::PegPlugin)
        .add_plugin(debug::DebugPlugin)
        .add_plugin(launcher::LauncherPlugin)
        .add_plugin(trajectory::TrajectoryPlugin)
        .add_plugin(ui::UiPlugin)
        .insert_resource(GameAssets::default())
        .insert_resource(GameStats::default())
        .add_system(exit_timeout_system)
        .add_startup_system(load_assets)
        .add_startup_system(setup_graphics.after(load_assets))
        .add_startup_system(setup_level)
        .run();
}

// Required for CI
#[cfg(feature = "exit_timeout")]
fn exit_timeout_system(time: Res<Time>, mut writer: EventWriter<bevy::app::AppExit>) {
    if time.time_since_startup() > std::time::Duration::from_secs(10) {
        println!("Didn't crash");
        writer.send(bevy::app::AppExit);
    }
}

// Required for CI
#[cfg(not(feature = "exit_timeout"))]
fn exit_timeout_system() {}

fn load_assets(asset_server: Res<AssetServer>, mut assets: ResMut<GameAssets>) {
    assets.peg_hit_sound = vec![
        asset_server.load("sfx/peg/impactGlass_medium_000.ogg"),
        asset_server.load("sfx/peg/impactGlass_medium_001.ogg"),
        asset_server.load("sfx/peg/impactGlass_medium_002.ogg"),
        asset_server.load("sfx/peg/impactGlass_medium_003.ogg"),
        asset_server.load("sfx/peg/impactGlass_medium_004.ogg"),
    ];
    assets.peg_pop_sound = asset_server.load("sfx/pop.ogg");

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
    assets.background_image = asset_server.load("sprites/background.png");
    assets.normal_font = asset_server.load("fonts/NotoSans.ttf");
}

fn setup_graphics(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::FixedVertical(SCREEN_HEIGHT),
            ..Default::default()
        },
        ..Default::default()
    });
    commands.spawn_bundle(SpriteBundle {
        texture: game_assets.background_image.clone(),
        transform: Transform::from_xyz(0.0, 0.0, -0.01),
        ..Default::default()
    });
}

fn spawn_wall(commands: &mut Commands, position: Vec2, width: f32, height: f32) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(width * 2.0, height * 2.0)),
                color: Color::GRAY,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Collider::cuboid(width, height))
        .insert(Transform::from_xyz(position.x, position.y, 0.0))
        .insert(GlobalTransform::default());
}

fn setup_level(mut commands: Commands) {
    spawn_wall(
        &mut commands,
        ARENA_POS + Vec2::new(0.0, ARENA_SIZE.y / 2.0),
        ARENA_SIZE.x / 2.0,
        10.0,
    );
    spawn_wall(
        &mut commands,
        ARENA_POS + Vec2::new(ARENA_SIZE.x / 2.0, 0.0),
        10.0,
        ARENA_SIZE.y / 2.0,
    );
    spawn_wall(
        &mut commands,
        ARENA_POS - Vec2::new(ARENA_SIZE.x / 2.0, 0.0),
        10.0,
        ARENA_SIZE.y / 2.0,
    );
}
