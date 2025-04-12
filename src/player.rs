use bevy::prelude::*;
use crate::camera::FpsCamera; // Import FpsCamera from the camera module

// Player component
#[derive(Component)]
pub struct Player {
    pub speed: f32,
    // Jump related fields might be added later if needed for scenarios
    // pub jump_force: f32,
    // pub grounded: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 5.0,
            // jump_force: 5.0,
            // grounded: false,
        }
    }
}

// System to setup the player entity
// Note: In many aim trainers, the player doesn't have a visible body or separate transform.
// The camera *is* the player's viewpoint. We'll keep a Player component for potential state
// but won't spawn a separate transform for it by default. Player movement will likely affect the Camera directly.
pub fn setup_player(mut commands: Commands) {
    // We associate the Player component with the Camera entity
    // If you need separate player physics later, you'll need to adjust this.
    // commands.spawn((
    //     Player::default(),
    //     // No TransformBundle here, movement affects the camera
    // ));
    // For now, let's assume the camera IS the player for movement purposes.
    // If you need player-specific data not tied to camera, add the component to the camera:
    // (Get camera entity in setup_camera and add Player component there,
    // or query for Camera3dBundle here and add Player component)
    // This approach requires careful system ordering or queries.

    // Simpler approach for now: Assume movement applies directly to the camera entity.
    // The Player component might just hold state like speed if needed.
     // Player is now a component, not a resource
     // commands.insert_resource(Player::default());
     // Instead, we'll add it to the camera entity in the camera setup
     println!("Player Resource Initialized (Speed: {})", Player::default().speed);

}

// System to handle player movement (moves the camera)
pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    // Move the camera directly, assuming it represents the player viewpoint
    mut camera_query: Query<&mut Transform, With<FpsCamera>>,
    // Access Player component from the camera entity instead of as a resource
    // player_settings: Res<Player>,
) {
    let delta_time = time.delta_seconds();

    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        let mut direction = Vec3::ZERO;
        // Get forward and right vectors, dereferencing Direction3d to Vec3
        let forward = *camera_transform.forward();
        let right = *camera_transform.right();

        // Forward/backward movement (using camera's local forward)
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += forward;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction -= forward;
        }

        // Left/right movement (using camera's local right)
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= right;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += right;
        }

        // Normalize direction and remove vertical component for horizontal movement
        direction.y = 0.0;
        if let Some(normalized_direction) = direction.try_normalize() {
             // Apply movement directly to the camera's transform
             // Use a fixed speed value since we're not using the Player resource anymore
             let player_speed = 5.0; // Default speed from Player
             camera_transform.translation += normalized_direction * player_speed * delta_time;
        }
    }
}