use args::Args;
use automated_test_pluggin::{AutomatedTestPluggin, StartTestCaseEventWithIndex};
use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;
use dev_tools_pluggin::DevToolsPlugin;
use game_constants_pluggin::*;
use level_pluggin::{LevelPluggin, StartLevelEventWithIndex, StartTestLevelEventWithIndex};
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
mod movement_pluggin;
mod snake_pluggin;
mod test_levels;
mod undo;

// Don't touch this piece, needed for Web
#[cfg(target_arch = "wasm32")]
mod web_main;

pub fn run(app: &mut App, args: &Args) {
    app.insert_resource(ClearColor(DARK_COLOR_PALETTE[4]))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Bird Snake".to_string(),
                width: 640.0,
                height: 420.0,
                ..default()
            },
            ..default()
        }))
        .add_plugin(TweeningPlugin)
        .add_plugin(GameConstantsPlugin)
        .add_plugin(DevToolsPlugin)
        .add_plugin(SnakePluggin)
        .add_plugin(LevelPluggin)
        .add_plugin(MovementPluggin)
        .add_system(bevy::window::close_on_esc);

    match args.command {
        Some(args::Commands::Test { test_case }) => {
            app.add_plugin(AutomatedTestPluggin);

            let start_test_case =
                move |mut event_writer: EventWriter<StartTestCaseEventWithIndex>| {
                    let start_test_case = test_case.unwrap_or(0);
                    event_writer.send(StartTestCaseEventWithIndex(start_test_case));
                };
            app.add_startup_system(start_test_case);
        }
        None => {
            match args.test_level {
                Some(test_level) => {
                    let startup =
                        move |mut event_writer: EventWriter<StartTestLevelEventWithIndex>| {
                            event_writer.send(StartTestLevelEventWithIndex(test_level));
                        };
                    app.add_startup_system(startup);
                }
                None => {
                    let start_level = args.level;
                    let startup = move |mut event_writer: EventWriter<StartLevelEventWithIndex>| {
                        let start_level = start_level.unwrap_or(0);
                        event_writer.send(StartLevelEventWithIndex(start_level));
                    };
                    app.add_startup_system(startup);
                }
            };
        }
    };

    app.run();
}
