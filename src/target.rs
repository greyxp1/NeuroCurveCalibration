use bevy::{prelude::*, pbr::NotShadowCaster};
use std::time::Duration;
use rand::prelude::*;

// --- Core Target Components & Events ---

#[derive(Component, Debug, Clone)]
pub struct Target {
    pub radius: f32,
    pub health: i32,
    pub points: i32,
    pub destroy_on_hit: bool,
    pub time_to_live: Option<Duration>,
    // Using Timer for spawn delay/animation, Option because not all targets might use it
    pub spawn_timer: Option<Timer>,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            radius: 0.5,
            health: 1,
            points: 100,
            destroy_on_hit: true,
            time_to_live: None, // No TTL by default
            spawn_timer: None, // No spawn delay/anim by default
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
    // Box { half_extents: Vec3 }, // Example for future use
    // Capsule { radius: f32, height: f32 }, // Example for future use
}

impl Hitbox {
    // Ray intersection check
    pub fn intersect_ray(&self, ray: bevy::math::Ray3d, transform: &GlobalTransform) -> Option<f32> {
        match self {
            Hitbox::Sphere { radius } => {
                // Ray-sphere intersection formula
                let origin_to_center = transform.translation() - ray.origin;
                let projection = origin_to_center.dot(*ray.direction);
                let distance_sq = origin_to_center.length_squared() - projection * projection;
                let radius_sq = radius * radius;

                if distance_sq > radius_sq {
                    return None; // Ray misses the sphere's projected circle
                }

                let half_chord_distance = (radius_sq - distance_sq).sqrt();
                let intersection1 = projection - half_chord_distance;
                let intersection2 = projection + half_chord_distance;

                if intersection1 > 1e-6 { // Check if intersection point is in front of the ray origin
                    Some(intersection1)
                } else if intersection2 > 1e-6 {
                    Some(intersection2)
                } else {
                    None // Both intersections are behind the ray origin
                }
            },
            // Add cases for Box, Capsule etc. if implemented
        }
    }
}


// --- Movement Components & Logic ---

// Use Bevy's Timer for lifetime tracking
#[derive(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

#[derive(Component, Debug, Clone)]
pub enum TargetMovement {
    Static,
    Linear {
        velocity: Vec3,
        bounds: Option<(Vec3, Vec3)>, // Optional min/max bounds
    },
    // Example: Circular motion
    // Circular { center: Vec3, radius: f32, speed: f32, axis: Vec3 },
}

impl Default for TargetMovement {
    fn default() -> Self {
        Self::Static
    }
}

// System to update target movement
pub fn update_target_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut TargetMovement)>,
) {
    let delta_time = time.delta_seconds();
    for (mut transform, mut movement) in query.iter_mut() {
        match &mut *movement {
            TargetMovement::Static => {} // Do nothing for static targets
            TargetMovement::Linear { velocity, bounds } => {
                transform.translation += *velocity * delta_time;
                // Simple axis-aligned bounding box check and velocity reversal
                if let Some((min_bounds, max_bounds)) = bounds {
                    for i in 0..3 {
                        if (transform.translation[i] < min_bounds[i] && velocity[i] < 0.0) ||
                           (transform.translation[i] > max_bounds[i] && velocity[i] > 0.0) {
                            velocity[i] *= -1.0; // Reverse direction on this axis
                            // Clamp position to prevent going too far out if velocity is high
                            transform.translation[i] = transform.translation[i].clamp(min_bounds[i], max_bounds[i]);
                        }
                    }
                }
            }
            // Add logic for other movement types (e.g., Circular) here
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
    pub target_lifetime_secs: Option<f32>, // Optional lifetime for spawned targets
    pub target_movement: TargetMovement, // Movement pattern for spawned targets
}

// System to update target spawners
pub fn update_target_spawners(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawners: Query<&mut TargetSpawner>,
    targets: Query<&Target>, // Query existing targets to check count
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

            let target_bundle = Target {
                radius: spawner.target_radius,
                time_to_live: spawner.target_lifetime_secs.map(Duration::from_secs_f32),
                ..Default::default()
            };

            // Make hitbox slightly larger than visual representation for easier hits
            let hitbox = Hitbox::Sphere { radius: target_bundle.radius * 1.2 };
            let movement = spawner.target_movement.clone(); // Clone movement pattern

            // Use Bevy's Lifetime component if TTL is set
            let lifetime_component = if let Some(ttl) = target_bundle.time_to_live {
                Some(Lifetime { timer: Timer::new(ttl, TimerMode::Once) })
            } else {
                None
            };

            let mut entity_commands = commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(bevy::math::primitives::Sphere { radius: target_bundle.radius })),
                    material: materials.add(StandardMaterial { base_color: spawner.target_color, emissive: spawner.target_color * 0.1, ..default() }),
                    transform: Transform::from_translation(position),
                    ..default()
                },
                target_bundle, // Spawn the Target component itself
                hitbox,
                movement,
                NotShadowCaster,
            ));

            if let Some(lifetime) = lifetime_component {
                entity_commands.insert(lifetime);
            }

            let entity_id = entity_commands.id();
            target_spawned_events.send(TargetSpawnedEvent { target_entity: entity_id, position });

            // Reset the timer for the next spawn
            spawner.spawn_timer.reset();
            // Optional: Randomize next spawn interval slightly?
            // let next_spawn_time = spawner.spawn_timer.duration().as_secs_f32() * rng.gen_range(0.8..1.2);
            // spawner.spawn_timer.set_duration(Duration::from_secs_f32(next_spawn_time));
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
    // Track shots explicitly for accurate miss counting
    pub shots_fired_this_frame: bool,
    pub hit_registered_this_frame: bool,
}

// System to setup the score tracker resource
pub fn setup_score_tracker(mut commands: Commands) {
    commands.init_resource::<ScoreTracker>();
}

// System to reset frame-specific score tracker flags
pub fn reset_score_tracker_flags(mut score_tracker: ResMut<ScoreTracker>) {
    score_tracker.shots_fired_this_frame = false;
    score_tracker.hit_registered_this_frame = false;
}

// System to update the score tracker
pub fn update_score_tracker(
    mut score_tracker: ResMut<ScoreTracker>,
    mut target_destroyed_events: EventReader<TargetDestroyedEvent>,
    mouse_button_input: Res<ButtonInput<MouseButton>>, // Keep checking mouse input here for misses
) {
    // Process destroyed targets first
    for destroyed_event in target_destroyed_events.read() {
        if destroyed_event.destroyed_by_hit {
            score_tracker.score += destroyed_event.points;
            score_tracker.hit_registered_this_frame = true; // Mark that a hit occurred
        }
    }

    // Check if a shot was fired this frame
    if mouse_button_input.just_pressed(MouseButton::Left) {
        score_tracker.shots_fired_this_frame = true;
    }

     // Update hit/miss counts based on frame flags
    if score_tracker.shots_fired_this_frame {
        if score_tracker.hit_registered_this_frame {
            score_tracker.hits += 1;
        } else {
            score_tracker.misses += 1;
        }
        // Recalculate accuracy
        let total_shots = score_tracker.hits + score_tracker.misses;
        if total_shots > 0 {
            score_tracker.accuracy = score_tracker.hits as f32 / total_shots as f32 * 100.0;
        }
    }
}


// System to display the score (e.g., console output)
pub fn display_score(score_tracker: Res<ScoreTracker>) {
    // Only print when a shot was fired this frame
    if score_tracker.shots_fired_this_frame {
        println!(
            "Score: {}, Accuracy: {:.1}% (Hits: {}, Misses: {})",
            score_tracker.score, score_tracker.accuracy, score_tracker.hits, score_tracker.misses
        );
    }
}

// --- Core Systems (related to Target) ---

// System to handle target lifetime using the Lifetime component
pub fn update_target_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut targets: Query<(Entity, &Target, &mut Lifetime)>, // Query Target to get points on timeout
    mut target_destroyed_events: EventWriter<TargetDestroyedEvent>,
) {
    for (entity, _target_info, mut lifetime) in targets.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.finished() {
            target_destroyed_events.send(TargetDestroyedEvent {
                target_entity: entity,
                points: 0, // No points for timeout
                destroyed_by_hit: false,
            });
            commands.entity(entity).despawn_recursive();
        }
    }
}


// System to handle target hit detection (using raycasting from camera)
pub fn detect_target_hits(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>, // Use Camera and GlobalTransform for raycasting
    target_query: Query<(Entity, &GlobalTransform, &Target, &Hitbox)>,
    mut target_hit_events: EventWriter<TargetHitEvent>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>, // Ensure we query the primary window
) {
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return; // Only check on left click press
    }

    let Ok(window) = windows.get_single() else { return; }; // Handle missing window
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return; }; // Handle missing camera

    // Use center of screen for hit detection to match the crosshair
    let cursor_position = Vec2::new(
        window.width() / 2.0,
        window.height() / 2.0
    );

    // Cast ray from camera through the center of the screen
    if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
        let mut closest_hit: Option<(f32, Entity, Vec3, &Target)> = None;

        // Check intersection with all targets
        for (entity, target_transform, target, hitbox) in target_query.iter() {
            // Simple check: is target still "spawning"? Skip if so.
            if let Some(timer) = &target.spawn_timer {
                 if !timer.finished() { continue; }
             }

            if let Some(hit_distance) = hitbox.intersect_ray(ray, target_transform) {
                let hit_position = ray.origin + ray.direction * hit_distance;
                // Update closest hit if this one is nearer
                if closest_hit.is_none() || hit_distance < closest_hit.unwrap().0 {
                    closest_hit = Some((hit_distance, entity, hit_position, target));
                }
            }
        }

        // Process the closest hit target
        if let Some((_distance, entity, hit_position, _target)) = closest_hit {
            target_hit_events.send(TargetHitEvent {
                target_entity: entity,
                hit_position,
                damage: 1, // Default damage = 1
            });
            // Target destruction/health update is handled in update_target_health
        }
        // Note: Misses are implicitly handled by update_score_tracker checking
        // if a shot was fired but no hit was registered this frame.
    }
}


// System to handle target health and destruction based on hit events
pub fn update_target_health(
    mut commands: Commands,
    mut target_hit_events: EventReader<TargetHitEvent>,
    mut targets: Query<(Entity, &mut Target)>, // Query mutable Target component
    mut target_destroyed_events: EventWriter<TargetDestroyedEvent>,
) {
    for hit_event in target_hit_events.read() {
        if let Ok((entity, mut target)) = targets.get_mut(hit_event.target_entity) {
            if target.destroy_on_hit {
                // Send destroyed event immediately
                 target_destroyed_events.send(TargetDestroyedEvent {
                    target_entity: entity,
                    points: target.points,
                    destroyed_by_hit: true,
                });
                 commands.entity(entity).despawn_recursive();
            } else {
                // Apply damage and check health
                target.health -= hit_event.damage;
                if target.health <= 0 {
                     target_destroyed_events.send(TargetDestroyedEvent {
                        target_entity: entity,
                        points: target.points,
                        destroyed_by_hit: true,
                    });
                    commands.entity(entity).despawn_recursive();
                }
                // Optional: Add feedback for non-destroying hits (e.g., color change)
            }
        }
    }
}

// Hit feedback systems removed


// Setup a basic target spawner entity for testing
pub fn setup_basic_target_spawner(mut commands: Commands) {
    commands.spawn(TargetSpawner {
        spawn_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        max_targets: 5, // Fewer targets for basic test
        spawn_area_min: Vec3::new(-5.0, 0.5, -8.0), // Area in front of player
        spawn_area_max: Vec3::new(5.0, 3.0, -3.0),
        target_radius: 0.4,
        target_color: Color::rgb(1.0, 0.2, 0.2),
        target_lifetime_secs: Some(5.0), // Give targets a lifetime
        target_movement: TargetMovement::Static, // Start with static targets
    });
}

// Target Plugin (Optional, bundles target-related systems)
pub struct TargetPlugin;

impl Plugin for TargetPlugin {
     fn build(&self, app: &mut App) {
         app.add_event::<TargetHitEvent>()
            .add_event::<TargetDestroyedEvent>()
            .add_event::<TargetSpawnedEvent>()
            .init_resource::<ScoreTracker>()
            .add_systems(Startup, (
                setup_score_tracker,
                setup_basic_target_spawner, // Setup a default spawner
            ))
            .add_systems(Update, (
                reset_score_tracker_flags.before(update_score_tracker), // Reset flags before updates
                detect_target_hits,
                update_target_health.after(detect_target_hits), // Process hits before destroying
                update_target_lifetime, // Handle TTL timeouts
                update_target_spawners,
                update_target_movement,
                update_score_tracker.after(update_target_health), // Update score after hits/destroys resolve
                display_score, // Display score (optional)
            ));
    }
}