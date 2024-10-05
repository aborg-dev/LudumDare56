//! Spawn the main level.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::distributions::{Distribution, Uniform};

use crate::asset_tracking::LoadResource;
use crate::audio::SoundEffect;
use crate::demo::creature::SpawnCreature;
use crate::demo::movement_pattern::MovementPattern;
use crate::screens::Screen;
use crate::AppSet;

use std::time::Duration;

const SHRINK_DURATION: Duration = Duration::from_secs(10);
const SPAWN_DURATION: Duration = Duration::from_secs(5);

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct SpawnTimer(Timer);

#[derive(Resource, Debug, Clone, PartialEq, Reflect, Default)]
#[reflect(Resource)]
struct WaveCounter {
    wave: u32,
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct WaveSound {
    #[dependency]
    sound: Handle<AudioSource>,
}

impl FromWorld for WaveSound {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            sound: assets.load("audio/sound_effects/wave_start.ogg"),
        }
    }
}

impl Default for SpawnTimer {
    fn default() -> Self {
        Self(Timer::new(SPAWN_DURATION, TimerMode::Repeating))
    }
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<SpawnTimer>();
    app.load_resource::<WaveSound>();
    app.add_systems(OnEnter(Screen::Gameplay), add_resources);
    app.add_systems(OnExit(Screen::Gameplay), remove_resources);

    app.add_systems(
        Update,
        (
            tick_spawn_timer.in_set(AppSet::TickTimers),
            check_spawn_timer.in_set(AppSet::Update),
        )
            .run_if(resource_exists::<WaveSound>)
            .run_if(in_state(Screen::Gameplay)),
    );
}

fn add_resources(mut commands: Commands) {
    commands.insert_resource(SpawnTimer::default());
    commands.insert_resource(WaveCounter::default());
}

fn remove_resources(mut commands: Commands) {
    commands.remove_resource::<SpawnTimer>();
    commands.remove_resource::<WaveCounter>();
}

/// A [`Command`] to spawn the level.
/// Functions that accept only `&mut World` as their parameter implement [`Command`].
/// We use this style when a command requires no configuration.
pub fn spawn_level(_world: &mut World) {}

fn tick_spawn_timer(time: Res<Time>, mut timer: ResMut<SpawnTimer>) {
    timer.0.tick(time.delta());
}

fn check_spawn_timer(
    timer: ResMut<SpawnTimer>,
    mut wave_counter: ResMut<WaveCounter>,
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    sound: Res<WaveSound>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };

    if timer.0.just_finished() || wave_counter.wave == 0 {
        commands.spawn((
            AudioBundle {
                source: sound.sound.clone(),
                settings: PlaybackSettings::DESPAWN,
            },
            SoundEffect,
        ));

        wave_counter.wave += 1;
        let mut rng = &mut rand::thread_rng();
        let size = window.size() - 256.0;
        let half_size = size / 2.0;
        let x_dist = Uniform::from(-half_size.x..half_size.x);
        let y_dist = Uniform::from(-half_size.y..half_size.y);
        let dist = Uniform::new(-1.0, 1.0);

        // Add either a constant or a periodic movement each wave
        let num_constant_movement: u32 = 3 + (wave_counter.wave + 1) / 2;
        let num_periodic_movement: u32 = 2 + wave_counter.wave / 2;
        // Add a circular movement every third round
        let num_circle_movement: u32 = wave_counter.wave / 3;

        let min_radius = 0.25;
        let max_radius = 1.0 + wave_counter.wave as f32;

        for _ in 0..num_constant_movement {
            let pos = Vec2 {
                x: x_dist.sample(&mut rng),
                y: y_dist.sample(&mut rng),
            };

            let speed = Vec2 {
                x: dist.sample(rng),
                y: dist.sample(rng),
            }
            .normalize();
            commands.add(SpawnCreature {
                max_speed: 400.0,
                pos,
                movement: MovementPattern::Constant { speed },
                shrink_duration: SHRINK_DURATION,
            });
        }
        for _ in 0..num_periodic_movement {
            let pos = Vec2 {
                x: x_dist.sample(&mut rng),
                y: y_dist.sample(&mut rng),
            };

            let max_speed = Vec2 {
                x: dist.sample(rng),
                y: dist.sample(rng),
            }
            .normalize()
                * 3.0;
            commands.add(SpawnCreature {
                max_speed: 400.0,
                pos,
                movement: MovementPattern::Periodic {
                    timer: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
                    max_speed,
                },
                shrink_duration: SHRINK_DURATION,
            });
        }
        for _ in 0..num_circle_movement {
            let pos = Vec2 {
                x: x_dist.sample(&mut rng),
                y: y_dist.sample(&mut rng),
            };

            commands.add(SpawnCreature {
                max_speed: 400.0,
                pos,
                movement: MovementPattern::Circle {
                    timer: Timer::new(Duration::from_millis(1500), TimerMode::Repeating),
                    radius: (dist.sample(rng) * max_radius).max(min_radius),
                },
                shrink_duration: SHRINK_DURATION,
            });
        }
    }
}
