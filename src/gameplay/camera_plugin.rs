use bevy::prelude::*;

struct CameraPluggin;

impl PLugin for CameraPluggin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_setup_system)
            .add_system(camera_follow_system);
    }
}

fn camera_setup_system(
    mut commands: Commands,
    event_start_level: EventReader<StartLevelEvent>,
    level_template: Res<LevelTemplate>,
    mut level_instance: ResMut<LevelInstance>,
) {
    if event_start_level.iter().next().is_none() {
        return;
    }

    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_xyz(
                level_template.grid.width() as f32 * GRID_TO_WORLD_UNIT * 0.5,
                level_template.grid.height() as f32 * GRID_TO_WORLD_UNIT * 0.5,
                0.0,
            ),
            projection: OrthographicProjection {
                 
            },
            ..default(),
        })
        .insert(LevelEntity);
}

fn camera_follow_system(
    level_template: Res<LevelTemplate>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
}
