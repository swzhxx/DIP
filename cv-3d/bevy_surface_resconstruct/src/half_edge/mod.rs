use std::borrow::Borrow;

use bevy::{
    prelude::Mesh,
    render::mesh::{Indices, MeshVertexAttributeId},
};

use bevy_inspector_egui::egui::epaint::Vertex;
use sparse21::Matrix as CooMatrix;

use tri_mesh::{
    prelude::{HalfEdgeID, InnerSpace, Vector3, VertexID, Walker, ID},
    MeshBuilder,
};

pub struct SurfaceHalfEdge {
    half_edge: tri_mesh::prelude::Mesh,
}

unsafe impl Sync for SurfaceHalfEdge {}
unsafe impl Send for SurfaceHalfEdge {}

impl SurfaceHalfEdge {
    pub fn new(mesh: &Mesh) -> Self {
        let vertices = Self::mesh_point3_vertices(mesh);
        let indices = Self::mesh_half_edge_indices(mesh);
        let half_edge_mesh = MeshBuilder::new()
            .with_indices(indices)
            .with_positions(vertices)
            .build()
            .unwrap();
        Self {
            half_edge: half_edge_mesh,
        }
    }

    pub fn minmial_surface(&mut self, vertex_id: VertexID) {
        let position = self.half_edge.vertex_position(vertex_id);
        let lambda = 0.001;
        let hn = self.mean_curvature_flow(vertex_id);
        let p = position - lambda * hn / 2.;
        self.half_edge.move_vertex_to(vertex_id, p)
    }
    fn mean_curvature_flow(&self, vertex_id: VertexID) -> Vector3<f64> {
        // let normal = self.half_edge.vertex_normal(vertex_id);
        let mut curvature = laplace_beltrami(&self.half_edge, vertex_id);
        curvature /= 2. * voronoi_area(&self.half_edge, vertex_id);
        curvature
    }

    pub fn global_minmial_surface(&mut self) {
        let vertices = self.half_edge.no_vertices();

        let mut coo = CooMatrix::new();
        let mut b_x = vec![0.; vertices];
        let mut b_y = vec![0.; vertices];
        let mut b_z = vec![0.; vertices];
        for vertex_id in self.half_edge.vertex_iter() {
            let position = self.half_edge.vertex_position(vertex_id);
            if self.half_edge.is_vertex_on_boundary(vertex_id) {
                coo.add_element(vertex_id.get() as usize, vertex_id.get() as usize, 1.);
                b_x[vertex_id.get() as usize] = position.x;
                b_y[vertex_id.get() as usize] = position.y;
                b_z[vertex_id.get() as usize] = position.z;
            } else {
                let mut total_weight = 0.;
                for half_edge_id in self.half_edge.vertex_halfedge_iter(vertex_id) {
                    let to_vertex_id = self
                        .half_edge
                        .walker_from_halfedge(half_edge_id)
                        .vertex_id()
                        .unwrap();

                    let weight = contan_weight(&self.half_edge, half_edge_id);
                    total_weight += weight;
                    coo.add_element(
                        vertex_id.get() as usize,
                        to_vertex_id.get() as usize,
                        weight,
                    );
                }
                coo.add_element(
                    vertex_id.get() as usize,
                    vertex_id.get() as usize,
                    -total_weight,
                );
            }
        }

        let solve_x = coo.solve(b_x).unwrap();
        let solve_y = coo.solve(b_y).unwrap();
        let solve_z = coo.solve(b_z).unwrap();
        let points = solve_x
            .iter()
            .zip(solve_y.iter().zip(solve_z.iter()))
            .map(|point| Vector3::new(*point.0, *((point.1).0), *((point.1).1)))
            .collect::<Vec<Vector3<f64>>>();
        for (index, vertex_id) in self.half_edge.vertex_iter().enumerate() {
            self.half_edge
                .move_vertex_to(vertex_id, points[index].clone())
        }
    }

    pub fn harmonic_map(&mut self) {
        let mut start_v: Option<VertexID> = None;
        let mesh = &mut self.half_edge;
        let vertices = mesh.no_vertices();
        for vertex in mesh.vertex_iter() {
            if mesh.is_vertex_on_boundary(vertex) {
                start_v = Some(vertex);
                break;
            }
        }
        if start_v.is_none() {
            return;
        }
        let mut boundary_vec: Vec<VertexID> = vec![];
        println!("find boundary start");

        // 寻找 boundary 的vertex
        {
            let mut now: Option<VertexID> = start_v;
            let mut prev: Option<VertexID> = None;

            for edge in mesh.vertex_halfedge_iter(start_v.unwrap()) {
                if mesh.is_edge_on_boundary(edge) {
                    prev = now;
                    let walker = mesh.walker_from_halfedge(edge);
                    now = Some(walker.vertex_id().unwrap());
                    break;
                }
            }
            while now != start_v {
                for edge in mesh.vertex_halfedge_iter(now.unwrap()) {
                    let walker = mesh.walker_from_halfedge(edge);
                    if mesh.is_edge_on_boundary(edge) && prev != walker.vertex_id() {
                        prev = now;
                        boundary_vec.push(prev.unwrap());
                        now = Some(walker.vertex_id().unwrap());
                        break;
                    }
                }
            }
        }
        println!("find boundary end {}", boundary_vec.len());
        let mut b_u = vec![0.; vertices];
        let mut b_v = vec![0.; vertices];

        // square init
        {
            let length = 1.;
            let delta_size = boundary_vec.len() / 4;
            let landmark_idx1 = delta_size;
            let landmark_idx2 = 2 * delta_size;
            let landmark_idx3 = 3 * delta_size;

            let mut delta = length / landmark_idx1 as f64;
            println!("delta {:?}", delta);
            for i in 0..landmark_idx1 {
                b_u[boundary_vec[i].get() as usize] = 0.;
                b_v[boundary_vec[i].get() as usize] = i as f64 * delta;
            }
            b_u[boundary_vec[landmark_idx1].get() as usize] = 0.;
            b_v[boundary_vec[landmark_idx1].get() as usize] = length;

            delta = length / (landmark_idx2 - landmark_idx1) as f64;
            for (i, j) in (landmark_idx1 + 1..landmark_idx2).enumerate() {
                // b_u[boundary_vec[i].get()] = i
                b_u[boundary_vec[j].get() as usize] = i as f64 * delta;
                b_v[boundary_vec[j].get() as usize] = length;
            }
            b_u[boundary_vec[landmark_idx2].get() as usize] = length;
            b_v[boundary_vec[landmark_idx2].get() as usize] = length;

            delta = length / (landmark_idx3 - landmark_idx2) as f64;
            for (i, j) in (landmark_idx2 + 1..landmark_idx3).enumerate() {
                b_u[boundary_vec[j].get() as usize] = length;
                b_v[boundary_vec[j].get() as usize] = length - i as f64 * delta;
            }
            b_u[boundary_vec[landmark_idx3].get() as usize] = length;
            b_v[boundary_vec[landmark_idx3].get() as usize] = 0.;

            delta = length / (boundary_vec.len() - landmark_idx3) as f64;
            for (i, j) in (landmark_idx3 + 1..boundary_vec.len()).enumerate() {
                b_u[boundary_vec[j].get() as usize] = length - i as f64 * delta;
                b_v[boundary_vec[j].get() as usize] = 0.;
            }
        }
        println!("{:?}", b_u);
        println!("{:?}", b_v);
        let mut coo = CooMatrix::new();
        for vertex in mesh.vertex_iter() {
            if mesh.is_vertex_on_boundary(vertex) {
                coo.add_element(vertex.get() as usize, vertex.get() as usize, 1.);
            } else {
                let mut total_weight = 0.;
                for edge_id in mesh.vertex_halfedge_iter(vertex) {
                    let weight = contan_weight(mesh, edge_id);
                    let walker = mesh.walker_from_halfedge(edge_id);
                    coo.add_element(
                        vertex.get() as usize,
                        walker.vertex_id().unwrap().get() as usize,
                        weight,
                    );
                    total_weight += weight;
                }
                coo.add_element(vertex.get() as usize, vertex.get() as usize, -total_weight);
            }
        }
        let disk_u = coo.solve(b_u).unwrap();
        let disk_v = coo.solve(b_v).unwrap();
        disk_u
            .iter()
            .zip(disk_v.iter())
            .enumerate()
            .for_each(|item| {
                println!("{:?}", item.1);
                mesh.move_vertex_to(
                    VertexID::new(item.0 as u32),
                    tri_mesh::prelude::Vec3::new(*(item.1 .0) as f64, *(item.1 .1) as f64, 0.),
                )
            });
    }
    fn mesh_point3_vertices(mesh: &Mesh) -> Vec<f64> {
        let vertex = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let vertex = match vertex {
            bevy::render::mesh::VertexAttributeValues::Float32x3(vertex) => {
                vertex.iter().fold(vec![], |mut acc, v| {
                    acc.push(v[0] as f64);
                    acc.push(v[1] as f64);
                    acc.push(v[2] as f64);
                    acc
                })
            }

            _ => unreachable!(),
        };
        vertex
    }
    fn mesh_half_edge_indices(mesh: &Mesh) -> Vec<u32> {
        let indices = mesh.indices().unwrap();
        let result = match indices {
            bevy::render::mesh::Indices::U32(indices) => {
                let mut result: Vec<u32> = vec![];
                for i in 0..indices.len() {
                    result.push(indices[i] as u32);
                }
                result
            }
            _ => {
                unreachable!()
            }
        };
        result
    }
    pub fn half_edge(&self) -> &tri_mesh::prelude::Mesh {
        &self.half_edge
    }
    pub fn half_edge_mut(&mut self) -> &mut tri_mesh::prelude::Mesh {
        &mut self.half_edge
    }
    pub fn cover_position_buffer_to_bevy_mesh(&self, mesh: &mut Mesh) {
        let postion_buffer = self
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
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, postion_buffer.clone());
    }
}

fn laplace_beltrami(mesh: &tri_mesh::prelude::Mesh, vertex_id: VertexID) -> Vector3<f64> {
    // let normal = mesh.vertex_normal(vertex_id);
    let mut laplace = Vector3::new(0., 0., 0.);

    if !mesh.is_vertex_on_boundary(vertex_id) {
        let mut weight = 0.;
        let mut total_weight = 0.;
        let position = mesh.vertex_position(vertex_id);
        for half_edge_id in mesh.vertex_halfedge_iter(vertex_id) {
            let walker = mesh.walker_from_halfedge(half_edge_id);
            weight = contan_weight(mesh, half_edge_id);
            total_weight += weight;
            laplace += weight * (position - mesh.vertex_position(walker.vertex_id().unwrap()));
        }
        laplace
    } else {
        laplace
    }
}

fn contan_weight(mesh: &tri_mesh::prelude::Mesh, edge_id: HalfEdgeID) -> f64 {
    let walker = mesh.walker_from_halfedge(edge_id);
    let vertex_id = walker.vertex_id().unwrap();
    let mut weight = 0.;

    let h0 = edge_id;
    let h1 = walker.clone().as_twin().halfedge_id().unwrap();

    let p0 = mesh.vertex_position(vertex_id);
    let p1 = mesh.vertex_position(walker.clone().as_twin().vertex_id().unwrap());
    if !mesh.is_edge_on_boundary(h0) {
        let p2_id = walker.clone().as_next().vertex_id().unwrap();
        let p2 = mesh.vertex_position(p2_id);
        let d0 = p0 - p2;
        let d1 = p1 - p2;
        let area = d0.cross(d1.clone()).magnitude();
        let cot = d0.dot(d1.clone()) / area;
        weight += cot;
    }

    if !mesh.is_edge_on_boundary(h1) {
        let p2 = mesh.vertex_position(walker.clone().as_twin().as_next().vertex_id().unwrap());
        let d0 = p0 - p2;
        let d1 = p1 - p2;
        let area = d0.cross(d1.clone()).magnitude();
        let cot = d0.dot(d1.clone()) / area;
        weight += cot;
    }
    weight
}

fn voronoi_area(mesh: &tri_mesh::prelude::Mesh, vertex_id: VertexID) -> f64 {
    // let mut walker = mesh.walker_from_vertex(vertex_id);
    let mut area = 0.;
    for nb_edge_id in mesh.vertex_halfedge_iter(vertex_id) {
        let mut walker = mesh.walker_from_halfedge(nb_edge_id);

        let h0 = walker.vertex_id().unwrap();
        let h1 = walker.as_next().vertex_id().unwrap();
        let h2 = walker.as_next().vertex_id().unwrap();

        let p = mesh.vertex_position(h2);
        let q = mesh.vertex_position(h0);
        let r = mesh.vertex_position(h1);

        let pq = p - q;
        let qr = r - q;
        let pr = r - p;

        let tri_area = pq.cross(pr.clone()).magnitude();
        if tri_area <= f64::MIN_POSITIVE {
            continue;
        }
        let dotp = pq.dot(pr.clone());
        let dotq = -1. * qr.dot(pq.clone());
        let dotr = qr.dot(pr.clone());

        // angle at p is obtuse
        if dotp < 0. {
            area += 0.25 * tri_area
        }
        // angle at q or r obtuse
        else if dotq < 0. || dotr < 0. {
            area += 0.125 * tri_area
        } else {
            let cotq = dotq / tri_area;
            let cotr = dotr / tri_area;
            area += 0.125 * (pr.magnitude2() * cotq + pq.magnitude2() * cotr);
        }
    }
    area
}

fn cot_vertor(v1: &Vector3<f64>, v2: &Vector3<f64>) -> f64 {
    let cos_theta = v1.dot(v2.clone()) / (v1.magnitude() * v2.magnitude());
    let theta = cos_theta.acos();
    1. / theta.tan()
}

// pub trait End {
//     type Item;
//     fn end(&self) -> Self::Item;
// }

// impl End for Walker<'_> {
//     type Item = Option<VertexID>;
//     fn end(&self) -> Self::Item {
//         self.clone().into_twin().vertex_id()
//     }
// }

// fn compute_sector_normal(
//     walker: Walker,
//     vertex_id: VertexID,
//     mesh: &tri_mesh::prelude::Mesh,
// ) -> Vector3<f64> {
//     let p = mesh.vertex_position(vertex_id);
//     let p_f_vertex_id = walker.into_next().vertex_id().unwrap();
//     let p_t_vertex_id = walker.into_next().vertex_id().unwrap();

//     todo!()
// }

// fn compute_sector_angle(walker: Walker, vertex_id: VertexID) -> f64 {
//     todo!()
// }

#[cfg(test)]
mod test {

    use tri_mesh::MeshBuilder;

    #[test]
    fn test_half_edge_mesh() {
        let point3 = [
            [40.5, 0.0, -15.5],
            [40.5, -10.0, -13.5],
            [41.5, -16.0, -19.5],
            [37.5, 1.0, -7.5],
            [-37.5, -3.0, -5.5],
            [-41.5, -3.0, -11.5],
            [-37.5, -5.0, -7.5],
            [-33.5, -6.0, -1.5],
            [-37.5, -7.0, -8.5],
            [-14.5, -13.0, -25.5],
            [-16.5, -3.0, -28.5],
            [-14.5, -4.0, -40.5],
            [10.5, 21.0, 21.5],
            [20.5, 3.0, 19.5],
            [17.5, 19.0, 14.5],
            [-20.5, -10.0, 23.5],
            [-29.5, -14.0, 13.5],
            [-12.5, -18.0, 23.5],
            [7.5, 3.0, 27.5],
            [2.5, -14.0, 28.5],
            [11.5, -7.0, 25.5],
            [-6.5, 3.0, 29.5],
            [-6.5, 13.0, 29.5],
            [28.5, -17.0, -1.5],
            [31.5, 1.0, 3.5],
            [26.5, 2.0, 12.5],
            [24.5, -14.0, 7.5],
            [32.5, -25.0, -13.5],
            [16.5, -27.0, -6.5],
            [17.5, -32.0, -13.5],
            [9.5, 28.0, 10.5],
            [22.5, 17.0, 6.5],
            [-3.5, 32.0, 17.5],
            [0.5, 29.0, 21.5],
            [9.5, 24.0, -18.5],
            [33.5, 16.0, -16.5],
            [27.5, 16.0, -5.5],
            [-22.5, 2.0, 23.5],
            [-28.5, -6.0, 16.5],
            [-15.5, -3.0, 27.5],
            [-7.5, -8.0, 31.5],
            [1.5, 19.0, -28.5],
            [-7.5, 20.0, -21.5],
            [1.5, 28.0, -7.5],
            [7.5, -35.0, -18.5],
            [6.5, -29.0, -12.5],
            [-4.5, -26.0, -21.5],
            [-5.5, -30.0, -27.5],
            [15.5, 23.0, -27.5],
            [0.5, 17.0, -37.5],
            [-10.5, 6.0, -32.5],
            [-12.5, -16.0, -30.5],
            [-12.5, -22.0, -36.5],
            [17.5, -21.0, 3.5],
            [-12.5, -27.0, 1.5],
            [-5.5, -24.0, -15.5],
            [4.5, -27.0, -7.5],
            [-3.5, -27.0, 14.5],
            [10.5, -23.0, 7.5],
            [3.5, -22.0, 22.5],
            [12.5, -16.0, 19.5],
            [-9.5, 33.0, 9.5],
            [-13.5, 28.0, -9.5],
            [-14.5, 22.0, -19.5],
            [-25.5, 23.0, -10.5],
            [-20.5, 28.0, -3.5],
            [-25.5, 16.0, -20.5],
            [-24.5, 10.0, -24.5],
            [-33.5, 12.0, -14.5],
            [-11.5, 12.0, -26.5],
            [-22.5, 6.0, -26.5],
            [-35.5, -3.0, -24.5],
            [-33.5, -9.0, -22.5],
            [-37.5, -7.0, -21.5],
            [-34.5, 7.0, -22.5],
            [-38.5, 2.0, -21.5],
            [-39.5, 4.0, -16.5],
            [-36.5, 8.0, -16.5],
            [-39.5, -3.0, -17.5],
            [-40.5, 1.0, -15.5],
            [-34.5, 11.0, -12.5],
            [-38.5, 3.0, -11.5],
            [-29.5, -15.0, -15.5],
            [-23.5, -11.0, -24.5],
            [-12.5, -24.0, -10.5],
            [-24.5, -22.0, -8.5],
            [-39.5, -6.0, -14.5],
            [-37.5, -10.0, -17.5],
            [-32.5, -13.0, -9.5],
            [-31.5, -12.0, -4.5],
            [-39.5, 1.0, -8.5],
            [-35.5, -2.0, -2.5],
            [-32.5, -15.0, 1.5],
            [-25.5, -22.0, 1.5],
            [-12.5, -26.0, 15.5],
            [-15.5, 26.0, 35.5],
            [-17.5, 22.0, 32.5],
            [-10.5, 22.0, 34.5],
            [-33.5, 11.0, 5.5],
            [-32.5, 17.0, 3.5],
            [-35.5, 12.0, -0.5],
            [-31.5, -14.0, 7.5],
            [-28.5, -19.0, 8.5],
            [-33.5, 5.0, -3.5],
            [-33.5, 11.0, -6.5],
            [-31.5, 19.0, -0.5],
            [-18.5, 28.0, 8.5],
            [-30.5, 20.0, 6.5],
            [-31.5, 13.0, 11.5],
            [-30.5, 7.0, 14.5],
            [-30.5, 1.0, 16.5],
            [-34.5, -2.0, 3.5],
            [0.5, 23.0, 26.5],
            [-6.5, 31.0, 30.5],
            [-5.5, 26.0, 33.5],
            [-10.5, 35.0, 17.5],
            [-11.5, 35.0, 24.5],
            [-12.5, 34.0, 31.5],
            [-13.5, 31.0, 36.5],
            [-18.5, 18.0, 28.5],
            [-20.5, 14.0, 22.5],
            [-12.5, 23.0, 21.5],
            [-0.5, -26.0, 29.5],
            [-17.5, -15.0, 29.5],
            [-15.5, -18.0, 34.5],
            [-12.5, -24.0, 38.5],
            [-9.5, -30.0, 38.5],
            [-8.5, -31.0, 31.5],
            [-10.5, -30.0, 23.5],
            [-6.5, -31.0, 24.5],
            [-4.5, -29.0, 34.5],
            [-2.5, -21.0, 35.5],
            [-8.5, -23.0, 40.5],
            [-8.5, -18.0, 38.5],
            [-13.5, 8.0, 27.5],
        ]
        .iter()
        .fold(vec![], |mut acc, item| {
            acc.push(item[0]);
            acc.push(item[1]);
            acc.push(item[2]);
            acc
        });
        let l = point3.len();
        let indices: Vec<u32> = [
            [1, 0, 2],
            [1, 3, 0],
            [5, 4, 6],
            [4, 7, 6],
            [8, 5, 6],
            [6, 7, 8],
            [9, 11, 10],
            [13, 12, 14],
            [16, 15, 17],
            [19, 18, 20],
            [18, 21, 22],
            [24, 23, 25],
            [26, 25, 23],
            [2, 27, 1],
            [27, 3, 1],
            [27, 23, 3],
            [3, 23, 24],
            [28, 23, 29],
            [29, 23, 27],
            [12, 30, 14],
            [13, 14, 25],
            [31, 25, 14],
            [14, 30, 31],
            [32, 30, 33],
            [33, 30, 12],
            [30, 34, 31],
            [26, 13, 25],
            [3, 35, 0],
            [3, 36, 35],
            [24, 36, 3],
            [36, 24, 31],
            [25, 31, 24],
            [37, 15, 38],
            [39, 40, 15],
            [41, 43, 42],
            [44, 28, 29],
            [45, 28, 44],
            [44, 46, 45],
            [46, 44, 47],
            [48, 35, 34],
            [36, 34, 35],
            [31, 34, 36],
            [30, 43, 34],
            [43, 41, 34],
            [49, 48, 41],
            [34, 41, 48],
            [11, 49, 50],
            [11, 51, 52],
            [52, 46, 47],
            [51, 46, 52],
            [53, 26, 23],
            [53, 23, 28],
            [55, 54, 56],
            [55, 56, 46],
            [45, 46, 56],
            [56, 28, 45],
            [56, 53, 28],
            [57, 56, 54],
            [56, 57, 58],
            [53, 56, 58],
            [59, 58, 57],
            [58, 59, 53],
            [53, 59, 60],
            [53, 60, 26],
            [20, 13, 60],
            [60, 13, 26],
            [18, 13, 20],
            [18, 12, 13],
            [32, 43, 30],
            [61, 43, 32],
            [43, 61, 62],
            [63, 43, 62],
            [42, 43, 63],
            [62, 64, 63],
            [62, 61, 65],
            [65, 64, 62],
            [63, 64, 66],
            [67, 63, 66],
            [66, 68, 67],
            [66, 64, 68],
            [69, 67, 70],
            [63, 67, 69],
            [63, 69, 42],
            [69, 41, 42],
            [41, 69, 49],
            [50, 49, 69],
            [10, 69, 70],
            [69, 10, 50],
            [10, 11, 50],
            [67, 71, 70],
            [71, 10, 70],
            [72, 10, 71],
            [72, 71, 73],
            [67, 68, 74],
            [71, 67, 74],
            [75, 71, 74],
            [74, 76, 75],
            [74, 68, 77],
            [76, 74, 77],
            [79, 78, 76],
            [76, 78, 75],
            [78, 71, 75],
            [71, 78, 73],
            [80, 77, 68],
            [80, 81, 77],
            [77, 81, 76],
            [76, 81, 79],
            [11, 9, 51],
            [9, 46, 51],
            [46, 9, 55],
            [82, 83, 72],
            [10, 72, 83],
            [9, 10, 83],
            [83, 82, 9],
            [9, 84, 55],
            [84, 54, 55],
            [85, 9, 82],
            [9, 85, 84],
            [85, 54, 84],
            [86, 8, 87],
            [86, 87, 78],
            [73, 78, 87],
            [73, 87, 72],
            [87, 82, 72],
            [88, 87, 8],
            [87, 88, 82],
            [88, 85, 82],
            [89, 85, 88],
            [4, 5, 90],
            [90, 5, 81],
            [81, 5, 79],
            [79, 5, 78],
            [78, 5, 86],
            [5, 8, 86],
            [91, 7, 4],
            [8, 7, 88],
            [88, 7, 89],
            [89, 7, 92],
            [92, 93, 89],
            [89, 93, 85],
            [93, 54, 85],
            [93, 94, 54],
            [95, 97, 96],
            [99, 98, 100],
            [7, 101, 92],
            [101, 93, 92],
            [102, 93, 101],
            [16, 102, 101],
            [100, 103, 104],
            [104, 103, 80],
            [80, 103, 81],
            [81, 103, 90],
            [90, 103, 4],
            [103, 91, 4],
            [98, 103, 100],
            [105, 68, 64],
            [68, 105, 80],
            [80, 105, 104],
            [104, 105, 100],
            [99, 100, 105],
            [106, 65, 61],
            [106, 107, 65],
            [65, 107, 64],
            [64, 107, 105],
            [105, 107, 99],
            [107, 98, 99],
            [107, 108, 98],
            [108, 109, 98],
            [110, 111, 109],
            [109, 111, 98],
            [98, 111, 103],
            [103, 111, 91],
            [91, 111, 7],
            [111, 101, 7],
            [111, 16, 101],
            [16, 111, 38],
            [38, 111, 110],
            [18, 22, 112],
            [12, 18, 112],
            [33, 113, 32],
            [112, 114, 33],
            [33, 114, 113],
            [12, 112, 33],
            [115, 61, 32],
            [115, 32, 116],
            [116, 32, 113],
            [117, 116, 113],
            [117, 113, 118],
            [114, 118, 113],
            [112, 22, 114],
            [118, 97, 95],
            [97, 118, 114],
            [114, 22, 97],
            [97, 22, 96],
            [96, 22, 119],
            [22, 120, 119],
            [120, 121, 119],
            [119, 121, 96],
            [96, 121, 95],
            [95, 121, 118],
            [118, 121, 117],
            [117, 121, 116],
            [116, 121, 115],
            [115, 121, 61],
            [61, 121, 106],
            [121, 107, 106],
            [120, 107, 121],
            [107, 120, 108],
            [108, 120, 109],
            [37, 109, 120],
            [109, 37, 110],
            [110, 37, 38],
            [19, 60, 59],
            [122, 19, 59],
            [38, 15, 16],
            [94, 16, 17],
            [16, 94, 102],
            [102, 94, 93],
            [57, 54, 94],
            [60, 19, 20],
            [40, 18, 19],
            [17, 15, 123],
            [17, 123, 124],
            [17, 124, 125],
            [17, 125, 126],
            [17, 126, 127],
            [17, 127, 128],
            [17, 128, 94],
            [128, 57, 94],
            [128, 127, 129],
            [129, 57, 128],
            [129, 59, 57],
            [129, 122, 59],
            [122, 129, 130],
            [130, 129, 127],
            [127, 126, 130],
            [126, 131, 130],
            [130, 131, 122],
            [131, 19, 122],
            [132, 126, 125],
            [132, 131, 126],
            [133, 131, 132],
            [133, 19, 131],
            [133, 40, 19],
            [125, 124, 132],
            [132, 124, 133],
            [124, 40, 133],
            [123, 40, 124],
            [15, 40, 123],
            [21, 18, 40],
            [22, 21, 134],
            [134, 120, 22],
            [134, 37, 120],
            [21, 37, 134],
            [21, 39, 37],
            [37, 39, 15],
            [39, 21, 40],
        ]
        .iter()
        .fold(vec![], |mut acc, v| {
            acc.push(v[0] as u32);
            acc.push(v[1] as u32);
            acc.push(v[2] as u32);
            acc
        });
        let mesh = MeshBuilder::new()
            .with_indices(indices)
            .with_positions(point3)
            .build()
            .unwrap();
        let positions = mesh.positions_buffer();
        assert_eq!(l, positions.len());
    }
}
