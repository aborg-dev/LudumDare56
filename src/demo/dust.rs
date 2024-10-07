use bevy::app::{App, Update};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::sprite::SpriteBundle;
use bevy::time::{Time, Timer, TimerMode};

use crate::AppSet;

use super::creature::CreatureAssets;
use super::movement::MovementController;

#[derive(Component, Clone, Reflect, Default)]
pub struct DustAnimation {
    pub timer: Timer,
}

// animation parameters
const DURATION_SEC: f32 = 0.25;
const SPEED: f32 = 50.0;
const DUST_Z: f32 = 0.5;

impl DustAnimation {
    pub fn init(app: &mut App) {
        app.add_systems(
            Update,
            (update_dust_animation, tick.in_set(AppSet::TickTimers)),
        );
    }

    pub fn spawn(commands: &mut Commands, assets: &CreatureAssets, start_pos: Vec2) {
        let clouds = 16;
        for i in 0..clouds {
            let angle = i as f32 * std::f32::consts::TAU / clouds as f32;
            commands.spawn((
                DustAnimation {
                    timer: Timer::from_seconds(DURATION_SEC, TimerMode::Once),
                },
                SpriteBundle {
                    texture: assets.dust.clone(),
                    transform: Transform::from_translation(start_pos.extend(DUST_Z)),
                    ..Default::default()
                },
                MovementController {
                    intent: Vec2::from_angle(angle),
                    max_speed: SPEED,
                    ..Default::default()
                },
            ));
        }
    }
}

fn update_dust_animation(
    mut commands: Commands,
    mut query: Query<(&DustAnimation, Entity, &mut Sprite)>,
) {
    for (animation, entity, mut image) in &mut query {
        if animation.timer.finished() {
            commands.entity(entity).despawn();
        } else {
            image.color.set_alpha(animation.timer.fraction_remaining());
        }
    }
}

fn tick(time: Res<Time>, mut query: Query<&mut DustAnimation>) {
    for mut animation in &mut query.iter_mut() {
        animation.timer.tick(time.delta());
    }
}
