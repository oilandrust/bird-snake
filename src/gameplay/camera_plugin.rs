use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    math::Vec3Swizzles,
    prelude::*,
};
use iyes_loopless::prelude::{ConditionHelpers, ConditionSet, IntoConditionalSystem};

use crate::{
    level::{level_instance::LevelInstance, level_template::LevelTemplate},
    GameState,
};

use super::{
    game_constants_pluggin::{to_world, GRID_TO_WORLD_UNIT},
    level_pluggin::{LevelEntity, StartLevelEventWithLevel},
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            camera_setup_system
                .run_in_state(GameState::Game)
                .run_if_resource_exists::<LevelInstance>(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Game)
                .run_if_resource_exists::<LevelInstance>()
                .with_system(camera_zoom_scroll_system)
                .with_system(camera_pan_system)
                .into(),
        );
    }
}

fn camera_setup_system(
    mut commands: Commands,
    mut event_start_level: EventReader<StartLevelEventWithLevel>,
    level_template: Res<LevelTemplate>,
) {
    if event_start_level.iter().next().is_none() {
        return;
    }

    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_xyz(
                level_template.grid.width() as f32 * GRID_TO_WORLD_UNIT * 0.5,
                level_template.grid.height() as f32 * GRID_TO_WORLD_UNIT * 0.5,
                50.0,
            ),
            projection: OrthographicProjection {
                scale: 0.72,
                ..default()
            },
            ..default()
        })
        .insert(LevelEntity);
}

fn camera_zoom_scroll_system(
    mut scroll_event: EventReader<MouseWheel>,
    mut camera: Query<&mut OrthographicProjection>,
) {
    let mut projection = camera.single_mut();

    const SCALE_MAX: f32 = 1.5;
    const SCALE_MIN: f32 = 0.5;

    for event in scroll_event.iter() {
        match event.unit {
            MouseScrollUnit::Line => {
                projection.scale -= 0.05 * event.y;
                projection.scale = projection.scale.clamp(SCALE_MIN, SCALE_MAX);
            }
            MouseScrollUnit::Pixel => {
                projection.scale -= 0.005 * event.y;
                projection.scale = projection.scale.clamp(SCALE_MIN, SCALE_MAX);
            }
        }
    }
}

fn camera_pan_system(
    mut motion_event: EventReader<MouseMotion>,
    buttons: Res<Input<MouseButton>>,
    mut camera: Query<&mut GlobalTransform, With<Camera>>,
    level_template: Res<LevelTemplate>,
) {
    if !buttons.pressed(MouseButton::Right) {
        return;
    }
    let mut camera_transform = camera.single_mut();

    let pos_max = to_world(IVec2::new(
        level_template.grid.width() as i32,
        level_template.grid.height() as i32,
    ));

    for event in motion_event.iter() {
        let mut new_pos = (camera_transform.translation()
            - 0.5 * Vec3::new(event.delta.x, -event.delta.y, 0.0))
        .xy();

        new_pos = new_pos.clamp(Vec2::ZERO, pos_max);
        let new_pos = new_pos.extend(camera_transform.translation().z);
        *camera_transform.translation_mut() = new_pos.into();
    }
}
