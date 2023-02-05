use bevy::{prelude::*, sprite::Material2dPlugin};
use iyes_loopless::prelude::{ConditionHelpers, IntoConditionalSystem};

use crate::{level::level_instance::LevelInstance, GameState};

use self::water::{animate_water, spawn_water_system, WaterMaterial};

pub mod water;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<WaterMaterial>::default())
            .add_system(spawn_water_system.run_in_state(GameState::Game))
            .add_system(
                animate_water
                    .run_in_state(GameState::Game)
                    .run_if_resource_exists::<LevelInstance>(),
            );
    }
}
