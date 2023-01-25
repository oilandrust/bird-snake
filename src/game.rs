use args::Args;
use automated_test_pluggin::{AutomatedTestPluggin, StartTestCaseEventWithIndex};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::TweeningPlugin;
use dev_tools_pluggin::DevToolsPlugin;
use game_constants_pluggin::*;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet},
    state::NextState,
};
use level_pluggin::{
    LevelEntity, LevelPluggin, StartLevelEventWithIndex, StartTestLevelEventWithIndex,
};
use menu::MenuPlugin;
use movement_pluggin::MovementPluggin;
use snake_pluggin::SnakePluggin;

pub mod args;
mod automated_test_pluggin;
mod commands;
mod dev_tools_pluggin;
mod game_constants_pluggin;
mod level_instance;
mod level_pluggin;
mod level_template;
mod levels;
mod menu;
mod movement_pluggin;
mod snake_pluggin;
mod test_levels;
mod undo;

// Don't touch this piece, needed for Web
#[cfg(target_arch = "wasm32")]
mod web_main;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Menu,
    Game,
}

pub struct GamePlugin {
    args: Args,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        match self.args.command {
            Some(args::Commands::Test { test_case }) => {
                app.add_plugin(AutomatedTestPluggin);

                let start_test_case =
                    move |mut event_writer: EventWriter<StartTestCaseEventWithIndex>| {
                        let start_test_case = test_case.unwrap_or(0);
                        event_writer.send(StartTestCaseEventWithIndex(start_test_case));
                    };
                app.add_enter_system(GameState::Game, start_test_case);
            }
            None => {
                match self.args.test_level {
                    Some(test_level) => {
                        let startup =
                            move |mut event_writer: EventWriter<StartTestLevelEventWithIndex>| {
                                event_writer.send(StartTestLevelEventWithIndex(test_level));
                            };
                        app.add_enter_system(GameState::Game, startup);
                    }
                    None => {
                        let start_level = self.args.level;
                        let startup =
                            move |mut event_writer: EventWriter<StartLevelEventWithIndex>| {
                                let start_level = start_level.unwrap_or(0);
                                event_writer.send(StartLevelEventWithIndex(start_level));
                            };
                        app.add_enter_system(GameState::Game, startup);
                    }
                };
            }
        };

        app.add_exit_system(GameState::Game, despawn_with::<LevelEntity>)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Game)
                    .with_system(back_to_menu_on_escape_system)
                    .into(),
            )
            .add_plugin(LevelPluggin)
            .add_plugin(SnakePluggin)
            .add_plugin(MovementPluggin)
            .add_plugin(GameConstantsPlugin)
            .add_plugin(DevToolsPlugin);
    }
}

fn back_to_menu_on_escape_system(mut commands: Commands, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(GameState::Menu));
    }
}

pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn run(app: &mut App, args: &Args) {
    let start_state = if args.command.is_none() && args.level.is_none() && args.test_level.is_none()
    {
        GameState::Menu
    } else {
        GameState::Game
    };

    app.insert_resource(ClearColor(DARK_COLOR_PALETTE[4]))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Bird Snake".to_string(),
                width: 640.0,
                height: 420.0,
                ..default()
            },
            ..default()
        }))
        .add_loopless_state(start_state)
        .add_plugin(MenuPlugin)
        .add_plugin(GamePlugin { args: args.clone() })
        .run();
}
