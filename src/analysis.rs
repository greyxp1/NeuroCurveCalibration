use bevy::prelude::*;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crate::{
    input::{MouseInputBuffer, MouseSample}, // Use MouseInputBuffer from input module
    target::{TargetHitEvent, ScoreTracker}, // Use events/resources from target module
};

// Constants for analysis (adjust as needed)
const MICRO_ADJUSTMENT_THRESHOLD_SPEED: f32 = 50.0; // Speed threshold (pixels/sec) for micro-adjustments
const OVERSHOOT_ANGLE_THRESHOLD: f32 = 10.0; // Angle change threshold (degrees) for overshoot detection
const SPEED_SAMPLE_WINDOW_DURATION: Duration = Duration::from_millis(500); // Time window for speed analysis
const MAX_HISTORY_SAMPLES: usize = 1000; // Max aim samples to store

// Data structure for detailed aim information per frame or event
#[derive(Debug, Clone)]
pub struct AimSample {
    pub timestamp: Instant,
    pub mouse_delta: Vec2, // Raw mouse delta for this frame/event
    pub mouse_speed: f32, // Calculated speed (pixels/sec)
    pub target_hit: bool, // Was a target hit in this frame/event?
    pub target_position: Option<Vec3>, // Position of the target hit (if any)
    pub camera_direction: Vec3, // Camera direction at the time of the sample
    // Add other relevant data as needed (e.g., target distance, angular distance)
}

// Data structure to represent detected micro-adjustments
#[derive(Debug, Clone)]
pub struct MicroAdjustment {
    pub timestamp: Instant,
    pub magnitude: f32, // Magnitude of the adjustment movement
    pub duration: Duration, // Duration of the adjustment
}

// Resource to store advanced aiming metrics over time
#[derive(Resource, Default)]
pub struct AdvancedMetrics {
    pub aim_samples: VecDeque<AimSample>, // History of aim samples
    pub micro_adjustments: Vec<MicroAdjustment>, // Detected micro-adjustments
    pub overshoots_detected: i32, // Count of detected overshoots
    pub undershoots_detected: i32, // Count of detected undershoots (optional)
    pub average_reaction_time_ms: Option<f32>, // Average time from target spawn to hit
    pub time_to_kill_ms: Vec<f32>, // Time taken to destroy targets
    pub last_target_spawn_time: Option<Instant>, // For reaction time calc
    pub last_target_hit_time: Option<Instant>, // For TTK calc
    // Add more metrics: flick accuracy, tracking accuracy, smoothness, etc.
}

// System to initialize the AdvancedMetrics resource
pub fn setup_advanced_metrics(mut commands: Commands) {
    commands.init_resource::<AdvancedMetrics>();
}

// System to collect aim data every frame (or based on events)
pub fn update_advanced_metrics(
    mut metrics: ResMut<AdvancedMetrics>,
    mut hit_events: EventReader<TargetHitEvent>,
    mouse_buffer: Res<MouseInputBuffer>,
    camera_query: Query<&Transform, With<crate::camera::FpsCamera>>, // Use FpsCamera from camera module
    time: Res<Time>, // Use Bevy's Time resource
) {
    let now = Instant::now();
    let Ok(camera_transform) = camera_query.get_single() else { return; }; // Need camera transform

    // --- Collect Mouse Input Sample ---
    let latest_mouse_sample = mouse_buffer.samples.back(); // Get the most recent sample
    let (mouse_delta, mouse_speed) = latest_mouse_sample.map_or((Vec2::ZERO, 0.0), |s| (s.delta, s.speed));

    // --- Check for Target Hit ---
    let mut target_hit_this_frame = false;
    let mut hit_position = None;
    for hit_event in hit_events.read() {
        target_hit_this_frame = true;
        hit_position = Some(hit_event.hit_position);
        metrics.last_target_hit_time = Some(now); // Update last hit time
        // Could calculate reaction time here if spawn time is tracked
        // if let Some(spawn_time) = metrics.last_target_spawn_time {
        //     let reaction = now.duration_since(spawn_time).as_secs_f32() * 1000.0;
        //     // Store or average reaction time
        // }
    }

    // --- Create and Store Aim Sample ---
    let sample = AimSample {
        timestamp: now,
        mouse_delta,
        mouse_speed,
        target_hit: target_hit_this_frame,
        target_position: hit_position,
        camera_direction: camera_transform.forward(),
    };

    metrics.aim_samples.push_back(sample);
    if metrics.aim_samples.len() > MAX_HISTORY_SAMPLES {
        metrics.aim_samples.pop_front();
    }
}

// System to detect overshooting based on rapid direction changes after passing a target
pub fn calculate_overshooting(
    mut metrics: ResMut<AdvancedMetrics>,
    // Requires access to recent aim samples and potentially target positions
) {
    if metrics.aim_samples.len() < 5 { return; } // Need a few samples

    // Iterate through recent samples, looking for patterns like:
    // 1. High speed movement towards a target area
    // 2. Movement continues past the target area
    // 3. Rapid direction change back towards the target area

    // This is complex to implement accurately. A simpler heuristic:
    // Look for large, rapid changes in mouse delta direction.
    let recent_samples: Vec<&AimSample> = metrics.aim_samples.iter().rev().take(5).collect();
    let delta1 = recent_samples[0].mouse_delta;
    let delta2 = recent_samples[1].mouse_delta;
    let delta3 = recent_samples[2].mouse_delta;

    if delta1.length_squared() > 1.0 && delta2.length_squared() > 1.0 {
         // Check angle between consecutive movements
        let angle = delta1.angle_between(delta2).to_degrees();
        // Check angle change between delta1/2 and delta2/3 for reversal
        let angle_prev = delta2.angle_between(delta3).to_degrees();


        // Detect sharp reversal (angle > 90 deg) after significant movement
        if angle.abs() > 90.0 && delta1.length() + delta2.length() > 10.0 { // Adjust threshold
             // More sophisticated: Check if this reversal happened after passing near a target
             // Requires correlating with target positions from AimSamples
            // println!("Potential Overshoot Detected (Sharp Angle Change)"); // Debug log
            // metrics.overshoots_detected += 1; // Increment cautiously
        }
    }
}


// System to detect micro-adjustments (small, quick movements often before a hit)
pub fn analyze_micro_adjustments(
    mut metrics: ResMut<AdvancedMetrics>,
    // Requires access to recent aim samples
) {
     if metrics.aim_samples.len() < 3 { return; }

    // Iterate through recent samples, looking for:
    // 1. A period of low-speed movement (below MICRO_ADJUSTMENT_THRESHOLD_SPEED)
    // 2. Followed by a target hit shortly after

    let window_size = 5; // Look at the last 5 samples
    if metrics.aim_samples.len() < window_size { return; }

    let recent_samples = metrics.aim_samples.iter().rev().take(window_size).collect::<Vec<_>>();

    // Check if the latest sample is a hit
    if recent_samples[0].target_hit {
        // Check if preceding samples had low speed
        let mut low_speed_streak = 0;
        for i in 1..window_size {
            if recent_samples[i].mouse_speed < MICRO_ADJUSTMENT_THRESHOLD_SPEED {
                low_speed_streak += 1;
            } else {
                break; // Streak broken
            }
        }

        // If there was a period of low speed right before the hit, consider it a micro-adjustment phase
        if low_speed_streak > 1 { // Require at least 2 low-speed samples before hit
             // Create and store MicroAdjustment data (needs more detail: magnitude, duration)
            // let adjustment = MicroAdjustment { ... };
            // metrics.micro_adjustments.push(adjustment);
            // println!("Potential Micro-Adjustment Detected Before Hit"); // Debug log
        }
    }
}

// System placeholder for analyzing speed-accuracy correlation
pub fn analyze_speed_accuracy_correlation(
    metrics: Res<AdvancedMetrics>,
    score: Res<ScoreTracker>,
    // Requires analysis of aim samples over time
) {
    // Calculate correlation between mouse_speed in AimSamples
    // and whether those samples (or subsequent shots) resulted in hits.
    // This requires more sophisticated statistical analysis over a larger sample set.

    // Simple placeholder: Print current accuracy vs average speed in last N samples
    if metrics.aim_samples.len() > 10 {
        let avg_speed: f32 = metrics.aim_samples.iter()
            .rev()
            .take(50) // Look at last 50 samples
            .map(|s| s.mouse_speed)
            .sum::<f32>() / 50.0;

        // Print occasionally
         if metrics.aim_samples.len() % 60 == 0 { // Print every ~second
             // println!("Avg Speed (last 50 samples): {:.2} px/s, Current Accuracy: {:.1}%", avg_speed, score.accuracy);
         }
    }
}


// Plugin structure for analysis systems
pub struct AnalysisPlugin;

impl Plugin for AnalysisPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AdvancedMetrics>() // Ensure resource is initialized
           .add_systems(Startup, setup_advanced_metrics)
           .add_systems(Update, (
               update_advanced_metrics, // Collect data first
               // Run analysis systems after data collection
               (calculate_overshooting,
                analyze_micro_adjustments,
                analyze_speed_accuracy_correlation,
               ).after(update_advanced_metrics),
           ));
    }
}