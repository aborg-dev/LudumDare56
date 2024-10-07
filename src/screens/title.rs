//! The title screen that appears when the game starts.

use bevy::prelude::*;

use crate::{screens::Screen, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), spawn_title_screen);
}

fn spawn_title_screen(mut commands: Commands, assets: Res<UiAssets>) {
    commands.spawn(SpriteBundle {
        texture: assets.background.clone(),
        ..Default::default()
    });

    commands
        .ui_root()
        .insert(StateScoped(Screen::Title))
        .with_children(|children| {
            children.large_message("Animal Arcade", &assets);

            children.button("Play").observe(enter_gameplay_screen);
            children.button("Credits").observe(enter_credits_screen);

            #[cfg(not(target_family = "wasm"))]
            children.button("Exit").observe(exit_app);
        });
}

fn enter_gameplay_screen(_trigger: Trigger<OnPress>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Gameplay);
}

fn enter_credits_screen(_trigger: Trigger<OnPress>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_trigger: Trigger<OnPress>, mut app_exit: EventWriter<AppExit>) {
    app_exit.send(AppExit::Success);
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct UiAssets {
    #[dependency]
    pub background: Handle<Image>,
    #[dependency]
    pub sign: Handle<Image>,
    #[dependency]
    pub font: Handle<Font>,
}

impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            background: assets.load("images/background.png"),
            sign: assets.load("images/sign.png"),
            font: assets.load("fonts/StyleScript-Regular.ttf"),
        }
    }
}
