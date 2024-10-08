//! Plugin handling the player character in particular.
//! Note that this is separate from the `movement` module as that could be used
//! for other characters as well.

use std::time::Duration;

use bevy::{
    audio::Volume,
    ecs::{system::RunSystemOnce as _, world::Command},
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    window::PrimaryWindow,
};
use rand::{distributions::Uniform, prelude::Distribution, Rng};

use crate::{
    asset_tracking::LoadResource,
    audio::SoundEffect,
    demo::{
        animation::CreatureAnimation,
        movement::{MovementController, ScreenBounce},
        movement_pattern::MovementPattern,
    },
    screens::{GameplayArea, Screen},
    AppSet,
};

use super::{
    creature_image::CreatureImage, custom_cursor::HideGameCursor, dust::DustAnimation,
    movement::ScreenWrap, movement_pattern::MovementPatternDefinition,
};

const BULLET_DURATION_SEC: f32 = 0.3;

const fn default_shrink_duration() -> u64 {
    10_000
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Creature>();
    app.load_resource::<CreatureAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            tick_bullets.in_set(AppSet::TickTimers),
            tick_death_animation.in_set(AppSet::TickTimers),
            record_player_click_input
                .run_if(resource_exists::<CreatureAssets>)
                .run_if(in_state(Screen::Gameplay))
                .in_set(AppSet::RecordInput),
            (
                update_bullet_animation,
                process_bullets_landing,
                process_bullets_falling.run_if(in_state(Screen::Gameplay)),
                end_game_on_too_many_creatures,
            )
                .chain()
                .run_if(resource_exists::<CreatureAssets>)
                .in_set(AppSet::Update),
            reaper,
        ),
    );

    DustAnimation::init(app);
}

fn tick_bullets(time: Res<Time>, mut query: Query<&mut Bullet>) {
    for mut bullet in &mut query.iter_mut() {
        bullet.timer.tick(time.delta());
    }
}

fn update_bullet_animation(
    mut query: Query<
        (&Bullet, &mut Transform, &mut MovementController),
        Without<FallingBulletMarker>,
    >,
) {
    for (bullet, mut transform, mut movement) in &mut query.iter_mut() {
        // we are throwing a ball
        // it should become smaller the further it is away and also adhere to gravity

        // 1) Size
        // use inverse linear approximation scale based on distance
        let min_size = 0.4;
        let scale = 1.0 - (1.0 - min_size) * bullet.timer.fraction();
        transform.scale = Vec2::splat(scale).extend(1.0);

        // 2) Gravity
        // bounce up and down, following parabola formula 1-((2x-1)^2) on range [0,1]
        // first derivative: -4x+2
        let x = bullet.timer.fraction();
        // lower height makes it appear / feel like a faster throw
        let height = 0.4;
        movement.intent.y = (2.0 - 4.0 * x) * height;
    }
}

fn process_bullets_falling(
    mut bullets: Query<(Entity, &Transform, &mut MovementController), With<FallingBulletMarker>>,
    mut commands: Commands,
    gameplay_area: Res<GameplayArea>,
    time: Res<Time>,
) {
    // remove bullets when they are out of the screen
    for (entity, transform, mut movement) in &mut bullets {
        if transform.translation.y < gameplay_area.main_area.min.y {
            commands.entity(entity).despawn();
        } else {
            let acceleration = 0.02;
            movement.intent.y -= time.delta().as_millis() as f32 * acceleration;
        }
    }
}

fn process_bullets_landing(
    creatures: Query<
        (Entity, &Transform, &CreatureImage),
        (With<Creature>, Without<DeathAnimation>),
    >,
    mut bullets: Query<(Entity, &Bullet, &Transform, &mut MovementController)>,
    mut commands: Commands,
    creature_assets: Res<CreatureAssets>,
) {
    let mut hits = Vec::new();
    let rng = &mut rand::thread_rng();
    for (entity, bullet, transform, mut movement) in &mut bullets {
        if !bullet.timer.just_finished() {
            continue;
        }
        DustAnimation::spawn(&mut commands, &creature_assets, transform.translation.xy());
        hits.push((entity, transform.translation.xy()));
        commands.entity(entity).insert(FallingBulletMarker);
        // bounce up
        movement.intent.y = 3.0;
        // and randomly to the side
        movement.intent.x = rng.sample(Uniform::new(-1.0, 1.0));
    }
    if hits.is_empty() {
        return;
    }

    // Bullet has landed.

    let mut found_target = false;
    for (entity, transform, image) in &creatures {
        let scaled_image_dimension = image.size().as_vec2() * transform.scale.truncate();
        let bounding_box =
            Rect::from_center_size(transform.translation.truncate(), scaled_image_dimension);
        if hits
            .iter()
            .any(|(_, click_pos)| bounding_box.contains(*click_pos))
        {
            commands.add(KillCreature(entity));
            found_target = true;
        }
    }

    if found_target {
        commands.spawn((
            AudioBundle {
                source: creature_assets.hit.clone(),
                settings: PlaybackSettings::DESPAWN.with_volume(Volume::new(0.5)),
            },
            SoundEffect,
        ));
    } else {
        commands.spawn((
            AudioBundle {
                source: creature_assets.miss.clone(),
                settings: PlaybackSettings::DESPAWN,
            },
            SoundEffect,
        ));
    }
}

fn end_game_on_too_many_creatures(
    creatures: Query<Entity, With<Creature>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    let count = creatures.iter().count();
    if count > 100 {
        next_screen.set(Screen::Score);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Creature;

/// A command to spawn the player character.
#[derive(Debug)]
pub struct SpawnCreature {
    pub image: CreatureImage,
    /// See [`MovementController::max_speed`].
    pub max_speed: f32,
    pub pos: Vec2,
    pub movement: MovementPattern,
    pub shrink_duration: Duration,
    /// true: wraps on the screen edge
    /// false (default): bounces on the screen edge
    pub wrap: bool,
}

/// A command to spawn the player character.
#[derive(Debug, Clone, Reflect, serde::Deserialize)]
pub struct CreatureDefinition {
    #[serde(default = "CreatureImage::fox")]
    pub image: CreatureImage,
    pub max_speed: f32,
    /// None is turned into a random position on screen
    pub pos: Option<Vec2>,
    pub movement: MovementPatternDefinition,
    #[serde(default = "default_shrink_duration")]
    pub shrink_duration_ms: u64,
    #[serde(default)]
    pub wrap: bool,
}

impl Command for SpawnCreature {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self, spawn_creature);
    }
}

fn spawn_creature(
    In(config): In<SpawnCreature>,
    mut commands: Commands,
    creature_assets: Res<CreatureAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // A texture atlas is a way to split one image with a grid into multiple
    // sprites. By attaching it to a [`SpriteBundle`] and providing an index, we
    // can specify which section of the image we want to see. We will use this
    // to animate our player character. You can learn more about texture atlases in
    // this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(
        config.image.size(),
        config.image.atlas_columns(),
        config.image.atlas_rows(),
        Some(UVec2::splat(1)),
        None,
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let creature_animation = CreatureAnimation::new(config.shrink_duration);

    let texture = match config.image {
        CreatureImage::Fox => creature_assets.fox.clone(),
        CreatureImage::Snake => creature_assets.snake.clone(),
        CreatureImage::Mouse => creature_assets.mouse.clone(),
        CreatureImage::Hedgehog => creature_assets.hedgehog.clone(),
        CreatureImage::Weasel => creature_assets.weasel.clone(),
        CreatureImage::Duck => creature_assets.ducky.clone(),
    };

    let scale = config.image.default_scale();
    let mut entity = commands.spawn((
        Name::new("Creature"),
        Creature,
        SpriteBundle {
            texture,
            transform: Transform::from_scale(Vec2::splat(scale).extend(1.0))
                .with_translation(config.pos.extend(1.0)),
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: 0,
        },
        MovementController {
            max_speed: config.max_speed,
            ..default()
        },
        config.movement,
        creature_animation,
        StateScoped(Screen::Gameplay),
        config.image,
    ));
    if config.wrap {
        entity.insert(ScreenWrap);
    } else {
        entity.insert(ScreenBounce);
    }
}

#[derive(Component, Clone, Reflect, Default)]
struct Bullet {
    pub timer: Timer,
}

#[derive(Component, Clone, Reflect, Default)]
struct FallingBulletMarker;

fn record_player_click_input(
    input: Res<ButtonInput<MouseButton>>,
    touches_input: Res<Touches>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    creature_assets: Res<CreatureAssets>,
    mut commands: Commands,
    bullets: Query<&Bullet, Without<FallingBulletMarker>>,
) {
    // There can be only one bullet at a time.
    if !bullets.is_empty() {
        return;
    } else {
        commands.insert_resource(HideGameCursor(false));
    }

    let (camera, camera_global_transform) = camera_query.single();
    let window = window_query.single();

    if input.just_pressed(MouseButton::Left) || touches_input.any_just_pressed() {
        if let Some(p) = window
            .cursor_position()
            .or_else(|| touches_input.first_pressed_position())
            .and_then(|cursor| camera.viewport_to_world_2d(camera_global_transform, cursor))
        {
            commands.spawn((
                Name::new("Bullet"),
                Bullet {
                    timer: Timer::from_seconds(BULLET_DURATION_SEC, TimerMode::Once),
                },
                SpriteBundle {
                    texture: creature_assets.ball.clone(),
                    transform: Transform::from_translation(p.extend(2.0)),
                    ..Default::default()
                },
                MovementController::default(),
            ));
            commands.spawn((
                AudioBundle {
                    source: creature_assets.shot.clone(),
                    settings: PlaybackSettings::DESPAWN,
                },
                SoundEffect,
            ));
            commands.insert_resource(HideGameCursor(true));
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
    pub fox: Handle<Image>,
    #[dependency]
    pub snake: Handle<Image>,
    #[dependency]
    pub mouse: Handle<Image>,
    #[dependency]
    pub weasel: Handle<Image>,
    #[dependency]
    pub hedgehog: Handle<Image>,
    #[dependency]
    pub ball: Handle<Image>,
    #[dependency]
    pub dust: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
    #[dependency]
    pub catch: Handle<AudioSource>,
    #[dependency]
    pub shot: Handle<AudioSource>,
    #[dependency]
    pub hit: Handle<AudioSource>,
    #[dependency]
    pub miss: Handle<AudioSource>,
}

impl CreatureAssets {
    pub const PATH_DUCKY: &'static str = "images/ducky.png";
    pub const PATH_FOX: &'static str = "images/fox.png";
    pub const PATH_SNAKE: &'static str = "images/snake.png";
    pub const PATH_MOUSE: &'static str = "images/mouse.png";
    pub const PATH_WEASEL: &'static str = "images/weasel.png";
    pub const PATH_HEDGEHOG: &'static str = "images/hedgehog.png";
    pub const PATH_BALL: &'static str = "images/ball.png";
    pub const PATH_DUST: &'static str = "images/dust.png";
    pub const PATH_STEP_1: &'static str = "audio/sound_effects/step1.ogg";
    pub const PATH_STEP_2: &'static str = "audio/sound_effects/step2.ogg";
    pub const PATH_STEP_3: &'static str = "audio/sound_effects/step3.ogg";
    pub const PATH_STEP_4: &'static str = "audio/sound_effects/step4.ogg";
    pub const PATH_CATCH: &'static str = "audio/sound_effects/catch.ogg";
    pub const PATH_SHOT: &'static str = "audio/sound_effects/shot.ogg";
    pub const PATH_HIT: &'static str = "audio/sound_effects/hit.ogg";
    pub const PATH_MISS: &'static str = "audio/sound_effects/miss.ogg";
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
            fox: assets.load_with_settings(
                CreatureAssets::PATH_FOX,
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve the pixel art style.
                    settings.sampler = ImageSampler::linear();
                },
            ),
            snake: assets.load_with_settings(
                CreatureAssets::PATH_SNAKE,
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve the pixel art style.
                    settings.sampler = ImageSampler::linear();
                },
            ),
            mouse: assets.load_with_settings(
                CreatureAssets::PATH_MOUSE,
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve the pixel art style.
                    settings.sampler = ImageSampler::linear();
                },
            ),
            hedgehog: assets.load_with_settings(
                CreatureAssets::PATH_HEDGEHOG,
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve the pixel art style.
                    settings.sampler = ImageSampler::linear();
                },
            ),
            weasel: assets.load_with_settings(
                CreatureAssets::PATH_WEASEL,
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve the pixel art style.
                    settings.sampler = ImageSampler::linear();
                },
            ),
            ball: assets.load(CreatureAssets::PATH_BALL),
            dust: assets.load(CreatureAssets::PATH_DUST),
            steps: vec![
                assets.load(CreatureAssets::PATH_STEP_1),
                assets.load(CreatureAssets::PATH_STEP_2),
                assets.load(CreatureAssets::PATH_STEP_3),
                assets.load(CreatureAssets::PATH_STEP_4),
            ],
            catch: assets.load(CreatureAssets::PATH_CATCH),
            shot: assets.load(CreatureAssets::PATH_SHOT),
            hit: assets.load(CreatureAssets::PATH_HIT),
            miss: assets.load(CreatureAssets::PATH_MISS),
        }
    }
}

#[derive(Reflect, Clone)]
struct KillCreature(Entity);

impl Command for KillCreature {
    fn apply(self, world: &mut World) {
        if let Some(mut atlas) = world.get_mut::<TextureAtlas>(self.0) {
            // index 1 is for shot creatures
            atlas.index = 1;
        }
        if let Some(mut movement) = world.get_mut::<MovementController>(self.0) {
            // freeze in place
            movement.intent_modifier = Vec2::ZERO;
        }
        if let Some(mut transform) = world.get_mut::<Transform>(self.0) {
            // put behind dust animation
            transform.translation.z = 0.4;
        }
        world.entity_mut(self.0).insert(DeathAnimation::new());
    }
}

#[derive(Component, Clone, Reflect, Default)]
pub struct DeathAnimation {
    pub timer: Timer,
    x_rotation_ms: f32,
    y_rotation_ms: f32,
}

fn reaper(
    mut commands: Commands,
    mut query: Query<(&DeathAnimation, &mut Transform, &CreatureImage, Entity)>,
    time: Res<Time>,
) {
    for (animation, mut transform, image, entity) in &mut query {
        if animation.timer.finished() {
            commands.entity(entity).despawn();
        }

        // quadratic animation: start animation fast and slot it down
        let explosive_entry = animation.timer.fraction_remaining().powi(2);

        let x_rotations =
            time.delta().as_millis() as f32 / animation.x_rotation_ms * explosive_entry;
        let y_rotations =
            time.delta().as_millis() as f32 / animation.y_rotation_ms * explosive_entry;
        transform.rotate_local_x(std::f32::consts::TAU * x_rotations);
        transform.rotate_local_y(std::f32::consts::TAU * y_rotations);

        // size reduction at quadratic speed from 1.0 to 0.25
        transform.scale = Vec3::splat(image.default_scale() * (0.25 + explosive_entry * 0.75));
    }
}

fn tick_death_animation(time: Res<Time>, mut query: Query<&mut DeathAnimation>) {
    for mut animation in &mut query.iter_mut() {
        animation.timer.tick(time.delta());
    }
}

impl DeathAnimation {
    fn new() -> Self {
        let rng = &mut rand::thread_rng();
        let dist = Uniform::from(75..400);

        DeathAnimation {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            x_rotation_ms: dist.sample(rng) as f32,
            y_rotation_ms: dist.sample(rng) as f32,
        }
    }
}
