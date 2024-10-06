//! The game's main screen states and transitions between them.

mod credits;
mod dev_view;
mod gameplay;
mod loading;
mod score;
mod splash;
mod title;

use bevy::prelude::*;
pub use gameplay::GameplayArea;

use crate::theme::palette::THEME_MALACHITE_DARK;

#[derive(Resource, Reflect, Clone, Default)]
pub struct GameScore {
    pub score: u32,
    pub win: bool,
}

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.enable_state_scoped_entities::<Screen>();

    // Accessed in gameplay and score screen.
    app.insert_resource(GameScore::default());
    app.insert_resource(ClearColor(THEME_MALACHITE_DARK));

    app.add_plugins((
        credits::plugin,
        gameplay::plugin,
        loading::plugin,
        splash::plugin,
        title::plugin,
        score::plugin,
        dev_view::plugin,
    ));
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum Screen {
    Splash,
    // TODO: Revert to splash on release.
    #[default]
    Loading,
    Title,
    Credits,
    Gameplay,
    Score,
    Dev,
}
