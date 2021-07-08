use js_sys::{Uint32Array, Uint8ClampedArray};
use ndarray::{prelude::*, OwnedRepr, ViewRepr};
use num_traits::Pow;

use rs_graph::traits::{Directed, Indexable};
use rs_graph::IndexGraph;

use rs_graph::{
    maxflow::dinic,
    traits::GraphSize,
    vecgraph::{Edge, Node, VecGraphBuilder},
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
    unsafe {
        web_sys::console::log_1(&format!("start0").into());
    }
    let get_node_index = |y, x| -> usize { y * width + x };
    let calc_edge_weight = |current: ArrayBase<ViewRepr<&u8>, Dim<[usize; 1]>>,
                            next: ArrayBase<ViewRepr<&u8>, Dim<[usize; 1]>>|
     -> f64 {
        let t: Array1<f64> = (&current - &next).map(|val| (*val as f64).pow(2.));
        let weight = 1. / (t.sum() + 0.00001);
        // unsafe {
        //     web_sys::console::log_1(&format!("weight {:?}", weight).into());
        // }
        weight
    };

    let data: Array3<u8> = Array::from_shape_vec((height, width, 4), data.to_vec()).unwrap();
    unsafe {
        web_sys::console::log_1(&format!("start").into());
    }
    let mut builder_g = VecGraphBuilder::with_capacities(0, 0);

    let mut upper: Vec<f64> = vec![];
    let mut nodes: Vec<Node> = vec![];
    for y in 0..height {
        for x in 0..width {
            let node = builder_g.add_node();
            nodes.push(node)
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
                upper.push(weight);
                builder_g.add_edge(nodes[currentIndex], nodes[nextIndex]);
                upper.push(weight);
                builder_g.add_edge(nodes[nextIndex], nodes[currentIndex]);
            }
            if y < height - 1 {
                let bottom_neighbor = data.slice(s![y + 1, x, ..]);
                let nextIndex = get_node_index(y + 1, x);
                let weight = calc_edge_weight(current, bottom_neighbor);
                upper.push(weight);
                builder_g.add_edge(nodes[currentIndex], nodes[nextIndex]);
                upper.push(weight);
                builder_g.add_edge(nodes[nextIndex], nodes[currentIndex]);
            }
        }
    }
    unsafe {
        web_sys::console::log_1(&format!("edge build completed").into());
    }
    let build_background_front_edge =
        |points: &mut Uint32Array,
         node: Node<u32>,
         upper: &mut Vec<f64>,
         builder_g: &mut VecGraphBuilder<u32>| {
            let front_vec = points.to_vec();
            let len = front_vec.len() as usize;
            let front = Array::from_shape_vec(((len / 2) as usize, 2), front_vec).unwrap();
            unsafe {
                web_sys::console::log_1(&format!("front build completed {:?}", front).into());
            }
            for i in 0..front.shape()[0] {
                let y = front[[i, 0]];
                let x = front[[i, 1]];

                let currentIndex = get_node_index(y as usize, x as usize);
                let anode = nodes[currentIndex];
                // unsafe {
                //     web_sys::console::log_1(&format!("anode {:?}", anode).into());
                // }
                upper.push(MAXIUM as f64);
                if anode == source {
                    builder_g.add_edge(anode, node);
                    upper.push(0.);
                    builder_g.add_edge(node, sink);
                } else {
                    builder_g.add_edge(node, anode);
                    upper.push(0.);
                    builder_g.add_edge(source, node);
                }

                // upper.push(MAXIUM as f64);
                // builder_g.add_edge(node, anode);
            }
        };
    // 构建前景节点
    {
        unsafe {
            web_sys::console::log_1(&format!("front build start").into());
        }
        build_background_front_edge(&mut front, source, &mut upper, &mut builder_g);
    }
    // 构建背景
    {
        unsafe {
            web_sys::console::log_1(&format!("background build start").into());
        }
        build_background_front_edge(&mut background, sink, &mut upper, &mut builder_g);
    }
    unsafe {
        web_sys::console::log_1(&format!("graph build completed").into());
    }
    let g = builder_g.into_graph();

    // maxflow mincut
    let (value, flow, mut mincut) = dinic(&g, source, sink, |e| (upper[e.index()] * 1000.) as u32);

    let mut fronts = vec![];
    // struct Dfs<'s, 'r, 'a> {
    //     f: &'s dyn Fn(&'r Dfs, &'a VecGraph<u32>, Node, &'a mut Vec<(usize, usize)>),
    // }

    // let dfs = Dfs {
    //     f: &|dfs: &Dfs, g: &VecGraph<u32>, startNode: Node, c: &mut Vec<(usize, usize)>| {
    //         g.outedges(source).for_each(|(e, n)| {
    //             {
    //                 c.push((g.node_id(startNode), g.node_id(n)));
    //             }
    //             (dfs.f)(dfs, g, n, c)
    //         });
    //     },
    // };

    // (dfs.f)(&dfs, &g, source, &mut fronts);

    // let dfs = |g: &VecGraph<u32>, startNode: Node, c: &mut Vec<(usize, usize)>| {
    //     g.outedges(source).for_each(|(e, n)| {
    //         {
    //             fronts.push((g.node_id(startNode), g.node_id(n)));
    //         }
    //         dfs(&g, n, fronts)
    //     });
    // };
    // dfs(&g, source, &mut fronts);
    // g.outedges(source)
    //     .for_each(|(e, n)| dfs(&g, n, &mut fronts));

    // let mut builder = VecGraphBuilder::with_capacities(0, 0);
    // g.nodes().for_each(|n| {
    //     // let nid = g.node2id(n);
    //     builder.add_node();
    // });
    // g.edges()
    //     .filter(|e| (upper[e.index()]).abs() > 0.0001)
    //     .for_each(|e| builder.add_edge(e));

    ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(&vec![0 as u8, 0 as u8, 0 as u8, 0 as u8]),
        1,
        1,
    )
}
