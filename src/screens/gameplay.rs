//! The screen state for the main gameplay.

use bevy::audio::Volume;
use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{asset_tracking::LoadResource, audio::Music, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<GameplayMusic>();
    app.init_resource::<DevGameplay>();
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (play_gameplay_music, reset_dev_portal),
    );
    app.add_systems(OnExit(Screen::Gameplay), stop_music);

    app.add_systems(
        Update,
        return_to_title_screen
            .run_if(in_state(Screen::Gameplay).and_then(input_just_pressed(KeyCode::Escape))),
    );
    app.add_systems(Update, dev_mode_portal);
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
            handle: assets.load("audio/music/Fluffing A Duck.ogg"),
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
                    settings: PlaybackSettings::LOOP.with_volume(Volume::ZERO),
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
