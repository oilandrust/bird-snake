use automated_test_pluggin::{AutomatedTestPluggin, StartTestCaseEventWithIndex};
use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;
use dev_tools_pluggin::DevToolsPlugin;
use game_constants_pluggin::*;
use level_pluggin::{LevelPluggin, StartLevelEventWithIndex, StartTestLevelEventWithIndex};
use movement_pluggin::MovementPluggin;
use snake_pluggin::SnakePluggin;

mod automated_test_pluggin;
mod commands;
mod dev_tools_pluggin;
mod game_constants_pluggin;
mod level_pluggin;
mod level_template;
mod levels;
mod movement_pluggin;
mod snake_pluggin;
mod test_levels;
mod undo;

use clap::{Parser, Subcommand};

/// Cli API.
/// Run a level
/// ./snake-bird -l 0
/// ./snake-bird --level 0
/// Run a test level
/// ./snake-bird -t 0
/// ./snake-bird --test_level 0
/// // Run the automated tests
/// ./snake-bird test
/// // Run the automated tests for a specific test case
/// ./snake-bird -t 0 test

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    level: Option<usize>,

    #[arg(short, long)]
    test_level: Option<usize>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run automated tests.
    Test {
        #[arg(short, long)]
        test_case: Option<usize>,
    },
}

fn main() {
    let args = Args::parse();

    let mut app = App::new();

    app.insert_resource(ClearColor(DARK_COLOR_PALETTE[4]))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Snake".to_string(),
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
        Some(Commands::Test { test_case }) => {
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
