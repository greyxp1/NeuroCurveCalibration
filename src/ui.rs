use bevy::{prelude::*, ui::FocusPolicy};

// Component to mark the crosshair
#[derive(Component, Default)]
pub struct CrosshairMarker;

// System to setup the UI including crosshair
pub fn setup_ui(mut commands: Commands) {
    // Root node
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            focus_policy: FocusPolicy::Pass, // Pass focus through this node
            ..default()
        })
        .with_children(|parent| {
            // Simple dot crosshair
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(4.0),
                        height: Val::Px(4.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    background_color: Color::rgba(1.0, 1.0, 1.0, 0.8).into(), // White with slight transparency
                    border_color: Color::rgba(0.0, 0.0, 0.0, 0.9).into(), // Black border
                    ..default()
                },
                CrosshairMarker,
            ));
        });
}

// Plugin to organize UI systems
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui);
    }
}
