//! The screen state for the main gameplay.

use crate::demo::level::{WaveCounter, WaveTimer};
use crate::theme::prelude::*;
use bevy::audio::Volume;
use bevy::window::PrimaryWindow;
use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{asset_tracking::LoadResource, audio::Music, screens::Screen};

pub const HEADER_SIZE: f32 = 65.0;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<GameplayMusic>();
    app.init_resource::<DevGameplay>();
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (play_gameplay_music, reset_dev_portal),
    );
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_game_background, set_gameplay_area),
    );
    app.add_systems(OnExit(Screen::Gameplay), (stop_music, remove_background));

    app.add_systems(
        Update,
        return_to_title_screen
            .run_if(in_state(Screen::Gameplay).and_then(input_just_pressed(KeyCode::Escape))),
    );
    app.add_systems(Update, dev_mode_portal);
    app.add_systems(
        Update,
        update_wave_timer.run_if(resource_exists::<WaveTimer>),
    );
    app.add_systems(
        Update,
        update_wave_number.run_if(resource_exists::<WaveCounter>),
    );
}

#[derive(Component, Debug, Clone, Reflect)]
struct WaveTimerLabel;

#[derive(Component, Debug, Clone, Reflect)]
struct WaveNumber;

// Modifies the UI to show the time left in the wave.
fn update_wave_timer(
    timer: Res<WaveTimer>,
    parent_query: Query<&Children, With<WaveTimerLabel>>,
    mut child_query: Query<&mut Text>,
) {
    let children = parent_query.single();
    for &child in children.iter() {
        let mut text = child_query.get_mut(child).unwrap();
        text.sections[0].value = format!("Time left: {:.0}s", timer.0.remaining_secs().ceil());
    }
}

fn update_wave_number(
    wave_counter: Res<WaveCounter>,
    parent_query: Query<&Children, With<WaveNumber>>,
    mut child_query: Query<&mut Text>,
) {
    let children = parent_query.single();
    for &child in children.iter() {
        let mut text = child_query.get_mut(child).unwrap();
        text.sections[0].value = format!("Wave: {}", wave_counter.wave);
    }
}

fn spawn_game_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background_image = asset_server.load("images/background.png");
    commands
        .spawn(SpriteBundle {
            texture: background_image,
            ..Default::default()
        })
        .insert(Background);

    commands
        .top_panel()
        .insert(StateScoped(Screen::Gameplay))
        .with_children(|children| {
            children
                .header("Time left: 30s".to_owned())
                .insert(WaveTimerLabel);

            children.header("Wave: 1".to_owned()).insert(WaveNumber);
        });
}

fn remove_background(mut commands: Commands, query: Query<Entity, With<Background>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
struct Background;

#[derive(Debug, Clone, Reflect, Resource)]
pub struct GameplayArea {
    pub main_area: Rect,
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct GameplayMusic {
    #[dependency]
    handle: Handle<AudioSource>,
    entity: Option<Entity>,
}

impl FromWorld for GameplayMusic {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            handle: assets.load("audio/music/music.ogg"),
            entity: None,
        }
    }
}

fn play_gameplay_music(mut commands: Commands, mut music: ResMut<GameplayMusic>) {
    music.entity = Some(
        commands
            .spawn((
                AudioBundle {
                    source: music.handle.clone(),
                    settings: PlaybackSettings::LOOP.with_volume(Volume::new(0.3)),
                },
                Music,
            ))
            .id(),
    );
}

fn stop_music(mut commands: Commands, mut music: ResMut<GameplayMusic>) {
    if let Some(entity) = music.entity.take() {
        commands.entity(entity).despawn_recursive();
    }
}

fn set_gameplay_area(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    let mut main_area = Rect::from_center_size(Vec2::ZERO, window.size());
    // subtract a few px for the header
    main_area.max.y -= HEADER_SIZE;
    commands.insert_resource(GameplayArea { main_area });
}

fn return_to_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

/// State for a dev view
#[derive(Resource, Reflect, Clone, Default)]
pub struct DevGameplay {
    enter_dev_mode_counter: u32,
}

fn dev_mode_portal(
    mut next_screen: ResMut<NextState<Screen>>,
    mut state: ResMut<DevGameplay>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // D is for dev
    if keys.just_pressed(KeyCode::KeyD) {
        state.enter_dev_mode_counter += 1;
    } else if keys.get_just_pressed().next().is_some() {
        // a different key was pressed, reset dev counter
        state.enter_dev_mode_counter = 0;
    }
    if state.enter_dev_mode_counter >= 3 {
        state.enter_dev_mode_counter = 0;
        next_screen.set(Screen::Dev);
    }
}

fn reset_dev_portal(mut state: ResMut<DevGameplay>) {
    state.enter_dev_mode_counter = 0;
}
