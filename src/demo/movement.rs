//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use bevy::prelude::*;

use super::creature_image::CreatureImage;
use crate::screens::{GameplayArea, Screen};
use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<(MovementController, ScreenWrap, ScreenBounce)>();

    app.add_systems(
        Update,
        (
            apply_movement,
            apply_screen_wrap.run_if(in_state(Screen::Gameplay)),
            apply_screen_bounce.run_if(in_state(Screen::Gameplay)),
        )
            .chain()
            .in_set(AppSet::Update),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Possible adjustments applied after the initial intent direction.
    pub intent_modifier: Vec2,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics
    /// engine.
    pub max_speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            intent_modifier: Vec2::ONE,
            // 400 pixels per second is a nice default, but we can still vary this per character.
            max_speed: 400.0,
        }
    }
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &mut Transform)>,
) {
    for (controller, mut transform) in &mut movement_query {
        let velocity = controller.max_speed * controller.intent * controller.intent_modifier;
        transform.translation += velocity.extend(0.0) * time.delta_seconds();
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScreenWrap;

fn apply_screen_wrap(
    gameplay_area: Res<GameplayArea>,
    mut wrap_query: Query<&mut Transform, With<ScreenWrap>>,
) {
    let size = gameplay_area.main_area.size() + 256.0;
    let half_size = size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScreenBounce;

fn apply_screen_bounce(
    gameplay_area: Res<GameplayArea>,
    mut query: Query<(&mut MovementController, &Transform, &CreatureImage), With<ScreenBounce>>,
) {
    for (mut movement, transform, image) in &mut query {
        let size =
            gameplay_area.main_area.size() - (image.size().as_vec2() * image.default_scale());
        let half_size = size / 2.0;
        let position = transform.translation.xy();
        if position.x.abs() > half_size.x {
            // x is out of border, we have to set the intent modifier such that
            // it goes away from the edge after it is multiplied with the
            // original intent FIXME: This is hacky, can we do better? The
            // problem is how to combine arbitrary intent changes based on
            // pattern and make it orthogonal to the bouncing behavior. Maybe
            // each pattern should define its own bouncing.
            movement.intent_modifier.x = movement.intent.x.signum() * -position.x.signum();
        }
        if position.y.abs() > half_size.y {
            movement.intent_modifier.y = movement.intent.y.signum() * -position.y.signum();
        }
    }
}
