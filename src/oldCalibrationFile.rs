// Combined code from aim_test module

// --- Necessary Use Statements (Combined and Deduplicated) ---
use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowMode},
    input::mouse::MouseMotion,
    pbr::NotShadowCaster,
    math::Ray,
    ecs::schedule::SystemSet, // Added from mod.rs
};
use std::collections::{VecDeque, HashMap};
use std::time::{Duration, Instant};
use std::f32::consts::PI;
use rand::prelude::*;
use serde::{Serialize, Deserialize}; // Added from sensitivity.rs & calibration.rs
use std::fs; // Added from calibration.rs
use std::path::Path; // Added from calibration.rs
use chrono::Local; // Added from calibration.rs

// --- analysis.rs Code ---

// Constants for analysis
const MICRO_ADJUSTMENT_THRESHOLD: f32 = 2.0; // Pixels
const OVERSHOOT_THRESHOLD: f32 = 10.0; // Pixels
const SPEED_SAMPLE_WINDOW: usize = 20; // Number of samples to analyze for speed
const MAX_HISTORY_SAMPLES: usize = 1000; // Maximum number of samples to store

// Advanced aim sample with more detailed information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AimSample {
    pub timestamp: Instant,
    // Only keeping fields that are actually used
    pub angular_distance: f32,
    pub mouse_speed: f32,
    pub is_hit: bool,
    pub reaction_time: Option<Duration>,
}

// Micro-adjustment detection
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MicroAdjustment {
    // Only keeping the field that's actually used
    pub before_hit: bool,
}

// Advanced metrics resource
#[derive(Resource)]
#[allow(dead_code)]
pub struct AdvancedMetrics {
    pub aim_samples: VecDeque<AimSample>,
    pub micro_adjustments: Vec<MicroAdjustment>,
    pub overshoots: i32,
    pub total_shots: i32,
    pub speed_accuracy_correlation: f32,
    pub flick_accuracy: f32,
    pub tracking_accuracy: f32,
    pub average_reaction_time: Option<Duration>,
    pub target_switch_time: Option<Duration>,
    pub consistency_score: f32,
    pub last_target_position: Option<Vec3>,
    pub last_hit_time: Option<Instant>,
}

impl Default for AdvancedMetrics {
    fn default() -> Self {
        Self {
            aim_samples: VecDeque::with_capacity(MAX_HISTORY_SAMPLES),
            micro_adjustments: Vec::new(),
            overshoots: 0,
            total_shots: 0,
            speed_accuracy_correlation: 0.0,
            flick_accuracy: 0.0,
            tracking_accuracy: 0.0,
            average_reaction_time: None,
            target_switch_time: None,
            consistency_score: 0.0,
            last_target_position: None,
            last_hit_time: None,
        }
    }
}

// System to setup advanced metrics
#[allow(dead_code)]
pub fn setup_advanced_metrics(mut commands: Commands) {
    commands.insert_resource(AdvancedMetrics::default());
}

// System to update advanced metrics
#[allow(dead_code)]
pub fn update_advanced_metrics(
    mut metrics: ResMut<AdvancedMetrics>,
    mut hit_events: EventReader<TargetHitEvent>, // Adjusted path
    mouse_input: Option<Res<MouseInputBuffer>>, // Adjusted path
    _time: Res<Time>,
) {
    // Store mouse speed for use in the loop
    let current_mouse_speed = if let Some(input) = mouse_input.as_ref() {
        // Calculate current speed from the latest sample if available
        input.samples.back().map_or(0.0, |sample| sample.speed)
    } else {
        0.0
    };
    // Process hit events
    for hit_event in hit_events.read() {
        // Record hit in metrics
        metrics.total_shots += 1;

        // Calculate reaction time if we have the last hit time
        let reaction_time = if let Some(last_hit_time) = metrics.last_hit_time {
            Some(last_hit_time.elapsed())
        } else {
            None
        };

        // Update last hit time
        metrics.last_hit_time = Some(Instant::now());

        // Create a new aim sample
        let sample = AimSample {
            timestamp: Instant::now(),
            angular_distance: 0.0, // Would calculate from mouse position to target
            mouse_speed: current_mouse_speed,
            is_hit: true,
            reaction_time,
        };

        // Add sample to history
        metrics.aim_samples.push_back(sample);

        // Limit the history size
        if metrics.aim_samples.len() > MAX_HISTORY_SAMPLES {
            metrics.aim_samples.pop_front();
        }

        // Update last target position
        metrics.last_target_position = Some(hit_event.hit_position);
    }
}

// System to calculate overshooting
#[allow(dead_code)]
pub fn calculate_overshooting(
    mut metrics: ResMut<AdvancedMetrics>,
    mouse_input: Option<Res<MouseInputBuffer>>, // Adjusted path
) {
    // This would analyze mouse movement patterns to detect overshooting
    // For now, we'll just use a simple heuristic based on direction changes

    if let Some(input) = mouse_input {
        // Check if we have enough samples
        if input.samples.len() < 3 {
            return;
        }

        // Get the last few mouse movements
        let last_movements: Vec<_> = input.samples.iter().rev().take(3).collect();

        // Check for direction change (simplified)
        if last_movements.len() >= 3 {
            let dir1 = last_movements[0].delta.signum();
            let dir2 = last_movements[1].delta.signum();

            // If direction changed and movement was significant
            if dir1.x != dir2.x && dir1.x.abs() > OVERSHOOT_THRESHOLD {
                metrics.overshoots += 1;
            }
        }
    }
}

// System to analyze micro-adjustments
#[allow(dead_code)]
pub fn analyze_micro_adjustments(
    mut metrics: ResMut<AdvancedMetrics>,
    mouse_input: Option<Res<MouseInputBuffer>>, // Adjusted path
) {
    // This would analyze mouse movement patterns to detect micro-adjustments
    // For now, we'll just use a simple heuristic based on small movements

    if let Some(input) = mouse_input {
        // Check if we have enough samples
        if input.samples.len() < 2 {
            return;
        }

        // Get the last mouse movement
        let last_movement = input.samples.back().unwrap();

        // Check if it's a micro-adjustment (small movement)
        if last_movement.delta.length() < MICRO_ADJUSTMENT_THRESHOLD {
            // Check if it's before a hit (simplified)
            let before_hit = metrics.last_hit_time.map_or(false, |time| {
                time.elapsed() < Duration::from_millis(100)
            });

            // Record the micro-adjustment
            metrics.micro_adjustments.push(MicroAdjustment {
                before_hit,
            });
        }
    }
}

// System to analyze speed-accuracy correlation
#[allow(dead_code)]
pub fn analyze_speed_accuracy_correlation(
    mut metrics: ResMut<AdvancedMetrics>,
    score_tracker: Option<Res<ScoreTracker>>, // Adjusted path
) {
    // This would analyze the correlation between mouse speed and accuracy
    // For now, we'll just use a simple heuristic

    // Get recent aim samples
    let recent_samples: Vec<_> = metrics.aim_samples.iter()
        .rev()
        .take(SPEED_SAMPLE_WINDOW)
        .collect();

    if recent_samples.len() < SPEED_SAMPLE_WINDOW / 2 {
        return; // Not enough samples
    }

    // Calculate average speed for hits and misses
    let mut hit_speeds = Vec::new();
    let mut miss_speeds = Vec::new();

    for sample in recent_samples {
        if sample.is_hit {
            hit_speeds.push(sample.mouse_speed);
        } else {
            miss_speeds.push(sample.mouse_speed);
        }
    }

    // Calculate averages
    let avg_hit_speed = if !hit_speeds.is_empty() {
        hit_speeds.iter().sum::<f32>() / hit_speeds.len() as f32
    } else {
        0.0
    };

    let avg_miss_speed = if !miss_speeds.is_empty() {
        miss_speeds.iter().sum::<f32>() / miss_speeds.len() as f32
    } else {
        0.0
    };

    // Calculate correlation (simplified)
    if avg_hit_speed > 0.0 && avg_miss_speed > 0.0 {
        // Higher correlation when hit speed is higher than miss speed
        metrics.speed_accuracy_correlation = avg_hit_speed / (avg_hit_speed + avg_miss_speed);
    } else {
        metrics.speed_accuracy_correlation = 0.5; // Neutral
    }

    // Update accuracy metrics if we have score data
    if let Some(score) = score_tracker {
        metrics.flick_accuracy = score.accuracy;
        // Other metrics would be calculated based on more detailed analysis
    }
}

pub struct AnalysisPlugin;

impl Plugin for AnalysisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_advanced_metrics)
           .add_systems(Update, (
               update_advanced_metrics,
               calculate_overshooting,
               analyze_micro_adjustments,
               analyze_speed_accuracy_correlation,
           ));
    }
}


// --- calibration.rs Code ---

#[derive(Resource)]
pub struct CalibrationSystem {
    pub active_profile: String,
    pub profiles: HashMap<String, CalibrationProfile>,
    pub current_calibration: Option<CalibrationSession>,
    pub calibration_history: Vec<CalibrationResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationProfile {
    pub name: String,
    pub description: String,
    pub dpi: f64,
    pub monitor_distance_cm: f64,
    pub monitor_width_cm: f64,
    pub monitor_height_cm: f64,
    pub fov_degrees: f64,
    pub input_latency_ms: f64,
    pub sensitivity_cm_per_360: f64,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for CalibrationProfile {
    fn default() -> Self {
        let now = chrono::Local::now().to_rfc3339();
        Self {
            name: "Default".to_string(),
            description: "Default calibration profile".to_string(),
            dpi: 800.0,
            monitor_distance_cm: 60.0,
            monitor_width_cm: 60.0,
            monitor_height_cm: 33.75,
            fov_degrees: 90.0,
            input_latency_ms: 0.0,
            sensitivity_cm_per_360: 30.0,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CalibrationSession {
    pub session_type: CalibrationSessionType,
    pub steps: Vec<CalibrationStep>,
    pub current_step: usize,
    pub data: HashMap<String, CalibrationData>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CalibrationSessionType {
    Full,
    Sensitivity,
    Dpi,
    MonitorDistance, // Combined MonitorDistance and Size
    Fov,
    InputLatency,
}

#[derive(Clone, Debug)]
pub struct CalibrationStep {
    pub name: String,
    pub description: String,
    pub instruction: String,
    pub completed: bool,
    pub data_key: String, // Key used to store data for this step
}

#[derive(Clone, Debug)]
pub enum CalibrationData {
    Number(f64),
    TimeSeries(Vec<(Instant, f64)>),
    MouseSamples(Vec<(Instant, Vec2, f64)>), // For DPI/Sensitivity
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationResult {
    pub session_type: String,
    pub timestamp: String,
    pub profile_name: String,
    pub values: HashMap<String, f64>,
    pub notes: String,
}

impl Default for CalibrationSystem {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert("Default".to_string(), CalibrationProfile::default());

        Self {
            active_profile: "Default".to_string(),
            profiles,
            current_calibration: None,
            calibration_history: Vec::new(),
        }
    }
}

impl CalibrationSystem {
    pub fn get_active_profile(&self) -> &CalibrationProfile {
        self.profiles.get(&self.active_profile).unwrap_or_else(|| {
            self.profiles.get("Default").expect("Default profile should always exist")
        })
    }

    pub fn get_active_profile_mut(&mut self) -> &mut CalibrationProfile {
        if !self.profiles.contains_key(&self.active_profile) {
            self.active_profile = "Default".to_string();
        }
        self.profiles.get_mut(&self.active_profile).expect("Active profile should exist")
    }

    pub fn set_active_profile(&mut self, name: &str) -> bool {
        if self.profiles.contains_key(name) {
            self.active_profile = name.to_string();
            true
        } else {
            false
        }
    }

     pub fn add_profile(&mut self, profile: CalibrationProfile) -> bool {
        if !self.profiles.contains_key(&profile.name) {
            self.profiles.insert(profile.name.clone(), profile);
            true
        } else {
            false
        }
    }

    pub fn remove_profile(&mut self, name: &str) -> bool {
        if name != "Default" && self.profiles.contains_key(name) {
            self.profiles.remove(name);
            if self.active_profile == name {
                self.active_profile = "Default".to_string();
            }
            true
        } else {
            false
        }
    }

    // Simplified start_calibration focusing on core types
     pub fn start_calibration(&mut self, session_type: CalibrationSessionType) -> bool {
        if self.current_calibration.is_some() {
            return false;
        }

        let steps = match session_type {
            CalibrationSessionType::Sensitivity => vec![
                CalibrationStep { name: "Sensitivity Measurement".to_string(), description: "".to_string(), instruction: "".to_string(), completed: false, data_key: "sensitivity".to_string() },
            ],
            CalibrationSessionType::Dpi => vec![
                CalibrationStep { name: "DPI Measurement".to_string(), description: "".to_string(), instruction: "".to_string(), completed: false, data_key: "dpi".to_string() },
            ],
            CalibrationSessionType::MonitorDistance => vec![
                CalibrationStep { name: "Monitor Distance".to_string(), description: "".to_string(), instruction: "".to_string(), completed: false, data_key: "monitor_distance".to_string() },
                CalibrationStep { name: "Monitor Size".to_string(), description: "".to_string(), instruction: "".to_string(), completed: false, data_key: "monitor_size".to_string() },
            ],
             CalibrationSessionType::Fov => vec![
                CalibrationStep { name: "FOV Calibration".to_string(), description: "".to_string(), instruction: "".to_string(), completed: false, data_key: "fov".to_string() },
            ],
            CalibrationSessionType::InputLatency => vec![
                CalibrationStep { name: "Input Latency".to_string(), description: "".to_string(), instruction: "".to_string(), completed: false, data_key: "latency".to_string() },
            ],
            // Full calibration includes all steps - simplified here
            CalibrationSessionType::Full => vec![
                // Add steps for each calibration type as needed
                CalibrationStep { name: "DPI Measurement".to_string(), description: "".to_string(), instruction: "".to_string(), completed: false, data_key: "dpi".to_string() },
                CalibrationStep { name: "Monitor Distance & Size".to_string(), description: "".to_string(), instruction: "".to_string(), completed: false, data_key: "monitor".to_string() }, // Example combined key
                CalibrationStep { name: "FOV Calibration".to_string(), description: "".to_string(), instruction: "".to_string(), completed: false, data_key: "fov".to_string() },
                CalibrationStep { name: "Sensitivity Measurement".to_string(), description: "".to_string(), instruction: "".to_string(), completed: false, data_key: "sensitivity".to_string() },
                CalibrationStep { name: "Input Latency".to_string(), description: "".to_string(), instruction: "".to_string(), completed: false, data_key: "latency".to_string() },
            ],
        };

        self.current_calibration = Some(CalibrationSession {
            session_type,
            steps,
            current_step: 0,
            data: HashMap::new(),
        });

        true
    }

     pub fn next_calibration_step(&mut self) -> bool {
        if let Some(ref mut session) = self.current_calibration {
            if session.current_step < session.steps.len() - 1 {
                session.steps[session.current_step].completed = true;
                session.current_step += 1;
                true
            } else {
                session.steps[session.current_step].completed = true;
                self.complete_calibration(); // Auto-complete on last step
                false
            }
        } else {
            false
        }
    }

    pub fn previous_calibration_step(&mut self) -> bool {
        if let Some(ref mut session) = self.current_calibration {
            if session.current_step > 0 {
                session.current_step -= 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn set_calibration_data(&mut self, key: &str, data: CalibrationData) -> bool {
        if let Some(ref mut session) = self.current_calibration {
            session.data.insert(key.to_string(), data);
            true
        } else {
            false
        }
    }

     pub fn complete_calibration(&mut self) -> bool {
        if let Some(session) = self.current_calibration.take() {
            let mut values = HashMap::new();
            for (key, data) in session.data {
                match data {
                    CalibrationData::Number(value) => { values.insert(key, value); },
                    // Simplified data processing
                    _ => {} // Ignore other data types for brevity
                }
            }

            let result = CalibrationResult {
                session_type: format!("{:?}", session.session_type),
                timestamp: chrono::Local::now().to_rfc3339(),
                profile_name: self.active_profile.clone(),
                values: values.clone(),
                notes: "".to_string(),
            };
            self.calibration_history.push(result);

            // Update profile
            let profile = self.get_active_profile_mut();
            if let Some(v) = values.get("dpi") { profile.dpi = *v; }
            if let Some(v) = values.get("monitor_distance") { profile.monitor_distance_cm = *v; }
            if let Some(v) = values.get("monitor_width") { profile.monitor_width_cm = *v; }
            if let Some(v) = values.get("monitor_height") { profile.monitor_height_cm = *v; }
            if let Some(v) = values.get("fov") { profile.fov_degrees = *v; }
            if let Some(v) = values.get("sensitivity") { profile.sensitivity_cm_per_360 = *v; }
            if let Some(v) = values.get("latency") { profile.input_latency_ms = *v; }
            profile.updated_at = chrono::Local::now().to_rfc3339();

            true
        } else {
            false
        }
    }

    pub fn cancel_calibration(&mut self) -> bool {
        if self.current_calibration.is_some() {
            self.current_calibration = None;
            true
        } else {
            false
        }
    }

    // Profile export/import can be kept if needed, depends on 'simple' requirement
    pub fn export_profiles(&self) -> String {
        let profiles: Vec<&CalibrationProfile> = self.profiles.values().collect();
        serde_json::to_string_pretty(&profiles).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn import_profiles(&mut self, json: &str) -> bool {
        match serde_json::from_str::<Vec<CalibrationProfile>>(json) {
            Ok(profiles) => {
                for profile in profiles {
                    self.profiles.insert(profile.name.clone(), profile);
                }
                true
            },
            Err(_) => false,
        }
    }
}

// --- DPI Calibration (Simplified from fov.rs - Needs refinement as original was in fov.rs) ---

#[derive(Resource, Default)]
pub struct DpiCalibrationTracker {
    pub active: bool,
    pub mouse_distance_pixels: f32,
    pub physical_distance_cm: f32, // User needs to measure and input this
    pub mouse_samples: Vec<(Instant, Vec2, f64)>, // Store raw samples
    pub completed: bool,
}

// Basic system stubs for DPI - Needs Input handling from Bevy
pub fn start_dpi_calibration(mut tracker: ResMut<DpiCalibrationTracker>, windows: Query<&Window>) {
    if !tracker.active {
        tracker.active = true;
        tracker.mouse_distance_pixels = 0.0;
        tracker.mouse_samples.clear();
        tracker.completed = false;
        // In real Bevy app, would get start position here
    }
}

pub fn track_dpi_calibration(mut tracker: ResMut<DpiCalibrationTracker>, windows: Query<&Window>) {
    if !tracker.active { return; }
    // In real Bevy app, would track mouse movement delta and add to mouse_distance_pixels
    // Store samples: tracker.mouse_samples.push(...)
}

pub fn complete_dpi_calibration(mut tracker: ResMut<DpiCalibrationTracker>) {
   // Called when user indicates completion (e.g., key press)
   if tracker.mouse_distance_pixels > 0.0 {
       tracker.completed = true;
       tracker.active = false;
   }
}

pub fn calculate_dpi(tracker: Res<DpiCalibrationTracker>, mut calibration_system: ResMut<CalibrationSystem>) {
    if tracker.completed && tracker.physical_distance_cm > 0.0 {
        let dpi = (tracker.mouse_distance_pixels as f64 / tracker.physical_distance_cm as f64) * 2.54;
        calibration_system.set_calibration_data("dpi", CalibrationData::Number(dpi));
        // Optionally store samples
        // calibration_system.set_calibration_data("dpi_samples", CalibrationData::MouseSamples(tracker.mouse_samples.clone()));
    }
}


// --- Monitor Calibration (Simplified from profile.rs) ---

#[derive(Resource, Default)]
pub struct MonitorCalibrationTracker {
    pub active: bool,
    pub monitor_distance_cm: f32,
    pub monitor_width_cm: f32,
    pub monitor_height_cm: f32,
    pub completed: bool,
}

// Basic system stubs for Monitor - Needs Input handling/UI for value entry
pub fn setup_monitor_calibration(mut commands: Commands, calibration_system: Res<CalibrationSystem>) {
    if let Some(ref session) = calibration_system.current_calibration {
        if session.session_type == CalibrationSessionType::MonitorDistance ||
           (session.session_type == CalibrationSessionType::Full &&
            session.steps[session.current_step].data_key.starts_with("monitor")) {
             let profile = calibration_system.get_active_profile();
             commands.insert_resource(MonitorCalibrationTracker {
                 active: true,
                 monitor_distance_cm: profile.monitor_distance_cm as f32,
                 monitor_width_cm: profile.monitor_width_cm as f32,
                 monitor_height_cm: profile.monitor_height_cm as f32,
                 completed: false,
            });
        }
    }
}

// In a real app, these would be updated via UI input fields or sliders
pub fn adjust_monitor_distance(mut tracker: ResMut<MonitorCalibrationTracker>, /* input: Res<Input<KeyCode>> */) {
   if !tracker.active { return; }
   // Logic to adjust tracker.monitor_distance_cm based on input
}
pub fn adjust_monitor_size(mut tracker: ResMut<MonitorCalibrationTracker>, /* input: Res<Input<KeyCode>> */) {
    if !tracker.active { return; }
   // Logic to adjust tracker.monitor_width_cm and tracker.monitor_height_cm based on input
}

pub fn complete_monitor_calibration(mut tracker: ResMut<MonitorCalibrationTracker>, mut calibration_system: ResMut<CalibrationSystem>) {
    // Called when user confirms values
    if tracker.active {
        tracker.completed = true;
        tracker.active = false;
        calibration_system.set_calibration_data("monitor_distance", CalibrationData::Number(tracker.monitor_distance_cm as f64));
        calibration_system.set_calibration_data("monitor_width", CalibrationData::Number(tracker.monitor_width_cm as f64));
        calibration_system.set_calibration_data("monitor_height", CalibrationData::Number(tracker.monitor_height_cm as f64));
    }
}


// --- FOV Calibration (Simplified from fov.rs) ---

#[derive(Resource, Default)]
pub struct FovCalibrationTracker {
    pub active: bool,
    pub fov_degrees: f32,
    // Removed reference object details for simplicity, assuming direct FOV input/adjustment
    pub completed: bool,
}

#[derive(Component)]
pub struct FovCalibrationReference {
    // Keep if visual reference is needed, otherwise remove
    pub fov_degrees: f32,
}

// Basic system stubs for FOV - Needs Input handling/UI and Camera interaction
pub fn setup_fov_calibration(mut commands: Commands, calibration_system: Res<CalibrationSystem>) {
    if let Some(ref session) = calibration_system.current_calibration {
        if session.session_type == CalibrationSessionType::Fov ||
           (session.session_type == CalibrationSessionType::Full &&
            session.steps[session.current_step].data_key == "fov") {
            let profile = calibration_system.get_active_profile();
            commands.insert_resource(FovCalibrationTracker {
                active: true,
                fov_degrees: profile.fov_degrees as f32,
                completed: false,
            });
            // Optionally spawn reference object
        }
    }
}

pub fn adjust_fov(mut tracker: ResMut<FovCalibrationTracker>, mut camera_query: Query<&mut Projection>, /* input: Res<Input<KeyCode>> */) {
    if !tracker.active { return; }
    let mut fov_changed = false;
    // Logic to adjust tracker.fov_degrees based on input
    // fov_changed = true;

    if fov_changed {
        for mut projection in camera_query.iter_mut() {
            if let Projection::Perspective(ref mut perspective) = *projection {
                perspective.fov = tracker.fov_degrees.to_radians();
            }
        }
        // Update reference object if it exists
    }
}

pub fn complete_fov_calibration(mut tracker: ResMut<FovCalibrationTracker>, mut calibration_system: ResMut<CalibrationSystem>) {
    // Called when user confirms FOV
     if tracker.active {
        tracker.completed = true;
        tracker.active = false;
        calibration_system.set_calibration_data("fov", CalibrationData::Number(tracker.fov_degrees as f64));
    }
}


// --- Sensitivity Calibration (Simplified from sensitivity.rs) ---

#[derive(Resource, Default)]
pub struct SensitivityCalibrationTracker {
    pub active: bool,
    pub mouse_distance_pixels: f32, // Renamed from mouse_distance for clarity
    pub total_rotation_degrees: f32, // Renamed from total_rotation
    pub target_rotation_degrees: f32, // Default: 360.0
    pub mouse_samples: Vec<(Instant, Vec2, f64)>, // Store raw samples
    pub completed: bool,
}

#[derive(Component)]
pub struct SensitivityRotationTarget; // Simplified marker component

// Basic system stubs for Sensitivity - Needs Input/MouseMotion handling and Camera/Target rotation
pub fn setup_sensitivity_calibration(mut commands: Commands, calibration_system: Res<CalibrationSystem>) {
    if let Some(ref session) = calibration_system.current_calibration {
         if session.session_type == CalibrationSessionType::Sensitivity ||
           (session.session_type == CalibrationSessionType::Full &&
            session.steps[session.current_step].data_key == "sensitivity") {
             commands.insert_resource(SensitivityCalibrationTracker{
                target_rotation_degrees: 360.0,
                ..Default::default()
             });
            // Optionally spawn a visual target to rotate
             commands.spawn(( PbrBundle { /* ... */ transform: Transform::default(), ..default() }, SensitivityRotationTarget ));
         }
    }
}

pub fn start_sensitivity_calibration(mut tracker: ResMut<SensitivityCalibrationTracker>) {
     if !tracker.active {
        tracker.active = true;
        tracker.mouse_distance_pixels = 0.0;
        tracker.total_rotation_degrees = 0.0;
        tracker.mouse_samples.clear();
        tracker.completed = false;
    }
}

pub fn track_sensitivity_calibration(
    mut tracker: ResMut<SensitivityCalibrationTracker>,
    mut mouse_motion_events: EventReader<bevy::input::mouse::MouseMotion>,
    // Query for camera or target to apply rotation based on mouse delta.x
) {
    if !tracker.active { return; }
    let mut total_delta_x = 0.0;
    for event in mouse_motion_events.read() {
        total_delta_x += event.delta.x;
        let speed = event.delta.length();
        tracker.mouse_distance_pixels += speed;
        tracker.mouse_samples.push((Instant::now(), event.delta, speed as f64));
    }

    // Convert delta_x to rotation (needs scaling factor based on game sensitivity settings)
    let rotation_degrees_this_frame = total_delta_x * 0.1; // EXAMPLE SCALING FACTOR - ADJUST!
    tracker.total_rotation_degrees += rotation_degrees_this_frame.abs();

    // Apply rotation_degrees_this_frame to camera/target transform

    if tracker.total_rotation_degrees >= tracker.target_rotation_degrees {
        tracker.completed = true;
        tracker.active = false;
    }
}


pub fn calculate_sensitivity(tracker: Res<SensitivityCalibrationTracker>, mut calibration_system: ResMut<CalibrationSystem>) {
    if tracker.completed {
        let profile = calibration_system.get_active_profile();
        let dpi = profile.dpi;
        if dpi > 0.0 && tracker.total_rotation_degrees > 0.0 {
             // Ensure we use the distance measured *for the target rotation*
            // This might require storing distance at the point completion was met
            // Simplified: using total distance measured during the active phase
             let pixels_per_target_rotation = tracker.mouse_distance_pixels * (tracker.target_rotation_degrees / tracker.total_rotation_degrees);
             let cm_per_360 = (pixels_per_target_rotation as f64 / tracker.target_rotation_degrees as f64) * (2.54 / dpi) * 360.0; // Calculate cm/360

            calibration_system.set_calibration_data("sensitivity", CalibrationData::Number(cm_per_360));
            // Optionally store samples
             // calibration_system.set_calibration_data("sensitivity_samples", CalibrationData::MouseSamples(tracker.mouse_samples.clone()));
        }
    }
}

// --- Latency Calibration (Simplified from mod.rs sub-module) ---

#[derive(Resource, Default)]
pub struct LatencyCalibrationTracker {
    pub active: bool,
    pub targets_spawned: usize,
    pub targets_clicked: usize,
    pub max_targets: usize, // e.g., 10
    pub spawn_timer: Timer, // Bevy timer
    pub reaction_times: Vec<Duration>,
    pub completed: bool,
}

#[derive(Component)]
pub struct LatencyCalibrationTarget {
    pub spawn_time: Instant,
}

// Basic system stubs for Latency - Needs Input/MouseClick handling, Target spawning/despawning
pub fn setup_latency_calibration(mut commands: Commands, calibration_system: Res<CalibrationSystem>) {
     if let Some(ref session) = calibration_system.current_calibration {
         if session.session_type == CalibrationSessionType::InputLatency ||
           (session.session_type == CalibrationSessionType::Full &&
            session.steps[session.current_step].data_key == "latency") {
             commands.insert_resource(LatencyCalibrationTracker{
                 max_targets: 10,
                 spawn_timer: Timer::new(Duration::from_secs(2), TimerMode::Repeating),
                ..Default::default()
             });
         }
    }
}

pub fn start_latency_calibration(mut tracker: ResMut<LatencyCalibrationTracker>) {
    if !tracker.active {
        tracker.active = true;
        tracker.targets_spawned = 0;
        tracker.targets_clicked = 0;
        tracker.reaction_times.clear();
        tracker.completed = false;
        tracker.spawn_timer.reset();
    }
}

pub fn spawn_latency_targets(mut commands: Commands, mut tracker: ResMut<LatencyCalibrationTracker>, time: Res<Time>, windows: Query<&Window>) {
    if !tracker.active || tracker.targets_spawned >= tracker.max_targets { return; }

    tracker.spawn_timer.tick(time.delta());

    if tracker.spawn_timer.just_finished() {
        let window = windows.single();
        let mut rng = thread_rng();
        let x = rng.gen_range(50.0..window.width() - 50.0);
        let y = rng.gen_range(50.0..window.height() - 50.0);

        // Spawn a simple UI node or 2D sprite as a target
        commands.spawn((
            NodeBundle {
                 style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(x - 25.0),
                    top: Val::Px(y - 25.0),
                    width: Val::Px(50.0),
                    height: Val::Px(50.0),
                    ..default()
                 },
                 background_color: Color::RED.into(),
                ..default()
             },
            LatencyCalibrationTarget { spawn_time: Instant::now() },
        ));
        tracker.targets_spawned += 1;

        // Randomize next spawn time
        let next_delay = rng.gen_range(1.0..3.0);
        tracker.spawn_timer = Timer::new(Duration::from_secs_f32(next_delay), TimerMode::Once);
    }
}

pub fn handle_target_clicks(
    mut commands: Commands,
    mut tracker: ResMut<LatencyCalibrationTracker>,
    target_query: Query<(Entity, &LatencyCalibrationTarget, &Node, &GlobalTransform)>, // Query Node for UI clicks
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Query<&Window>,
) {
    if !tracker.active || !mouse_button_input.just_pressed(MouseButton::Left) { return; }

    if let Some(cursor_pos) = windows.single().cursor_position() {
        for (entity, target_info, node, transform) in target_query.iter() {
            let target_pos = transform.translation().truncate();
            let target_size = node.calculated_size();
             let half_size = target_size / 2.0;
            let target_rect = Rect::from_center_size(target_pos, target_size);

            // Check if click is within the target's bounds
             if cursor_pos.x >= target_rect.min.x && cursor_pos.x <= target_rect.max.x &&
                cursor_pos.y >= target_rect.min.y && cursor_pos.y <= target_rect.max.y {

                let reaction_time = Instant::now().duration_since(target_info.spawn_time);
                tracker.reaction_times.push(reaction_time);
                tracker.targets_clicked += 1;
                commands.entity(entity).despawn_recursive();

                if tracker.targets_clicked >= tracker.max_targets {
                    tracker.completed = true;
                    tracker.active = false;
                }
                break; // Process one click per frame
            }
        }
    }
}


pub fn calculate_latency(tracker: Res<LatencyCalibrationTracker>, mut calibration_system: ResMut<CalibrationSystem>) {
    if tracker.completed && !tracker.reaction_times.is_empty() {
        let total_time: Duration = tracker.reaction_times.iter().sum();
        let avg_time = total_time / tracker.reaction_times.len() as u32;
        let latency_ms = avg_time.as_secs_f64() * 1000.0;
        calibration_system.set_calibration_data("latency", CalibrationData::Number(latency_ms));
        // Optionally store samples
        // let time_series: Vec<(Instant, f64)> = tracker.reaction_times.iter().map(...).collect();
        // calibration_system.set_calibration_data("latency_samples", CalibrationData::TimeSeries(time_series));
    }
}

// --- Cleanup ---
// Generic cleanup function for removing resources and entities
fn cleanup_calibration<T: Resource, C: Component>(
    mut commands: Commands,
    tracker: Option<Res<T>>,
    component_query: Query<Entity, With<C>>,
) {
     if let Some(t) = tracker {
         // Assuming a 'completed' field exists on the tracker
         // This needs reflection or a trait to work generically, simplified here:
         // if t.completed { ... } // Cannot access fields generically

         // Simple approach: Remove if the resource exists (usually means calibration ended)
         commands.remove_resource::<T>();
         for entity in component_query.iter() {
            commands.entity(entity).despawn_recursive();
         }
     } else {
         // If tracker doesn't exist, ensure components are gone too
         for entity in component_query.iter() {
            commands.entity(entity).despawn_recursive();
         }
     }
}


// --- Bevy Plugin ---

pub struct CombinedCalibrationPlugin;

impl Plugin for CombinedCalibrationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CalibrationSystem>()
            .init_resource::<DpiCalibrationTracker>() // Initialize all trackers
            .init_resource::<MonitorCalibrationTracker>()
            .init_resource::<FovCalibrationTracker>()
            .init_resource::<SensitivityCalibrationTracker>()
            .init_resource::<LatencyCalibrationTracker>()

            // Add systems - Grouped by calibration type for clarity
            // Order matters for dependencies (e.g., calculate needs tracker completion)

            // DPI Systems (Placeholder - requires input implementation)
            .add_systems(Update, (
                start_dpi_calibration,
                track_dpi_calibration.after(start_dpi_calibration),
                complete_dpi_calibration.after(track_dpi_calibration),
                calculate_dpi.after(complete_dpi_calibration),
                // cleanup_calibration::<DpiCalibrationTracker, DpiCalibrationUI> // Need UI Component
             ).run_if(resource_exists::<DpiCalibrationTracker>))

            // Monitor Systems
             .add_systems(Update, (
                setup_monitor_calibration, // Should run based on CalibrationSystem state
                adjust_monitor_distance.after(setup_monitor_calibration),
                adjust_monitor_size.after(setup_monitor_calibration),
                complete_monitor_calibration.after(adjust_monitor_distance).after(adjust_monitor_size),
                 // cleanup_calibration::<MonitorCalibrationTracker, MonitorCalibrationUI> // Need UI Component
             ).run_if(resource_exists::<MonitorCalibrationTracker>))

            // FOV Systems
            .add_systems(Update, (
                setup_fov_calibration, // Should run based on CalibrationSystem state
                adjust_fov.after(setup_fov_calibration),
                complete_fov_calibration.after(adjust_fov),
                 cleanup_calibration::<FovCalibrationTracker, FovCalibrationReference> // Using Reference as component marker
             ).run_if(resource_exists::<FovCalibrationTracker>))

            // Sensitivity Systems
            .add_systems(Update, (
                setup_sensitivity_calibration, // Should run based on CalibrationSystem state
                start_sensitivity_calibration.after(setup_sensitivity_calibration),
                track_sensitivity_calibration.after(start_sensitivity_calibration),
                calculate_sensitivity.after(track_sensitivity_calibration),
                 cleanup_calibration::<SensitivityCalibrationTracker, SensitivityRotationTarget>
             ).run_if(resource_exists::<SensitivityCalibrationTracker>))

            // Latency Systems
            .add_systems(Update, (
                setup_latency_calibration, // Should run based on CalibrationSystem state
                start_latency_calibration.after(setup_latency_calibration),
                spawn_latency_targets.after(start_latency_calibration),
                handle_target_clicks.after(spawn_latency_targets), // Needs target query results
                calculate_latency.after(handle_target_clicks), // Depends on tracker.completed being set in clicks
                cleanup_calibration::<LatencyCalibrationTracker, LatencyCalibrationTarget>
            ).run_if(resource_exists::<LatencyCalibrationTracker>))
            ;
    }
}

// --- camera.rs Code ---

// Camera component for the first-person view
#[derive(Component)]
pub struct FpsCamera {
    pub sensitivity: f32,
    // Removed unused fov field
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
) {
    let mut delta = Vec2::ZERO;

    // Accumulate all mouse motion this frame
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }

    // Only update if we have some motion
    if delta != Vec2::ZERO {
        for (mut transform, mut camera) in camera_query.iter_mut() {
            // Update camera rotation based on mouse movement
            camera.yaw -= delta.x * camera.sensitivity;
            camera.pitch -= delta.y * camera.sensitivity;

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

// --- input.rs Code ---

// Structure to hold mouse input samples
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MouseSample {
    pub timestamp: Instant,
    pub delta: Vec2,
    pub speed: f32,
}

// Resource to store mouse input history
#[derive(Resource)]
#[allow(dead_code)]
pub struct MouseInputBuffer {
    pub samples: VecDeque<MouseSample>,
    pub max_samples: usize,
    pub sensitivity: f32,
    pub raw_input: bool,
}

impl Default for MouseInputBuffer {
    fn default() -> Self {
        Self {
            samples: VecDeque::with_capacity(100),
            max_samples: 100,
            sensitivity: 1.0,
            raw_input: true,
        }
    }
}

// System to initialize mouse input
#[allow(dead_code)]
pub fn setup_mouse_input(mut commands: Commands) {
    commands.insert_resource(MouseInputBuffer::default());
}

// System to capture and process mouse input
pub fn process_mouse_input(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_buffer: ResMut<MouseInputBuffer>,
    _time: Res<Time>, // Unused but kept for future use
) {
    for event in mouse_motion_events.read() {
        let now = Instant::now();
        let delta = Vec2::new(event.delta.x, event.delta.y);
        let speed = delta.length();

        // Apply sensitivity
        let adjusted_delta = delta * mouse_buffer.sensitivity;

        // Create and store sample
        let sample = MouseSample {
            timestamp: now,
            delta: adjusted_delta,
            speed,
        };

        // Add to buffer and maintain max size
        mouse_buffer.samples.push_back(sample);
        if mouse_buffer.samples.len() > mouse_buffer.max_samples {
            mouse_buffer.samples.pop_front();
        }
    }
}

// System to capture mouse button input
pub fn process_mouse_buttons(
    mouse_button_input: Res<Input<MouseButton>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        // Handle mouse click (will be used for shooting/target interaction)
    }
}


// --- player.rs Code ---

// Player component
#[derive(Component)]
#[allow(dead_code)]
pub struct Player {
    pub speed: f32,
    #[allow(dead_code)]
    pub jump_force: f32,
    #[allow(dead_code)]
    pub grounded: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 5.0,
            jump_force: 5.0,
            grounded: false,
        }
    }
}

// System to setup the player
pub fn setup_player(mut commands: Commands) {
    commands.spawn((
        Player::default(),
        TransformBundle::from(Transform::from_xyz(0.0, 1.7, 0.0)),
    ));
}

// System to handle player movement
pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    camera_query: Query<&Transform, With<FpsCamera>>, // Adjusted path
    mut player_query: Query<(&mut Transform, &Player), Without<FpsCamera>>, // Adjusted path
) {
    let delta_time = time.delta_seconds();

    if let Ok(camera_transform) = camera_query.get_single() {
        for (mut player_transform, player) in player_query.iter_mut() {
            let mut direction = Vec3::ZERO;

            // Forward/backward movement
            if keyboard_input.pressed(KeyCode::W) {
                direction += camera_transform.forward();
            }
            if keyboard_input.pressed(KeyCode::S) {
                direction += camera_transform.back();
            }

            // Left/right movement
            if keyboard_input.pressed(KeyCode::A) {
                direction += camera_transform.left();
            }
            if keyboard_input.pressed(KeyCode::D) {
                direction += camera_transform.right();
            }

            // Normalize direction and remove vertical component for horizontal movement
            if direction != Vec3::ZERO {
                direction = direction.normalize();
                direction.y = 0.0;
            }

            // Apply movement
            player_transform.translation += direction * player.speed * delta_time;

            // Update camera position to follow player
            player_transform.rotation = camera_transform.rotation;
        }
    }
}

// --- sensitivity.rs Code ---

// Game enum for sensitivity conversion
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Game {
    Valorant,
    CSGO,
    Overwatch,
    Apex,
    Quake,
    // Additional games can be added here
}

// Main sensitivity system resource
#[derive(Resource)]
pub struct SensitivitySystem {
    // Base measurements
    pub dpi: f64,
    pub base_cm_per_360: f64,
    pub counts_per_360: f64,

    // Current active profile
    pub active_profile: String,
    pub profiles: HashMap<String, SensitivityProfile>,

    // Game-specific conversion factors
    pub game_sensitivity_scales: HashMap<Game, f64>,
}

impl Default for SensitivitySystem {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        let default_profile = SensitivityProfile::default();
        profiles.insert("Default".to_string(), default_profile);

        let mut game_sensitivity_scales = HashMap::new();
        game_sensitivity_scales.insert(Game::Valorant, 1.0);
        game_sensitivity_scales.insert(Game::CSGO, 3.18);
        game_sensitivity_scales.insert(Game::Overwatch, 3.33);
        game_sensitivity_scales.insert(Game::Apex, 3.18);
        game_sensitivity_scales.insert(Game::Quake, 3.18);

        Self {
            dpi: 1600.0,
            base_cm_per_360: 30.0,
            counts_per_360: 0.0, // Will be calculated
            active_profile: "Default".to_string(),
            profiles,
            game_sensitivity_scales,
        }
    }
}

#[allow(dead_code)]
impl SensitivitySystem {
    // Calculate counts per 360 based on DPI and cm/360
    pub fn calculate_counts_per_360(&mut self) {
        // Formula: counts_per_360 = DPI * (2.54 * cm_per_360)
        self.counts_per_360 = self.dpi * (2.54 * self.base_cm_per_360);
    }

    // Get the current active profile
    pub fn get_active_profile(&self) -> &SensitivityProfile {
        self.profiles.get(&self.active_profile).unwrap_or_else(|| {
            // Fallback to default if active profile doesn't exist
            self.profiles.get("Default").expect("Default profile should always exist")
        })
    }

    // Get the current active profile as mutable
    pub fn get_active_profile_mut(&mut self) -> &mut SensitivityProfile {
        if !self.profiles.contains_key(&self.active_profile) {
            self.active_profile = "Default".to_string();
        }
        self.profiles.get_mut(&self.active_profile).expect("Active profile should exist")
    }

    // Set a new active profile
    pub fn set_active_profile(&mut self, profile_name: &str) -> bool {
        if self.profiles.contains_key(profile_name) {
            self.active_profile = profile_name.to_string();
            true
        } else {
            false
        }
    }

    // Add a new profile
    pub fn add_profile(&mut self, name: &str, profile: SensitivityProfile) -> bool {
        if !self.profiles.contains_key(name) {
            self.profiles.insert(name.to_string(), profile);
            true
        } else {
            false
        }
    }

    // Remove a profile (cannot remove Default)
    #[allow(dead_code)]
    pub fn remove_profile(&mut self, name: &str) -> bool {
        if name != "Default" && self.profiles.contains_key(name) {
            self.profiles.remove(name);
            if self.active_profile == name {
                self.active_profile = "Default".to_string();
            }
            true
        } else {
            false
        }
    }

    // Convert sensitivity to a specific game
    pub fn convert_to_game(&self, game: &Game) -> f64 {
        let scale = self.game_sensitivity_scales.get(game).copied().unwrap_or(1.0);
        let active_profile = self.get_active_profile();
        active_profile.curve_parameters.min_sens / scale
    }

    // Set sensitivity from a specific game
    pub fn set_from_game(&mut self, game: &Game, game_sens: f64) {
        let scale = self.game_sensitivity_scales.get(game).copied().unwrap_or(1.0);
        let new_base_sens = game_sens * scale;
        let active_profile = self.get_active_profile_mut();
        active_profile.curve_parameters.min_sens = new_base_sens;
    }

    // Calculate sensitivity for a given mouse speed
    #[allow(dead_code)]
    pub fn calculate_sensitivity(&self, mouse_speed: f64) -> f64 {
        let active_profile = self.get_active_profile();
        active_profile.curve.calculate_sensitivity(mouse_speed)
    }
}

// Curve parameters structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveParameters {
    pub min_sens: f64,
    pub max_sens: f64,
    pub range: f64,
    pub growth_base: f64,
    pub offset: f64,
    pub smoothing: f64,  // Added smoothing parameter
}

impl Default for CurveParameters {
    fn default() -> Self {
        Self {
            min_sens: 0.6,
            max_sens: 7.0,
            range: 40.0,
            growth_base: 1.05,
            offset: 0.0,
            smoothing: 0.0,
        }
    }
}

// Sensitivity curve implementation
#[derive(Component, Debug, Clone)]
pub struct SensitivityCurve {
    pub parameters: CurveParameters,
    pub lookup_table: Vec<f64>,
    pub lookup_table_size: usize,
}

impl Default for SensitivityCurve {
    fn default() -> Self {
        let parameters = CurveParameters::default();
        let lookup_table_size = 257; // Default size
        let lookup_table = generate_sensitivity_curve(
            lookup_table_size,
            parameters.growth_base,
            parameters.range,
            parameters.min_sens,
            parameters.max_sens,
            parameters.offset,
            true, // Use plateau by default
        );

        Self {
            parameters,
            lookup_table,
            lookup_table_size,
        }
    }
}

#[allow(dead_code)]
impl SensitivityCurve {
    // Create a new curve with custom parameters
    pub fn new(parameters: CurveParameters, lookup_table_size: usize) -> Self {
        let lookup_table = generate_sensitivity_curve(
            lookup_table_size,
            parameters.growth_base,
            parameters.range,
            parameters.min_sens,
            parameters.max_sens,
            parameters.offset,
            true, // Use plateau by default
        );

        Self {
            parameters,
            lookup_table,
            lookup_table_size,
        }
    }

    // Update the curve with new parameters
    pub fn update_parameters(&mut self, parameters: CurveParameters) {
        self.parameters = parameters;
        self.regenerate_lookup_table();
    }

    // Regenerate the lookup table
    pub fn regenerate_lookup_table(&mut self) {
        self.lookup_table = generate_sensitivity_curve(
            self.lookup_table_size,
            self.parameters.growth_base,
            self.parameters.range,
            self.parameters.min_sens,
            self.parameters.max_sens,
            self.parameters.offset,
            true, // Use plateau by default
        );
    }

    // Calculate sensitivity for a given mouse speed
    #[allow(dead_code)]
    pub fn calculate_sensitivity(&self, mouse_speed: f64) -> f64 {
        let index = mouse_speed.min((self.lookup_table.len() - 1) as f64).max(0.0) as usize;
        self.lookup_table[index]
    }

    // Get the curve data as (x, y) points for visualization
    pub fn get_curve_points(&self) -> Vec<(f64, f64)> {
        (0..self.lookup_table.len())
            .map(|i| (i as f64, self.lookup_table[i]))
            .collect()
    }
}

// Function to generate a sensitivity curve
pub fn generate_sensitivity_curve(
    n: usize,
    growth_base: f64,
    range: f64,
    min_sens: f64,
    max_sens: f64,
    offset: f64,
    plateau: bool,
) -> Vec<f64> {
    if n == 0 || range <= 0.0 {
        return vec![0.0; n];
    }

    let mut result = Vec::with_capacity(n);
    let offset_points = (offset.ceil() as usize).min(n);
    result.resize(offset_points, min_sens);

    if offset_points >= n {
        return result;
    }

    let remaining = n - offset_points;
    let range_size = range.ceil() as usize;
    let expo_len = if plateau && remaining > range_size {
        range_size
    } else {
        remaining
    };

    let sens_diff = max_sens - min_sens;
    let inv_range = 1.0 / range;

    result.reserve(expo_len);

    if growth_base <= 1.0 {
        // Use smooth curve for growth_base <= 1.0
        for i in 0..expo_len {
            let t = (i as f64 * inv_range).min(1.0);
            let t2 = t * t;
            result.push(min_sens + sens_diff * (t2 * (3.0 - 2.0 * t)));
        }
    } else {
        // Use exponential curve for growth_base > 1.0
        let base_factor = 1.0 / (growth_base.powf(range) - 1.0);
        for i in 0..expo_len {
            let t = ((growth_base.powf(i as f64 * inv_range * range) - 1.0) * base_factor).min(1.0);
            result.push(min_sens + sens_diff * t);
        }
    }

    // Add plateau if needed
    if plateau && remaining > expo_len {
        result.resize(n, max_sens);
    }

    result
}

// Sensitivity profile structure
#[derive(Component, Debug, Clone)]
pub struct SensitivityProfile {
    pub name: String,
    pub description: String,
    pub curve_parameters: CurveParameters,
    pub curve: SensitivityCurve,
    pub y_x_ratio: f64,
    pub rotation: f64,
    pub angle_snapping: f64,
}

impl Default for SensitivityProfile {
    fn default() -> Self {
        let curve_parameters = CurveParameters::default();
        let curve = SensitivityCurve::default();

        Self {
            name: "Default".to_string(),
            description: "Default sensitivity profile".to_string(),
            curve_parameters,
            curve,
            y_x_ratio: 1.0,
            rotation: 0.0,
            angle_snapping: 0.0,
        }
    }
}

#[allow(dead_code)]
impl SensitivityProfile {
    // Create a new profile with custom parameters
    pub fn new(
        name: &str,
        description: &str,
        curve_parameters: CurveParameters,
        y_x_ratio: f64,
        rotation: f64,
        angle_snapping: f64,
    ) -> Self {
        let curve = SensitivityCurve::new(curve_parameters.clone(), 257);

        Self {
            name: name.to_string(),
            description: description.to_string(),
            curve_parameters,
            curve,
            y_x_ratio,
            rotation,
            angle_snapping,
        }
    }

    // Update the curve parameters
    pub fn update_curve_parameters(&mut self, parameters: CurveParameters) {
        self.curve_parameters = parameters.clone();
        self.curve.update_parameters(parameters);
    }

    // Apply the profile to mouse input
    pub fn apply_to_mouse_input(&self, input: Vec2) -> Vec2 {
        let mut result = input;

        // Apply Y/X ratio
        result.y *= self.y_x_ratio as f32;

        // Apply rotation if needed
        if self.rotation != 0.0 {
            let rad = self.rotation.to_radians() as f32;
            let cos_rad = rad.cos();
            let sin_rad = rad.sin();
            let x = result.x;
            let y = result.y;
            result.x = x * cos_rad - y * sin_rad;
            result.y = x * sin_rad + y * cos_rad;
        }

        // Apply angle snapping if needed
        if self.angle_snapping > 0.0 {
            let snap_angle = self.angle_snapping.to_radians() as f32;
            let length = result.length();
            if length > 0.0 {
                let angle = result.y.atan2(result.x);
                let snapped_angle = (angle / snap_angle).round() * snap_angle;
                result.x = length * snapped_angle.cos();
                result.y = length * snapped_angle.sin();
            }
        }

        result
    }

    // Calculate sensitivity for a given mouse speed
    pub fn calculate_sensitivity(&self, mouse_speed: f64) -> f64 {
        self.curve.calculate_sensitivity(mouse_speed)
    }
}

// Serializable profile for saving/loading
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableProfile {
    pub name: String,
    pub description: String,
    pub curve_parameters: CurveParameters,
    pub y_x_ratio: f64,
    pub rotation: f64,
    pub angle_snapping: f64,
}

impl From<&SensitivityProfile> for SerializableProfile {
    fn from(profile: &SensitivityProfile) -> Self {
        Self {
            name: profile.name.clone(),
            description: profile.description.clone(),
            curve_parameters: profile.curve_parameters.clone(),
            y_x_ratio: profile.y_x_ratio,
            rotation: profile.rotation,
            angle_snapping: profile.angle_snapping,
        }
    }
}

impl From<SerializableProfile> for SensitivityProfile {
    fn from(serializable: SerializableProfile) -> Self {
        Self::new(
            &serializable.name,
            &serializable.description,
            serializable.curve_parameters,
            serializable.y_x_ratio,
            serializable.rotation,
            serializable.angle_snapping,
        )
    }
}

// Game sensitivity conversion component
#[derive(Component, Debug, Clone)]
#[allow(dead_code)]
pub struct GameSensitivity {
    // Conversion factors for different games
    pub conversion_factors: HashMap<Game, f64>,
    // Current game sensitivities
    pub game_sensitivities: HashMap<Game, f64>,
}

impl Default for GameSensitivity {
    fn default() -> Self {
        let mut conversion_factors = HashMap::new();
        // These are approximate conversion factors between games
        // 1.0 is the base (Valorant)
        conversion_factors.insert(Game::Valorant, 1.0);
        conversion_factors.insert(Game::CSGO, 3.18);
        conversion_factors.insert(Game::Overwatch, 3.33);
        conversion_factors.insert(Game::Apex, 3.18);
        conversion_factors.insert(Game::Quake, 3.18);

        let mut game_sensitivities = HashMap::new();
        // Default sensitivities for each game
        game_sensitivities.insert(Game::Valorant, 0.5);
        game_sensitivities.insert(Game::CSGO, 1.6);
        game_sensitivities.insert(Game::Overwatch, 5.0);
        game_sensitivities.insert(Game::Apex, 1.6);
        game_sensitivities.insert(Game::Quake, 1.6);

        Self {
            conversion_factors,
            game_sensitivities,
        }
    }
}

#[allow(dead_code)]
impl GameSensitivity {
    // Convert a sensitivity value from one game to another
    pub fn convert_between_games(&self, from_game: &Game, to_game: &Game, sens_value: f64) -> f64 {
        let from_factor = self.conversion_factors.get(from_game).copied().unwrap_or(1.0);
        let to_factor = self.conversion_factors.get(to_game).copied().unwrap_or(1.0);

        // Convert to normalized value, then to target game
        (sens_value * from_factor) / to_factor
    }

    // Set a game sensitivity and update all others
    pub fn set_game_sensitivity(&mut self, game: &Game, sens_value: f64) {
        // Update the specified game sensitivity
        self.game_sensitivities.insert(game.clone(), sens_value);

        // Get the conversion factor for the updated game
        let base_factor = self.conversion_factors.get(game).copied().unwrap_or(1.0);
        let normalized_sens = sens_value * base_factor;

        // Update all other game sensitivities
        for (other_game, factor) in &self.conversion_factors {
            if other_game != game {
                let other_sens = normalized_sens / factor;
                self.game_sensitivities.insert(other_game.clone(), other_sens);
            }
        }
    }

    // Get sensitivity for a specific game
    pub fn get_game_sensitivity(&self, game: &Game) -> f64 {
        self.game_sensitivities.get(game).copied().unwrap_or_else(|| {
            // If not found, return a default value
            match game {
                Game::Valorant => 0.5,
                Game::CSGO => 1.6,
                Game::Overwatch => 5.0,
                Game::Apex => 1.6,
                Game::Quake => 1.6,
            }
        })
    }

    // Calculate cm/360 for a given game sensitivity
    pub fn calculate_cm_per_360(&self, game: &Game, dpi: f64) -> f64 {
        let game_sens = self.get_game_sensitivity(game);
        let factor = self.conversion_factors.get(game).copied().unwrap_or(1.0);

        // Formula: cm_per_360 = (2.54 * 360 * factor) / (dpi * game_sens)
        (2.54 * 360.0) / (dpi * game_sens * factor)
    }

    // Set sensitivity from cm/360
    pub fn set_from_cm_per_360(&mut self, game: &Game, cm_per_360: f64, dpi: f64) {
        let factor = self.conversion_factors.get(game).copied().unwrap_or(1.0);

        // Formula: game_sens = (2.54 * 360 * factor) / (dpi * cm_per_360)
        let game_sens = (2.54 * 360.0) / (dpi * cm_per_360 * factor);
        self.set_game_sensitivity(game, game_sens);
    }
}

// Component for sensitivity testing UI
#[derive(Component)]
pub struct SensitivityTestUI;

// SYSTEM IMPLEMENTATIONS

// System to setup the sensitivity system
pub fn setup_sensitivity_system(mut commands: Commands) {
    let mut sens_system = SensitivitySystem::default();
    sens_system.calculate_counts_per_360();
    commands.insert_resource(sens_system);
}

// System to update the sensitivity system
pub fn update_sensitivity_system(
    mut sens_system: ResMut<SensitivitySystem>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    // Example: Toggle between profiles with F1, F2, etc.
    if keyboard_input.just_pressed(KeyCode::F1) {
        sens_system.set_active_profile("Default");
    }
}

// System to update game sensitivities based on user input
pub fn update_game_sensitivities(
    mut sens_system: ResMut<SensitivitySystem>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    // Example: Adjust sensitivity for different games with keyboard shortcuts
    if keyboard_input.just_pressed(KeyCode::F5) {
        // Example: Set Valorant sensitivity to 0.5
        sens_system.set_from_game(&Game::Valorant, 0.5);
    } else if keyboard_input.just_pressed(KeyCode::F6) {
        // Example: Set CSGO sensitivity to 1.6
        sens_system.set_from_game(&Game::CSGO, 1.6);
    }
}

// System to visualize the sensitivity curve
pub fn visualize_sensitivity_curve(
    sens_system: Res<SensitivitySystem>,
) {
    let active_profile = sens_system.get_active_profile();
    let curve_points = active_profile.curve.get_curve_points();

    // This would be integrated with a UI system to display the curve
    // For now, we just print some debug info
    if cfg!(debug_assertions) {
        println!("Active profile: {}", sens_system.active_profile);
        println!("Base sensitivity: {}", active_profile.curve_parameters.min_sens);
        println!("Max sensitivity: {}", active_profile.curve_parameters.max_sens);
        println!("Curve points: {} entries", curve_points.len());
    }
}

// System to save profiles
#[allow(dead_code)]
pub fn save_profiles(
    sens_system: Res<SensitivitySystem>,
) {
    // Convert profiles to serializable format
    let serializable_profiles: Vec<SerializableProfile> = sens_system.profiles
        .values()
        .map(|profile| SerializableProfile::from(profile))
        .collect();

    // This would save to a file in a real implementation
    if cfg!(debug_assertions) {
        println!("Would save {} profiles", serializable_profiles.len());
    }
}

// System to load profiles
pub fn load_profiles(
    mut sens_system: ResMut<SensitivitySystem>,
) {
    // This would load from a file in a real implementation
    // For now, we just create a sample profile
    let sample_profile = SensitivityProfile::new(
        "Gaming",
        "Profile optimized for gaming",
        CurveParameters {
            min_sens: 0.5,
            max_sens: 5.0,
            range: 30.0,
            growth_base: 1.03,
            offset: 0.0,
            smoothing: 0.1,
        },
        1.0,
        0.0,
        0.0,
    );

    sens_system.add_profile("Gaming", sample_profile);
}

// System to setup the sensitivity testing UI
pub fn setup_sensitivity_test_ui(_commands: Commands, _asset_server: Res<AssetServer>) {
    // This would create a UI for testing sensitivity in a real implementation
    // For now, we just print a message
    if cfg!(debug_assertions) {
        println!("Setting up sensitivity test UI");
    }
}

// System to update the sensitivity testing UI
pub fn update_sensitivity_test_ui(
    sens_system: Res<SensitivitySystem>,
    _query: Query<&mut Text, With<SensitivityTestUI>>,
) {
    // This would update the UI with current sensitivity values
    // For now, we just print some debug info occasionally
    if cfg!(debug_assertions) {
        let active_profile = sens_system.get_active_profile();

        // Only print occasionally to avoid console spam
        if rand::random::<f32>() < 0.001 {
            println!("Active profile: {}", sens_system.active_profile);
            println!("Base sensitivity: {}", active_profile.curve_parameters.min_sens);
            println!("Max sensitivity: {}", active_profile.curve_parameters.max_sens);
            println!("DPI: {}", sens_system.dpi);
            println!("cm/360: {}", sens_system.base_cm_per_360);

            // Print game-specific sensitivities
            println!("Game sensitivities:");
            for game in [Game::Valorant, Game::CSGO, Game::Overwatch, Game::Apex, Game::Quake].iter() {
                let game_sens = sens_system.convert_to_game(game);
                println!("  {:?}: {:.2}", game, game_sens);
            }
        }
    }
}

// System to handle sensitivity test UI interactions
pub fn handle_sensitivity_test_ui_interaction(
    mut sens_system: ResMut<SensitivitySystem>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    // Example: Adjust sensitivity with keyboard shortcuts
    let active_profile = sens_system.get_active_profile_mut();
    let mut changed = false;

    // Adjust base sensitivity
    if keyboard_input.pressed(KeyCode::ControlLeft) {
        if keyboard_input.just_pressed(KeyCode::Up) {
            active_profile.curve_parameters.min_sens += 0.1;
            changed = true;
        } else if keyboard_input.just_pressed(KeyCode::Down) {
            active_profile.curve_parameters.min_sens = (active_profile.curve_parameters.min_sens - 0.1).max(0.1);
            changed = true;
        }

        // Adjust max sensitivity
        if keyboard_input.just_pressed(KeyCode::Right) {
            active_profile.curve_parameters.max_sens += 0.5;
            changed = true;
        } else if keyboard_input.just_pressed(KeyCode::Left) {
            active_profile.curve_parameters.max_sens = (active_profile.curve_parameters.max_sens - 0.5).max(active_profile.curve_parameters.min_sens);
            changed = true;
        }
    }

    // Adjust curve range
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        if keyboard_input.just_pressed(KeyCode::Up) {
            active_profile.curve_parameters.range += 5.0;
            changed = true;
        } else if keyboard_input.just_pressed(KeyCode::Down) {
            active_profile.curve_parameters.range = (active_profile.curve_parameters.range - 5.0).max(5.0);
            changed = true;
        }

        // Adjust growth base
        if keyboard_input.just_pressed(KeyCode::Right) {
            active_profile.curve_parameters.growth_base += 0.01;
            changed = true;
        } else if keyboard_input.just_pressed(KeyCode::Left) {
            active_profile.curve_parameters.growth_base = (active_profile.curve_parameters.growth_base - 0.01).max(1.0);
            changed = true;
        }
    }

    // Update the curve if parameters changed
    if changed {
        active_profile.curve.update_parameters(active_profile.curve_parameters.clone());
    }
}

// Main plugin to register all the sensitivity systems
pub struct SensitivityPlugin;

impl Plugin for SensitivityPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (setup_sensitivity_system, setup_sensitivity_test_ui))
            .add_systems(Update, (
                update_sensitivity_system,
                update_game_sensitivities,
                visualize_sensitivity_curve,
                update_sensitivity_test_ui,
                handle_sensitivity_test_ui_interaction,
            ));
    }
}


// --- target.rs Code ---

// --- Core Target Components & Events ---

#[derive(Component, Debug, Clone)]
pub struct Target {
    pub radius: f32,
    pub health: i32,
    pub points: i32,
    pub destroy_on_hit: bool,
    pub time_to_live: Option<Duration>,
    pub spawn_time_instant: Option<Timer>, // Adjusted type to Timer
}

impl Default for Target {
    fn default() -> Self {
        Self {
            radius: 0.5,
            health: 1,
            points: 100,
            destroy_on_hit: true,
            time_to_live: None,
            spawn_time_instant: None,
        }
    }
}

#[derive(Event, Debug)]
pub struct TargetHitEvent {
    pub target_entity: Entity,
    pub hit_position: Vec3,
    pub damage: i32,
}

#[derive(Event, Debug)]
pub struct TargetDestroyedEvent {
    pub target_entity: Entity,
    pub points: i32,
    pub destroyed_by_hit: bool,
}

#[derive(Event, Debug)]
pub struct TargetSpawnedEvent {
    pub target_entity: Entity,
    pub position: Vec3,
}

// --- Hitbox Component & Logic ---

#[derive(Component, Debug, Clone)]
pub enum Hitbox {
    Sphere { radius: f32 },
    // Add other hitbox types (Box, Capsule, Compound) if needed, keeping it simple for now
}

impl Hitbox {
    // Simplified ray intersection check for Sphere only
    pub fn intersect_ray(&self, ray: Ray, transform: &GlobalTransform) -> Option<f32> {
        match self {
            Hitbox::Sphere { radius } => {
                let origin = ray.origin - transform.translation();
                let a = ray.direction.length_squared();
                let b = 2.0 * origin.dot(ray.direction);
                let c = origin.length_squared() - radius * radius;
                let discriminant = b * b - 4.0 * a * c;

                if discriminant < 0.0 { return None; }

                let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
                let t2 = (-b + discriminant.sqrt()) / (2.0 * a);

                if t1 > 0.0 { Some(t1) } else if t2 > 0.0 { Some(t2) } else { None }
            },
            // Cases for other hitbox types omitted for simplicity
        }
    }
}

// --- Movement Components & Logic ---

#[derive(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

#[derive(Component, Debug, Clone)]
pub enum TargetMovement {
    Static,
    Linear {
        velocity: Vec3,
        bounds: Option<(Vec3, Vec3)>,
    },
    // Add other movement types (Circular, Oscillating, etc.) if needed
}

impl Default for TargetMovement {
    fn default() -> Self {
        Self::Static
    }
}

// System to update target movement (simplified)
pub fn update_target_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut TargetMovement)>,
) {
    let delta_time = time.delta_seconds();
    for (mut transform, mut movement) in query.iter_mut() {
        match &mut *movement {
            TargetMovement::Static => {}
            TargetMovement::Linear { velocity, bounds } => {
                transform.translation += *velocity * delta_time;
                // Simplified bounds check (just reverse velocity component)
                if let Some((min_bounds, max_bounds)) = bounds {
                    for i in 0..3 {
                        if transform.translation[i] < min_bounds[i] || transform.translation[i] > max_bounds[i] {
                            velocity[i] *= -1.0;
                            // Clamp position to prevent going too far out
                            transform.translation[i] = transform.translation[i].clamp(min_bounds[i], max_bounds[i]);
                        }
                    }
                }
            }
            // Cases for other movement types omitted
        }
    }
}

// --- Spawner Logic ---

#[derive(Component, Debug)]
pub struct TargetSpawner {
    pub spawn_timer: Timer,
    pub max_targets: usize,
    pub spawn_area_min: Vec3,
    pub spawn_area_max: Vec3,
    pub target_radius: f32,
    pub target_color: Color,
}

// System to update target spawners (simplified)
pub fn update_target_spawners(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawners: Query<&mut TargetSpawner>,
    targets: Query<&Target>,
    mut target_spawned_events: EventWriter<TargetSpawnedEvent>,
) {
    let current_target_count = targets.iter().count();
    for mut spawner in spawners.iter_mut() {
        spawner.spawn_timer.tick(time.delta());
        if spawner.spawn_timer.just_finished() && current_target_count < spawner.max_targets {
            let mut rng = thread_rng();
            let position = Vec3::new(
                rng.gen_range(spawner.spawn_area_min.x..spawner.spawn_area_max.x),
                rng.gen_range(spawner.spawn_area_min.y..spawner.spawn_area_max.y),
                rng.gen_range(spawner.spawn_area_min.z..spawner.spawn_area_max.z),
            );

            let target = Target {
                radius: spawner.target_radius,
                time_to_live: Some(Duration::from_secs(5)), // Example lifetime
                ..Default::default()
            };
            let hitbox = Hitbox::Sphere { radius: target.radius };
            let movement = TargetMovement::Static; // Default to static for simplicity

            let entity = commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::UVSphere { radius: target.radius, ..default() })),
                    material: materials.add(StandardMaterial { base_color: spawner.target_color, ..default() }),
                    transform: Transform::from_translation(position),
                    ..default()
                },
                target,
                hitbox,
                movement,
            )).id();

            target_spawned_events.send(TargetSpawnedEvent { target_entity: entity, position });
        }
    }
}

// --- Scoring Logic ---

#[derive(Resource, Debug, Default)]
pub struct ScoreTracker {
    pub score: i32,
    pub hits: i32,
    pub misses: i32,
    pub accuracy: f32,
}

// System to setup the score tracker (needed for resource initialization)
pub fn setup_score_tracker(mut commands: Commands) {
    commands.insert_resource(ScoreTracker::default());
}


// System to update the score tracker (simplified)
pub fn update_score_tracker(
    mut score_tracker: ResMut<ScoreTracker>,
    mut target_destroyed_events: EventReader<TargetDestroyedEvent>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    let mut current_hits = 0;
    for destroyed_event in target_destroyed_events.read() {
        if destroyed_event.destroyed_by_hit {
            score_tracker.score += destroyed_event.points;
            current_hits += 1;
        }
    }

    let shots_fired = mouse_button_input.just_pressed(MouseButton::Left);
    if shots_fired {
        if current_hits > 0 {
            score_tracker.hits += current_hits;
        } else {
            score_tracker.misses += 1;
        }
        let total_shots = score_tracker.hits + score_tracker.misses;
        if total_shots > 0 {
            score_tracker.accuracy = score_tracker.hits as f32 / total_shots as f32 * 100.0;
        }
    }
}

// System to display the score (simplified console output)
pub fn display_score(score_tracker: Res<ScoreTracker>) {
    if score_tracker.is_changed() {
        println!("Score: {}, Accuracy: {:.1}% (Hits: {}, Misses: {})",
            score_tracker.score, score_tracker.accuracy, score_tracker.hits, score_tracker.misses);
    }
}

// --- Core Systems (related to Target) ---

// System to handle target lifetime
pub fn update_target_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut targets: Query<(Entity, &mut Target)>,
    mut target_destroyed_events: EventWriter<TargetDestroyedEvent>,
) {
    for (entity, mut target) in targets.iter_mut() {
        // Handle spawn timer
        if let Some(timer) = &mut target.spawn_time_instant {
            timer.tick(time.delta());
            if timer.finished() {
                target.spawn_time_instant = None; // Clear timer
            } else {
                continue; // Still spawning
            }
        }

        // Handle time to live
        if let Some(ttl_duration) = &mut target.time_to_live {
            if ttl_duration.as_secs_f32() > 0.0 {
                 *ttl_duration = ttl_duration.saturating_sub(time.delta());
                 if ttl_duration.is_zero() {
                    target_destroyed_events.send(TargetDestroyedEvent {
                        target_entity: entity,
                        points: 0,
                        destroyed_by_hit: false,
                    });
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

// System to handle target hit detection
pub fn detect_target_hits(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    target_query: Query<(Entity, &GlobalTransform, &Target, &Hitbox)>,
    mut target_hit_events: EventWriter<TargetHitEvent>,
    mut target_destroyed_events: EventWriter<TargetDestroyedEvent>,
    windows: Query<&Window>,
) {
    if !mouse_button_input.just_pressed(MouseButton::Left) { return; }

    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_position) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            let mut hit_detected = false; // Track if any hit occurred this click
            // Find the closest hit target
            let mut closest_hit: Option<(f32, Entity, Vec3, &Target)> = None;

            for (entity, transform, target, hitbox) in target_query.iter() {
                 // Skip targets still spawning
                if let Some(timer) = &target.spawn_time_instant {
                    if !timer.finished() { continue; }
                }

                if let Some(hit_distance) = hitbox.intersect_ray(ray, transform) {
                    hit_detected = true;
                    let hit_position = ray.origin + ray.direction * hit_distance;
                    if closest_hit.is_none() || hit_distance < closest_hit.unwrap().0 {
                         closest_hit = Some((hit_distance, entity, hit_position, target));
                    }
                }
            }

            // Process the closest hit target if one exists
            if let Some((_distance, entity, hit_position, target)) = closest_hit {
                 target_hit_events.send(TargetHitEvent {
                    target_entity: entity,
                    hit_position,
                    damage: 1, // Assuming 1 damage per hit
                });

                if target.destroy_on_hit {
                    target_destroyed_events.send(TargetDestroyedEvent {
                        target_entity: entity,
                        points: target.points,
                        destroyed_by_hit: true,
                    });
                    commands.entity(entity).despawn_recursive();
                }
            }
            // Removed implicit miss tracking here, handled by update_score_tracker
        }
    }
}


// System to handle target health (simplified, assumes destroy_on_hit most of the time)
pub fn update_target_health(
    mut commands: Commands,
    mut target_hit_events: EventReader<TargetHitEvent>,
    mut targets: Query<(Entity, &mut Target)>,
    mut target_destroyed_events: EventWriter<TargetDestroyedEvent>,
) {
    for hit_event in target_hit_events.read() {
        if let Ok((entity, mut target)) = targets.get_mut(hit_event.target_entity) {
             if !target.destroy_on_hit { // Only process health if not destroyed immediately
                 target.health -= hit_event.damage;
                if target.health <= 0 {
                    target_destroyed_events.send(TargetDestroyedEvent {
                        target_entity: entity,
                        points: target.points,
                        destroyed_by_hit: true,
                    });
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

// System to provide simple hit feedback (spawns a small sphere)
pub fn provide_hit_feedback(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut target_hit_events: EventReader<TargetHitEvent>,
) {
    for hit_event in target_hit_events.read() {
        commands.spawn(( // Removed unused `hit_effect` variable
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.1, ..default() })),
                material: materials.add(StandardMaterial { base_color: Color::YELLOW, emissive: Color::YELLOW, ..default() }),
                transform: Transform::from_translation(hit_event.hit_position),
                ..default()
            },
            NotShadowCaster,
            Lifetime { timer: Timer::new(Duration::from_millis(200), TimerMode::Once) },
        ));
    }
}

// System to update hit effect lifetimes
pub fn update_hit_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut hit_effects: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in hit_effects.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// Setup a basic spawner (simplified from spawner.rs setup)
pub fn setup_basic_target_spawner(mut commands: Commands) {
    commands.spawn(TargetSpawner {
        spawn_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        max_targets: 10,
        spawn_area_min: Vec3::new(-5.0, 0.5, -5.0),
        spawn_area_max: Vec3::new(5.0, 3.0, 5.0),
        target_radius: 0.5,
        target_color: Color::RED,
    });
}

// --- Aim Trainer Logic (Assuming it uses Target components/events) ---
// NOTE: Aim Trainer logic was referenced in mod.rs but not provided.
// Adding placeholders based on mod.rs systems.

#[derive(Component)]
pub struct AimTrainerScenario; // Placeholder component

pub fn setup_aim_trainer(mut commands: Commands) {
    // Placeholder: Setup resources or entities needed for the aim trainer
    println!("Aim Trainer Setup (Placeholder)");
}

pub fn update_aim_trainer() {
    // Placeholder: Update aim trainer state, scenarios, etc.
}

pub fn handle_aim_trainer_shortcuts() {
    // Placeholder: Handle keyboard input for aim trainer control
}

pub fn display_aim_trainer_ui() {
    // Placeholder: Display aim trainer UI elements
}

// NOTE: The 'main' function from target.rs is omitted here.
// The Bevy app should be run using the AimTestPlugin structure below.


// --- timing.rs Code ---

// Resource to track frame timing information
#[derive(Resource)]
#[allow(dead_code)]
pub struct FrameTimingInfo {
    pub frame_times: VecDeque<Duration>,
    pub max_samples: usize,
    pub average_frame_time: Duration,
    pub average_fps: f32,
    pub vsync_enabled: bool,
    pub target_fps: Option<f32>,
}

impl Default for FrameTimingInfo {
    fn default() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            max_samples: 120,
            average_frame_time: Duration::from_secs(0),
            average_fps: 0.0,
            vsync_enabled: true,
            target_fps: None,
        }
    }
}

// System to setup frame timing
#[allow(dead_code)]
pub fn setup_frame_timing(mut commands: Commands) {
    commands.insert_resource(FrameTimingInfo::default());
}

// System to update frame timing information
pub fn update_frame_timing(
    time: Res<Time>,
    mut timing_info: ResMut<FrameTimingInfo>,
) {
    let frame_time = time.delta();

    // Add current frame time to the buffer
    timing_info.frame_times.push_back(frame_time);
    if timing_info.frame_times.len() > timing_info.max_samples {
        timing_info.frame_times.pop_front();
    }

    // Calculate average frame time
    if !timing_info.frame_times.is_empty() {
        let total_time: Duration = timing_info.frame_times.iter().sum();
        timing_info.average_frame_time = total_time / timing_info.frame_times.len() as u32;

        // Calculate average FPS
        let frame_time_secs = timing_info.average_frame_time.as_secs_f32();
        if frame_time_secs > 0.0 {
            timing_info.average_fps = 1.0 / frame_time_secs;
        }
    }
}

// System to toggle vsync
pub fn toggle_vsync(
    keyboard_input: Res<Input<KeyCode>>,
    mut timing_info: ResMut<FrameTimingInfo>,
) {
    if keyboard_input.just_pressed(KeyCode::V) {
        timing_info.vsync_enabled = !timing_info.vsync_enabled;
        // Note: Actually changing vsync requires modifying the window settings
        // This will be connected to the window module
        println!("VSync Toggled (Config Only): {}", timing_info.vsync_enabled); // Added feedback
    }
}


// --- window.rs Code ---

// Resource to store window configuration
#[derive(Resource)]
#[allow(dead_code)]
pub struct WindowConfig {
    pub fullscreen: bool,
    pub vsync: bool,
    pub cursor_grabbed: bool,
    pub width: f32,
    pub height: f32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            fullscreen: false,
            vsync: true, // Match FrameTimingInfo default
            cursor_grabbed: true,
            width: 1280.0,
            height: 720.0,
        }
    }
}

// System to setup window configuration
#[allow(dead_code)]
pub fn setup_window_config(mut commands: Commands) {
    commands.insert_resource(WindowConfig::default());
}

// System to apply window configuration
pub fn apply_window_config(
    window_config: Res<WindowConfig>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut timing_info: ResMut<FrameTimingInfo>, // Added to sync vsync
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        // Set window mode (fullscreen or windowed)
        window.mode = if window_config.fullscreen {
            WindowMode::Fullscreen
        } else {
            WindowMode::Windowed
        };

        // Set cursor grab mode
        window.cursor.grab_mode = if window_config.cursor_grabbed {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };

        // Hide cursor when grabbed
        window.cursor.visible = !window_config.cursor_grabbed;

        // Set window size if not fullscreen
        if !window_config.fullscreen {
            window.resolution.set(window_config.width, window_config.height);
        }

        // Apply vsync setting (Requires WindowPlugin configuration usually)
        // This system *shows* the intent, but actual vsync is often set during App build
         if window_config.is_changed() || timing_info.is_changed() {
            // Sync timing_info's vsync state with window_config
            if timing_info.vsync_enabled != window_config.vsync {
                 timing_info.vsync_enabled = window_config.vsync; // Keep them in sync
            }
            // Attempt to set VSync (Best effort - might need Plugin setup)
            window.present_mode = if window_config.vsync {
                 bevy::window::PresentMode::AutoVsync
             } else {
                 bevy::window::PresentMode::AutoNoVsync
             };
             println!("Applied Window Config: VSync = {}", window_config.vsync); // Added feedback
         }
    }
}

// System to toggle cursor grab with Escape key
pub fn toggle_cursor_grab(
    keyboard_input: Res<Input<KeyCode>>,
    mut window_config: ResMut<WindowConfig>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        window_config.cursor_grabbed = !window_config.cursor_grabbed;

        if let Ok(mut window) = primary_window.get_single_mut() {
            window.cursor.grab_mode = if window_config.cursor_grabbed {
                CursorGrabMode::Locked
            } else {
                CursorGrabMode::None
            };

            window.cursor.visible = !window_config.cursor_grabbed;
        }
    }
}

// --- world.rs Code ---

// System to setup the 3D environment
pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add a ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 50.0,
            subdivisions: 0, // Set subdivisions explicitly
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.3, 0.3),
            perceptual_roughness: 1.0,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    // Add directional light
    commands.spawn(DirectionalLightBundle { // Changed from PointLight based on setup_scene
        directional_light: DirectionalLight { // Use DirectionalLight component
            illuminance: 10000.0, // Adjusted intensity
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y), // Point towards origin
        ..default()
    });

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2, // Adjusted brightness
    });
}


// System to create a simple target for testing
pub fn spawn_test_target(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a spherical target
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: 0.5,
            sectors: 32, // Added sectors/stacks for smoothness
            stacks: 32,
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(1.0, 0.0, 0.0),
            emissive: Color::rgb(0.5, 0.0, 0.0), // Make it glow slightly
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 1.7, -5.0), // Position it in front of camera
        ..default()
    });
}


// --- mod.rs Code (Adapted) ---

// System set for operations that must run on the main thread
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct OnRootThread;

pub struct AimTestPlugin;

impl Plugin for AimTestPlugin {
    fn build(&self, app: &mut App) {
        // Add necessary Bevy plugins (if not already added by main app)
        // app.add_plugins(DefaultPlugins); // Usually added by the main application

        // Add custom plugins defined within this combined file
        app.add_plugins(AnalysisPlugin)
           .add_plugins(CombinedCalibrationPlugin) // Added CombinedCalibrationPlugin
           .add_plugins(SensitivityPlugin); // Added SensitivityPlugin

        // Add resources (ensure defaults match if defined elsewhere)
        app.init_resource::<MouseInputBuffer>() // Use init_resource for default
           .init_resource::<WindowConfig>()
           .init_resource::<FrameTimingInfo>()
           .init_resource::<ScoreTracker>(); // Add ScoreTracker init

        // Add events
        app.add_event::<TargetHitEvent>()
           .add_event::<TargetDestroyedEvent>()
           .add_event::<TargetSpawnedEvent>();

        // Add startup systems (using direct function names)
        app.add_systems(Startup, (
            setup_camera,
            setup_player,
            setup_world,
            spawn_test_target, // Keep this for initial testing?
            // setup_mouse_input, // Handled by init_resource
            // setup_window_config, // Handled by init_resource
            // setup_frame_timing, // Handled by init_resource
            // setup_score_tracker, // Handled by init_resource
            setup_sensitivity_system, // From SensitivityPlugin
            load_profiles, // From SensitivityPlugin
            setup_sensitivity_test_ui, // From SensitivityPlugin
            setup_basic_target_spawner,
            setup_aim_trainer, // Placeholder from target.rs section
            // Calibration setups are handled by CombinedCalibrationPlugin
        ));

        // Add update systems - split into multiple sets if needed
        // Note: Systems from sub-plugins (Analysis, Calibration, Sensitivity) are added by those plugins
        app.add_systems(Update, (
            // Core systems
            update_camera,
            player_movement,
            process_mouse_input,
            process_mouse_buttons,
            // Window and timing systems
            apply_window_config,
            toggle_cursor_grab,
            update_frame_timing,
            toggle_vsync,
            // Target systems
            detect_target_hits,
            update_target_health,
            update_target_lifetime,
            provide_hit_feedback,
            update_hit_effects,
            update_target_spawners,
            update_score_tracker,
            display_score,
            update_target_movement,
            // Aim trainer systems (Placeholders)
            update_aim_trainer,
            handle_aim_trainer_shortcuts,
            display_aim_trainer_ui,
            // -- Systems added by sub-plugins --
            // Analysis systems (from AnalysisPlugin)
            // Calibration systems (from CombinedCalibrationPlugin)
            // Sensitivity systems (from SensitivityPlugin)
            // Ensure Bevy utility systems run
            bevy::window::close_on_esc,
        ));

        // Example of adding systems to a specific set (if needed, though CalibrationPlugin handles its own)
        // .add_systems(Update, (
        //     some_main_thread_system,
        // ).in_set(OnRootThread))
        // ;
    }
}

// --- Optional: Example Main Function ---
// If you want to run this combined code as a standalone app,
// you would need a main function like this. Usually, you'd add
// the AimTestPlugin to an existing Bevy App builder.

/*
fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // Add default plugins here if running standalone
        .add_plugins(AimTestPlugin) // Add our combined plugin
        .run();
}
*/