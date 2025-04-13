use bevy::{prelude::*, render::camera::Exposure, time::Stopwatch, window::CursorGrabMode};
use bevy_diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy_fps_controller::controller::*;
use bevy_rapier3d::prelude::*;
use rand::{distr::Uniform, prelude::*};
use std::f32::consts::TAU;

const SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.625, 0.0);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct FpsControllerSetup;

#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Component, Default)]
pub struct Target;

#[derive(Component)]
struct PointsDisplay;

#[derive(Component)]
struct FpsDisplay;

#[derive(Default, Resource)]
struct Points { pub value: i32 }

#[derive(Component)]
struct ShootTracker { stopwatch: Stopwatch, spray_count: usize }

fn main() {
    App::new()
        .insert_resource(AmbientLight { color: Color::WHITE, brightness: 2000.0 })
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .insert_resource(Points::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(FpsControllerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, fps_controller_setup.in_set(FpsControllerSetup))
        .add_systems(Update, respawn)
        .add_systems(Update, manage_cursor)
        .add_systems(Update, click_targets)
        .add_systems(Update, update_fps_display)
        .add_systems(Update, update_points_display)
        .run();
}

fn fps_controller_setup(mut commands: Commands) {
    // Standard player height is 1.8 meters, let's make it 2.5 meters (taller)
    let height = 10.0; // Much taller player
    let logical_entity = commands
        .spawn((
            Collider::cylinder(height / 2.0, 0.5),
            Friction { coefficient: 0.0, combine_rule: CoefficientCombineRule::Min },
            Restitution { coefficient: 0.0, combine_rule: CoefficientCombineRule::Min },
            ActiveEvents::COLLISION_EVENTS, Velocity::zero(), RigidBody::Dynamic,
            Sleeping::disabled(), LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0), GravityScale(0.0), Ccd { enabled: true }, // Disable gravity
            Transform::from_translation(SPAWN_POINT), LogicalPlayer,
            FpsControllerInput { pitch: -TAU / 12.0, yaw: TAU * 5.0 / 8.0, ..default() },
            FpsController { air_acceleration: 80.0, ..default() },
        ))
        .insert(CameraConfig { height_offset: 4.0 }) // Place camera much higher for taller player
        .insert(ShootTracker { stopwatch: Stopwatch::new(), spray_count: 0 })
        .insert(SpatialListener::new(0.5))
        .id();

    commands.spawn((
        Camera3d::default(), Camera { order: 0, ..default() },
        Projection::Perspective(PerspectiveProjection { fov: TAU / 4.5, ..default() }), // Wider FOV for taller player
        Exposure::SUNLIGHT, RenderPlayer { logical_entity },
    ));
}

fn setup(
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut materials2d: ResMut<Assets<ColorMaterial>>,
) {
    // Set window title
    window.single_mut().title = String::from("Kovaak's-Inspired Aim Trainer");

    // Enhanced lighting setup for massive arena
    // Main directional light - increased height
    commands.spawn((DirectionalLight { illuminance: light_consts::lux::OVERCAST_DAY, shadows_enabled: true, ..default() },
                   Transform::from_xyz(0.0, 40.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z)));

    // Primary center light - massive intensity
    commands.spawn((PointLight { color: Color::srgb(0.9, 0.9, 1.0), intensity: 10000.0, range: 120.0, ..default() },
                   Transform::from_xyz(0.0, 20.0, 0.0)));

    // Corner fill lights - using a loop to reduce code
    let corner_positions = [(-45.0, -45.0), (45.0, -45.0), (-45.0, 45.0), (45.0, 45.0)];
    for (x, z) in corner_positions.iter() {
        commands.spawn((PointLight { color: Color::srgb(0.8, 0.8, 1.0), intensity: 5000.0, range: 80.0, ..default() },
                       Transform::from_xyz(*x, 20.0, *z)));
    }

    // Grid of fill lights for even illumination
    for x in [-30.0, 0.0, 30.0].iter() {
        for z in [-30.0, 0.0, 30.0].iter() {
            // Skip the center (0,0) as it already has the primary light
            if *x == 0.0 && *z == 0.0 { continue; }

            commands.spawn((PointLight { color: Color::srgb(0.8, 0.8, 1.0), intensity: 3000.0, range: 60.0, ..default() },
                           Transform::from_xyz(*x, 15.0, *z)));
        }
    }
    commands.spawn((Camera2d, Camera { order: 2, ..default() }));

    // Arena setup with materials and dimensions - massive training area
    let arena_width = 100.0;
    let arena_depth = 100.0;
    let arena_height = 25.0;
    let wall_thickness = 1.0;

    // Materials
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.15, 0.15), perceptual_roughness: 0.9, cull_mode: None, ..default()
    });

    let wall_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.25), perceptual_roughness: 0.8, cull_mode: None, ..default()
    });

    let grid_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.35), emissive: Color::srgb(0.3, 0.3, 0.35).into(),
        unlit: true, ..default()
    });

    // Ground
    commands.spawn((Collider::cuboid(arena_width/2.0, 0.1, arena_depth/2.0), RigidBody::Fixed,
                   Transform::from_translation(Vec3::new(0.0, -0.5, 0.0))));
    commands.spawn((Mesh3d(meshes.add(Cuboid::new(arena_width, 0.1, arena_depth))),
                   MeshMaterial3d(ground_material.clone()),
                   Transform::from_translation(Vec3::new(0.0, -0.5, 0.0))));

    // Grid lines - thicker for massive arena
    let line_thickness = 0.2;
    let line_height = 0.05;

    // Create grid lines using loops - larger spacing for massive room
    for i in (-48..=48).step_by(4) {
        let x = i as f32;
        // X axis lines
        commands.spawn((Mesh3d(meshes.add(Cuboid::new(line_thickness, line_height, arena_depth))),
                       MeshMaterial3d(grid_material.clone()),
                       Transform::from_translation(Vec3::new(x, -0.45, 0.0))));

        // Z axis lines
        commands.spawn((Mesh3d(meshes.add(Cuboid::new(arena_width, line_height, line_thickness))),
                       MeshMaterial3d(grid_material.clone()),
                       Transform::from_translation(Vec3::new(0.0, -0.45, x))));
    }

    // Walls and ceiling - using a loop to reduce code
    let wall_configs = [
        // [width, height, depth, x, y, z] for each wall
        [arena_width, arena_height, wall_thickness, 0.0, arena_height/2.0 - 0.5, arena_depth/2.0],   // Back
        [arena_width, arena_height, wall_thickness, 0.0, arena_height/2.0 - 0.5, -arena_depth/2.0],  // Front
        [wall_thickness, arena_height, arena_depth, -arena_width/2.0, arena_height/2.0 - 0.5, 0.0],  // Left
        [wall_thickness, arena_height, arena_depth, arena_width/2.0, arena_height/2.0 - 0.5, 0.0],   // Right
        [arena_width, wall_thickness, arena_depth, 0.0, arena_height - 0.5, 0.0]                     // Ceiling
    ];

    for [width, height, depth, x, y, z] in wall_configs {
        // Physics collider
        commands.spawn((Collider::cuboid(width/2.0, height/2.0, depth/2.0), RigidBody::Fixed,
                       Transform::from_translation(Vec3::new(x, y, z))));
        // Visual mesh
        commands.spawn((Mesh3d(meshes.add(Cuboid::new(width, height, depth))),
                       MeshMaterial3d(wall_material.clone()),
                       Transform::from_translation(Vec3::new(x, y, z))));
    }

    // Spawn more targets for the massive arena
    for _ in 0..10 {
        spawn_random_target(&mut commands, &mut meshes, &mut materials);
    }

    // UI elements - crosshair and displays
    let crosshair_color = Color::srgb(0.0, 1.0, 1.0);
    let crosshair_material = materials2d.add(crosshair_color);

    // Crosshair (horizontal and vertical lines)
    for (width, height) in [(10.0, 2.0), (2.0, 10.0)] {
        commands.spawn((
            Mesh2d(meshes.add(Cuboid::new(width, height, 0.0))),
            MeshMaterial2d(crosshair_material.clone()),
            Transform::default(),
        ));
    }

    // Text displays
    commands.spawn((Text::new("Points: 0"),
                   Node { position_type: PositionType::Absolute, bottom: Val::Px(5.), left: Val::Px(15.), ..default() },
                   PointsDisplay));

    commands.spawn((Text::new("FPS: 0"),
                   Node { position_type: PositionType::Absolute, top: Val::Px(5.), right: Val::Px(15.), ..default() },
                   FpsDisplay));
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

    if btn.just_pressed(MouseButton::Left) {
        // Lock cursor for gameplay
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
        controller_query.iter_mut().for_each(|mut c| c.enable_input = true);
    } else if key.just_pressed(KeyCode::Escape) {
        // Release cursor when ESC pressed
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        controller_query.iter_mut().for_each(|mut c| c.enable_input = false);
    }
}

// Spray pattern for weapon recoil
const SPRAY_DIRECTIONS: [Vec3; 12] = [
    Vec3::ZERO, Vec3::new(-0.01, 0.025, 0.0), Vec3::new(-0.02, 0.05, 0.0), Vec3::new(-0.03, 0.055, 0.0),
    Vec3::new(-0.032, 0.065, 0.0), Vec3::new(-0.034, 0.075, 0.0), Vec3::new(-0.038, 0.08, 0.0),
    Vec3::new(-0.042, 0.082, 0.0), Vec3::new(-0.046, 0.085, 0.0), Vec3::new(-0.042, 0.087, 0.0),
    Vec3::new(-0.039, 0.090, 0.0), Vec3::new(-0.038, 0.093, 0.0),
];

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

    // Early returns for not shooting or cooldown
    if !buttons.pressed(MouseButton::Left) { shoot_tracker.spray_count = 0; return; }
    if shoot_tracker.stopwatch.elapsed_secs() <= 0.1 { return; }

    // Get spray direction
    let spray = if shoot_tracker.spray_count >= SPRAY_DIRECTIONS.len() {
        let mut rng = rand::rng();
        Vec3::new(rng.sample(Uniform::new(-0.065f32, 0.065).unwrap()),
                 rng.sample(Uniform::new(-0.065f32, 0.065).unwrap()), 0.0)
    } else {
        SPRAY_DIRECTIONS[shoot_tracker.spray_count]
    };
    shoot_tracker.spray_count += 1;

    // Cast ray
    let camera_transform = camera.single();
    let ray_pos = camera_transform.translation;
    let ray_dir = camera_transform.forward().as_vec3() + camera_transform.rotation * spray;
    let filter = QueryFilter::new().exclude_sensors().exclude_rigid_body(player_handle);

    // Handle hit
    if let Some((entity, _)) = rapier_context.single().cast_ray(ray_pos, ray_dir, 100.0, true, filter) {
        if targets.get(entity).is_ok() {
            commands.entity(entity).despawn_recursive();
            spawn_random_target(&mut commands, &mut meshes, &mut materials);
            points.value += 1;
        } else {
            points.value -= 1;
        }
    } else {
        points.value -= 1;
    }

    shoot_tracker.stopwatch.reset();
}

fn spawn_random_target(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();

    // Arena dimensions and target properties - for massive arena
    let arena_width = 95.0; // Slightly smaller than actual arena to keep targets away from walls
    let arena_depth = 95.0;
    let arena_height = 23.0;
    let wall_bias = 0.85; // Higher bias to place targets closer to walls
    let size = 1.5; // Larger targets for better visibility in the massive room

    // Get random height position (same for all walls)
    let y = rng.sample(Uniform::new(1.0, arena_height - 2.0).unwrap());

    // Choose wall and position target
    let wall_choice = rng.random_range(0..5);
    let pos = match wall_choice {
        0 => Vec3::new(-arena_width/2.0 * wall_bias, y, rng.sample(Uniform::new(-arena_depth/2.0 + 2.0, arena_depth/2.0 - 2.0).unwrap())), // Left
        1 => Vec3::new(arena_width/2.0 * wall_bias, y, rng.sample(Uniform::new(-arena_depth/2.0 + 2.0, arena_depth/2.0 - 2.0).unwrap())),  // Right
        2 => Vec3::new(rng.sample(Uniform::new(-arena_width/2.0 + 2.0, arena_width/2.0 - 2.0).unwrap()), y, arena_depth/2.0 * wall_bias),   // Back
        3 => Vec3::new(rng.sample(Uniform::new(-arena_width/2.0 + 2.0, arena_width/2.0 - 2.0).unwrap()), y, -arena_depth/2.0 * wall_bias),  // Front
        _ => Vec3::new(rng.sample(Uniform::new(-arena_width/2.0 + 2.0, arena_width/2.0 - 2.0).unwrap()), y,
                      rng.sample(Uniform::new(-arena_depth/2.0 + 2.0, arena_depth/2.0 - 2.0).unwrap())), // Random
    };

    // Create enhanced target material for better visibility in massive arena
    let target_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.1, 0.1),
        emissive: Color::srgb(1.0, 0.2, 0.2).into(), // Brighter glow
        perceptual_roughness: 0.0, // Perfectly smooth
        metallic: 0.5, // More metallic for better highlights
        reflectance: 1.0, // Maximum reflectance
        unlit: false, // Allow lighting to affect it
        ..default()
    });

    // Spawn target entity
    commands.spawn((
        Collider::ball(size), RigidBody::Fixed, Transform::from_translation(pos), Target,
        Mesh3d(meshes.add(Sphere::new(size))), MeshMaterial3d(target_material),
    ));
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
