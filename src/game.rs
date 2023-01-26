use args::Args;
use bevy::prelude::*;
use gameplay::game_constants_pluggin::*;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet},
    state::NextState,
};
use level::level_pluggin::{
    ClearLevelEvent, LevelEntity, LevelPluggin, StartLevelEventWithIndex,
    StartTestLevelEventWithIndex,
};
use menu::MenuPlugin;
use gameplay::movement_pluggin::MovementPluggin;
use gameplay::snake_pluggin::SnakePluggin;
use tools::automated_test_pluggin::{AutomatedTestPluggin, StartTestCaseEventWithIndex};
use tools::dev_tools_pluggin::DevToolsPlugin;

pub mod args;
mod gameplay;
mod level;
mod menu;
mod tools;

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
    }
}

fn back_to_menu_on_escape_system(
    mut event_clear_level: EventWriter<ClearLevelEvent>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        event_clear_level.send(ClearLevelEvent);
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
        .add_loopless_state_before_stage(CoreStage::PreUpdate, start_state)
        .add_plugin(MenuPlugin)
        .add_plugin(GamePlugin { args: args.clone() })
        .run();
}
