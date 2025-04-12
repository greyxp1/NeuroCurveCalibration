use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow}, // Added PrimaryWindow
};

// Camera component for the first-person view
#[derive(Component)]
pub struct FpsCamera {
    pub sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for FpsCamera {
    fn default() -> Self {
        Self {
            sensitivity: 0.1,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

// System to setup the camera
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.7, 0.0), // Eye height of ~1.7 meters
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 75.0_f32.to_radians(),
                aspect_ratio: 1.0, // Will be updated by Bevy automatically
                near: 0.1,
                far: 1000.0,
            }),
            ..default()
        },
        FpsCamera::default(),
    ));
}

// System to update the camera based on mouse input
pub fn update_camera(
    mut camera_query: Query<(&mut Transform, &mut FpsCamera)>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    primary_window: Query<&Window, With<PrimaryWindow>>, // Query window to check cursor grab
) {
    let mut delta = Vec2::ZERO;

    // Only process camera movement if the cursor is grabbed
    if let Ok(window) = primary_window.get_single() {
        if window.cursor.grab_mode != CursorGrabMode::Locked {
            mouse_motion_events.clear(); // Consume events if cursor not grabbed
            return;
        }
    } else {
         mouse_motion_events.clear(); // Consume events if no window
        return;
    }


    // Accumulate all mouse motion this frame
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }

    // Only update if we have some motion
    if delta != Vec2::ZERO {
        for (mut transform, mut camera) in camera_query.iter_mut() {
            // Update camera rotation based on mouse movement
            // Note: Sensitivity application might be handled globally elsewhere
            // depending on the final sensitivity system integration.
            // Assuming direct application here for now.
            camera.yaw -= delta.x * camera.sensitivity * 0.002; // Small factor to make sensitivity reasonable
            camera.pitch -= delta.y * camera.sensitivity * 0.002;

            // Clamp pitch to avoid camera flipping
            camera.pitch = camera.pitch.clamp(-89.9_f32.to_radians(), 89.9_f32.to_radians());

            // Apply rotation to transform
            transform.rotation = Quat::from_euler(
                EulerRot::YXZ,
                camera.yaw,
                camera.pitch,
                0.0,
            );
        }
    }
}