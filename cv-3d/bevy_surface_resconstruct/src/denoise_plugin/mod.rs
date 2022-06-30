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
struct BunnyObj(Handle<Mesh>, Option<SurfaceHalfEdge>);

impl DenoisePlugin {
    fn denoise() {
        // println!("denoise")
    }

    fn to_half_edge(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut query: Query<&mut BunnyObj>,
    ) {
        for mut bunny_obj_handle in query.iter_mut() {
            if bunny_obj_handle.1.is_some() {
                // let surface = bunny_obj_handle.1.as_mut().unwrap();
                // for vertex in surface.half_edge_mut().vertex_iter() {
                //     if surface.half_edge().is_vertex_on_boundary(vertex) {
                //         continue;
                //     }
                //     surface.minmial_surface(vertex);
                // }
                // let postion_buffer = surface
                //     .half_edge()
                //     .positions_buffer()
                //     .iter()
                //     .enumerate()
                //     .fold(vec![], |mut acc, (usize, value)| {
                //         if usize % 3 == 0 {
                //             acc.push(vec![])
                //         }
                //         acc.last_mut().unwrap().push(*value as f32);
                //         acc
                //     })
                //     .iter()
                //     .map(|item| [item[0], item[1], item[2]])
                //     .collect::<Vec<[f32; 3]>>();
                // meshes
                //     .get_mut(&bunny_obj_handle.0)
                //     .unwrap()
                //     .insert_attribute(Mesh::ATTRIBUTE_POSITION, postion_buffer.clone());
                return;
            }
            if let Some(bunny_mesh) = meshes.get_mut(&bunny_obj_handle.0) {
                let mut surface_half_edge = SurfaceHalfEdge::new(bunny_mesh);
                surface_half_edge.global_minial_surface();
                let postion_buffer = surface_half_edge
                    .half_edge()
                    .positions_buffer()
                    .iter()
                    .enumerate()
                    .fold(vec![], |mut acc, (usize, value)| {
                        if usize % 3 == 0 {
                            acc.push(vec![])
                        }
                        acc.last_mut().unwrap().push(*value as f32);
                        acc
                    })
                    .iter()
                    .map(|item| [item[0], item[1], item[2]])
                    .collect::<Vec<[f32; 3]>>();
                bunny_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, postion_buffer.clone());
                bunny_obj_handle.1 = Some(surface_half_edge)
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
        let mesh_handle = asset_server.load("Nefertiti_face.obj");
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
