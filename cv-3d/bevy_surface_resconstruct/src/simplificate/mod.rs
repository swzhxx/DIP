use std::{collections::HashMap, ops::Add};

use bevy::render::render_graph::Edge;
use nalgebra::{AbstractRotation, Matrix4, Vector3, Vector4};
use sparse21::Matrix;
use tri_mesh::{
    mesh::vertex_measures,
    prelude::{HalfEdgeID, VertexID, Walker, ID},
};

use crate::half_edge::{self, SurfaceHalfEdge};

pub struct Simpleificate {
    mesh: tri_mesh::prelude::Mesh,
}

impl Simpleificate {
    pub fn surface_simplification(&mut self, ratio: f64) {
        assert!(ratio > 0. && ratio <= 1.);
        let vertices = self.mesh.no_vertices();
        let faces = self.mesh.no_faces();
        let it_num = ((1. - ratio) * vertices as f64) as u32;
        //1. Compute the Q matrices for all the initial vertices
        let mut Q: Vec<Matrix4<f64>> = Self::make_temporary_property(vertices as u32);
        let mut v = Self::make_temporary_property::<Vector4<f64>>(vertices as u32);
        let mut flag = Self::make_temporary_property::<u32>(vertices as u32);
        let mut p = Self::make_temporary_property::<Vector4<f64>>(faces as u32);
        let mut deleted_map: HashMap<u32, bool> = HashMap::new();
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
            flag[vertex_id.get() as usize] = 0;
        }
        // 2. Select all valid pairs (only vertices in an edge are considered)
        // 3. Compute the optimal contraction target
        let mut q = vec![];
        for edge_id in self.mesh.edge_iter() {
            let mut walker = self.mesh.walker_from_halfedge(edge_id);
            let v1 = walker.clone().vertex_id().unwrap();
            let v0 = walker.as_twin().vertex_id().unwrap();

            let mut tq = Q[v0.get() as usize] + Q[v1.get() as usize];
            let b = Vector4::new(0., 0., 0., 1.);
            tq[(3, 0)] = 0.;
            tq[(3, 1)] = 0.;
            tq[(3, 2)] = 0.;
            tq[(3, 3)] = 1.;
            let mut vnew: Vector4<f64> = Default::default();
            if let Some(inverse) = tq.try_inverse() {
                vnew = inverse * &b;
            } else {
                vnew = v[v0.get() as usize] + v[v1.get() as usize] / 2.;
            }
            let np = nalgebra::Vector3::new(vnew[0], vnew[1], vnew[2]);
            let ts = EdgeCollapseStructure {
                hf: edge_id,
                cost: (vnew.transpose() * &tq * &vnew)[0],
                np: np,
                vto_flag: 0,
                vfrom: v1,
                Q_new: tq,
                vto: v1,
            };
            q.push(ts);
        }
        for _i in 0..it_num {
            let s = q.pop().unwrap();
            if deleted_map.get(&(s.hf.get() as u32)).is_some() {
                continue;
            }
            let mut tvh;
            /***     if(mesh.is_collapse_ok(s.hf))
            {
                mesh.collapse(s.hf);
                tvh = s.vto;
                flag[s.vto] ++;
                flag[s.vfrom] ++;
            }

            else if(mesh.is_collapse_ok(mesh.opposite_halfedge_handle(s.hf)))
            {
                mesh.collapse(mesh.opposite_halfedge_handle(s.hf));
                tvh = s.vfrom;
                flag[s.vto] ++;
                flag[s.vfrom] ++;
            }
            else
            {
                continue;
            }
                 */
            if true {
                // why ?
                tvh = s.vto.clone();
                flag[tvh.get() as usize] += 1;
                flag[tvh.get() as usize] += 1;
            } else {
                tvh = s.vfrom.clone();
                flag[tvh.get() as usize] += 1;
                flag[tvh.get() as usize] += 1;
            }

            let new_vertex_id = self.mesh.collapse_edge(s.hf);
            Q[tvh.get() as usize] = s.Q_new;
            v[tvh.get() as usize][0] = s.np[0];
            v[tvh.get() as usize][1] = s.np[1];
            v[tvh.get() as usize][2] = s.np[2];
            v[tvh.get() as usize][3] = 1.;

            let walker = self.mesh.walker_from_vertex(tvh);

            for vertex_id in TriMeshVertexAroundIter::new(walker) {
                let mut walker = self.mesh.walker_from_vertex(vertex_id);
                let edge = walker.as_twin().halfedge_id().unwrap();
                let tt = vertex_id;
                let newQ = s.Q_new + Q[tt.get() as usize];
                let mut tQ = newQ;
                let b = Vector4::new(0., 0., 0., 1.);
                tQ[(3, 0)] = 0.;
                tQ[(3, 1)] = 0.;
                tQ[(3, 2)] = 0.;
                tQ[(3, 3)] = 1.;
                let lu = tQ.full_piv_lu();
                let mut vnew: Vector4<f64> = Default::default();
                if lu.is_invertible() {
                    vnew = tQ.try_inverse().unwrap() * &b;
                } else {
                    vnew = (v[tvh.get() as usize] + v[tt.get() as usize]) / 2.0
                }
                let np = Vector3::new(vnew[0], vnew[1], vnew[2]);
                let ts = EdgeCollapseStructure {
                    hf: edge,
                    vto: tt,
                    cost: (vnew.transpose() * &newQ * &vnew)[0],
                    np: np,
                    vto_flag: flag[tt.get() as usize],
                    vfrom: tvh,
                    Q_new: newQ,
                };
                q.push(ts);
            }
        }
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

struct EdgeCollapseStructure {
    hf: tri_mesh::prelude::HalfEdgeID,
    vto: VertexID,
    cost: f64,
    np: nalgebra::Vector3<f64>,
    vto_flag: u32,
    vfrom: VertexID,
    Q_new: Matrix4<f64>,
}

impl Add for &EdgeCollapseStructure {
    type Output = f64;

    fn add(self, rhs: Self) -> Self::Output {
        self.cost + rhs.cost
    }
}

pub struct TriMeshVertexAroundIter<'a> {
    step: u32,
    walker: tri_mesh::prelude::Walker<'a>,
    start: VertexID,
}
impl<'a> TriMeshVertexAroundIter<'a> {
    fn new(walker: tri_mesh::prelude::Walker<'a>) -> Self {
        Self {
            step: 0,
            start: walker.clone().vertex_id().unwrap(),
            walker: walker,
        }
    }
}
impl<'a> Iterator for TriMeshVertexAroundIter<'a> {
    type Item = VertexID;

    fn next(&mut self) -> Option<Self::Item> {
        let next: Option<VertexID> = self.walker.as_twin().as_next().vertex_id();
        if next.as_ref().unwrap().get() == self.start.get() {
            None
        } else {
            next
        }
    }
}
