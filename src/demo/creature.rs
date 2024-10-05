//! Plugin handling the player character in particular.
//! Note that this is separate from the `movement` module as that could be used
//! for other characters as well.

use bevy::{
    ecs::{system::RunSystemOnce as _, world::Command},
    math::bounding::Aabb2d,
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    window::PrimaryWindow,
};

use crate::{
    asset_tracking::LoadResource,
    demo::{
        animation::CreatureAnimation,
        movement::{MovementController, ScreenWrap},
    },
    screens::Screen,
    AppSet,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Creature>();
    app.load_resource::<CreatureAssets>();
    app.insert_resource(ClickController::default());

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            record_player_click_input.in_set(AppSet::RecordInput),
            process_clicks_on_creatures.in_set(AppSet::Update),
        ),
    );
}

fn process_clicks_on_creatures(
    click_controller: Res<ClickController>,
    creatures: Query<(Entity, &Transform), With<Creature>>,
    mut commands: Commands,
) {
    if click_controller.position.is_none() {
        return;
    }

    for (entity, transform) in &creatures {
        let p = click_controller.position.unwrap();
        let half_size = 32.0 * 8.0 / 2.0;
        let aabb2d = Aabb2d::new(transform.translation.truncate(), Vec2::splat(half_size));
        if aabb2d.closest_point(p) == p {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Creature;

/// A command to spawn the player character.
#[derive(Debug)]
pub struct SpawnCreature {
    /// See [`MovementController::max_speed`].
    pub max_speed: f32,
    pub pos: Vec2,
}

impl Command for SpawnCreature {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self, spawn_player);
    }
}

fn spawn_player(
    In(config): In<SpawnCreature>,
    mut commands: Commands,
    player_assets: Res<CreatureAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // A texture atlas is a way to split one image with a grid into multiple
    // sprites. By attaching it to a [`SpriteBundle`] and providing an index, we
    // can specify which section of the image we want to see. We will use this
    // to animate our player character. You can learn more about texture atlases in
    // this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = CreatureAnimation::new();

    commands.spawn((
        Name::new("Creature"),
        Creature,
        SpriteBundle {
            texture: player_assets.ducky.clone(),
            transform: Transform::from_scale(Vec2::splat(8.0).extend(1.0))
                .with_translation(config.pos.extend(1.0)),
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: player_animation.get_atlas_index(),
        },
        MovementController {
            max_speed: config.max_speed,
            ..default()
        },
        ScreenWrap,
        player_animation,
        StateScoped(Screen::Gameplay),
    ));
}

#[derive(Resource, Reflect, Clone, Default)]
pub struct ClickController {
    pub position: Option<Vec2>,
}

fn record_player_click_input(
    input: Res<ButtonInput<MouseButton>>,
    touches_input: Res<Touches>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut click_controller: ResMut<ClickController>,
) {
    let (camera, camera_global_transform) = camera_query.single();
    let window = window_query.single();

    click_controller.position = None;
    if input.just_pressed(MouseButton::Left) {
        if let Some(p) = window
            .cursor_position()
            .or_else(|| touches_input.first_pressed_position())
            .and_then(|cursor| camera.viewport_to_world_2d(camera_global_transform, cursor))
        {
            click_controller.position = Some(p);
        }
    }
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct CreatureAssets {
    // This #[dependency] attribute marks the field as a dependency of the Asset.
    // This means that it will not finish loading until the labeled asset is also loaded.
    #[dependency]
    pub ducky: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl CreatureAssets {
    pub const PATH_DUCKY: &'static str = "images/ducky.png";
    pub const PATH_STEP_1: &'static str = "audio/sound_effects/step1.ogg";
    pub const PATH_STEP_2: &'static str = "audio/sound_effects/step2.ogg";
    pub const PATH_STEP_3: &'static str = "audio/sound_effects/step3.ogg";
    pub const PATH_STEP_4: &'static str = "audio/sound_effects/step4.ogg";
}

impl FromWorld for CreatureAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load_with_settings(
                CreatureAssets::PATH_DUCKY,
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve the pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            steps: vec![
                assets.load(CreatureAssets::PATH_STEP_1),
                assets.load(CreatureAssets::PATH_STEP_2),
                assets.load(CreatureAssets::PATH_STEP_3),
                assets.load(CreatureAssets::PATH_STEP_4),
            ],
        }
    }
}