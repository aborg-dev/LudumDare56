use crate::asset_tracking::LoadResource;
use crate::demo::level::{DevMode, LevelDefinition, SpawnLevel};
use crate::{screens::Screen, theme::prelude::*};
use bevy::asset::{LoadedFolder, UntypedAssetId, VisitAssetDependencies};
use bevy::prelude::*;

use super::title::UiAssets;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Dev), spawn_dev_screen);
    app.load_resource::<LevelsFolder>();
}

fn spawn_dev_screen(
    mut commands: Commands,
    levels: Res<LevelsFolder>,
    folder_assets: Res<Assets<LoadedFolder>>,
    assets: Res<UiAssets>,
) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Dev))
        .with_children(|children| {
            children.header("Developer Settings", &assets);

            children
                .button("Back to game")
                .observe(enter_gameplay_screen);

            children
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceEvenly,
                        column_gap: Val::Px(3.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|columns| {
                    for column_index in 0..3 {
                        columns
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::all(Val::Px(10.0)),
                                    margin: UiRect::all(Val::Px(15.0)),
                                    row_gap: Val::Px(3.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|column| {
                                let mut button_count = 0;
                                levels
                                    .0
                                    .visit_dependencies(&mut |asset_id: UntypedAssetId| {
                                        if let Ok(folder_id) = asset_id.try_typed::<LoadedFolder>()
                                        {
                                            if let Some(folder) = folder_assets.get(folder_id) {
                                                for handle in &folder.handles {
                                                    if let Ok(level_handle) = handle
                                                        .clone()
                                                        .try_typed::<LevelDefinition>(
                                                    ) {
                                                        let path = level_handle.path().unwrap();
                                                        let file_name = path
                                                            .path()
                                                            .file_stem()
                                                            .unwrap()
                                                            .to_str()
                                                            .unwrap();
                                                        let name = file_name
                                                            .strip_suffix(".level")
                                                            .unwrap_or(file_name);

                                                        if button_count % 3 == column_index {
                                                            column.button(name).observe(
                                                                enter_level(level_handle.clone()),
                                                            );
                                                        }
                                                        button_count += 1;
                                                    }
                                                }
                                            }
                                        }
                                    });
                            });
                    }
                });

            // levels
            //     .0
            //     .visit_dependencies(&mut |asset_id: UntypedAssetId| {
            //         if let Ok(folder_id) = asset_id.try_typed::<LoadedFolder>() {
            //             if let Some(folder) = folder_assets.get(folder_id) {
            //                 for handle in &folder.handles {
            //                     if let Ok(level_handle) =
            //                         handle.clone().try_typed::<LevelDefinition>()
            //                     {
            //                         // e.g. "levels/00_level_name.level.ron"
            //                         let path = level_handle.path().unwrap();
            //                         // e.g. "00_level_name.level"
            //                         let file_name =
            //                             path.path().file_stem().unwrap().to_str().unwrap();
            //                         let name =
            //                             file_name.strip_suffix(".level").unwrap_or(file_name);
            //                         children.button(name).observe(enter_level(level_handle));
            //                     }
            //                 }
            //             }
            //         }
            //     });
        });
}

fn enter_gameplay_screen(
    _trigger: Trigger<OnPress>,
    mut commands: Commands,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    commands.insert_resource(DevMode(false));
    next_screen.set(Screen::Gameplay);
}

fn enter_level(
    level_handle: Handle<LevelDefinition>,
) -> impl Fn(Trigger<OnPress>, Commands, ResMut<NextState<Screen>>) {
    move |_trigger: Trigger<OnPress>,
          mut commands: Commands,
          mut next_screen: ResMut<NextState<Screen>>| {
        commands.insert_resource(DevMode(true));
        next_screen.set(Screen::Gameplay);
        commands.add(SpawnLevel(level_handle.clone()));
    }
}

#[derive(Clone, Resource, Asset, Reflect)]
struct LevelsFolder(#[dependency] Handle<LoadedFolder>);

impl FromWorld for LevelsFolder {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        LevelsFolder(assets.load_folder("levels"))
    }
}
