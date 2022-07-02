use bevy::pbr::wireframe::{Wireframe, WireframeConfig};
use bevy::prelude::*;
use bevy::{
    pbr::StandardMaterial,
    prelude::{AssetServer, Plugin},
};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
// use rust_3d::{EId, HalfEdge};

use crate::half_edge::SurfaceHalfEdge;

pub struct DenoisePlugin;

impl Plugin for DenoisePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Self::load_bunny_obj)
            .add_system(Self::to_half_edge)
            .add_system(Self::denoise)
            .add_plugin(InspectorPlugin::<DenoisePluginUI>::new());
    }
}

#[derive(Component)]
struct BunnyObj(Handle<Mesh>, Option<SurfaceHalfEdge>);

#[derive(Inspectable)]
struct DenoisePluginUI {
    global_minmial_surface: bool,
    local_minmial_surface: bool,
    harmonic_map: bool,
}
impl Default for DenoisePluginUI {
    fn default() -> Self {
        Self {
            global_minmial_surface: false,
            local_minmial_surface: false,
            harmonic_map: false,
        }
    }
}

impl DenoisePlugin {
    fn denoise() {
        // println!("denoise")
    }

    fn to_half_edge(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut query: Query<&mut BunnyObj>,
        mut ui: ResMut<DenoisePluginUI>,
    ) {
        for mut bunny_obj_handle in query.iter_mut() {
            if bunny_obj_handle.1.is_some() && ui.local_minmial_surface {
                let mesh = meshes.get_mut(&bunny_obj_handle.0).unwrap();
                let surface = bunny_obj_handle.1.as_mut().unwrap();
                for vertex in surface.half_edge_mut().vertex_iter() {
                    if surface.half_edge().is_vertex_on_boundary(vertex) {
                        continue;
                    }
                    surface.minmial_surface(vertex);
                }
                surface.cover_position_buffer_to_bevy_mesh(mesh);
                return;
            } else if bunny_obj_handle.1.is_some() && ui.is_changed() && ui.global_minmial_surface {
                ui.local_minmial_surface = false;
                let mesh = meshes.get_mut(&bunny_obj_handle.0).unwrap();
                let surface = bunny_obj_handle.1.as_mut().unwrap();
                surface.global_minmial_surface();
                surface.cover_position_buffer_to_bevy_mesh(mesh);
                return;
            } else if ui.harmonic_map && bunny_obj_handle.1.is_some() && ui.is_changed() {
                let mesh = meshes.get_mut(&bunny_obj_handle.0).unwrap();
                let surface = bunny_obj_handle.1.as_mut().unwrap();
                surface.harmonic_map();
                surface.cover_position_buffer_to_bevy_mesh(mesh);
            } else if let Some(bunny_mesh) = meshes.get_mut(&bunny_obj_handle.0) {
                let surface_half_edge = SurfaceHalfEdge::new(bunny_mesh);
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
