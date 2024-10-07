//! A credits screen that can be accessed from the title screen.

use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, audio::Music, screens::Screen, theme::prelude::*};

use super::title::UiAssets;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Credits), spawn_credits_screen);

    app.load_resource::<CreditsMusic>();
    app.add_systems(OnEnter(Screen::Credits), play_credits_music);
    app.add_systems(OnExit(Screen::Credits), stop_music);
}

fn spawn_credits_screen(mut commands: Commands, assets: Res<UiAssets>) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Credits))
        .with_children(|children| {
            children.header("Made by", &assets);
            children.label_on_background(
                "Jakob Meier - Coding\nAndrew Borg - Coding\nMartin Villavicencio - Art",
                3,
            );

            children.header("Assets", &assets);
            children.label_on_background("Button SFx - CC0 by Jaszunio15\nGameplay SFx - lovely people at Freesound\nMusic - by JewelBeat", 3);

            children.button("Back").observe(enter_title_screen);
        });
}

fn enter_title_screen(_trigger: Trigger<OnPress>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct CreditsMusic {
    #[dependency]
    music: Handle<AudioSource>,
    entity: Option<Entity>,
}

impl FromWorld for CreditsMusic {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Monkeys Spinning Monkeys.ogg"),
            entity: None,
        }
    }
}

fn play_credits_music(mut commands: Commands, mut music: ResMut<CreditsMusic>) {
    music.entity = Some(
        commands
            .spawn((
                AudioBundle {
                    source: music.music.clone(),
                    settings: PlaybackSettings::LOOP,
                },
                Music,
            ))
            .id(),
    );
}

fn stop_music(mut commands: Commands, mut music: ResMut<CreditsMusic>) {
    if let Some(entity) = music.entity.take() {
        commands.entity(entity).despawn_recursive();
    }
}
