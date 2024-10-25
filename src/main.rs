/// This is a bare-bones app that allows dropping images onto a canvas.
/// This is a minimal example of a bug with wayland DnD handling.

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<CursorWorldPosition>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_zoom,
            global_cursor,
            file_drop
                .after(global_cursor)  // Making extra sure we run file_drop after cursor pos update. Doesn't help at all.
        ))
        .run();
}

#[derive(Component)]
struct CanvasCamera;

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), CanvasCamera));
}

fn handle_zoom(
    mut evr_scroll: EventReader<MouseWheel>,
    mut query_camera: Query<&mut OrthographicProjection, With<CanvasCamera>>,
) {
    for ev in evr_scroll.read() {
        let value = ev.y;
        let mut p = query_camera.single_mut();
        p.scale -= p.scale * 0.1 * value;
        p.scale = p.scale.clamp(0.1, 400.0);
    }
}

/// For storing world cursor position
#[derive(Default, Resource, Debug)]
struct CursorWorldPosition(Vec2);

/// This system updates the cursor position in the world.
fn global_cursor(
    mut world_coords: ResMut<CursorWorldPosition>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CanvasCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        // On wayland this doesn't print anything when hovering a file over a window
        log::info!("Cursor world position updated! {:?}", world_coords);
        world_coords.0 = world_position;
    }
}

fn file_drop(
    mut evr_dnd: EventReader<FileDragAndDrop>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world_cursor: Res<CursorWorldPosition>, // world_cursor is not always updated when file is dropped
) {
    for ev in evr_dnd.read() {
        match ev {
            FileDragAndDrop::DroppedFile { window, path_buf } => {
                log::info!("Dropped file: {:?} at {:?}", path_buf, window);
                let texture_handle = asset_server.load(path_buf.to_str().unwrap().to_string());

                commands.spawn(
                    SpriteBundle {
                        texture: texture_handle,
                        transform: Transform::from_xyz(world_cursor.0.x, world_cursor.0.y, 0.0),
                        ..default()
                    });
            }
            FileDragAndDrop::HoveredFile {
                window: _,
                path_buf: _,
            } => {
                // On wayland this sometimes prints multiple times for one drop
                log::info!("Hovered file");
            }
            FileDragAndDrop::HoveredFileCanceled { window: _ } => {
                log::info!("File canceled!");
            }
        }
    }
}
