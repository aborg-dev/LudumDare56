//! Spawn the main level.

use bevy::window::PrimaryWindow;
use bevy::{ecs::world::Command, prelude::*};
use rand::distributions::{Distribution, Uniform};

use crate::demo::creature::SpawnCreature;
use crate::demo::movement_pattern::MovementPattern;
use crate::screens::Screen;
use crate::AppSet;

const SPAWN_DURATION_SECS: f32 = 1.0;

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct SpawnTimer(Timer);

impl Default for SpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            SPAWN_DURATION_SECS,
            TimerMode::Repeating,
        ))
    }
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<SpawnTimer>();
    app.init_resource::<SpawnTimer>();
    app.add_systems(
        Update,
        (
            tick_spawn_timer.in_set(AppSet::TickTimers),
            check_spawn_timer.in_set(AppSet::Update),
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// A [`Command`] to spawn the level.
/// Functions that accept only `&mut World` as their parameter implement [`Command`].
/// We use this style when a command requires no configuration.
pub fn spawn_level(world: &mut World) {
    SpawnCreature {
        max_speed: 400.0,
        pos: Vec2 { x: 0.0, y: 0.0 },
        movement: MovementPattern::Constant(Vec2 { x: 1.0, y: 0.5 }),
    }
    .apply(world);
}

fn tick_spawn_timer(time: Res<Time>, mut timer: ResMut<SpawnTimer>) {
    timer.0.tick(time.delta());
}

fn check_spawn_timer(
    timer: ResMut<SpawnTimer>,
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };

    if timer.0.just_finished() {
        let mut rng = &mut rand::thread_rng();
        let size = window.size();
        // TODO: This doesn't fully work, need to figure out why.
        // Don't spawn on the edges.
        let sprite_half_size = 256.0 / 2.0;
        let x_dist = Uniform::from(sprite_half_size..size.x - sprite_half_size);
        let y_dist = Uniform::from(sprite_half_size..size.y - sprite_half_size);
        let pos = Vec2 {
            x: x_dist.sample(&mut rng),
            y: y_dist.sample(&mut rng),
        };

        let dist = Uniform::new(-1.0, 1.0);
        let direction = Vec2 {
            x: dist.sample(rng),
            y: dist.sample(rng),
        };
        commands.add(SpawnCreature {
            max_speed: 400.0,
            pos,
            movement: MovementPattern::Constant(direction),
        });
    }
}
