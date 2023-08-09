use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_kira_audio::{Audio, AudioControl};
use bevy_rapier2d::prelude::*;
use bevy_tweening::{Animator, EaseFunction, Lens, Tween};
use std::collections::VecDeque;
use std::time::Duration;

use crate::common::{GameState, GameStats, InGameState};
use crate::path::{Path, PathAgent, PathPoint};
use crate::sounds::{play_collision_sound, CollisionSound, SoundType};
use crate::{assets::GameAssets, PEG_RADIUS};

pub struct PegPlugin;

impl Plugin for PegPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PegDespawnEvent>()
            .insert_resource(PegDespawnQueue::default())
            .add_systems(OnEnter(GameState::InGame), spawn_peg_system)
            .add_systems(
                Update,
                (
                    transform_peg,
                    peg_hit_system
                        .run_if(in_state(InGameState::Ball))
                        .after(play_collision_sound),
                    peg_cleanup.run_if(in_state(InGameState::Cleanup)),
                    peg_despawn.after(peg_cleanup),
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Event)]
pub struct PegDespawnEvent(pub Entity);

#[derive(Default, Bundle, Clone)]
pub struct PegPreset {
    img: Handle<Image>,
    sprite: Sprite,
    collision_sound: CollisionSound,
    collider: Collider,
}

#[derive(Default, PartialEq, Eq, Hash)]
pub enum PegState {
    #[default]
    Active,
    Hit,
}

#[derive(Component, Default)]
pub struct Peg {
    pub is_target: bool,
    pub state: PegState,
    pub presets: std::sync::Arc<HashMap<PegState, PegPreset>>,
}

#[derive(Resource, Default)]
pub struct PegDespawnQueue(VecDeque<Entity>);

#[derive(Bundle)]
pub struct PegBundle {
    #[bundle()]
    pub sprite_bundle: SpriteBundle,

    pub collider: Collider,
    pub name: Name,
    pub collision_sound: CollisionSound,
    pub peg: Peg,
    pub body: RigidBody,
}

impl Default for PegBundle {
    fn default() -> Self {
        Self {
            collider: Collider::ball(PEG_RADIUS),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(PEG_RADIUS * 2.0, PEG_RADIUS * 2.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            name: Name::new("Peg"),
            collision_sound: Default::default(),
            peg: Peg::default(),
            body: RigidBody::Fixed,
        }
    }
}

fn spawn_peg_system(mut commands: Commands, game_assets: Res<GameAssets>) {
    let pegs_count = 15 * 7;
    let target_pegs_count = pegs_count / 8;

    let mut round_peg_presets = HashMap::new();
    let mut round_target_peg_presets = HashMap::new();
    let mut round_peg_preset = PegPreset {
        img: game_assets.peg.image.clone(),
        sprite: Sprite {
            custom_size: Some(Vec2::new(PEG_RADIUS * 2.0, PEG_RADIUS * 2.0)),
            ..Default::default()
        },
        collision_sound: CollisionSound {
            sound: SoundType::Random(game_assets.peg.hit_sound.clone()),
            ..Default::default()
        },
        collider: Collider::ball(PEG_RADIUS),
    };
    let mut round_target_peg_preset = round_peg_preset.clone();
    round_target_peg_preset.sprite.color = Color::ORANGE;
    round_peg_presets.insert(PegState::Active, round_peg_preset.clone());
    round_target_peg_presets.insert(PegState::Active, round_target_peg_preset.clone());

    round_peg_preset.collision_sound.sound = SoundType::None;
    round_peg_preset.sprite.color = Color::MIDNIGHT_BLUE;
    round_target_peg_preset.collision_sound.sound = SoundType::None;
    round_target_peg_preset.sprite.color = Color::ORANGE_RED;
    round_peg_presets.insert(PegState::Hit, round_peg_preset);
    round_target_peg_presets.insert(PegState::Hit, round_target_peg_preset);
    let round_peg_presets = std::sync::Arc::new(round_peg_presets);
    let round_target_peg_presets = std::sync::Arc::new(round_target_peg_presets);

    let mut rect_peg_presets = HashMap::new();
    let mut rect_target_peg_presets = HashMap::new();
    let mut rect_peg_preset = PegPreset {
        img: game_assets.peg.image.clone(),
        sprite: Sprite {
            custom_size: Some(Vec2::new(PEG_RADIUS * 2.0 * 1.5, PEG_RADIUS * 2.0)),
            ..Default::default()
        },
        collision_sound: CollisionSound {
            sound: SoundType::Random(game_assets.peg.hit_sound.clone()),
            ..Default::default()
        },
        collider: Collider::cuboid(PEG_RADIUS * 1.5, PEG_RADIUS),
    };
    let mut rect_target_peg_preset = rect_peg_preset.clone();
    rect_target_peg_preset.sprite.color = Color::GREEN;
    rect_peg_presets.insert(PegState::Active, rect_peg_preset.clone());
    rect_target_peg_presets.insert(PegState::Active, rect_target_peg_preset.clone());

    rect_peg_preset.collision_sound.sound = SoundType::None;
    rect_peg_preset.sprite.color = Color::MIDNIGHT_BLUE;
    rect_target_peg_preset.collision_sound.sound = SoundType::None;
    rect_target_peg_preset.sprite.color = Color::DARK_GREEN;
    rect_peg_presets.insert(PegState::Hit, rect_peg_preset);
    rect_target_peg_presets.insert(PegState::Hit, rect_target_peg_preset);
    let rect_peg_presets = std::sync::Arc::new(rect_peg_presets);
    let rect_target_peg_presets = std::sync::Arc::new(rect_target_peg_presets);

    let mut pegs = Vec::with_capacity(pegs_count);
    for i in 0..15 {
        for j in 1..8 {
            let tr = Transform::from_xyz(
                (14 / 2 - i) as f32 * PEG_RADIUS * 5.0,
                -j as f32 * PEG_RADIUS * 5.0,
                0.0,
            );
            let presets = fastrand::choice([&round_peg_presets, &rect_peg_presets])
                .unwrap()
                .clone();
            pegs.push(PegBundle {
                peg: Peg {
                    presets,
                    ..Default::default()
                },
                sprite_bundle: SpriteBundle {
                    transform: tr,
                    ..Default::default()
                },
                body: RigidBody::Fixed,
                ..Default::default()
            })
        }
    }

    fastrand::shuffle(&mut pegs);

    for (i, peg) in pegs.iter_mut().enumerate() {
        if i < target_pegs_count {
            if std::sync::Arc::ptr_eq(&peg.peg.presets, &round_peg_presets) {
                peg.peg.presets = round_target_peg_presets.clone();
            } else {
                peg.peg.presets = rect_target_peg_presets.clone();
            }
        }
    }

    commands.spawn_batch(pegs.into_iter());

    commands
        .spawn((
            Path::new(100.0, true),
            TransformBundle::default(),
            VisibilityBundle::default(),
        ))
        .with_children(|cb| {
            for i in [
                Vec2::new(-300.0, 300.0),
                Vec2::new(300.0, 300.0),
                Vec2::new(300.0, -300.0),
                Vec2::new(-300.0, -300.0),
            ] {
                cb.spawn((
                    PathPoint::default(),
                    TransformBundle::from_transform(Transform::from_translation(i.extend(0.0))),
                ));
            }
            for i in 0..20 {
                cb.spawn((
                    PegBundle {
                        peg: Peg {
                            presets: rect_peg_presets.clone(),
                            ..Default::default()
                        },
                        sprite_bundle: SpriteBundle {
                            ..Default::default()
                        },
                        body: RigidBody::KinematicPositionBased,
                        ..Default::default()
                    },
                    PathAgent {
                        t: 4.0 * i as f32 / 20.0,
                    },
                ));
            }
        });
}

fn peg_cleanup(
    mut commands: Commands,
    time: Res<Time>,
    mut despawn_events: EventWriter<PegDespawnEvent>,
    mut peg_sprites: Query<&mut Sprite, With<Peg>>,
    mut despawn_timer: Local<Option<Timer>>,
    mut peg_despawn_queue: ResMut<PegDespawnQueue>,
) {
    let despawn_timer =
        despawn_timer.get_or_insert_with(|| Timer::from_seconds(0.1, TimerMode::Repeating));
    despawn_timer.tick(time.delta());
    loop {
        if let Some(peg) = peg_despawn_queue.0.front() {
            let inflated_size = PEG_RADIUS * 2.0 + despawn_timer.percent() * PEG_RADIUS * 1.5;
            if let Ok(mut s) = peg_sprites.get_mut(*peg) {
                s.custom_size = Some(Vec2::splat(inflated_size));
            } else {
                // Skips already depspawned entities
                peg_despawn_queue.0.pop_front();
                continue;
            }
            if despawn_timer.just_finished() {
                despawn_events.send(PegDespawnEvent(*peg));
                peg_despawn_queue.0.pop_front();
            }
        } else {
            despawn_timer.reset();
            peg_despawn_queue.0.clear();
            commands.insert_resource(NextState(Some(InGameState::Launcher)));
        }
        break;
    }
}

fn peg_despawn(
    mut commands: Commands,
    mut despawn_events: EventReader<PegDespawnEvent>,
    mut game_stats: ResMut<GameStats>,
    pegs: Query<Entity, With<Peg>>,
    game_assets: Res<GameAssets>,
    audio: Res<Audio>,
) {
    for PegDespawnEvent(entity) in despawn_events.iter() {
        if !pegs.contains(*entity) {
            continue;
        }
        let Some(mut entity_commands) = commands.get_entity(*entity) else {continue};
        audio.play(game_assets.peg.pop_sound.clone());
        game_stats.player_score += 1;
        entity_commands.despawn();
    }
}

fn peg_hit_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut pegs: Query<(Entity, &Sprite, &mut Peg)>,
    mut peg_despawn_queue: ResMut<PegDespawnQueue>,
) {
    for event in collision_events.iter() {
        let CollisionEvent::Started(e1, e2, _) = event else {continue};
        let mut comps = pegs.get_mut(*e1);
        if comps.is_err() {
            comps = pegs.get_mut(*e2);
        }
        let Ok((entity, sprite, mut peg)) = comps else {continue};

        if peg.state == PegState::Hit {
            continue;
        }

        struct SpriteSizeLens {
            start: Vec2,
            end: Vec2,
        }

        impl Lens<Sprite> for SpriteSizeLens {
            fn lerp(&mut self, target: &mut Sprite, ratio: f32) {
                target.custom_size = Some(self.start + (self.end - self.start) * ratio);
            }
        }

        peg.state = PegState::Hit;
        let hit_tween = Tween::new(
            EaseFunction::CubicIn,
            Duration::from_secs_f32(0.1),
            SpriteSizeLens {
                start: sprite.custom_size.unwrap_or_default(),
                end: sprite.custom_size.unwrap_or_default() * 1.5,
            },
        )
        .then(Tween::new(
            EaseFunction::CubicOut,
            Duration::from_secs_f32(0.1),
            SpriteSizeLens {
                end: sprite.custom_size.unwrap_or_default(),
                start: sprite.custom_size.unwrap_or_default() * 1.5,
            },
        ));
        commands.entity(entity).insert(Animator::new(hit_tween));
        peg_despawn_queue.0.push_back(entity);
    }
}

fn transform_peg(
    mut commands: Commands,
    mut pegs: Query<(Entity, &Peg), Or<(Added<Peg>, Changed<Peg>)>>,
) {
    pegs.iter_mut().for_each(|(e, peg)| {
        let Some(preset) = peg.presets.get(&peg.state) else {return};
        commands.entity(e).insert(preset.clone());
    });
}
