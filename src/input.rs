use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
};
use std::collections::VecDeque;
use std::time::Instant;

// Structure to hold mouse input samples
#[derive(Debug, Clone)]
pub struct MouseSample {
    pub timestamp: Instant,
    pub delta: Vec2,
    pub speed: f32, // Speed in pixels/frame (or similar unit)
}

// Resource to store mouse input history
#[derive(Resource)]
pub struct MouseInputBuffer {
    pub samples: VecDeque<MouseSample>,
    pub max_samples: usize,
    // Sensitivity related fields might be moved to a dedicated Sensitivity resource later
    pub sensitivity_multiplier: f32,
    pub raw_input: bool, // Placeholder for future use
}

impl Default for MouseInputBuffer {
    fn default() -> Self {
        Self {
            samples: VecDeque::with_capacity(100),
            max_samples: 100,
            sensitivity_multiplier: 1.0, // Base multiplier
            raw_input: true,
        }
    }
}

// System to initialize mouse input resource
pub fn setup_mouse_input(mut commands: Commands) {
    commands.init_resource::<MouseInputBuffer>(); // Use init_resource for default
}

// System to capture and process mouse movement input
pub fn process_mouse_input(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_buffer: ResMut<MouseInputBuffer>,
    time: Res<Time>, // Use Time to calculate speed more accurately (pixels/second)
) {
    let delta_seconds = time.delta_seconds();
    if delta_seconds == 0.0 { return; } // Avoid division by zero if frame rate is too high or time stalls

    for event in mouse_motion_events.read() {
        let now = Instant::now();
        let delta = event.delta; // Raw pixel delta for this event
        let speed = delta.length() / delta_seconds; // Speed in pixels/second

        // Apply sensitivity multiplier (can be adjusted by sensitivity curve later)
        let adjusted_delta = delta * mouse_buffer.sensitivity_multiplier;

        // Create and store sample
        let sample = MouseSample {
            timestamp: now,
            delta: adjusted_delta, // Store adjusted delta for potential use in camera/movement
            speed, // Store calculated speed
        };

        // Add to buffer and maintain max size
        mouse_buffer.samples.push_back(sample);
        if mouse_buffer.samples.len() > mouse_buffer.max_samples {
            mouse_buffer.samples.pop_front();
        }
    }
}

// System to capture mouse button input (just logs for now)
pub fn process_mouse_buttons(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        // Logic for shooting/interaction is handled in target.rs (detect_target_hits)
        // println!("Left mouse button pressed"); // Optional debug log
    }
    // Add other button handling if needed
}