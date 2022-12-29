use bevy::prelude::*;

use bevy_inspector_egui::{Inspectable, InspectorPlugin};

pub const GRID_TO_WORLD_UNIT: f32 = 25.;
pub const GRID_TO_WORLD_UNIT_INVERSE: f32 = 1. / GRID_TO_WORLD_UNIT;
pub const SNAKE_SIZE: Vec2 = Vec2::splat(GRID_TO_WORLD_UNIT);
pub const GRID_CELL_SIZE: Vec2 = SNAKE_SIZE;
pub const MOVE_START_VELOCITY: f32 = 4.0;
pub const JUMP_START_VELOCITY: f32 = 65.0;
pub const GRAVITY: f32 = 300.0;

pub fn to_world(position: IVec2) -> Vec2 {
    (position.as_vec2() + 0.5) * GRID_TO_WORLD_UNIT
}

#[derive(Resource, Inspectable)]
pub struct GameConstants {
    #[inspectable(min = 0.0, max = 10.0)]
    pub move_velocity: f32,

    #[inspectable(min = 0.0, max = 300.0)]
    pub jump_velocity: f32,

    #[inspectable(min = 0.0, max = 900.0)]
    pub gravity: f32,
}

impl Default for GameConstants {
    fn default() -> Self {
        Self {
            move_velocity: MOVE_START_VELOCITY,
            jump_velocity: JUMP_START_VELOCITY,
            gravity: GRAVITY,
        }
    }
}
pub struct GameConstantsPlugin;

impl Plugin for GameConstantsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InspectorPlugin::<GameConstants>::new());
    }
}
