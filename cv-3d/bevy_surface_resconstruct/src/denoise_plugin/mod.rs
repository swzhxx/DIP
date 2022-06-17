use bevy::pbr::wireframe::{Wireframe, WireframeConfig};
use bevy::prelude::*;
use bevy::{
    pbr::StandardMaterial,
    prelude::{AssetServer, Plugin},
};
// use rust_3d::{EId, HalfEdge};

use crate::half_edge::SurfaceHalfEdge;

pub struct DenoisePlugin;

impl Plugin for DenoisePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Self::load_bunny_obj)
            .add_system(Self::to_half_edge)
            .add_system(Self::denoise);
    }
}

#[derive(Component)]
struct BunnyObj(Handle<Mesh>, Option<f32>);

impl DenoisePlugin {
    fn denoise() {
        // println!("denoise")
    }

    fn to_half_edge(
        mut commands: Commands,
        meshes: ResMut<Assets<Mesh>>,
        query: Query<&mut BunnyObj>,
    ) {
        for bunny_obj_handle in query.iter() {
            if bunny_obj_handle.1.is_some() {
                continue;
            }
            if let Some(bunny_mesh) = meshes.get(&bunny_obj_handle.0) {
                let surface_half_edge = SurfaceHalfEdge::new(bunny_mesh);
                let count = surface_half_edge.half_edge().vertex_count();
                println!("{:?}", count)
            }
        }
    }

    fn load_bunny_obj(
        mut commands: Commands,
        mut wireframe_config: ResMut<WireframeConfig>,
        asset_server: Res<AssetServer>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        wireframe_config.global = false;
        let mesh_handle = asset_server.load("Cat_head.obj");
        commands
            .spawn_bundle(PbrBundle {
                mesh: mesh_handle.clone(),
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
        commands.spawn().insert(BunnyObj(mesh_handle.clone(), None));
    }
}
