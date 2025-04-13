use bevy::{prelude::*, render::camera::Exposure, time::Stopwatch, window::CursorGrabMode};
use bevy_diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy_fps_controller::controller::*;
use bevy_rapier3d::prelude::*;
use rand::{distr::Uniform, prelude::*};
use std::f32::consts::TAU;

// Game constants
const SENSITIVITY_CM_PER_360: f32 = 10.0;
const MOUSE_DPI: f32 = 1600.0;
const SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.625, 0.0);
const ARENA_WIDTH: f32 = 200.0;
const ARENA_DEPTH: f32 = 200.0;
const ARENA_HEIGHT: f32 = 50.0;
const WALL_THICKNESS: f32 = 1.0;
const TARGET_SIZE: f32 = 1.5;
const PLAYER_HEIGHT: f32 = 10.0;
const PLAYER_RADIUS: f32 = 0.5;
const CAMERA_HEIGHT_OFFSET: f32 = 4.0;
const CAMERA_FOV: f32 = TAU / 4.5;
const CENTER_SIZE: f32 = 8.0;
const GRID_SPACING: f32 = 4.0;

// Component and resource definitions
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
struct ShootTracker { stopwatch: Stopwatch }

fn main() {
    App::new()
        .insert_resource(AmbientLight { color: Color::WHITE, brightness: 2000.0 })
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .insert_resource(Points::default())
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
        .add_systems(Update, (respawn, manage_cursor, click_targets,
                             update_fps_display, update_points_display))
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
            FpsControllerInput { pitch: -TAU / 12.0, yaw: TAU * 5.0 / 8.0, ..default() },
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

    // UI elements - crosshair and text displays
    let crosshair_material = materials2d.add(Color::srgb(0.0, 1.0, 1.0));
    for (width, height) in [(10.0, 2.0), (2.0, 10.0)] {
        commands.spawn((Mesh2d(meshes.add(Cuboid::new(width, height, 0.0))),
                       MeshMaterial2d(crosshair_material.clone()),
                       Transform::default()));
    }

    // Text displays
    commands.spawn((Text::new("Points: 0"),
                   Node { position_type: PositionType::Absolute, bottom: Val::Px(5.), left: Val::Px(15.), ..default() },
                   PointsDisplay));
    commands.spawn((Text::new("FPS: 0"),
                   Node { position_type: PositionType::Absolute, top: Val::Px(5.), right: Val::Px(15.), ..default() },
                   FpsDisplay));
    commands.spawn((Text::new(format!("Sensitivity: {:.1} cm/360 @ {} DPI", SENSITIVITY_CM_PER_360, MOUSE_DPI as i32)),
                   Node { position_type: PositionType::Absolute, bottom: Val::Px(5.), right: Val::Px(15.), ..default() }));
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
    let mut rng = rand::rng();
    let y = rng.sample(Uniform::new(1.0, ARENA_HEIGHT - 2.0).unwrap());
    let margin = 5.0; // Margin from walls

    // Calculate random position within arena bounds
    let pos = Vec3::new(
        rng.sample(Uniform::new(-ARENA_WIDTH/2.0 + margin, ARENA_WIDTH/2.0 - margin).unwrap()),
        y,
        rng.sample(Uniform::new(-ARENA_DEPTH/2.0 + margin, ARENA_DEPTH/2.0 - margin).unwrap())
    );

    // Create target material
    let target_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.1, 0.1),
        emissive: Color::srgb(1.0, 0.2, 0.2).into(),
        perceptual_roughness: 0.0, metallic: 0.5, reflectance: 1.0,
        ..default()
    });

    // Spawn target entity
    commands.spawn((
        Collider::ball(TARGET_SIZE),
        RigidBody::Fixed,
        Transform::from_translation(pos),
        Target,
        Mesh3d(meshes.add(Sphere::new(TARGET_SIZE))),
        MeshMaterial3d(target_material),
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


