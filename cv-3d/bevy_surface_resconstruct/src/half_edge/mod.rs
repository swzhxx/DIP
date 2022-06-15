use std::borrow::Borrow;

use bevy::{
    prelude::Mesh,
    render::mesh::{Indices, MeshVertexAttributeId},
};
use cgmath::Point3;
use half_edge_mesh::{Edge, HalfEdgeMesh, Ptr, Vert};

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
                let mut result: Vec<Vec<usize>> = vec![];
                for i in 0..indices.len() {
                    if i % 3 == 0 {
                        let item = vec![];
                        {
                            match result.last() {
                                Some(item) => {
                                    debug_assert!(item.len() == 3, "face must be 3 vertex");
                                }
                                None => {}
                            }
                        }

                        result.push(item);
                    }
                    result.last_mut().unwrap().push(indices[i] as usize);
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
    pub fn half_edge(&self) -> &HalfEdgeMesh {
        &self.half_edge
    }
}

pub trait IsOnBounday {
    fn is_on_boundary(&self) -> bool;
}

impl IsOnBounday for Vert {
    fn is_on_boundary(&self) -> bool {
        let edge = {
            match self.edge.upgrade() {
                Some(e) => e,
                None => return true,
            }
        };

        let edge_pair = &(*edge).borrow().pair;
        if edge_pair.as_ref().is_none() {
            return true;
        }
        let face = (*edge).borrow().face.upgrade();
        if face.is_none() {
            return true;
        }
        let edge_pair = edge_pair.upgrade();
        if edge_pair.is_none() {
            return true;
        }
        let face2 = (*edge_pair.unwrap()).borrow().face.upgrade();
        if face2.is_none() {
            return true;
        }
        let face = (*face.unwrap()).borrow().id;
        let face2 = (*face2.unwrap()).borrow().id;
        if face == face2 {
            false
        } else {
            true
        }
    }
}
