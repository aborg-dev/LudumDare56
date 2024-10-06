//! Creature sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::prelude::*;
use std::time::Duration;

use crate::{demo::creature::CreatureAssets, AppSet};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CreatureAnimation>();
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSet::TickTimers),
            (update_animation_shrinking)
                .chain()
                .run_if(resource_exists::<CreatureAssets>)
                .in_set(AppSet::Update),
        ),
    );
}

fn update_animation_shrinking(
    mut _creature_query: Query<(&mut Transform, &mut CreatureAnimation)>,
) {
    // for (mut transform, animation) in &mut creature_query {
    //     let shrink_factor = (1.0 - animation.timer.elapsed_secs() / 10.0).max(0.5);
    //     transform.scale = Vec2::splat(0.5 * shrink_factor).extend(1.0);
    // }
}

/// Update the animation timer.
fn update_animation_timer(time: Res<Time>, mut query: Query<&mut CreatureAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Component that tracks player's animation state.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CreatureAnimation {
    timer: Timer,
}

impl CreatureAnimation {
    pub fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
        }
    }

    /// Update animation timers.
    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
    }
}
