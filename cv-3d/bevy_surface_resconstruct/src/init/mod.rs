use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use bevy_inspector_egui::{
    Inspectable, InspectorPlugin, RegisterInspectable, WorldInspectorPlugin,
};
use bevy_obj::*;

use crate::orbit_controls::{OrbitCamera, OrbitCameraPlugin};
pub struct SurfaceDefaultPlugin;

impl SurfaceDefaultPlugin {
    fn add_camera(mut commands: Commands) {
        commands.spawn_bundle(PointLightBundle {
            transform: Transform::from_translation(Vec3::new(3.0, 4.0, 3.0)),
            ..Default::default()
        });

        commands
            .spawn_bundle(PerspectiveCameraBundle {
                transform: Transform::from_translation(Vec3::new(1.5, 2.7, 4.0))
                    .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
                ..Default::default()
            })
            .insert(OrbitCamera::default());
    }
}

impl Plugin for SurfaceDefaultPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins)
            .add_plugin(ObjPlugin)
            .add_plugin(OrbitCameraPlugin)
            .add_plugin(WireframePlugin)
            .add_startup_system(Self::add_camera);
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
