//! Spawn the main level.

use bevy::ecs::system::RunSystemOnce;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use rand::distributions::{Distribution, Uniform};

use crate::asset_tracking::LoadResource;
use crate::audio::SoundEffect;
use crate::demo::creature::CreatureDefinition;
use crate::demo::creature::SpawnCreature;
use crate::screens::GameScore;
use crate::screens::GameplayArea;
use crate::screens::Screen;
use crate::AppSet;

use std::time::Duration;

use super::creature::Creature;
use super::creature::DeathAnimation;

const WAVE_DURATION: Duration = Duration::from_secs(20);

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct WaveTimer(pub Timer);

#[derive(Resource, Debug, Clone, PartialEq, Reflect, Default)]
#[reflect(Resource)]
pub struct WaveCounter {
    pub wave: u32,
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct WaveSound {
    #[dependency]
    sound: Handle<AudioSource>,
}

#[derive(Debug, Resource, Clone, Reflect, Asset)]
pub struct Levels {
    /// The levels that run through when we play the game normally. (As opposed
    /// to loading other levels in dev mode.)
    #[dependency]
    game_levels: Vec<Handle<LevelDefinition>>,
}

/// A definition of a single level, loaded from a RON file or directly defined in Rust
#[derive(Debug, Clone, Reflect, Asset, serde::Deserialize)]
pub struct LevelDefinition {
    creatures: Vec<CreatureDefinition>,
}

#[derive(Clone, Reflect, Resource, Default, PartialEq)]
pub struct DevMode(pub bool);

impl FromWorld for Levels {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        // To add more levels for the game, create a file in assets/levels/ that
        // ends with .level.rin and add its path here.
        let levels = [
            "levels/00_easy_start.level.ron",
            "levels/01_more_easy_creatures.level.ron",
            "levels/02_few_periodic.level.ron",
            "levels/03_mixed_periodic.level.ron",
            "levels/04_mixed_circles.level.ron",
            "levels/05.level.ron",
            "levels/06.level.ron",
            "levels/07.level.ron",
        ];
        Levels {
            game_levels: levels
                .into_iter()
                .map(|path| assets.load::<LevelDefinition>(path))
                .collect::<Vec<_>>(),
        }
    }
}

impl FromWorld for WaveSound {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            sound: assets.load("audio/sound_effects/wave_start.ogg"),
        }
    }
}

impl Default for WaveTimer {
    fn default() -> Self {
        Self(Timer::new(WAVE_DURATION, TimerMode::Once))
    }
}

pub(super) fn plugin(app: &mut App) {
    // Configure that ***.level.ron files loaded as assets map to a `LevelDefinition`.
    app.add_plugins(RonAssetPlugin::<LevelDefinition>::new(&["level.ron"]));

    app.register_type::<WaveTimer>();
    app.load_resource::<WaveSound>();
    app.load_resource::<Levels>();
    app.init_resource::<DevMode>();
    app.add_systems(OnEnter(Screen::Gameplay), add_resources);
    app.add_systems(OnExit(Screen::Gameplay), remove_resources);

    app.add_systems(
        Update,
        (
            tick_wave_timer.in_set(AppSet::TickTimers),
            (check_wave_spawn, check_wave_timer)
                .chain()
                .in_set(AppSet::Update)
                .run_if(resource_equals(DevMode(false))),
        )
            .run_if(resource_exists::<WaveSound>)
            .run_if(in_state(Screen::Gameplay)),
    );
}

fn add_resources(mut commands: Commands) {
    commands.insert_resource(WaveTimer::default());
    commands.insert_resource(WaveCounter::default());
}

fn remove_resources(mut commands: Commands) {
    commands.remove_resource::<WaveTimer>();
    commands.remove_resource::<WaveCounter>();
}

fn tick_wave_timer(time: Res<Time>, mut timer: ResMut<WaveTimer>) {
    timer.0.tick(time.delta());
}

fn check_wave_spawn(
    mut timer: ResMut<WaveTimer>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut commands: Commands,
    mut wave_counter: ResMut<WaveCounter>,
    mut game_score: ResMut<GameScore>,
    level_handles: Res<Levels>,
    creatures: Query<Entity, With<Creature>>,
) {
    // If it's a first wave or the wave was cleared.
    if wave_counter.wave == 0 || creatures.iter().len() == 0 {
        // Last level done.
        let Some(level_handle) = level_handles.game_levels.get(wave_counter.wave as usize) else {
            game_score.win = true;
            next_screen.set(Screen::Score);
            return;
        };
        game_score.score = wave_counter.wave;
        wave_counter.wave += 1;
        commands.add(SpawnLevel(level_handle.clone()));
        timer.0.reset();
    }
}

fn check_wave_timer(
    mut next_screen: ResMut<NextState<Screen>>,
    timer: Res<WaveTimer>,
    alive_creatures: Query<&Creature, Without<DeathAnimation>>,
    mut game_score: ResMut<GameScore>,
) {
    // This means we've lost.
    if timer.0.just_finished() && !alive_creatures.is_empty() {
        game_score.win = false;
        next_screen.set(Screen::Score);
    }
}

pub(crate) struct SpawnLevel(pub Handle<LevelDefinition>);

impl Command for SpawnLevel {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self, spawn_level);
    }
}

fn spawn_level(
    In(SpawnLevel(level_handle)): In<SpawnLevel>,
    mut commands: Commands,
    gameplay_area: Res<GameplayArea>,
    levels: Res<Assets<LevelDefinition>>,
    sound: Res<WaveSound>,
) {
    let Some(level) = levels.get(&level_handle) else {
        // level not loaded, yet
        println!("Loading order wrong, level {level_handle:?} has not been loaded when it should have been spawned");
        return;
    };

    commands.spawn((
        AudioBundle {
            source: sound.sound.clone(),
            settings: PlaybackSettings::DESPAWN,
        },
        SoundEffect,
    ));

    let mut rng = &mut rand::thread_rng();
    let size = gameplay_area.main_area.size() - 256.0;
    let half_size = size / 2.0;
    let x_dist = Uniform::from(-half_size.x..half_size.x);
    let y_dist = Uniform::from(-half_size.y..half_size.y);
    let mut random_screen_pos = || Vec2 {
        x: x_dist.sample(&mut rng),
        y: y_dist.sample(&mut rng),
    };

    for creature in &level.creatures {
        commands.add(SpawnCreature {
            image: creature.image,
            max_speed: creature.max_speed,
            pos: creature.pos.unwrap_or_else(&mut random_screen_pos),
            movement: creature.movement.build(),
            shrink_duration: Duration::from_millis(creature.shrink_duration_ms),
            wrap: creature.wrap,
        });
    }
}
