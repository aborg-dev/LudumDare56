//! A plugin to add movement patterns to anything with a `MovementController`.
use bevy::app::App;
use bevy::math::Vec2;
use bevy::prelude::{Component, Query};
use bevy::reflect::Reflect;

use super::movement::MovementController;

/// A specific pattern for how entities move.
///
/// The entity requires a `MovementController` for the moving to work.
#[derive(Debug, Component, Reflect)]
pub enum MovementPattern {
    Constant(Vec2),
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementPattern>();
    app.add_systems(bevy::app::Update, move_creatures);
}

fn move_creatures(mut controller_query: Query<(&mut MovementController, &MovementPattern)>) {
    for (mut movement_controller, creature_property) in &mut controller_query {
        match creature_property {
            MovementPattern::Constant(vec2) => {
                movement_controller.intent = *vec2;
            }
        }
    }
}
