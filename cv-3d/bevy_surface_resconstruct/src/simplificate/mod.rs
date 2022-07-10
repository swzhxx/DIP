use nalgebra::{Matrix4, Vector4};
use sparse21::Matrix;
use tri_mesh::prelude::ID;

use crate::half_edge::{self, SurfaceHalfEdge};

pub struct Simpleificate {
    mesh: tri_mesh::prelude::Mesh,
}

impl Simpleificate {
    pub fn surface_simplification(&mut self, ratio: f64) {
        assert!(ratio > 0. && ratio <= 1.);
        let vertices = self.mesh.no_vertices();
        let faces = self.mesh.no_faces();
        //1. Compute the Q matrices for all the initial vertices
        let mut Q: Vec<Matrix4<f64>> = Self::make_temporary_property(vertices as u32);
        let v = Self::make_temporary_property::<Vector4<f64>>(vertices as u32);
        let flag = Self::make_temporary_property::<bool>(vertices as u32);
        let mut p = Self::make_temporary_property::<Vector4<f64>>(faces);

        for face_id in self.mesh.face_iter() {
            let walker = self.mesh.walker_from_face(face_id);
            let normal = self.mesh.face_normal(face_id);
            let tp = walker.vertex_id().unwrap();
            let tp = self.mesh.vertex_position(tp);
            let d = -(normal.x * tp.x + normal.y * tp.y + normal.z * tp.z);
            p[face_id.get() as usize][0] = normal.x;
            p[face_id.get() as usize][1] = normal.y;
            p[face_id.get() as usize][2] = normal.z;
            p[face_id.get() as usize][3] = d;
        }
        for vertex_id in self.mesh.vertex_iter() {
            let position = self.mesh.vertex_position(vertex_id);
            let mut mat = Matrix4::default();
            for face_id in self.mesh.face_iter() {
                mat = p[face_id.get() as usize] * &(p[face_id.get() as usize].transpose());
            }
            Q[vertex_id.get() as usize] = mat;
            v[vertex_id.get() as usize][0] = position[0];
            v[vertex_id.get() as usize][1] = position[1];
            v[vertex_id.get() as usize][2] = position[2];
            v[vertex_id.get() as usize][3] = 1.;
            flag[vertex_id.get() as usize] = false;
        }
        // 2. Select all valid pairs (only vertices in an edge are considered)
        // 3. Compute the optimal contraction target
        todo!();
    }

    pub fn make_temporary_property<T>(num: u32) -> Vec<T>
    where
        T: Default,
    {
        let mut res: Vec<T> = vec![];
        for i in 0..num {
            res.push(Default::default())
        }
        res
    }
}
