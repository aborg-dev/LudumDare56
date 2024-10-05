//! Spawn the main level.

use bevy::{ecs::world::Command, prelude::*};

use crate::demo::creature::SpawnCreature;

pub(super) fn plugin(_app: &mut App) {
    // No setup required for this plugin.
    // It's still good to have a function here so that we can add some setup
    // later if needed.
}

/// A [`Command`] to spawn the level.
/// Functions that accept only `&mut World` as their parameter implement [`Command`].
/// We use this style when a command requires no configuration.
pub fn spawn_level(world: &mut World) {
    SpawnCreature {
        max_speed: 400.0,
        pos: Vec2 { x: 0.0, y: 0.0 },
    }
    .apply(world);

    SpawnCreature {
        max_speed: 400.0,
        pos: Vec2 { x: 300.0, y: 400.0 },
    }
    .apply(world);
}
