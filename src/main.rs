use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowMode},
    input::mouse::MouseMotion,
    pbr::NotShadowCaster,
    math::Ray,
};
use std::collections::{VecDeque, HashMap};
use std::time::{Duration, Instant};
use std::f32::consts::PI;
use rand::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json; // For outputting results
use chrono::Local;

// --- Module Declarations (Create these files in src/ ) ---
mod camera;
mod input;
mod player;
mod target;
mod analysis;
mod sensitivity; // Contains curve generation logic needed by target/analysis?
mod world;
mod calibration_session; // Contains logic for running scenarios

// Import necessary components/resources/systems from modules
use camera::{FpsCamera, setup_camera, update_camera};
use input::{MouseInputBuffer, MouseSample, setup_mouse_input, process_mouse_input, process_mouse_buttons};
use player::{Player, setup_player, player_movement};
use target::{
    Target, Hitbox, TargetMovement, Lifetime, TargetSpawner, ScoreTracker,
    TargetHitEvent, TargetDestroyedEvent, TargetSpawnedEvent,
    setup_score_tracker, update_target_movement, update_target_spawners,
    update_score_tracker, display_score, update_target_lifetime,
    detect_target_hits, update_target_health, provide_hit_feedback,
    update_hit_effects, setup_basic_target_spawner
};
use analysis::{AimSample, MicroAdjustment, AdvancedMetrics, setup_advanced_metrics, update_advanced_metrics, calculate_overshooting, analyze_micro_adjustments, analyze_speed_accuracy_correlation, AnalysisPlugin};
use sensitivity::{SensitivityCurve, CurveParameters, generate_sensitivity_curve}; // Keep if needed
use world::{setup_world};
use calibration_session::{CalibrationState, setup_calibration_session, run_calibration_scenarios, output_results_and_exit}; // NEW module

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

        // --- Add Events ---
        .add_event::<TargetHitEvent>()
        .add_event::<TargetDestroyedEvent>()
        .add_event::<TargetSpawnedEvent>()

        // --- Add Resources ---
        .init_resource::<MouseInputBuffer>()
        .init_resource::<ScoreTracker>()
        .init_resource::<AdvancedMetrics>() // From analysis
        .init_resource::<CalibrationState>() // From calibration_session

        // --- Add Plugins ---
        // .add_plugins(AnalysisPlugin) // Plugin logic can be integrated directly or kept

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
        .add_systems(Update, (
            // Input & Camera & Player
            process_mouse_input,
            process_mouse_buttons,
            update_camera,
            player_movement, // If player movement is needed in scenarios

            // Target Logic
            detect_target_hits,
            update_target_health,
            update_target_lifetime,
            provide_hit_feedback,
            update_hit_effects,
            update_target_spawners,
            update_target_movement,

            // Scoring & Analysis
            update_score_tracker,
            display_score, // Mostly for debugging in console
            update_advanced_metrics,
            calculate_overshooting,
            analyze_micro_adjustments,
            analyze_speed_accuracy_correlation,

            // Calibration Flow
            run_calibration_scenarios, // Main logic for running the test
            output_results_and_exit.run_if(run_once()), // Check if calibration finished

            // Utilities
            toggle_cursor_grab, // Use Esc to release cursor for debugging
            bevy::window::close_on_esc, // Allow closing with Esc
        ))
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
    keys: Res<Input<KeyCode>>,
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