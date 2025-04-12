// src/calibration_session.rs
use bevy::prelude::*;
use std::collections::HashMap;
use serde_json; // Add serde_json for serialization
use crate::{ScoreTracker, AdvancedMetrics}; // Import necessary types

// --- Resources ---

#[derive(Resource, Default)]
pub struct CalibrationState {
    pub current_scenario: usize,
    pub total_scenarios: usize, // Define how many scenarios you have
    pub scenario_timer: Timer,
    pub is_complete: bool,
    // Store results temporarily before outputting
    pub collected_metrics: HashMap<String, f64>,
    // You might need to load initial curve settings from args or a file
    // pub initial_curve: Option<CurveParameters>,
}

// --- Systems ---

pub fn setup_calibration_session(mut commands: Commands) {
    commands.insert_resource(CalibrationState {
        current_scenario: 0,
        total_scenarios: 5, // Example: 5 scenarios
        scenario_timer: Timer::from_seconds(30.0, TimerMode::Once), // Example: 30s per scenario
        is_complete: false,
        collected_metrics: HashMap::new(),
        // initial_curve: None, // TODO: Load initial curve settings if needed
    });
    println!("Calibration Session Setup.");
    // TODO: Set up the first scenario (e.g., spawn specific targets)
}

pub fn run_calibration_scenarios(
    mut state: ResMut<CalibrationState>,
    time: Res<Time>,
    // Add queries/resources needed to run/monitor scenarios
    score: Res<ScoreTracker>,
    _adv_metrics: Res<AdvancedMetrics>, // Prefix with underscore to indicate intentional non-use
    // Query for targets, player, etc.
) {
    if state.is_complete {
        return;
    }

    state.scenario_timer.tick(time.delta());

    // TODO: Implement logic for the current scenario
    // - Spawn/manage targets based on state.current_scenario
    // - Monitor player performance

    if state.scenario_timer.finished() {
        // Scenario finished, collect data
        let current_scenario = state.current_scenario; // Store locally to avoid borrow issues
        println!("Scenario {} finished.", current_scenario + 1);
        state.collected_metrics.insert(format!("scenario_{}_score", current_scenario), score.score as f64);
        state.collected_metrics.insert(format!("scenario_{}_accuracy", current_scenario), score.accuracy as f64);
        // Add more metrics from ScoreTracker and AdvancedMetrics...


        // Move to next scenario or complete
        state.current_scenario += 1;
        if state.current_scenario >= state.total_scenarios {
            state.is_complete = true;
            println!("All scenarios complete.");
        } else {
            // Reset timer and set up next scenario
            state.scenario_timer.reset();
            println!("Starting Scenario {}.", state.current_scenario + 1);
            // TODO: Cleanup previous scenario entities
            // TODO: Setup next scenario entities
        }
    }
}

// System runs once when state.is_complete is true to output results
pub fn output_results_and_exit(
    state: Res<CalibrationState>,
    // Include other resources needed for final metrics calculation if any
) {
    if !state.is_complete {
        return; // Only run when complete
    }

    println!("Calibration finished. Outputting results...");

    // --- IMPORTANT: Output results in a format Tauri can parse ---
    // Example: Outputting combined metrics as JSON to stdout
    match serde_json::to_string(&state.collected_metrics) {
        Ok(json_output) => {
            println!("AIM_TRAINER_RESULTS_START"); // Marker for Tauri to find start
            println!("{}", json_output);
            println!("AIM_TRAINER_RESULTS_END"); // Marker for Tauri to find end
        }
        Err(e) => {
            eprintln!("Error serializing results to JSON: {}", e);
            // Fallback: Print individual metrics
            for (key, value) in &state.collected_metrics {
                 println!("{}: {}", key, value);
            }
        }
    }

    // Exit the Bevy application
    // Use std::process::exit(0) for a clean exit code
     std::process::exit(0);

    // Or use Bevy's AppExit event (might be cleaner)
    // mut exit: EventWriter<bevy::app::AppExit>
    // exit.send(bevy::app::AppExit);
}