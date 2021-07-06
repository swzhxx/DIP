use js_sys::{Uint32Array, Uint8ClampedArray};
use ndarray::{prelude::*, OwnedRepr, ViewRepr};
use num_traits::Pow;

use rs_graph::traits::Indexable;
use rs_graph::vecgraph::Node;
use rs_graph::{
    maxflow::dinic,
    vecgraph::{Edge, VecGraphBuilder},
    Builder, LinkedListGraph, Net, VecGraph,
};
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::ImageData;
const MAXIUM: usize = 100000000;

// use petgraph::{
//   data::Build,
//   graph::{node_index, Graph, Node, NodeIndex},
// };
// #[wasm_bindgen(js_name=graphCuts)]
// pub fn graph_cuts(
//     data: Clamped<Uint8ClampedArray>,
//     width: usize,
//     height: usize,
//     mut front: Uint32Array,
//     mut background: Uint32Array,
// ) -> Result<ImageData, JsValue> {
//     let get_node_index = |y, x| -> usize { y * width + x };
//     let calc_edge_weight = |current: ArrayBase<ViewRepr<&u8>, Dim<[usize; 1]>>,
//                             next: ArrayBase<ViewRepr<&u8>, Dim<[usize; 1]>>|
//      -> f64 {
//         let t: Array1<f64> = (&current - &next).map(|val| (*val as f64).pow(2.));
//         t.sum().sqrt()
//     };

//     let data: Array3<u8> = Array::from(data.to_vec())
//         .into_shape((height, width, 4))
//         .unwrap();

//     // 构建图
//     let mut g: Graph<String, f64, petgraph::Undirected> = Graph::new_undirected();

//     for y in 0..height {
//         for x in 0..width {
//             g.add_node(format!("y:{:?},x:{:?}", y, x).into());
//         }
//     }

//     let sink = g.add_node("sink".to_string());
//     let source = g.add_node("source".to_string());
//     //构建边
//     for y in 0..height {
//         for x in 0..width {
//             let current = data.slice(s![y, x, ..]);
//             let currentIndex = node_index(get_node_index(y, x));
//             if x < width - 1 {
//                 let right_neighbor = data.slice(s![y, x + 1, ..]);
//                 let weight = calc_edge_weight(current, right_neighbor);
//                 let nextIndex = node_index(get_node_index(y, x + 1));
//                 g.add_edge(currentIndex, nextIndex, weight);
//             }
//             if y < height - 1 {
//                 let bottom_neighbor = data.slice(s![y + 1, x, ..]);
//                 let nextIndex = node_index(get_node_index(y + 1, x));
//                 let weight = calc_edge_weight(current, bottom_neighbor);
//                 g.add_edge(currentIndex, nextIndex, weight);
//             }
//         }
//     }

//     let build_background_front_edge =
//         |points: &mut Uint32Array, node, g: &mut Graph<String, f64, petgraph::Undirected>| {
//             let front_vec = points.to_vec();
//             let len = front_vec.len() as usize;
//             let front = Array::from(front_vec)
//                 .into_shape(((len / 2) as usize, 2))
//                 .unwrap();
//             for i in 0..len {
//                 let y = front[[i, 0]];
//                 let x = front[[i, 1]];
//                 let currentIndex = node_index(get_node_index(y as usize, x as usize));
//                 g.add_edge(currentIndex, source, MAXIUM as f64);
//             }
//         };
//     // 构建前景节点
//     {
//         build_background_front_edge(&mut front, source, &mut g);
//     }
//     // 构建背景
//     {
//         build_background_front_edge(&mut background, sink, &mut g);
//     }
//     unsafe {
//         web_sys::console::log_1(&format!("graph build completed").into());
//     }

//     // maxflow min cut
//     todo!()
// }

#[wasm_bindgen(js_name=graphCuts)]
pub fn graph_cuts(
    data: Uint8ClampedArray,
    width: usize,
    height: usize,
    mut front: Uint32Array,
    mut background: Uint32Array,
) -> Result<ImageData, JsValue> {
    let get_node_index = |y, x| -> usize { y * width + x };
    let calc_edge_weight = |current: ArrayBase<ViewRepr<&u8>, Dim<[usize; 1]>>,
                            next: ArrayBase<ViewRepr<&u8>, Dim<[usize; 1]>>|
     -> f64 {
        let t: Array1<f64> = (&current - &next).map(|val| (*val as f64).pow(2.));
        1. / (t.sum() + 1.)
    };

    let data: Array3<u8> = Array::from(data.to_vec())
        .into_shape((height, width, 4))
        .unwrap();

    let mut builder_g = VecGraphBuilder::with_capacities(0, 0);

    let mut upper: Vec<f64> = vec![];

    for y in 0..height {
        for x in 0..width {
            builder_g.add_node();
        }
    }
    let source = builder_g.add_node();
    let sink = builder_g.add_node();

    //edge capacity
    for y in 0..height {
        for x in 0..width {
            let current = data.slice(s![y, x, ..]);
            let currentIndex = get_node_index(y, x);
            if x < width - 1 {
                let right_neighbor = data.slice(s![y, x + 1, ..]);
                let weight = calc_edge_weight(current, right_neighbor);
                let nextIndex = get_node_index(y, x + 1);
                upper.push(weight)
            }
            if y < height - 1 {
                let bottom_neighbor = data.slice(s![y + 1, x, ..]);
                let nextIndex = get_node_index(y + 1, x);
                let weight = calc_edge_weight(current, bottom_neighbor);
                upper.push(weight)
            }
        }
    }

    let build_background_front_edge =
        |points: &mut Uint32Array, node: Node<u32>, upper: &mut Vec<f64>| {
            let front_vec = points.to_vec();
            let len = front_vec.len() as usize;
            let front = Array::from(front_vec)
                .into_shape(((len / 2) as usize, 2))
                .unwrap();
            for i in 0..len {
                let y = front[[i, 0]];
                let x = front[[i, 1]];
                let currentIndex = get_node_index(y as usize, x as usize);
                upper.push(MAXIUM as f64);
            }
        };
    // 构建前景节点
    {
        build_background_front_edge(&mut front, source, &mut upper);
    }
    // 构建背景
    {
        build_background_front_edge(&mut background, sink, &mut upper);
    }
    unsafe {
        web_sys::console::log_1(&format!("graph build completed").into());
    }
    let g = builder_g.into_graph();

    // maxflow mincut 
    let (value, flow, mut mincut) = dinic(&g, source, sink, |e| (upper[e.index()] * 10000.) as u32);
    // g.add_edge(source, sink);
    todo!()
}
