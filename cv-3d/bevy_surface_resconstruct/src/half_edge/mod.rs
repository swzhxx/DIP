use std::borrow::Borrow;

use bevy::{
    prelude::Mesh,
    render::mesh::{Indices, MeshVertexAttributeId},
};

use bevy_inspector_egui::egui::epaint::Vertex;
use tri_mesh::{
    prelude::{HalfEdgeID, InnerSpace, Vector3, VertexID, Walker},
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

    pub fn minmial_surface(&self, vertex_id: VertexID) -> Vector3<f64> {
        todo!()
    }
    pub fn mean_curvature(&self, vertex_id: VertexID) -> Vector3<f64> {
        todo!()
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
}

fn laplace_beltrami(mesh: &tri_mesh::prelude::Mesh, vertex_id: VertexID) -> Vector3<f64> {
    // let normal = mesh.vertex_normal(vertex_id);
    let mut laplace = Vector3::new(0., 0., 0.);
    if !mesh.is_vertex_on_boundary(vertex_id) {
        let mut weight = 0.;
        let mut total_weight = 0.;
        let mut walker = mesh.walker_from_vertex(vertex_id);
        let p = mesh.vertex_position(vertex_id);

        loop {
            match walker.halfedge_id() {
                Some(edge_id) => {
                    weight = contan_weight(mesh, edge_id);
                    total_weight += weight;
                    laplace += weight * mesh.vertex_position(walker.vertex_id().unwrap());
                    walker = walker.into_next();
                }
                None => break,
            }
            if walker.vertex_id().unwrap() == vertex_id {
                break;
            }
        }
        laplace -= total_weight * mesh.vertex_position(vertex_id);
        // laplace vornoi area
        laplace /= 2. * vornoi_area(mesh, vertex_id);

        laplace
    } else {
        laplace
    }
}

fn contan_weight(mesh: &tri_mesh::prelude::Mesh, edge_id: HalfEdgeID) -> f64 {
    let weight = 0.;
    let walker = mesh.walker_from_halfedge(edge_id);
    let vertex_id = walker.vertex_id().unwrap();
    let mut weight = 0.;
    let v_position = mesh.vertex_position(vertex_id);

    for nb_edge_id in mesh.vertex_halfedge_iter(vertex_id) {
        let nb_vertex_id = mesh.walker_from_halfedge(nb_edge_id).vertex_id().unwrap();
        let mut pp: VertexID;
        let mut np: VertexID;
        let mut walker = mesh.walker_from_vertex(nb_vertex_id);
        // let adjve = walker.halfedge_id().unwrap();
        loop {
            if walker.end().is_some() && walker.end().unwrap() == vertex_id {
                pp = walker.end().unwrap();
                break;
            } else {
                walker = walker.into_twin().into_next();
            }
        }
        loop {
            let next_end = walker.as_next().end();
            if next_end.is_some() && next_end.unwrap() == vertex_id {
                np = walker.end().unwrap();
                break;
            } else {
                walker = walker.into_next();
            }
        }

        let adj_position = mesh.vertex_position(vertex_id);
        let pp_position = mesh.vertex_position(pp);
        let np_position = mesh.vertex_position(np);
        let cot_alpha = cot_vertor(&(adj_position - &pp_position), &(v_position - &pp_position));
        let cot_beta = cot_vertor(&(adj_position - &np_position), &(v_position - &np_position));
        weight += cot_alpha + cot_beta;
    }
    weight
}

fn vornoi_area(mesh: &tri_mesh::prelude::Mesh, vertex_id: VertexID) -> f64 {
    // let mut walker = mesh.walker_from_vertex(vertex_id);
    let mut area = 0.;
    for nb_edge_id in mesh.vertex_halfedge_iter(vertex_id) {
        let mut walker = mesh.walker_from_halfedge(nb_edge_id);
        let q = walker.vertex_id().unwrap();
        let q_position = mesh.vertex_position(q);
        let r = walker.as_next().vertex_id().unwrap();
        let r_position = mesh.vertex_position(r);
        let p = walker.as_next().vertex_id().unwrap();
        let p_position = mesh.vertex_position(p);

        let pq = p_position - q_position;
        let qr = r_position - q_position;
        let pr = r_position - p_position;

        let tri_area = pq.cross(pr.clone()).magnitude();

        let dotp = pq.dot(pr.clone());
        let dotq = qr.dot(pq.clone());
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
            area += 0.125 * (pr.magnitude() * cotq + pq.magnitude() * cotr);
        }
    }
    area
}

fn cot_vertor(v1: &Vector3<f64>, v2: &Vector3<f64>) -> f64 {
    let cos_theta = v1.dot(v2.clone()) / (v1.magnitude() * v2.magnitude());
    let theta = cos_theta.acos();
    1. / theta.tan()
}

pub trait End {
    type Item;
    fn end(&self) -> Self::Item;
}

impl End for Walker<'_> {
    type Item = Option<VertexID>;
    fn end(&self) -> Self::Item {
        self.clone().as_twin().as_next().vertex_id()
    }
}

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
