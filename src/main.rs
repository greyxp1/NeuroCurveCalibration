use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

// --- Module Declarations (Create these files in src/ ) ---
mod camera;
mod input;
mod player;
mod target;
mod analysis;
mod sensitivity; // Contains curve generation logic needed by target/analysis?
mod world;
mod calibration_session; // Contains logic for running scenarios
mod ui; // UI elements like crosshair

// Import necessary components/resources/systems from modules
use camera::{setup_camera, update_camera};
use input::{MouseInputBuffer, setup_mouse_input, process_mouse_input};
use player::{setup_player};
use target::{
    ScoreTracker, TargetHitEvent, TargetDestroyedEvent, TargetSpawnedEvent,
    setup_score_tracker, update_target_movement, update_target_spawners,
    update_score_tracker, display_score, update_target_lifetime,
    detect_target_hits, update_target_health, provide_hit_feedback,
    update_hit_effects, setup_basic_target_spawner, reset_score_tracker_flags
};
use analysis::{AdvancedMetrics, setup_advanced_metrics, update_advanced_metrics, calculate_overshooting, analyze_micro_adjustments, analyze_speed_accuracy_correlation};
use sensitivity::{CurveParameters, apply_sensitivity_curve}; // Updated imports
use world::{setup_world};
use calibration_session::{CalibrationState, setup_calibration_session, run_calibration_scenarios, output_results_and_exit}; // NEW module
use ui::UiPlugin; // Import the UI plugin

// --- Main Application ---

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "NeuroCurve Aim Trainer".into(),
                resolution: (1280.0, 720.0).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                // Ensures the window closes cleanly
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        // Add our custom UI plugin
        .add_plugins(UiPlugin)

        // --- Add Events ---
        .add_event::<TargetHitEvent>()
        .add_event::<TargetDestroyedEvent>()
        .add_event::<TargetSpawnedEvent>()

        // --- Add Resources ---
        .init_resource::<MouseInputBuffer>()
        .init_resource::<ScoreTracker>()
        .init_resource::<AdvancedMetrics>() // From analysis
        .init_resource::<CalibrationState>() // From calibration_session
        .init_resource::<CurveParameters>() // Initialize sensitivity curve parameters

        // --- Startup Systems ---
        .add_systems(Startup, (
            setup_camera,
            setup_player,
            setup_world,
            setup_mouse_input,
            setup_score_tracker,
            setup_advanced_metrics, // From analysis
            setup_basic_target_spawner, // Example spawner
            setup_calibration_session, // Initialize calibration state
            apply_deferred, // Apply changes made in setup
            grab_cursor, // Grab cursor after setup
        ).chain()) // Ensure order if needed

        // --- Update Systems ---
        // Add systems individually to avoid tuple size issues
        .add_systems(Update, process_mouse_input)
        .add_systems(Update, apply_sensitivity_curve)
        .add_systems(Update, update_camera)

        // Target systems
        .add_systems(Update, detect_target_hits)
        .add_systems(Update, update_target_health)
        .add_systems(Update, update_target_lifetime)
        .add_systems(Update, provide_hit_feedback)
        .add_systems(Update, update_hit_effects)
        .add_systems(Update, update_target_spawners)
        .add_systems(Update, update_target_movement)

        // Scoring & Analysis
        .add_systems(Update, reset_score_tracker_flags.before(update_score_tracker))
        .add_systems(Update, update_score_tracker)
        .add_systems(Update, display_score)
        .add_systems(Update, update_advanced_metrics)
        .add_systems(Update, calculate_overshooting)
        .add_systems(Update, analyze_micro_adjustments)
        .add_systems(Update, analyze_speed_accuracy_correlation)

        // Calibration
        .add_systems(Update, run_calibration_scenarios)
        .add_systems(Update, output_results_and_exit.run_if(run_once()))

        // Utilities
        .add_systems(Update, toggle_cursor_grab)
        .add_systems(Update, bevy::window::close_on_esc)

        .run();
}

// --- Utility Systems ---

fn grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }
}

fn toggle_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Ok(mut window) = primary_window.get_single_mut() {
            match window.cursor.grab_mode {
                CursorGrabMode::None => {
                    window.cursor.grab_mode = CursorGrabMode::Locked;
                    window.cursor.visible = false;
                }
                _ => {
                    window.cursor.grab_mode = CursorGrabMode::None;
                    window.cursor.visible = true;
                }
            }
        }
    }
}
