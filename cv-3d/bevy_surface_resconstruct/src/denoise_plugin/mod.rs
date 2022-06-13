use bevy::pbr::wireframe::{Wireframe, WireframeConfig};
use bevy::prelude::*;
use bevy::{
    pbr::StandardMaterial,
    prelude::{AssetServer, Plugin},
};
pub struct DenoisePlugin;

impl Plugin for DenoisePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Self::load_bunny_obj)
            .add_system(Self::denoise);
    }
}

impl DenoisePlugin {
    fn denoise() {
        println!("denoise")
    }

    fn load_bunny_obj(
        mut commands: Commands,
        mut wireframe_config: ResMut<WireframeConfig>,
        asset_server: Res<AssetServer>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        wireframe_config.global = false;
        commands
            .spawn_bundle(PbrBundle {
                mesh: asset_server.load("stanford-bunny.obj"),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgba(1., 1., 1., 0.1),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .insert(Wireframe);
        commands.spawn_bundle(PointLightBundle {
            transform: Transform::from_translation(Vec3::new(3.0, 4.0, 3.0)),
            ..Default::default()
        });
    }
}
