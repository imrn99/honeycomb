mod camera;
mod picking;

use bevy::prelude::*;
use bevy_mod_outline::OutlinePlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_mod_picking::selection::SelectionPluginSettings;

use crate::gui::WindowVisible;
use crate::systems::{
    render_dart_enabled, render_darts, render_edge_enabled, render_edges, render_vertex_enabled,
    render_vertices,
};

/// Plugin handling scene setup and updates.
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        // camera
        app.insert_resource(AmbientLight {
            color: Color::NONE,
            brightness: 0.0,
        })
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            camera::update_camera.run_if(|window_visible: Res<WindowVisible>| !window_visible.0),
        );

        // picking
        app.add_plugins(DefaultPickingPlugins.build())
            .add_plugins(OutlinePlugin)
            .insert_resource(SelectionPluginSettings::default())
            .add_systems(Update, picking::update_picking);

        // content rendering
        app.add_systems(Update, render_darts.run_if(render_dart_enabled))
            .add_systems(Update, render_vertices.run_if(render_vertex_enabled))
            .add_systems(Update, render_edges.run_if(render_edge_enabled));
    }
}
/// Scene setup routine.
pub fn setup_scene(mut commands: Commands) {
    let camera_transform = Transform::from_xyz(0.0, 0.0, 5.0);

    commands.spawn((
        camera::PanOrbitCamera {
            radius: camera_transform.translation.length(),
            ..Default::default()
        },
        Camera3dBundle {
            transform: camera_transform.looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));
}
