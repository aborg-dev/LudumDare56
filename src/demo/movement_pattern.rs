//! A plugin to add movement patterns to anything with a `MovementController`.
use bevy::app::App;
use bevy::math::Vec2;
use bevy::prelude::{Component, IntoSystemConfigs, Query, Res};
use bevy::reflect::Reflect;
use bevy::time::{Time, Timer};
use core::f32;
use std::time::Duration;

use crate::AppSet;

use super::movement::MovementController;

/// A specific pattern for how entities move.
///
/// The entity requires a `MovementController` for the moving to work.
#[derive(Debug, Component, Reflect, Clone)]
pub enum MovementPattern {
    Constant { speed: Vec2 },
    Periodic { timer: Timer, max_speed: Vec2 },
    Circle { timer: Timer, radius: f32 },
}

#[derive(Debug, Clone, Reflect, serde::Deserialize)]
pub enum MovementPatternDefinition {
    Constant { speed: Vec2 },
    Periodic { duration_ms: u64, max_speed: Vec2 },
    Circle { duration_ms: u64, radius: f32 },
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementPattern>();
    app.add_systems(
        bevy::app::Update,
        (
            move_creatures.in_set(AppSet::Update),
            update_timer.in_set(AppSet::TickTimers),
        ),
    );
}

fn move_creatures(mut controller_query: Query<(&mut MovementController, &MovementPattern)>) {
    for (mut movement_controller, creature_property) in &mut controller_query {
        match creature_property {
            MovementPattern::Constant { speed } => {
                movement_controller.intent = *speed;
            }
            MovementPattern::Periodic { max_speed, timer } => {
                // use positive range of sinus for the speed
                let fraction = f32::sin(f32::consts::PI * timer.fraction());
                movement_controller.intent = *max_speed * fraction;
            }
            MovementPattern::Circle { timer, radius } => {
                // use a constant speed to go around the circle
                let angle = timer.fraction() * f32::consts::TAU;
                movement_controller.intent = Vec2 {
                    x: -f32::sin(angle) * radius,
                    y: f32::cos(angle) * radius,
                };
            }
        }
    }
}

fn update_timer(time: Res<Time>, mut query: Query<&mut MovementPattern>) {
    let delta = time.delta();
    for mut pattern in &mut query {
        match *pattern {
            MovementPattern::Constant { .. } => (),
            MovementPattern::Periodic { ref mut timer, .. }
            | MovementPattern::Circle { ref mut timer, .. } => {
                timer.tick(delta);
            }
        }
    }
}

impl MovementPatternDefinition {
    pub fn build(&self) -> MovementPattern {
        match self {
            MovementPatternDefinition::Constant { speed } => {
                MovementPattern::Constant { speed: *speed }
            }
            MovementPatternDefinition::Periodic {
                duration_ms,
                max_speed,
            } => MovementPattern::Periodic {
                timer: Timer::new(
                    Duration::from_millis(*duration_ms),
                    bevy::time::TimerMode::Repeating,
                ),
                max_speed: *max_speed,
            },
            MovementPatternDefinition::Circle {
                duration_ms,
                radius,
            } => MovementPattern::Circle {
                timer: Timer::new(
                    Duration::from_millis(*duration_ms),
                    bevy::time::TimerMode::Repeating,
                ),
                radius: *radius,
            },
        }
    }
}
