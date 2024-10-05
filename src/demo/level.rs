//! Spawn the main level.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::distributions::{Distribution, Uniform};

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

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct WaveCounter {
    wave: u32,
}

impl Default for WaveCounter {
    fn default() -> Self {
        Self { wave: 0 }
    }
}

impl Default for SpawnTimer {
    fn default() -> Self {
        Self(Timer::new(SPAWN_DURATION, TimerMode::Repeating))
    }
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<SpawnTimer>();
    app.add_systems(OnEnter(Screen::Gameplay), add_resources);
    app.add_systems(OnExit(Screen::Gameplay), remove_resources);

    app.add_systems(
        Update,
        (
            tick_spawn_timer.in_set(AppSet::TickTimers),
            check_spawn_timer.in_set(AppSet::Update),
        )
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
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };

    if timer.0.just_finished() || wave_counter.wave == 0 {
        wave_counter.wave += 1;
        let mut rng = &mut rand::thread_rng();
        let size = window.size() - 256.0;
        let half_size = size / 2.0;
        let x_dist = Uniform::from(-half_size.x..half_size.x);
        let y_dist = Uniform::from(-half_size.y..half_size.y);
        let dist = Uniform::new(-1.0, 1.0);

        for _ in 0..5 + wave_counter.wave {
            let pos = Vec2 {
                x: x_dist.sample(&mut rng),
                y: y_dist.sample(&mut rng),
            };
            let direction = Vec2 {
                x: dist.sample(rng),
                y: dist.sample(rng),
            }
            .normalize();
            commands.add(SpawnCreature {
                max_speed: 400.0,
                pos,
                movement: MovementPattern::Constant(direction),
                shrink_duration: SHRINK_DURATION,
            });
        }
    }
}
