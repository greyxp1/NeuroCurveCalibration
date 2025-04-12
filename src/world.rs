use bevy::prelude::*;

// System to setup the basic 3D environment (ground, lighting)
pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground Plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 50.0, subdivisions: 1 })), // Explicit subdivisions
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.2, 0.2, 0.25), // Slightly bluish grey
            perceptual_roughness: 0.8, // Less reflective
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    // Directional Light (simulates sun)
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 15000.0, // Adjust brightness as needed
            shadows_enabled: true,
            ..default()
        },
        // Position the light source high and angled
        transform: Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Ambient Light (provides overall illumination)
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.8, 0.8, 1.0), // Slightly cool ambient light
        brightness: 0.15, // Adjust brightness
    });

     // Optional: Add some simple shapes for reference
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    //     transform: Transform::from_xyz(0.0, 0.5, -3.0),
    //     ..default()
    // });
}

// (Removed spawn_test_target as it's handled by the target module/spawner now)