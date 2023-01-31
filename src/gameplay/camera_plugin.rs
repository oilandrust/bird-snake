use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use iyes_loopless::prelude::{ConditionHelpers, ConditionSet, IntoConditionalSystem};

use crate::{
    level::{level_instance::LevelInstance, level_template::LevelTemplate},
    GameState,
};

use super::{
    game_constants_pluggin::GRID_TO_WORLD_UNIT,
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
                .with_system(camera_follow_system)
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
            ..default()
        })
        .insert(LevelEntity);
}

fn camera_follow_system(
    level_template: Res<LevelTemplate>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
}

fn camera_zoom_scroll_system(
    mut level: ResMut<LevelTemplate>,
    mut scroll_evr: EventReader<MouseWheel>,
    mut camera: Query<(&mut OrthographicProjection, &mut GlobalTransform)>,
) {
    let (mut proj, mut camera_transform) = camera.single_mut();

    use bevy::input::mouse::MouseScrollUnit;
    for ev in scroll_evr.iter() {
        match ev.unit {
            MouseScrollUnit::Line => {
                proj.scale -= 0.05 * ev.y;
            }
            MouseScrollUnit::Pixel => {
                println!(
                    "Scroll (pixel units): vertical: {}, horizontal: {}",
                    ev.y, ev.x
                );
            }
        }
    }
}

fn camera_pan_system(
    mut motion_evr: EventReader<MouseMotion>,
    buttons: Res<Input<MouseButton>>,
    mut camera: Query<(&mut Camera, &mut GlobalTransform)>,
) {
    if !buttons.pressed(MouseButton::Right) {
        return;
    }
    let (_, mut camera_transform) = camera.single_mut();

    for ev in motion_evr.iter() {
        let new_pos = camera_transform.translation() - Vec3::new(ev.delta.x, -ev.delta.y, 0.0);
        *camera_transform.translation_mut() = new_pos.into();
    }
}
