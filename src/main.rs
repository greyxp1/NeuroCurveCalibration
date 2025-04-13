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
        .insert_resource(AmbientLight { color: Color::WHITE, brightness: 6000.0 })
        .insert_resource(ClearColor(Color::srgb(0.83, 0.96, 0.96)))
        .insert_resource(Points::default())
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin::default(),
                     RapierPhysicsPlugin::<NoUserData>::default(), FpsControllerPlugin))
        .add_systems(Startup, (setup, fps_controller_setup.in_set(FpsControllerSetup)))
        .add_systems(Update, (respawn, manage_cursor, click_targets,
                             update_points_display, update_fps_display))
        .run();
}

fn fps_controller_setup(mut commands: Commands) {
    let height = 3.0;
    let logical_entity = commands
        .spawn((
            Collider::cylinder(height / 2.0, 0.5),
            Friction { coefficient: 0.0, combine_rule: CoefficientCombineRule::Min },
            Restitution { coefficient: 0.0, combine_rule: CoefficientCombineRule::Min },
            ActiveEvents::COLLISION_EVENTS, Velocity::zero(), RigidBody::Dynamic,
            Sleeping::disabled(), LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0), GravityScale(0.0), Ccd { enabled: true },
            Transform::from_translation(SPAWN_POINT), LogicalPlayer,
            FpsControllerInput { pitch: -TAU / 12.0, yaw: TAU * 5.0 / 8.0, ..default() },
            FpsController { air_acceleration: 80.0, ..default() },
        ))
        .insert(CameraConfig { height_offset: -0.5 })
        .insert(ShootTracker { stopwatch: Stopwatch::new(), spray_count: 0 })
        .insert(SpatialListener::new(0.5))
        .id();

    commands.spawn((
        Camera3d::default(), Camera { order: 0, ..default() },
        Projection::Perspective(PerspectiveProjection { fov: TAU / 5.0, ..default() }),
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
    window.single_mut().title = String::from("Minimal FPS Controller Example");

    // Lighting and cameras
    commands.spawn((DirectionalLight { illuminance: light_consts::lux::FULL_DAYLIGHT, shadows_enabled: true, ..default() },
                   Transform::from_xyz(4.0, 14.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y)));
    commands.spawn((Camera2d, Camera { order: 2, ..default() }));

    // Ground and wall
    let ground_material = materials.add(StandardMaterial { base_color: Color::srgb(0.5, 0.5, 0.5), ..Default::default() });

    // Ground
    commands.spawn((Collider::cuboid(20.0, 0.1, 20.0), RigidBody::Fixed,
                   Transform::from_translation(Vec3::new(0.0, -0.5, 0.0))));
    commands.spawn((Mesh3d(meshes.add(Cuboid::new(40.0, 0.1, 40.0))), MeshMaterial3d(ground_material.clone()),
                   Transform::from_translation(Vec3::new(0.0, -0.5, 0.0))));

    // Wall
    commands.spawn((Collider::cuboid(5.0, 2.5, 0.5), RigidBody::Fixed,
                   Transform::from_translation(Vec3::new(0.0, 0.0, 10.0))));
    commands.spawn((Mesh3d(meshes.add(Cuboid::new(10.0, 5.0, 1.0))), MeshMaterial3d(ground_material),
                   Transform::from_translation(Vec3::new(0.0, 0.0, 10.0))));

    // Spawn targets
    for _ in 0..3 {
        spawn_random_target(&mut commands, &mut meshes, &mut materials);
    }

    // UI elements
    commands.spawn((Mesh2d(meshes.add(Circle::new(2.0))), // Crosshair
                   MeshMaterial2d(materials2d.add(Color::srgb(0.5, 0.7, 1.0))),
                   Transform::default()));

    // Points display
    commands.spawn((Text::new("Points: 0"),
                   Node { position_type: PositionType::Absolute, bottom: Val::Px(5.), left: Val::Px(15.), ..default() },
                   PointsDisplay));

    // FPS counter
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
    if let Ok(mut window) = window_query.get_single_mut() {
        if btn.just_pressed(MouseButton::Left) {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            controller_query.iter_mut().for_each(|mut c| c.enable_input = true);
        } else if key.just_pressed(KeyCode::Escape) {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            controller_query.iter_mut().for_each(|mut c| c.enable_input = false);
        }
    }
}

const SPRAY_DIRECTIONS: [Vec3; 12] = [
    Vec3::new(0.0, 0.0, 0.0), Vec3::new(-0.01, 0.025, 0.0), Vec3::new(-0.02, 0.05, 0.0),
    Vec3::new(-0.03, 0.055, 0.0), Vec3::new(-0.032, 0.065, 0.0), Vec3::new(-0.034, 0.075, 0.0),
    Vec3::new(-0.038, 0.08, 0.0), Vec3::new(-0.042, 0.082, 0.0), Vec3::new(-0.046, 0.085, 0.0),
    Vec3::new(-0.042, 0.087, 0.0), Vec3::new(-0.039, 0.090, 0.0), Vec3::new(-0.038, 0.093, 0.0),
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
    let player_handle = player_query.single();
    let mut shoot_tracker = shoot_stopwatch.get_mut(player_handle).expect("LogicalPlayer needs ShootTracker");
    shoot_tracker.stopwatch.tick(time.delta());

    if !buttons.pressed(MouseButton::Left) {
        shoot_tracker.spray_count = 0;
        return;
    }

    if shoot_tracker.stopwatch.elapsed_secs() <= 0.1 {
        return;
    }

    let camera_transform = camera.single();
    let ray_pos = camera_transform.translation;

    // Get spray direction
    let spray = if shoot_tracker.spray_count >= SPRAY_DIRECTIONS.len() {
        let mut rng = rand::rng();
        let range = Uniform::new(-0.065f32, 0.065).unwrap();
        Vec3::new(rng.sample(range), rng.sample(range), 0.0)
    } else {
        SPRAY_DIRECTIONS[shoot_tracker.spray_count]
    };
    shoot_tracker.spray_count += 1;

    // Calculate ray direction and setup query
    let ray_dir = camera_transform.forward().as_vec3() + camera_transform.rotation * spray;
    let filter = QueryFilter::new().exclude_sensors().exclude_rigid_body(player_handle);

    // Cast ray and handle hit
    if let Some((entity, _)) = rapier_context.single().cast_ray(ray_pos, ray_dir, 100.0, true, filter) {
        if let Ok(_) = targets.get(entity) {
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
    let pos = Vec3::new(
        rng.sample(Uniform::new(-4.0f32, 4.0).unwrap()),
        rng.sample(Uniform::new(2.0f32, 5.0).unwrap()),
        rng.sample(Uniform::new(1.0f32, 2.0).unwrap())
    );
    let size = rng.sample(Uniform::new(0.3f32, 0.8).unwrap());

    let target_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        ..Default::default()
    });

    commands.spawn((
        Collider::ball(size),
        RigidBody::Fixed,
        Transform::from_translation(pos),
        Target,
        Mesh3d(meshes.add(Sphere::new(size))),
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
