use bevy::prelude::*;
use bevy_kira_audio::{AudioPlugin, AudioSource};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

const PLAYER_BALL_RADIUS: f32 = 6.0;
const PIXELS_PER_METER: f32 = 100.0;

mod ball;
mod input_state;
mod peg;
mod trajectory;
mod ui;

#[derive(Default)]
pub struct GameAssets {
    pub peg_hit_sound: Vec<Handle<AudioSource>>,
}

#[derive(Default)]
pub struct GameState {
    pub player_score: usize,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(input_state::InputStatePlugin)
        .add_plugin(peg::PegPlugin)
        .add_plugin(ball::BallPlugin)
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
        asset_server.load("sfx/peg/impactSoft_heavy_001.ogg"),
        asset_server.load("sfx/peg/impactSoft_heavy_002.ogg"),
        asset_server.load("sfx/peg/impactSoft_heavy_003.ogg"),
        asset_server.load("sfx/peg/impactSoft_heavy_004.ogg"),
    ];
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
        .insert(Transform::from_xyz(0.0, 300.0, 0.0));
    commands
        .spawn()
        .insert(Collider::cuboid(20.0, 500.0))
        .insert(Transform::from_xyz(-400.0, 0.0, 0.0));
    commands
        .spawn()
        .insert(Collider::cuboid(20.0, 500.0))
        .insert(Transform::from_xyz(400.0, 0.0, 0.0));
}
