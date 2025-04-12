use bevy::{
    prelude::*,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, EntityCountDiagnosticsPlugin},
    window::PresentMode,
    core::FrameCount,
};

// Resource to store performance settings
#[derive(Resource)]
pub struct PerformanceSettings {
    pub vsync_mode: PresentMode,
    pub reduce_draw_calls: bool,
    pub last_optimization_time: f32,
    pub optimization_interval: f32, // How often to check performance in seconds
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            vsync_mode: PresentMode::AutoVsync,
            reduce_draw_calls: false,
            last_optimization_time: 0.0,
            optimization_interval: 5.0, // Check every 5 seconds
        }
    }
}

// Component for FPS counter
#[derive(Component)]
pub struct FpsCounter;

// System to setup the FPS counter UI
pub fn setup_fps_counter(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "0",
                TextStyle {
                    font_size: 16.0,
                    color: Color::GREEN,
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        }),
        FpsCounter,
    ));
}

// System to update the FPS counter
pub fn update_fps_counter(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsCounter>>,
) {
    // Update once per frame
    if let Ok(mut text) = query.get_single_mut() {
        // Try to get FPS diagnostic
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            // Try to get smoothed value first
            if let Some(value) = fps.smoothed() {
                text.sections[1].value = format!("{value:.1}");

                // Update color based on FPS
                text.sections[1].style.color = if value >= 120.0 {
                    Color::GREEN
                } else if value >= 60.0 {
                    Color::YELLOW
                } else {
                    Color::RED
                };
            }
            // If smoothed not available, try to get raw value
            else if let Some(raw_value) = fps.value() {
                text.sections[1].value = format!("{raw_value:.1}");
            }
            // If no values available yet, show frame time
            else {
                let dt = time.delta_seconds();
                if dt > 0.0 {
                    let fps_estimate = 1.0 / dt;
                    text.sections[1].value = format!("{fps_estimate:.1}*");
                }
            }
        }
        // If diagnostic not available, calculate from delta time
        else {
            let dt = time.delta_seconds();
            if dt > 0.0 {
                let fps_estimate = 1.0 / dt;
                text.sections[1].value = format!("{fps_estimate:.1}*");
            }
        }
    }
}

// Performance monitoring in background
pub fn monitor_performance(
    diagnostics: Res<DiagnosticsStore>,
) {
    // Still monitor FPS for adaptive performance adjustments
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            // Log significant performance changes to console
            if value < 30.0 {
                // Only log once per second to avoid spam
                if (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() % 5) == 0 {
                    println!("Performance warning: Low FPS detected ({:.1})", value);
                }
            }
        }
    }
}

// System to toggle vsync mode with F1 key
pub fn toggle_vsync(
    keys: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<PerformanceSettings>,
    mut windows: Query<&mut Window>,
) {
    if keys.just_pressed(KeyCode::F1) {
        settings.vsync_mode = match settings.vsync_mode {
            PresentMode::AutoVsync => {
                println!("VSync: Off (Immediate)");
                PresentMode::Immediate
            }
            PresentMode::Immediate => {
                println!("VSync: On (AutoVsync)");
                PresentMode::AutoVsync
            }
            _ => {
                println!("VSync: On (AutoVsync)");
                PresentMode::AutoVsync
            }
        };

        // Apply to all windows
        for mut window in windows.iter_mut() {
            window.present_mode = settings.vsync_mode;
        }
    }
}

// System to dynamically adjust performance settings based on FPS
pub fn adaptive_performance(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut settings: ResMut<PerformanceSettings>,
    mut windows: Query<&mut Window>,
    frame_count: Res<FrameCount>,
) {
    // Only check periodically to avoid constant changes
    if time.elapsed_seconds() - settings.last_optimization_time < settings.optimization_interval {
        return;
    }

    // Skip the first few frames to let the game stabilize
    if frame_count.0 < 100 {
        return;
    }

    settings.last_optimization_time = time.elapsed_seconds();

    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            // If FPS is too low, try to improve performance
            if value < 50.0 {
                // Switch to immediate mode to reduce vsync overhead
                if settings.vsync_mode != PresentMode::Immediate {
                    settings.vsync_mode = PresentMode::Immediate;
                    for mut window in windows.iter_mut() {
                        window.present_mode = settings.vsync_mode;
                    }
                    println!("Performance: Disabled VSync to improve FPS");
                }

                // Enable reduced draw calls
                if !settings.reduce_draw_calls {
                    settings.reduce_draw_calls = true;
                    println!("Performance: Reduced draw calls to improve FPS");
                }
            }
            // If FPS is very high, we can afford to enable vsync for smoother experience
            else if value > 120.0 && settings.vsync_mode == PresentMode::Immediate {
                settings.vsync_mode = PresentMode::AutoVsync;
                for mut window in windows.iter_mut() {
                    window.present_mode = settings.vsync_mode;
                }
                println!("Performance: Enabled VSync for smoother experience");
            }
        }
    }
}

// Plugin to organize performance systems
pub struct PerformancePlugin;

// Combined plugin that includes both performance optimization and FPS counter
impl Plugin for PerformancePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerformanceSettings>()
           // Make sure to add the diagnostic plugins
           .add_plugins(FrameTimeDiagnosticsPlugin::default())
           .add_plugins(EntityCountDiagnosticsPlugin::default())
           .add_systems(Startup, setup_fps_counter)
           .add_systems(Update, (
               update_fps_counter,
               monitor_performance,
               toggle_vsync,
               adaptive_performance,
           ));
    }
}

// Removed unused configure_for_performance function
