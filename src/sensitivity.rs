use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap; // Keep HashMap if needed for profiles/game settings

// --- Sensitivity Curve Parameters ---

// Using f32 for Bevy compatibility, adjust if f64 precision is truly needed
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CurveParameters {
    pub min_sens: f32,
    pub max_sens: f32,
    pub range: f32, // Input speed range over which sens increases (e.g., pixels/sec)
    pub growth_base: f32, // Exponent base for curve shape (>1 = exponential, 1 = linear, <1 = decelerating)
    pub offset: f32, // Input speed offset before curve starts
    pub plateau: bool, // Whether sensitivity stays at max_sens after range
    // Smoothing might be handled separately in input processing or camera updates
}

impl Default for CurveParameters {
    fn default() -> Self {
        Self {
            min_sens: 1.0, // Base sensitivity multiplier
            max_sens: 3.0, // Max sensitivity multiplier
            range: 500.0, // Sensitivity increases up to 500 pixels/sec input speed
            growth_base: 1.02, // Slight exponential curve
            offset: 50.0, // No sensitivity change below 50 pixels/sec
            plateau: true, // Stay at max_sens after reaching range limit
        }
    }
}

// --- Sensitivity Curve Logic ---

// This function calculates the sensitivity multiplier based on input speed
// It replaces the need for a precomputed lookup table in this simplified version
pub fn calculate_sensitivity_multiplier(speed: f32, params: &CurveParameters) -> f32 {
    if speed <= params.offset {
        return params.min_sens;
    }

    let input_after_offset = speed - params.offset;
    let sens_diff = params.max_sens - params.min_sens;

    if sens_diff <= 0.0 || params.range <= 0.0 {
        return params.min_sens; // Avoid division by zero or invalid range
    }

    // Normalized progress within the range [0, 1]
    let t = (input_after_offset / params.range).clamp(0.0, 1.0);

    let multiplier_increase = if params.growth_base <= 1.0 {
        // Linear or decelerating (using smoothstep-like interpolation for base=1)
        let t2 = t * t;
        sens_diff * (t2 * (3.0 - 2.0 * t)) // Smoothstep for linear-like transition
    } else {
        // Exponential growth
        let base_factor = 1.0 / (params.growth_base.powf(params.range) - 1.0); // Denominator correction? Check formula
        let expo_t = ((params.growth_base.powf(input_after_offset) - 1.0) * base_factor).clamp(0.0, 1.0); // Apply expo to input speed
        sens_diff * expo_t
        // Alternative simple exponential: sens_diff * t.powf(params.growth_base) - needs tuning
    };

    let calculated_sens = params.min_sens + multiplier_increase;

    if params.plateau {
        calculated_sens.min(params.max_sens) // Clamp to max_sens if plateauing
    } else {
        // If not plateauing, allow exceeding max_sens based on formula (unusual)
        // Ensure we don't exceed max_sens if t >= 1.0 even without plateau
        if t >= 1.0 {
             params.max_sens
        } else {
            calculated_sens
        }
    }
}


// --- Bevy Integration ---

// Placeholder: System to apply sensitivity curve to mouse input buffer or camera
// This system needs to run *after* process_mouse_input and *before* update_camera
pub fn apply_sensitivity_curve(
    mut mouse_buffer: ResMut<crate::input::MouseInputBuffer>, // Modify the buffer directly
    params: Res<CurveParameters>, // Get curve parameters
    // Or query camera directly: mut camera_query: Query<&mut crate::camera::FpsCamera>,
) {
    // Option 1: Modify MouseInputBuffer sensitivity_multiplier based on latest speed
    if let Some(latest_sample) = mouse_buffer.samples.back() {
         mouse_buffer.sensitivity_multiplier = calculate_sensitivity_multiplier(latest_sample.speed, &params);
         // The process_mouse_input system will then use this multiplier for new samples
         // OR process_mouse_input could call calculate_sensitivity_multiplier directly
    }

    // Option 2: Apply sensitivity directly in the camera update system
    // (Requires passing CurveParameters resource to update_camera)

    // Option 3: Modify the delta in the existing samples (less ideal)
    // for sample in mouse_buffer.samples.iter_mut() {
    //     let multiplier = calculate_sensitivity_multiplier(sample.speed, &params);
    //     // Re-apply or adjust delta based on multiplier? Complicated.
    // }
}

// --- Game Sensitivity Conversion (Simplified / Placeholder) ---
// Full game sensitivity profiles and conversions are complex and likely belong
// in the main Tauri application logic, potentially passed to the aim trainer at launch.

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Game {
    Valorant,
    CSGO,
    Overwatch,
    // ... add more games
}

// Placeholder resource for game-specific settings if needed inside Bevy
#[derive(Resource, Default)]
pub struct GameSensitivitySettings {
    pub current_game: Option<Game>,
    pub base_sens: f32, // e.g., cm/360 from Tauri app
    pub game_specific_multiplier: f32, // Multiplier for the selected game
}


// --- Plugin ---

pub struct SensitivityPlugin;

impl Plugin for SensitivityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurveParameters>() // Initialize with default parameters
           // .init_resource::<GameSensitivitySettings>() // Optional: if game settings are needed here
           .add_systems(Update,
                apply_sensitivity_curve
                    .after(crate::input::process_mouse_input) // After raw input is processed
                    .before(crate::camera::update_camera) // Before camera rotation is calculated
            );
            // Add systems for updating CurveParameters via debug UI/commands if needed
    }
}

// Original generate_sensitivity_curve function (kept for reference, but replaced by calculate_sensitivity_multiplier)
/*
pub fn generate_sensitivity_curve(
    n: usize,
    growth_base: f64,
    range: f64,
    min_sens: f64,
    max_sens: f64,
    offset: f64,
    plateau: bool,
) -> Vec<f64> {
    // ... implementation ...
}
*/