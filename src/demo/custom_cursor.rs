use bevy::prelude::*;

use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), setup);
    app.add_systems(OnExit(Screen::Gameplay), reset_cursor);
    app.add_systems(Update, move_cursor.run_if(in_state(Screen::Gameplay)));
    app.init_resource::<HideGameCursor>();
}

#[derive(Component)]
struct GameCursor;

#[derive(Resource, Default)]
pub struct HideGameCursor(pub bool);

// taken from https://github.com/bevyengine/bevy/discussions/8613
fn setup(mut windows: Query<&mut Window>, mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut window: Mut<Window> = windows.single_mut();
    window.cursor.visible = false;
    let cursor_spawn: Vec3 = Vec3::ZERO;

    commands.spawn((
        ImageBundle {
            image: asset_server.load("images/ball.png").into(),
            style: Style {
                position_type: PositionType::Absolute,
                // position: UiRect::all(Val::Auto),
                ..default()
            },
            z_index: ZIndex::Global(15),
            transform: Transform::from_translation(cursor_spawn),
            ..default()
        },
        GameCursor,
    ));
}

fn reset_cursor(
    mut windows: Query<&mut Window>,
    mut commands: Commands,
    cursor_query: Query<(&GameCursor, Entity)>,
) {
    let mut window: Mut<Window> = windows.single_mut();
    window.cursor.visible = true;
    for (_cursor, entity) in &cursor_query {
        commands.entity(entity).despawn();
    }
}

// taken and modified from https://github.com/bevyengine/bevy/discussions/8613
fn move_cursor(
    window: Query<&Window>,
    mut cursor: Query<&mut Style, With<GameCursor>>,
    hide: Res<HideGameCursor>,
) {
    let window: &Window = window.single();
    if let Some(position) = window.cursor_position() {
        let mut img_style = cursor.single_mut();
        img_style.left = Val::Px(position.x - 24.0); // subtract half size of cursor image
        img_style.top = Val::Px(position.y - 24.0);
        if hide.0 {
            img_style.display = Display::None;
        } else {
            img_style.display = Display::DEFAULT;
        }
    }
}
