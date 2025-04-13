use bevy::{prelude::*, render::camera::Exposure, time::{Stopwatch, Timer, TimerMode}, window::CursorGrabMode};
use bevy_diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy_fps_controller::controller::*;
use bevy_rapier3d::prelude::*;
use rand::{distr::Uniform, prelude::*};
use std::f32::consts::TAU;

// Game constants
const SENSITIVITY_CM_PER_360: f32 = 10.0;
const MOUSE_DPI: f32 = 1600.0;
const ARENA_WIDTH: f32 = 200.0;
const ARENA_DEPTH: f32 = 200.0;
const ARENA_HEIGHT: f32 = 50.0;
// Position player in the middle of the arena, facing the front wall (negative Z)
const SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.625, 0.0);
const WALL_THICKNESS: f32 = 1.0;
const TARGET_SIZE: f32 = 1.5;
const PLAYER_HEIGHT: f32 = 10.0;
const PLAYER_RADIUS: f32 = 0.5;
const CAMERA_HEIGHT_OFFSET: f32 = 4.0;
// Valorant uses a vertical FOV of 90 degrees (approximately 1.57 radians)
const CAMERA_FOV: f32 = std::f32::consts::FRAC_PI_2; // 90 degrees in radians
const CENTER_SIZE: f32 = 8.0;
const GRID_SPACING: f32 = 4.0;

// Scenario constants
const SCENARIO_DURATION: f32 = 30.0; // Duration of each scenario in seconds
const SCENARIO_DELAY: f32 = 5.0; // Delay between scenarios in seconds

// Component and resource definitions
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct FpsControllerSetup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ScenarioType {
    // Clicking scenarios
    DynamicClicking,  // Targets move in unpredictable patterns
    StaticClicking,   // Targets are stationary
    LinearClicking,   // Targets move in straight lines

    // Tracking scenarios
    PreciseTracking,  // Slow, precise movements requiring accuracy
    ReactiveTracking, // Quick, sudden movements requiring fast reactions
    ControlTracking,  // Smooth, consistent movements requiring control

    // Switching scenarios
    SpeedSwitching,   // Fast target switching with emphasis on speed
    EvasiveSwitching, // Targets that try to evade the crosshair
    StabilitySwitching // Targets that require stability between switches
}

#[derive(Resource)]
struct ScenarioState {
    current_type: Option<ScenarioType>,
    scenario_timer: Timer,
    delay_timer: Timer,
    current_index: usize,
    is_active: bool,
    has_started: bool,
    scenarios: Vec<ScenarioType>,
}

impl Default for ScenarioState {
    fn default() -> Self {
        Self {
            current_type: None,
            scenario_timer: Timer::from_seconds(SCENARIO_DURATION, TimerMode::Once),
            delay_timer: Timer::from_seconds(SCENARIO_DELAY, TimerMode::Once),
            current_index: 0,
            is_active: false,
            has_started: false,
            scenarios: vec![
                ScenarioType::DynamicClicking,
                ScenarioType::StaticClicking,
                ScenarioType::LinearClicking,
                ScenarioType::PreciseTracking,
                ScenarioType::ReactiveTracking,
                ScenarioType::ControlTracking,
                ScenarioType::SpeedSwitching,
                ScenarioType::EvasiveSwitching,
                ScenarioType::StabilitySwitching,
            ],
        }
    }
}

#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Component, Default)]
pub struct Target;

#[derive(Component, Debug)]
struct TargetMovement {
    velocity: Vec3,
    pattern: MovementPattern,
    timer: f32,
    start_position: Vec3,
    max_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MovementPattern {
    Static,
    Linear,
    Circular,
    Random,
    Reactive,
    Smooth,
    Evasive,
}

#[derive(Component)]
struct ScenarioDisplay;

#[derive(Component)]
struct PointsDisplay;

#[derive(Component)]
struct FpsDisplay;

#[derive(Default, Resource)]
struct Points { pub value: i32 }

#[derive(Component)]
struct ShootTracker { stopwatch: Stopwatch }

fn main() {
    App::new()
        .insert_resource(AmbientLight { color: Color::WHITE, brightness: 2000.0 })
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .insert_resource(Points::default())
        .insert_resource(ScenarioState::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Aim Trainer"),
                present_mode: bevy::window::PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((FrameTimeDiagnosticsPlugin::default(),
                     RapierPhysicsPlugin::<NoUserData>::default(), FpsControllerPlugin))
        .add_systems(Startup, (setup, fps_controller_setup.in_set(FpsControllerSetup)))
        .add_systems(Update, (
            respawn,
            manage_cursor,
            click_targets,
            update_fps_display,
            update_points_display,
            manage_scenarios,
            update_scenario_display,
            update_target_movements,
        ))
        .run();
}



fn fps_controller_setup(mut commands: Commands) {
    // Create player entity
    let logical_entity = spawn_player(&mut commands);

    // Create camera entity
    spawn_camera(&mut commands, logical_entity);
}

fn spawn_player(commands: &mut Commands) -> Entity {
    // Calculate sensitivity based on cm/360
    // Formula: sensitivity = (2*PI) / (cm_per_360 / 2.54 * dpi)
    let sensitivity = TAU / (SENSITIVITY_CM_PER_360 / 2.54 * MOUSE_DPI);

    commands
        .spawn((
            Collider::cylinder(PLAYER_HEIGHT / 2.0, PLAYER_RADIUS),
            Friction { coefficient: 0.0, combine_rule: CoefficientCombineRule::Min },
            Restitution { coefficient: 0.0, combine_rule: CoefficientCombineRule::Min },
            ActiveEvents::COLLISION_EVENTS, Velocity::zero(), RigidBody::Dynamic,
            Sleeping::disabled(), LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0), GravityScale(0.0), Ccd { enabled: true },
            Transform::from_translation(SPAWN_POINT), LogicalPlayer,
            // Set pitch to 0 (looking straight ahead) and yaw to 0 (facing negative Z direction)
            FpsControllerInput { pitch: 0.0, yaw: 0.0, ..default() },
            FpsController {
                air_acceleration: 80.0,
                sensitivity, // Use our calculated cm/360 sensitivity
                ..default()
            },
        ))
        .insert(CameraConfig { height_offset: CAMERA_HEIGHT_OFFSET })
        .insert(ShootTracker { stopwatch: Stopwatch::new() })
        .insert(SpatialListener::new(0.5))
        .id()
}

fn spawn_camera(commands: &mut Commands, logical_entity: Entity) {
    commands.spawn((
        Camera3d::default(),
        Camera { order: 0, ..default() },
        Projection::Perspective(PerspectiveProjection { fov: CAMERA_FOV, ..default() }),
        Exposure::SUNLIGHT,
        RenderPlayer { logical_entity },
    ));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut materials2d: ResMut<Assets<ColorMaterial>>,
) {
    // Setup 2D camera for UI
    commands.spawn((Camera2d, Camera { order: 2, ..default() }));

    // Lighting setup
    setup_lighting(&mut commands);

    // Materials
    let materials_handles = setup_materials(&mut materials);

    // Arena construction
    setup_arena(&mut commands, &mut meshes, &materials_handles);

    // Spawn targets
    for _ in 0..10 {
        spawn_random_target(&mut commands, &mut meshes, &mut materials);
    }

    // UI elements - dot crosshair and text displays
    let crosshair_material = materials2d.add(Color::srgb(0.0, 1.0, 1.0));

    // Create a small dot as the crosshair
    let dot_size = 2.0; // Size of the dot in pixels
    commands.spawn((Mesh2d(meshes.add(Cuboid::new(dot_size, dot_size, 0.0))),
                   MeshMaterial2d(crosshair_material),
                   Transform::default()));

    // Text displays
    commands.spawn((Text::new("Points: 0"),
                   Node { position_type: PositionType::Absolute, bottom: Val::Px(5.), left: Val::Px(15.), ..default() },
                   PointsDisplay));
    commands.spawn((Text::new("FPS: 0"),
                   Node { position_type: PositionType::Absolute, top: Val::Px(5.), right: Val::Px(15.), ..default() },
                   FpsDisplay));
    commands.spawn((Text::new(format!("Sensitivity: {:.1} cm/360 @ {} DPI", SENSITIVITY_CM_PER_360, MOUSE_DPI as i32)),
                   Node { position_type: PositionType::Absolute, bottom: Val::Px(5.), right: Val::Px(15.), ..default() }));
    commands.spawn((Text::new("Press SPACE to start scenarios"),
                   Node { position_type: PositionType::Absolute, top: Val::Px(50.), left: Val::Px(15.), ..default() },
                   ScenarioDisplay));
}

fn setup_lighting(commands: &mut Commands) {
    // Main directional light
    commands.spawn((DirectionalLight { illuminance: light_consts::lux::OVERCAST_DAY, shadows_enabled: true, ..default() },
                   Transform::from_xyz(0.0, 40.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z)));

    // Center light
    commands.spawn((PointLight { color: Color::srgb(0.9, 0.9, 1.0), intensity: 10000.0, range: 120.0, ..default() },
                   Transform::from_xyz(0.0, 20.0, 0.0)));

    // Corner lights
    for (x, z) in [(-ARENA_WIDTH/4.0, -ARENA_DEPTH/4.0), (ARENA_WIDTH/4.0, -ARENA_DEPTH/4.0),
                  (-ARENA_WIDTH/4.0, ARENA_DEPTH/4.0), (ARENA_WIDTH/4.0, ARENA_DEPTH/4.0)] {
        commands.spawn((PointLight { color: Color::srgb(0.8, 0.8, 1.0), intensity: 5000.0, range: 80.0, ..default() },
                       Transform::from_xyz(x, 20.0, z)));
    }
}

struct MaterialHandles {
    ground: Handle<StandardMaterial>,
    wall: Handle<StandardMaterial>,
    grid: Handle<StandardMaterial>,
    center_floor: Handle<StandardMaterial>,
}

fn setup_materials(materials: &mut ResMut<Assets<StandardMaterial>>) -> MaterialHandles {
    MaterialHandles {
        ground: materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.15, 0.15), perceptual_roughness: 0.9, cull_mode: None, ..default()
        }),
        wall: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.25), perceptual_roughness: 0.8, cull_mode: None, ..default()
        }),
        grid: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.35), emissive: Color::srgb(0.3, 0.3, 0.35).into(),
            unlit: true, ..default()
        }),
        center_floor: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0), // Pure bright red
            emissive: Color::srgb(0.5, 0.0, 0.0).into(), // Add glow effect
            perceptual_roughness: 0.3, // More shiny
            metallic: 0.2,
            reflectance: 0.5, // More reflective
            cull_mode: None,
            ..default()
        }),
    }
}



fn setup_arena(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &MaterialHandles) {
    // Main ground collider (covers the entire floor)
    commands.spawn((Collider::cuboid(ARENA_WIDTH/2.0, 0.1, ARENA_DEPTH/2.0), RigidBody::Fixed,
                   Transform::from_translation(Vec3::new(0.0, -0.5, 0.0))));

    // Outer ground (dark gray)
    commands.spawn((Mesh3d(meshes.add(Cuboid::new(ARENA_WIDTH, 0.1, ARENA_DEPTH))),
                   MeshMaterial3d(materials.ground.clone()),
                   Transform::from_translation(Vec3::new(0.0, -0.5, 0.0))));

    // Center floor (red) - positioned slightly above the main floor to be more visible
    commands.spawn((Mesh3d(meshes.add(Cuboid::new(CENTER_SIZE, 0.1, CENTER_SIZE))),
                   MeshMaterial3d(materials.center_floor.clone()),
                   Transform::from_translation(Vec3::new(0.0, -0.44, 0.0))));

    // Grid lines
    let line_thickness = 0.2;
    let line_height = 0.05;

    // Calculate grid range based on arena dimensions
    let half_width = ARENA_WIDTH / 2.0;
    let half_depth = ARENA_DEPTH / 2.0;

    // Create grid lines along X axis (width)
    for x in (-half_width as i32..=half_width as i32).step_by(GRID_SPACING as usize) {
        let x_pos = x as f32;
        commands.spawn((Mesh3d(meshes.add(Cuboid::new(line_thickness, line_height, ARENA_DEPTH))),
                       MeshMaterial3d(materials.grid.clone()),
                       Transform::from_translation(Vec3::new(x_pos, -0.45, 0.0))));
    }

    // Create grid lines along Z axis (depth)
    for z in (-half_depth as i32..=half_depth as i32).step_by(GRID_SPACING as usize) {
        let z_pos = z as f32;
        commands.spawn((Mesh3d(meshes.add(Cuboid::new(ARENA_WIDTH, line_height, line_thickness))),
                       MeshMaterial3d(materials.grid.clone()),
                       Transform::from_translation(Vec3::new(0.0, -0.45, z_pos))));
    }

    // Walls and ceiling
    let wall_configs = [
        [ARENA_WIDTH, ARENA_HEIGHT, WALL_THICKNESS, 0.0, ARENA_HEIGHT/2.0 - 0.5, ARENA_DEPTH/2.0],
        [ARENA_WIDTH, ARENA_HEIGHT, WALL_THICKNESS, 0.0, ARENA_HEIGHT/2.0 - 0.5, -ARENA_DEPTH/2.0],
        [WALL_THICKNESS, ARENA_HEIGHT, ARENA_DEPTH, -ARENA_WIDTH/2.0, ARENA_HEIGHT/2.0 - 0.5, 0.0],
        [WALL_THICKNESS, ARENA_HEIGHT, ARENA_DEPTH, ARENA_WIDTH/2.0, ARENA_HEIGHT/2.0 - 0.5, 0.0],
        [ARENA_WIDTH, WALL_THICKNESS, ARENA_DEPTH, 0.0, ARENA_HEIGHT - 0.5, 0.0]
    ];

    for [width, height, depth, x, y, z] in wall_configs {
        commands.spawn((Collider::cuboid(width/2.0, height/2.0, depth/2.0), RigidBody::Fixed,
                       Transform::from_translation(Vec3::new(x, y, z))));
        commands.spawn((Mesh3d(meshes.add(Cuboid::new(width, height, depth))),
                       MeshMaterial3d(materials.wall.clone()),
                       Transform::from_translation(Vec3::new(x, y, z))));
    }
}



fn respawn(mut query: Query<(&mut Transform, &mut Velocity)>) {
    for (mut transform, mut velocity) in &mut query {
        if transform.translation.y <= -50.0 {
            velocity.linvel = Vec3::ZERO;
            transform.translation = SPAWN_POINT;
        }
    }
}

fn manage_cursor(
    btn: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window>,
    mut controller_query: Query<&mut FpsController>,
) {
    let Ok(mut window) = window_query.get_single_mut() else { return };
    let Ok(mut controller) = controller_query.get_single_mut() else { return };

    if btn.just_pressed(MouseButton::Left) {
        set_cursor_state(&mut window, &mut controller, true);
    } else if key.just_pressed(KeyCode::Escape) {
        set_cursor_state(&mut window, &mut controller, false);
    }
}

fn set_cursor_state(window: &mut Window, controller: &mut FpsController, gameplay_active: bool) {
    window.cursor_options.grab_mode = if gameplay_active { CursorGrabMode::Locked } else { CursorGrabMode::None };
    window.cursor_options.visible = !gameplay_active;
    controller.enable_input = gameplay_active;
}



fn click_targets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rapier_context: ReadRapierContext,
    player_query: Query<Entity, With<LogicalPlayer>>,
    camera: Query<&Transform, With<RenderPlayer>>,
    buttons: Res<ButtonInput<MouseButton>>,
    targets: Query<Entity, With<Target>>,
    mut points: ResMut<Points>,
    mut shoot_stopwatch: Query<&mut ShootTracker>,
    time: Res<Time>,
) {
    // Get player and update shoot tracker
    let player_handle = player_query.single();
    let mut shoot_tracker = shoot_stopwatch.get_mut(player_handle).expect("LogicalPlayer needs ShootTracker");
    shoot_tracker.stopwatch.tick(time.delta());

    // Early returns
    if !buttons.pressed(MouseButton::Left) { return; }
    if shoot_tracker.stopwatch.elapsed_secs() <= 0.1 { return; }

    // Get camera transform once
    let camera_transform = camera.single();

    // Cast ray and handle hit
    let ray_pos = camera_transform.translation;
    let ray_dir = camera_transform.forward().as_vec3();
    let filter = QueryFilter::new().exclude_sensors().exclude_rigid_body(player_handle);

    // Calculate maximum possible distance in the arena (diagonal + some margin)
    let max_distance = (ARENA_WIDTH.powi(2) + ARENA_DEPTH.powi(2) + ARENA_HEIGHT.powi(2)).sqrt() * 1.5;

    // Process hit result with increased ray distance
    process_hit_result(
        rapier_context.single().cast_ray(ray_pos, ray_dir, max_distance, true, filter),
        &mut commands, &mut meshes, &mut materials, &targets, &mut points
    );

    shoot_tracker.stopwatch.reset();
}



fn process_hit_result(
    hit_result: Option<(Entity, f32)>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    targets: &Query<Entity, With<Target>>,
    points: &mut ResMut<Points>,
) {
    if let Some((entity, _)) = hit_result {
        if targets.get(entity).is_ok() {
            commands.entity(entity).despawn_recursive();
            spawn_random_target(commands, meshes, materials);
            points.value += 1;
        } else {
            points.value -= 1;
        }
    } else {
        points.value -= 1;
    }
}

fn spawn_random_target(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Use the helper function to spawn within FOV
    let mut rng = rand::rng();

    // Randomly choose a movement pattern
    let patterns = [
        MovementPattern::Static,
        MovementPattern::Linear,
        MovementPattern::Circular,
        MovementPattern::Random,
    ];
    let pattern = patterns[rng.random_range(0..patterns.len())];
    let max_speed = if pattern == MovementPattern::Static { 0.0 } else { rng.random_range(3.0..10.0) };

    spawn_target_in_fov(commands, meshes, materials, pattern, max_speed);
}

fn update_points_display(points: Res<Points>, mut query: Query<&mut Text, With<PointsDisplay>>) {
    if let Ok(mut text) = query.get_single_mut() {
        text.0 = format!("Points: {}", points.value);
    }
}

fn update_fps_display(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsDisplay>>) {
    if let Ok(mut text) = query.get_single_mut() {
        let fps = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
            .map_or(0, |v| v as i32);
        text.0 = format!("FPS: {}", fps);
    }
}

fn manage_scenarios(mut scenario_state: ResMut<ScenarioState>, time: Res<Time>,
                   mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>,
                   mut materials: ResMut<Assets<StandardMaterial>>,
                   targets: Query<Entity, With<Target>>,
                   keyboard: Res<ButtonInput<KeyCode>>) {
    // Start the test sequence when the user presses Space
    if keyboard.just_pressed(KeyCode::Space) && !scenario_state.has_started {
        scenario_state.has_started = true;
        scenario_state.current_index = 0;
        scenario_state.is_active = false;
        scenario_state.delay_timer.reset();

        // Clear any existing targets
        for entity in &targets {
            commands.entity(entity).despawn_recursive();
        }

        // Start with delay before first scenario
        println!("Starting aim test sequence. First scenario in {} seconds...", SCENARIO_DELAY);
    }

    // If test hasn't started, don't proceed
    if !scenario_state.has_started {
        return;
    }

    // Handle delay between scenarios
    if !scenario_state.is_active {
        scenario_state.delay_timer.tick(time.delta());

        if scenario_state.delay_timer.just_finished() {
            // Start the next scenario
            if scenario_state.current_index < scenario_state.scenarios.len() {
                let scenario_type = scenario_state.scenarios[scenario_state.current_index];
                scenario_state.current_type = Some(scenario_type);
                scenario_state.is_active = true;
                scenario_state.scenario_timer.reset();

                // Spawn targets for this scenario
                spawn_scenario_targets(&mut commands, &mut meshes, &mut materials, scenario_type, &targets);

                println!("Starting scenario: {:?}", scenario_type);
            } else {
                // All scenarios completed
                scenario_state.has_started = false;
                scenario_state.current_type = None;
                println!("All scenarios completed!");
            }
        }
    } else {
        // Handle active scenario
        scenario_state.scenario_timer.tick(time.delta());

        if scenario_state.scenario_timer.just_finished() {
            // End current scenario
            scenario_state.is_active = false;
            scenario_state.current_index += 1;
            scenario_state.delay_timer.reset();

            // Clear targets
            for entity in &targets {
                commands.entity(entity).despawn_recursive();
            }

            if scenario_state.current_index < scenario_state.scenarios.len() {
                println!("Scenario completed. Next scenario in {} seconds...", SCENARIO_DELAY);
            }
        } else {
            // Update targets based on current scenario
            update_scenario_targets(&mut commands, &mut meshes, &mut materials,
                                   scenario_state.current_type.unwrap(), time.delta_secs(), &targets);
        }
    }
}

// Helper function to spawn targets in front of the player within FOV
fn spawn_target_in_fov(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>,
                     materials: &mut ResMut<Assets<StandardMaterial>>,
                     pattern: MovementPattern, max_speed: f32) -> Vec3 {
    let mut rng = rand::rng();

    // Target wall is always in front of player (negative Z)
    let z = -ARENA_DEPTH/2.0 + 5.0; // 5 units from the wall

    // Calculate a position within the player's FOV
    // For a 90-degree FOV, the width at distance |z| is approximately 2*|z|
    let fov_width = 2.0 * z.abs();
    let x = rng.sample(Uniform::new(-fov_width/2.0, fov_width/2.0).unwrap());
    let y = rng.sample(Uniform::new(5.0, ARENA_HEIGHT - 5.0).unwrap()); // Keep targets at reasonable heights

    let pos = Vec3::new(x, y, z);
    spawn_target_with_movement(commands, meshes, materials, pos, pattern, max_speed);
    pos
}

fn spawn_scenario_targets(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>,
                         materials: &mut ResMut<Assets<StandardMaterial>>, scenario_type: ScenarioType,
                         targets: &Query<Entity, With<Target>>) {
    // Clear any existing targets first
    for entity in targets.iter() {
        commands.entity(entity).despawn_recursive();
    }

    match scenario_type {
        ScenarioType::StaticClicking => {
            // Spawn 5 static targets in fixed positions within FOV
            for i in 0..5 {
                let angle = (i as f32) * (TAU / 5.0);
                let z = -ARENA_DEPTH/2.0 + 5.0; // 5 units from the wall
                let radius = z.abs() * 0.8; // 80% of the distance to the wall

                let pos = Vec3::new(
                    radius * angle.cos(),
                    10.0 + (i as f32 * 3.0) % 15.0, // Vary heights
                    z
                );
                spawn_target_with_movement(commands, meshes, materials, pos, MovementPattern::Static, 0.0);
            }
        },
        ScenarioType::DynamicClicking => {
            // Spawn exactly 3 moving targets with random movement
            for _ in 0..3 {
                spawn_target_in_fov(commands, meshes, materials, MovementPattern::Random, 10.0);
            }
        },
        ScenarioType::LinearClicking => {
            // Spawn 3 targets that move in straight lines
            for i in 0..3 {
                let z = -ARENA_DEPTH/2.0 + 5.0;
                let pos = Vec3::new(
                    (i as f32 - 1.0) * 15.0,
                    10.0 + (i as f32 * 5.0),
                    z
                );
                spawn_target_with_movement(commands, meshes, materials, pos, MovementPattern::Linear, 5.0);
            }
        },
        ScenarioType::PreciseTracking => {
            // Spawn a single target that moves slowly and predictably
            let z = -ARENA_DEPTH/2.0 + 10.0;
            spawn_target_with_movement(commands, meshes, materials, Vec3::new(0.0, 15.0, z), MovementPattern::Circular, 3.0);
        },
        ScenarioType::ReactiveTracking => {
            // Spawn a single target that moves with sudden direction changes
            let z = -ARENA_DEPTH/2.0 + 10.0;
            spawn_target_with_movement(commands, meshes, materials, Vec3::new(0.0, 15.0, z), MovementPattern::Reactive, 15.0);
        },
        ScenarioType::ControlTracking => {
            // Spawn a single target that moves in smooth patterns
            let z = -ARENA_DEPTH/2.0 + 10.0;
            spawn_target_with_movement(commands, meshes, materials, Vec3::new(0.0, 15.0, z), MovementPattern::Smooth, 8.0);
        },
        ScenarioType::SpeedSwitching => {
            // Spawn multiple targets that appear and disappear quickly
            let z = -ARENA_DEPTH/2.0 + 5.0;
            for i in 0..3 {
                let pos = Vec3::new(
                    (i as f32 - 1.0) * 20.0,
                    10.0 + (i as f32 * 5.0),
                    z
                );
                spawn_target_with_movement(commands, meshes, materials, pos, MovementPattern::Static, 0.0);
            }
        },
        ScenarioType::EvasiveSwitching => {
            // Spawn targets that move away from the crosshair
            let z = -ARENA_DEPTH/2.0 + 10.0;
            for i in 0..3 {
                let pos = Vec3::new(
                    (i as f32 - 1.0) * 20.0,
                    10.0 + (i as f32 * 5.0),
                    z
                );
                spawn_target_with_movement(commands, meshes, materials, pos, MovementPattern::Evasive, 12.0);
            }
        },
        ScenarioType::StabilitySwitching => {
            // Spawn targets that require precision between switches
            let z = -ARENA_DEPTH/2.0 + 5.0;
            for i in 0..3 {
                let pos = Vec3::new(
                    (i as f32 - 1.0) * 20.0,
                    10.0 + (i as f32 * 5.0),
                    z
                );
                spawn_target_with_movement(commands, meshes, materials, pos, MovementPattern::Static, 0.0);
            }
        },
    }
}

fn update_scenario_targets(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>,
                          materials: &mut ResMut<Assets<StandardMaterial>>,
                          scenario_type: ScenarioType, _delta_time: f32,
                          targets: &Query<Entity, With<Target>>) {
    // This function would update target behavior based on the current scenario
    // For now, we'll just implement basic functionality
    match scenario_type {
        ScenarioType::DynamicClicking => {
            // Only spawn new targets if we have fewer than the maximum
            let target_count = targets.iter().count();
            if target_count < 3 && rand::random::<f32>() < 0.1 {
                // Use the helper function to spawn within FOV
                spawn_target_in_fov(commands, meshes, materials, MovementPattern::Random, 10.0);
            }
        },
        _ => {}
    }
}

fn update_target_movements(time: Res<Time>, mut query: Query<(&mut Transform, &mut TargetMovement)>,
                          camera_query: Query<&Transform, (With<RenderPlayer>, Without<TargetMovement>)>) {
    let delta = time.delta_secs();
    let camera_transform = camera_query.get_single().ok();

    for (mut transform, mut movement) in &mut query {
        movement.timer += delta;

        match movement.pattern {
            MovementPattern::Static => {
                // No movement
            },
            MovementPattern::Linear => {
                // Move in a straight line, bounce off invisible boundaries
                if movement.velocity.length_squared() < 0.001 {
                    // Initialize velocity if not set
                    let mut rng = rand::rng();
                    movement.velocity = Vec3::new(
                        rng.sample(Uniform::new(-1.0, 1.0).unwrap()),
                        0.0,
                        rng.sample(Uniform::new(-1.0, 1.0).unwrap())
                    ).normalize() * movement.max_speed;
                }

                // Update position
                transform.translation += movement.velocity * delta;

                // Keep targets within the front wall area (player's FOV)
                let margin = 5.0;
                let z_min = -ARENA_DEPTH/2.0 + margin;
                let z_max = z_min + 30.0; // Only allow targets to move within 30 units of the front wall

                // Calculate FOV width at the target's z position
                let fov_width = 2.0 * transform.translation.z.abs();
                let bounds_min = Vec3::new(-fov_width/2.0, 5.0, z_min);
                let bounds_max = Vec3::new(fov_width/2.0, ARENA_HEIGHT - 5.0, z_max);

                if transform.translation.x < bounds_min.x || transform.translation.x > bounds_max.x {
                    movement.velocity.x = -movement.velocity.x;
                }
                if transform.translation.y < bounds_min.y || transform.translation.y > bounds_max.y {
                    movement.velocity.y = -movement.velocity.y;
                }
                if transform.translation.z < bounds_min.z || transform.translation.z > bounds_max.z {
                    movement.velocity.z = -movement.velocity.z;
                }

                // Clamp position to boundaries
                transform.translation = transform.translation.clamp(bounds_min, bounds_max);
            },
            MovementPattern::Circular => {
                // Move in a circular pattern, but keep it in front of the player
                let radius = 15.0;
                let speed = movement.max_speed / radius; // Angular velocity
                let angle = movement.timer * speed;

                // Calculate position in a horizontal circle
                let z_base = -ARENA_DEPTH/2.0 + 15.0; // Keep it near the front wall
                transform.translation = Vec3::new(
                    movement.start_position.x + radius * angle.cos(),
                    movement.start_position.y + radius * 0.3 * angle.sin(), // Small vertical movement
                    z_base + radius * 0.2 * (1.0 - angle.cos()) // Small z variation, but stay near the wall
                );

                // Ensure it stays within FOV
                let fov_width = 2.0 * transform.translation.z.abs();
                let bounds_min = Vec3::new(-fov_width/2.0, 5.0, -ARENA_DEPTH/2.0 + 5.0);
                let bounds_max = Vec3::new(fov_width/2.0, ARENA_HEIGHT - 5.0, -ARENA_DEPTH/2.0 + 35.0);
                transform.translation = transform.translation.clamp(bounds_min, bounds_max);
            },
            MovementPattern::Random => {
                // Random movement with occasional direction changes
                if movement.velocity.length_squared() < 0.001 || movement.timer > 2.0 {
                    // Initialize or change velocity
                    let mut rng = rand::rng();
                    movement.velocity = Vec3::new(
                        rng.sample(Uniform::new(-1.0, 1.0).unwrap()),
                        rng.sample(Uniform::new(-0.2, 0.2).unwrap()),
                        rng.sample(Uniform::new(-1.0, 1.0).unwrap())
                    ).normalize() * movement.max_speed;
                    movement.timer = 0.0;
                }

                // Update position
                transform.translation += movement.velocity * delta;

                // Keep targets within the front wall area (player's FOV)
                let margin = 5.0;
                let z_min = -ARENA_DEPTH/2.0 + margin;
                let z_max = z_min + 30.0; // Only allow targets to move within 30 units of the front wall

                // Calculate FOV width at the target's z position
                let fov_width = 2.0 * transform.translation.z.abs();
                let bounds_min = Vec3::new(-fov_width/2.0, 5.0, z_min);
                let bounds_max = Vec3::new(fov_width/2.0, ARENA_HEIGHT - 5.0, z_max);

                if transform.translation.x < bounds_min.x || transform.translation.x > bounds_max.x {
                    movement.velocity.x = -movement.velocity.x;
                }
                if transform.translation.y < bounds_min.y || transform.translation.y > bounds_max.y {
                    movement.velocity.y = -movement.velocity.y;
                }
                if transform.translation.z < bounds_min.z || transform.translation.z > bounds_max.z {
                    movement.velocity.z = -movement.velocity.z;
                }

                // Clamp position to boundaries
                transform.translation = transform.translation.clamp(bounds_min, bounds_max);
            },
            MovementPattern::Reactive => {
                // Sudden, unpredictable movements
                if movement.timer > 0.5 {
                    // Change direction abruptly
                    let mut rng = rand::rng();
                    movement.velocity = Vec3::new(
                        rng.sample(Uniform::new(-1.0, 1.0).unwrap()),
                        rng.sample(Uniform::new(-0.3, 0.3).unwrap()),
                        rng.sample(Uniform::new(-1.0, 1.0).unwrap())
                    ).normalize() * movement.max_speed;
                    movement.timer = 0.0;
                }

                // Update position
                transform.translation += movement.velocity * delta;

                // Keep targets within the front wall area (player's FOV)
                let margin = 5.0;
                let z_min = -ARENA_DEPTH/2.0 + margin;
                let z_max = z_min + 30.0; // Only allow targets to move within 30 units of the front wall

                // Calculate FOV width at the target's z position
                let fov_width = 2.0 * transform.translation.z.abs();
                let bounds_min = Vec3::new(-fov_width/2.0, 5.0, z_min);
                let bounds_max = Vec3::new(fov_width/2.0, ARENA_HEIGHT - 5.0, z_max);

                if transform.translation.x < bounds_min.x || transform.translation.x > bounds_max.x {
                    movement.velocity.x = -movement.velocity.x;
                }
                if transform.translation.y < bounds_min.y || transform.translation.y > bounds_max.y {
                    movement.velocity.y = -movement.velocity.y;
                }
                if transform.translation.z < bounds_min.z || transform.translation.z > bounds_max.z {
                    movement.velocity.z = -movement.velocity.z;
                }

                // Clamp position to boundaries
                transform.translation = transform.translation.clamp(bounds_min, bounds_max);
            },
            MovementPattern::Smooth => {
                // Smooth, predictable movement patterns
                let time = movement.timer;
                let radius_x = 15.0;
                let radius_y = 8.0;

                // Figure-8 pattern in the X-Y plane (horizontal figure-8)
                let z_base = -ARENA_DEPTH/2.0 + 15.0; // Keep it near the front wall
                transform.translation = Vec3::new(
                    movement.start_position.x + radius_x * (2.0 * time).sin(),
                    movement.start_position.y + radius_y * (time).sin() * (time).cos(),
                    z_base // Keep z position fixed near the wall
                );

                // Ensure it stays within FOV
                let fov_width = 2.0 * transform.translation.z.abs();
                let bounds_min = Vec3::new(-fov_width/2.0, 5.0, -ARENA_DEPTH/2.0 + 5.0);
                let bounds_max = Vec3::new(fov_width/2.0, ARENA_HEIGHT - 5.0, -ARENA_DEPTH/2.0 + 35.0);
                transform.translation = transform.translation.clamp(bounds_min, bounds_max);
            },
            MovementPattern::Evasive => {
                // Try to evade the player's aim
                if let Some(camera) = camera_transform {
                    // Calculate direction from target to camera
                    let to_camera = (camera.translation - transform.translation).normalize();

                    // Move perpendicular to this direction
                    let perpendicular = Vec3::new(to_camera.z, 0.0, -to_camera.x).normalize() * movement.max_speed;

                    // Update position
                    transform.translation += perpendicular * delta;

                    // Keep targets within the front wall area (player's FOV)
                    let margin = 5.0;
                    let z_min = -ARENA_DEPTH/2.0 + margin;
                    let z_max = z_min + 30.0; // Only allow targets to move within 30 units of the front wall

                    // Calculate FOV width at the target's z position
                    let fov_width = 2.0 * transform.translation.z.abs();
                    let bounds_min = Vec3::new(-fov_width/2.0, 5.0, z_min);
                    let bounds_max = Vec3::new(fov_width/2.0, ARENA_HEIGHT - 5.0, z_max);

                    // Clamp position to boundaries
                    transform.translation = transform.translation.clamp(bounds_min, bounds_max);
                }
            },
        }
    }
}



fn spawn_target_with_movement(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>,
                            materials: &mut ResMut<Assets<StandardMaterial>>, position: Vec3,
                            pattern: MovementPattern, max_speed: f32) {
    // Create target material
    let target_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.1, 0.1),
        emissive: Color::srgb(1.0, 0.2, 0.2).into(),
        perceptual_roughness: 0.0, metallic: 0.5, reflectance: 1.0,
        ..default()
    });

    // Create movement component if not static
    let components = (
        Collider::ball(TARGET_SIZE),
        RigidBody::Dynamic,
        GravityScale(0.0), // Disable gravity
        Sleeping::disabled(),
        Transform::from_translation(position),
        Target,
        Mesh3d(meshes.add(Sphere::new(TARGET_SIZE))),
        MeshMaterial3d(target_material),
    );

    // Add movement component if not static
    if pattern != MovementPattern::Static {
        let movement = TargetMovement {
            velocity: Vec3::ZERO,
            pattern,
            timer: 0.0,
            start_position: position,
            max_speed,
        };
        commands.spawn(components).insert(movement);
    } else {
        commands.spawn(components);
    }
}

fn update_scenario_display(scenario_state: Res<ScenarioState>, mut query: Query<&mut Text, With<ScenarioDisplay>>) {
    if let Ok(mut text) = query.get_single_mut() {
        if scenario_state.has_started {
            if scenario_state.is_active {
                let scenario_type = scenario_state.current_type.unwrap();
                let remaining = scenario_state.scenario_timer.remaining_secs();
                text.0 = format!("Current: {:?} - {:.1}s", scenario_type, remaining);
            } else {
                let next_index = scenario_state.current_index;
                if next_index < scenario_state.scenarios.len() {
                    let next_scenario = scenario_state.scenarios[next_index];
                    let remaining = scenario_state.delay_timer.remaining_secs();
                    text.0 = format!("Next: {:?} - {:.1}s", next_scenario, remaining);
                } else {
                    text.0 = "All scenarios completed!".to_string();
                }
            }
        } else {
            text.0 = "Press SPACE to start scenarios".to_string();
        }
    }
}


