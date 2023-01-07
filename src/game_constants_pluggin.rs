use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

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

pub fn to_grid(position: Vec2) -> IVec2 {
    (position * GRID_TO_WORLD_UNIT_INVERSE - 0.5)
        .round()
        .as_ivec2()
}

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct GameConstants {
    #[inspector(min = 0.0, max = 10.0)]
    pub move_velocity: f32,

    #[inspector(min = 0.0, max = 300.0)]
    pub jump_velocity: f32,

    #[inspector(min = 0.0, max = 900.0)]
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
        app.register_type::<GameConstants>();
        app.insert_resource(GameConstants::default());
    }
}
