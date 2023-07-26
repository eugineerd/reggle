use std::sync::Arc;

use bevy::{asset::LoadState, prelude::*};
use bevy_kira_audio::AudioSource;

use crate::common::GameState;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            .add_systems(OnEnter(GameState::LoadingAssets), load_assets)
            .add_systems(
                PreUpdate,
                check_load_status.run_if(in_state(GameState::LoadingAssets)),
            );
    }
}

#[derive(Default)]
pub struct PegAssets {
    pub hit_sound: Arc<Vec<Handle<AudioSource>>>,
    pub image: Handle<Image>,
    pub hit_image: Handle<Image>,
    pub pop_sound: Handle<AudioSource>,
}

#[derive(Default)]
pub struct BallAssets {
    pub hit_sound: Arc<Vec<Handle<AudioSource>>>,
    pub image: Handle<Image>,
}

#[derive(Default)]
pub struct LauncherAssets {
    pub image: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct GameAssets {
    pub peg: PegAssets,
    pub ball: BallAssets,
    pub launcher: LauncherAssets,
    pub background_image: Handle<Image>,
    pub normal_font: Handle<Font>,
}

fn load_assets(asset_server: Res<AssetServer>, mut assets: ResMut<GameAssets>) {
    assets.peg.hit_sound = Arc::new(
        vec![
            "sfx/peg/impactGlass_medium_000.ogg",
            "sfx/peg/impactGlass_medium_001.ogg",
            "sfx/peg/impactGlass_medium_002.ogg",
            "sfx/peg/impactGlass_medium_003.ogg",
            "sfx/peg/impactGlass_medium_004.ogg",
        ]
        .into_iter()
        .map(|s| asset_server.load(s))
        .collect(),
    );
    assets.peg.pop_sound = asset_server.load("sfx/pop.ogg");

    assets.peg.image = asset_server.load("sprites/peg/normal.png");
    assets.peg.hit_image = asset_server.load("sprites/peg/hit.png");

    assets.ball.hit_sound = Arc::new(
        vec![
            "sfx/ball/impactSoft_heavy_001.ogg",
            "sfx/ball/impactSoft_heavy_002.ogg",
            "sfx/ball/impactSoft_heavy_003.ogg",
            "sfx/ball/impactSoft_heavy_004.ogg",
        ]
        .into_iter()
        .map(|s| asset_server.load(s))
        .collect(),
    );

    assets.ball.image = asset_server.load("sprites/ball.png");
    assets.launcher.image = asset_server.load("sprites/launcher.png");
    assets.background_image = asset_server.load("sprites/background.png");
    assets.normal_font = asset_server.load("fonts/NotoSans.ttf");
}

fn check_load_status(
    asset_server: Res<AssetServer>,
    assets: Res<GameAssets>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let s = asset_server.get_load_state(&assets.background_image);
    info!("{:?}", s);
    match s {
        LoadState::Loaded => next_state.set(state.next()),
        LoadState::Failed => panic!("Failed to load"),
        _ => (),
    }
}
