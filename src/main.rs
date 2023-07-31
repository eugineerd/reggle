#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_tweening::TweeningPlugin;

const PLAYER_BALL_RADIUS: f32 = 10.0;
const LAUNCHER_BASE_POWER: f32 = 500.0;
const PEG_RADIUS: f32 = 13.0;
const PIXELS_PER_METER: f32 = 100.0;
const SCREEN_HEIGHT: f32 = 1000.0;
const ARENA_SIZE: Vec2 = Vec2::new(1000.0, 800.0);
const ARENA_POS: Vec2 = Vec2::new(0.0, -100.0);

mod assets;
mod ball;
mod common;
mod debug;
mod input;
mod launcher;
mod path;
mod peg;
mod sounds;
mod trajectory;
mod ui;

use common::*;

fn main() {
    let mut app = App::new();

    app.add_state::<GameState>()
        .add_state::<InGameState>()
        .insert_resource(assets::GameAssets::default())
        .insert_resource(GameStats::default())
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0., -500.),
            ..Default::default()
        })
        .add_plugins((
            // Engine
            DefaultPlugins,
            AudioPlugin,
            ShapePlugin,
            TweeningPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.),
            // Game
            assets::AssetsPlugin,
            input::GameInputPlugin,
            ball::BallPlugin,
            peg::PegPlugin,
            debug::DebugPlugin,
            launcher::LauncherPlugin,
            // trajectory::TrajectoryPlugin,
            ui::UiPlugin,
            path::PathPlugin,
            sounds::SoundsPlugin,
        ))
        .add_systems(OnEnter(GameState::InGame), (setup_graphics, setup_level));

    #[cfg(feature = "exit_timeout")]
    app.add_systems(Update, exit_timeout_system);

    app.run();
}

// Required for CI
#[cfg(feature = "exit_timeout")]
fn exit_timeout_system(time: Res<Time>, mut writer: EventWriter<bevy::app::AppExit>) {
    if bevy::utils::Instant::now() - time.startup() > std::time::Duration::from_secs(10) {
        println!("Didn't crash");
        writer.send(bevy::app::AppExit);
    }
}

fn setup_graphics(mut commands: Commands, game_assets: Res<assets::GameAssets>) {
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                scaling_mode: bevy::render::camera::ScalingMode::FixedVertical(SCREEN_HEIGHT),
                far: 1000.0,
                near: -1000.0,
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        MainCamera,
    ));
    commands.spawn(SpriteBundle {
        texture: game_assets.background_image.clone(),
        transform: Transform::from_xyz(0.0, 0.0, -100.0),
        ..Default::default()
    });
}

fn spawn_wall(commands: &mut Commands, position: Vec2, width: f32, height: f32) {
    commands
        .spawn(SpriteBundle {
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
