use std::collections::VecDeque;

use bevy::{prelude::*, time::FixedTimestep};

use crate::{
    game_constants_pluggin::*,
    level_pluggin::{
        load_level_system, CurrentLevelId, StartLevelEventWithLevel, LOAD_LEVEL_STAGE,
    },
    movement_pluggin::MoveCommandEvent,
    test_levels::*,
};

#[derive(Clone)]
struct TestInputCommand(IVec2);

#[derive(Resource, Clone)]
struct TestCase {
    level: &'static str,
    moves: VecDeque<TestInputCommand>,
}

#[derive(Resource)]
struct TestCases {
    cases: Vec<TestCase>,
}

macro_rules! test_case {
    ($name:ident, $($move:ident,)+) => {
        TestCase {
            level: $name,
            moves: VecDeque::from([$(TestInputCommand($move),)+]),
        }
    };
}

macro_rules! test_cases {
    ($($case:expr,)*) => {
        TestCases {
            cases: vec![
                $($case,)*
            ],
        }
    };
}

pub struct StartTestCaseEventWithIndex(pub usize);

pub struct AutomatedTestPluggin;

impl Plugin for AutomatedTestPluggin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartTestCaseEventWithIndex>()
            .add_startup_system(init_automation)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(1.0))
                    .with_system(moc_player_input),
            )
            .add_system_to_stage(LOAD_LEVEL_STAGE, start_test_case.before(load_level_system));
    }
}

fn moc_player_input(
    mut test_case: ResMut<TestCase>,
    mut move_command_event: EventWriter<MoveCommandEvent>,
) {
    let Some(next_move) = test_case.moves.pop_front() else {
        return;
    };

    move_command_event.send(MoveCommandEvent(next_move.0));
}

fn start_test_case(
    test_cases: Res<TestCases>,
    mut commands: Commands,
    mut event_start_level: EventWriter<StartLevelEventWithLevel>,
    mut event_reader: EventReader<StartTestCaseEventWithIndex>,
) {
    let Some(event) = event_reader.iter().next() else {
        return;
    };

    commands.insert_resource(CurrentLevelId(event.0));

    let new_test_case = &test_cases.cases[event.0];
    commands.insert_resource(new_test_case.clone());

    event_start_level.send(StartLevelEventWithLevel(new_test_case.level.to_owned()));
}

fn init_automation(mut commands: Commands) {
    let test_cases = test_cases! {
        test_case!(FALL_ON_SNAKE_BUG, RIGHT,),
        test_case!(REACH_GOAL_FALLING, RIGHT,),
        test_case!(FALL_ON_SPIKE, RIGHT, RIGHT,),
    };

    commands.insert_resource(test_cases);
}
