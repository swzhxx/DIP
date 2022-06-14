use bevy::{
    prelude::Mesh,
    render::mesh::{Indices, MeshVertexAttributeId},
};
use cgmath::Point3;
use half_edge_mesh::HalfEdgeMesh;

pub struct SurfaceHalfEdge<'a> {
    mesh: &'a Mesh,
    vertices: Vec<Point3<f32>>,
    indices: Vec<[usize; 3]>,
    half_edge: HalfEdgeMesh,
}
impl<'a> SurfaceHalfEdge<'a> {
    pub fn new(mesh: &'a Mesh) -> Self {
        let vertices = Self::mesh_point3_vertices(mesh);
        let indices = Self::mesh_half_edge_indices(mesh);
        let half_edge = HalfEdgeMesh::from_face_vertex_mesh(&vertices, &indices);
        Self {
            vertices,
            mesh,
            indices,
            half_edge,
        }
    }

    fn mesh_point3_vertices(mesh: &Mesh) -> Vec<Point3<f32>> {
        // let vertices = mesh.count_vertices();
        // let v = vec![];
        let vertex = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let vertex = match vertex {
            bevy::render::mesh::VertexAttributeValues::Float32x3(vertex) => vertex
                .iter()
                .map(|vertex| Point3::new(vertex[0], vertex[1], vertex[2]))
                .collect::<Vec<Point3<f32>>>(),
            _ => unreachable!(),
        };
        vertex
    }
    fn mesh_half_edge_indices(mesh: &Mesh) -> Vec<[usize; 3]> {
        let indices = mesh.indices().unwrap();
        let result = match indices {
            bevy::render::mesh::Indices::U32(indices) => {
                let mut result = vec![];
                for i in 0..indices.len() {
                    if result.len() % 3 == 0 {
                        let mut item = vec![];
                        result.push(item);
                    }
                    result.last_mut().unwrap().push(i);
                }
                result
                    .iter()
                    .map(|v| return [v[0], v[1], v[2]])
                    .collect::<Vec<[usize; 3]>>()
            }
            _ => {
                unreachable!()
            }
        };
        result
    }
    fn half_edge(&self) -> &HalfEdgeMesh {
        &self.half_edge
    }
}
