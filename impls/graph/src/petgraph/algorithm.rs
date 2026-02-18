//! Adaptation of the algorithm for [`petgraph::algo::simple_path::all_simple_paths_multi`] to accept
//! a cost function interacting with the edge weight.

use std::{
    collections::HashSet,
    hash::{BuildHasher, Hash},
    iter::from_fn,
};

use indexmap::IndexSet;
use petgraph::{
    Direction::Outgoing,
    visit::{EdgeRef, IntoEdgeReferences, IntoEdgesDirected, NodeCount},
};

/// Calculate all simple paths from a source node to any of several target nodes.
///
/// This function is a variant of [`all_simple_paths`] that accepts a `HashSet` of
/// target nodes instead of a single one. A path is yielded as soon as it reaches any
/// node in the `to` set.
///
/// # Performance Considerations
///
/// The efficiency of this function hinges on the graph's structure. It provides significant
/// performance gains on graphs where paths share long initial segments (e.g., trees and DAGs),
/// as the benefit of a single traversal outweighs the `HashSet` lookup overhead.
///
/// Conversely, in dense graphs where paths diverge quickly or for targets very close
/// to the source, the lookup overhead could make repeated calls to [`all_simple_paths`]
/// a faster alternative.
///
/// **Note**: If security is not a concern, a faster hasher (e.g., `FxBuildHasher`)
/// can be specified to minimize the `HashSet` lookup overhead.
///
/// # Arguments
/// * `graph`: an input graph.
/// * `from`: an initial node of desired paths.
/// * `to`: a `HashSet` of target nodes. A path is yielded as soon as it reaches any node in this set.
/// * `min_intermediate_nodes`: the minimum number of nodes in the desired paths.
/// * `max_intermediate_nodes`: the maximum number of nodes in the desired paths (optional).
/// * `initial_cost`: the starting cost value before any edges are traversed.
/// * `min_cost`: an optional threshold. If the accumulated cost drops below this value (via `PartialOrd`), the branch
///   is pruned — it is neither yielded nor explored further.
/// * `cost_fn`: an accumulator function `(accumulated_cost, &edge_weight, edge_count) -> new_cost` applied at each
///   edge. The `edge_count` is the 1-based hop number from the source (i.e., 1 for the first edge, 2 for the second,
///   etc.).
/// # Returns
/// Returns an iterator that produces `(path, cost)` tuples for all simple paths from `from` node to any node in the
/// `to` set, which contains at least `min_intermediate_nodes` and at most `max_intermediate_nodes` intermediate nodes,
/// if given, or limited by the graph's order otherwise. The cost is the result of folding `cost_fn` over the edge
/// weights along the path. Paths whose accumulated cost falls below `min_cost` at any point are excluded.
///
/// # Complexity
/// * Time complexity: for computing the first **k** paths, the running time will be **O(k|V| + k|E|)**.
/// * Auxillary space: **O(|V|)**.
///
/// where **|V|** is the number of nodes and **|E|** is the number of edges.
///
/// # Example
/// ```rust,ignore
/// use petgraph::prelude::*;
/// use std::collections::HashSet;
/// use std::collections::hash_map::RandomState;
///
/// let mut graph = DiGraph::<&str, i32>::new();
///
/// let a = graph.add_node("a");
/// let b = graph.add_node("b");
/// let c = graph.add_node("c");
/// let d = graph.add_node("d");
/// graph.extend_with_edges(&[(a, b, 1), (b, c, 1), (b, d, 1)]);
///
/// // Find paths from "a" to either "c" or "d", accumulating edge costs.
/// let targets = HashSet::from_iter([c, d]);
/// let mut paths = all_simple_paths_multi::<Vec<_>, _, RandomState, _, _>(
///     &graph, a, &targets, 0, None, 0i32, None, |cost, weight, _| cost + weight,
/// )
///     .collect::<Vec<_>>();
///
/// paths.sort_by_key(|(p, _)| p.clone());
/// let expected_paths = vec![
///     (vec![a, b, c], 2),
///     (vec![a, b, d], 2),
/// ];
///
/// assert_eq!(paths, expected_paths);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn all_simple_paths_multi<'a, TargetColl, G, S, F, C>(
    graph: G,
    from: G::NodeId,
    to: &'a HashSet<G::NodeId, S>,
    min_intermediate_nodes: usize,
    max_intermediate_nodes: Option<usize>,
    initial_cost: C,
    min_cost: Option<C>,
    cost_fn: F,
) -> impl Iterator<Item = (TargetColl, C)> + 'a
where
    G: NodeCount + IntoEdgesDirected + 'a,
    <G as IntoEdgesDirected>::EdgesDirected: 'a,
    G::NodeId: Eq + Hash,
    TargetColl: FromIterator<G::NodeId>,
    S: BuildHasher + Default,
    C: Clone + PartialOrd + 'a,
    F: Fn(C, &<<G as IntoEdgeReferences>::EdgeRef as EdgeRef>::Weight, usize) -> C + 'a,
{
    let max_nodes = if let Some(l) = max_intermediate_nodes {
        l + 2
    } else {
        graph.node_count()
    };

    let min_nodes = min_intermediate_nodes + 2;

    // list of visited nodes
    let mut visited: IndexSet<G::NodeId, S> = IndexSet::from_iter(Some(from));
    // list of edges from currently exploring path nodes,
    // last elem is list of edges of last visited node
    let mut stack = vec![graph.edges_directed(from, Outgoing)];
    // accumulated cost at each depth level, parallel to visited
    let mut costs: Vec<C> = vec![initial_cost];

    from_fn(move || {
        while let Some(edges) = stack.last_mut() {
            if let Some(edge) = edges.next() {
                let child = edge.target();

                if visited.contains(&child) {
                    continue;
                }

                let current_nodes = visited.len();
                let new_cost = cost_fn(costs.last().unwrap().clone(), edge.weight(), current_nodes);

                // Prune branch if cost drops below threshold
                if let Some(ref min) = min_cost
                    && new_cost < *min {
                        continue;
                    }

                let mut valid_path: Option<(TargetColl, C)> = None;

                // Check if we've reached a target node
                if to.contains(&child) && (current_nodes + 1) >= min_nodes {
                    valid_path = Some((
                        visited.iter().cloned().chain(Some(child)).collect::<TargetColl>(),
                        new_cost.clone(),
                    ));
                }

                // Expand the search only if within max length and unexplored target nodes remain
                if (current_nodes < max_nodes) && to.iter().any(|n| *n != child && !visited.contains(n)) {
                    visited.insert(child);
                    stack.push(graph.edges_directed(child, Outgoing));
                    costs.push(new_cost);
                }

                // yield the valid path if found
                if valid_path.is_some() {
                    return valid_path;
                }
            } else {
                // All edges of the current node have been explored
                stack.pop();
                visited.pop();
                costs.pop();
            }
        }
        None
    })
}

#[cfg(test)]
mod test {
    use std::collections::{HashSet, hash_map::RandomState};

    use petgraph::prelude::{DiGraph, UnGraph};

    use super::all_simple_paths_multi;

    #[test]
    fn undirected_graph_should_find_all_paths_to_multiple_targets() {
        let graph = UnGraph::<i32, i32>::from_edges([(0, 1), (1, 2), (2, 3), (2, 4)]);
        let targets = HashSet::from_iter([3.into(), 4.into()]);
        let paths: HashSet<Vec<_>> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        )
        .map(|(v, _): (Vec<_>, _)| v.into_iter().map(|i| i.index()).collect())
        .collect();
        let expected: HashSet<Vec<_>> = HashSet::from_iter([vec![0, 1, 2, 3], vec![0, 1, 2, 4]]);
        assert_eq!(paths, expected);
    }

    #[test]
    fn directed_graph_should_find_all_paths_to_multiple_targets() {
        let graph = DiGraph::<i32, ()>::from_edges([(0, 1), (1, 2), (2, 3), (2, 4)]);
        let targets = HashSet::from_iter([3.into(), 4.into()]);
        let paths: HashSet<Vec<_>> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        )
        .map(|(v, _): (Vec<_>, _)| v.into_iter().map(|i| i.index()).collect())
        .collect();
        let expected: HashSet<Vec<_>> = HashSet::from_iter([vec![0, 1, 2, 3], vec![0, 1, 2, 4]]);
        assert_eq!(paths, expected);
    }

    #[test]
    fn undirected_graph_should_respect_max_intermediate_nodes() {
        let graph = UnGraph::<i32, ()>::from_edges([(0, 1), (1, 2), (2, 3), (2, 4)]);
        let targets = HashSet::from_iter([3.into(), 4.into()]);
        let paths: HashSet<Vec<_>> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            0,
            Some(2),
            0,
            None,
            |c, _, _| c,
        )
        .map(|(v, _): (Vec<_>, _)| v.into_iter().map(|i| i.index()).collect())
        .collect();
        let expected: HashSet<Vec<_>> = HashSet::from_iter([vec![0, 1, 2, 3], vec![0, 1, 2, 4]]);
        assert_eq!(paths, expected);
    }

    #[test]
    fn directed_graph_should_respect_max_intermediate_nodes() {
        let graph = DiGraph::<i32, ()>::from_edges([(0, 1), (1, 2), (2, 3), (2, 4)]);
        let targets = HashSet::from_iter([3.into(), 4.into()]);
        let paths: HashSet<Vec<_>> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            0,
            Some(2),
            0,
            None,
            |c, _, _| c,
        )
        .map(|(v, _): (Vec<_>, _)| v.into_iter().map(|i| i.index()).collect())
        .collect();
        let expected: HashSet<Vec<_>> = HashSet::from_iter([vec![0, 1, 2, 3], vec![0, 1, 2, 4]]);
        assert_eq!(paths, expected);
    }

    #[test]
    fn inline_targets_should_yield_both_short_and_long_paths() {
        let graph = UnGraph::<i32, ()>::from_edges([(0, 1), (1, 2), (2, 3)]);
        let targets = HashSet::from_iter([2.into(), 3.into()]);
        let paths: HashSet<Vec<_>> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        )
        .map(|(v, _): (Vec<_>, _)| v.into_iter().map(|i| i.index()).collect())
        .collect();
        let expected: HashSet<Vec<_>> = HashSet::from_iter([vec![0, 1, 2], vec![0, 1, 2, 3]]);
        assert_eq!(paths, expected);
    }

    #[test]
    fn cyclic_graph_should_yield_only_simple_paths() {
        let graph = DiGraph::<i32, ()>::from_edges([(0, 1), (1, 2), (2, 0), (1, 3)]);
        let targets = HashSet::from_iter([2.into(), 3.into()]);
        let paths: HashSet<Vec<_>> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        )
        .map(|(v, _): (Vec<_>, _)| v.into_iter().map(|i| i.index()).collect())
        .collect();
        let expected = HashSet::from_iter([vec![0, 1, 2], vec![0, 1, 3]]);
        assert_eq!(paths, expected);
    }

    #[test]
    fn source_in_target_set_should_not_yield_zero_length_path() {
        let graph = UnGraph::<i32, ()>::from_edges([(0, 1), (1, 2)]);
        let targets = HashSet::from_iter([0.into(), 1.into(), 2.into()]);
        let paths: HashSet<Vec<_>> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        )
        .map(|(v, _): (Vec<_>, _)| v.into_iter().map(|i| i.index()).collect())
        .collect();
        let expected = HashSet::from_iter([vec![0, 1], vec![0, 1, 2]]);
        assert_eq!(paths, expected);
    }

    #[test]
    fn non_trivial_graph_should_find_all_simple_paths() {
        let graph = DiGraph::<i32, ()>::from_edges([
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (0, 5),
            (1, 5),
            (1, 3),
            (5, 4),
            (4, 2),
            (4, 3),
        ]);
        let targets = HashSet::from_iter([2.into(), 3.into()]);
        let paths: HashSet<Vec<_>> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            1.into(),
            &targets,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        )
        .map(|(v, _): (Vec<_>, _)| v.into_iter().map(|i| i.index()).collect())
        .collect();
        let expected = HashSet::from_iter([
            vec![1, 2],
            vec![1, 5, 4, 2],
            vec![1, 3, 4, 2],
            vec![1, 3],
            vec![1, 2, 3],
            vec![1, 5, 4, 3],
            vec![1, 5, 4, 2, 3],
        ]);
        assert_eq!(paths, expected);
    }

    #[test]
    fn min_intermediate_nodes_should_exclude_shorter_paths() {
        let graph = UnGraph::<i32, ()>::from_edges([(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
        let targets = HashSet::from_iter([1.into(), 3.into()]);
        let paths: HashSet<Vec<_>> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            2,
            None,
            0,
            None,
            |c, _, _| c,
        )
        .map(|(v, _): (Vec<_>, _)| v.into_iter().map(|i| i.index()).collect())
        .collect();
        let expected = HashSet::from_iter([vec![0, 2, 3, 1], vec![0, 3, 2, 1], vec![0, 1, 2, 3], vec![0, 2, 1, 3]]);
        assert_eq!(paths, expected);
    }

    #[test]
    fn multiplicative_cost_should_accumulate_along_path() {
        // 0 --0.9--> 1 --0.8--> 2 --0.7--> 3
        //                  \--0.6--> 4
        let mut graph = DiGraph::<(), f64>::new();
        let n: Vec<_> = (0..5).map(|_| graph.add_node(())).collect();
        graph.extend_with_edges([
            (n[0], n[1], 0.9),
            (n[1], n[2], 0.8),
            (n[2], n[3], 0.7),
            (n[1], n[4], 0.6),
        ]);

        let targets = HashSet::from_iter([n[3], n[4]]);
        let results: Vec<(Vec<_>, f64)> =
            all_simple_paths_multi::<_, _, RandomState, _, _>(&graph, n[0], &targets, 0, None, 1.0, None, |c, w, _| {
                c * w
            })
            .map(|(v, cost): (Vec<_>, f64)| (v.into_iter().map(|i| i.index()).collect(), cost))
            .collect();

        // Path 0->1->2->3: cost = 1.0 * 0.9 * 0.8 * 0.7 = 0.504
        // Path 0->1->4:     cost = 1.0 * 0.9 * 0.6 = 0.54
        assert_eq!(results.len(), 2);
        for (path, cost) in &results {
            match path.as_slice() {
                [0, 1, 2, 3] => assert!((cost - 0.504).abs() < 1e-9),
                [0, 1, 4] => assert!((cost - 0.54).abs() < 1e-9),
                other => panic!("unexpected path: {other:?}"),
            }
        }
    }

    #[test]
    fn min_cost_should_prune_path_falling_below_threshold() {
        // 0 --0.9--> 1 --0.8--> 2 --0.7--> 3
        //                  \--0.6--> 4
        // With min_cost = 0.51, path 0->1->2->3 (cost 0.504) is pruned at the 2->3 edge,
        // but 0->1->4 (cost 0.54) survives.
        let mut graph = DiGraph::<(), f64>::new();
        let n: Vec<_> = (0..5).map(|_| graph.add_node(())).collect();
        graph.extend_with_edges([
            (n[0], n[1], 0.9),
            (n[1], n[2], 0.8),
            (n[2], n[3], 0.7),
            (n[1], n[4], 0.6),
        ]);

        let targets = HashSet::from_iter([n[3], n[4]]);
        let results: Vec<(Vec<_>, f64)> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            n[0],
            &targets,
            0,
            None,
            1.0,
            Some(0.51),
            |c, w, _| c * w,
        )
        .map(|(v, cost): (Vec<_>, f64)| (v.into_iter().map(|i| i.index()).collect(), cost))
        .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, vec![0, 1, 4]);
        assert!((results[0].1 - 0.54).abs() < 1e-9);
    }

    #[test]
    fn min_cost_should_prune_entire_branch_on_low_first_edge() {
        // 0 --0.1--> 1 --0.9--> 2
        //      \--0.9--> 3 --0.9--> 2
        // With min_cost = 0.5, the 0->1 edge (cost 0.1) is pruned immediately,
        // so only 0->3->2 (cost 0.81) is found.
        let mut graph = DiGraph::<(), f64>::new();
        let n: Vec<_> = (0..4).map(|_| graph.add_node(())).collect();
        graph.extend_with_edges([
            (n[0], n[1], 0.1),
            (n[1], n[2], 0.9),
            (n[0], n[3], 0.9),
            (n[3], n[2], 0.9),
        ]);

        let targets = HashSet::from_iter([n[2]]);
        let results: Vec<(Vec<_>, f64)> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            n[0],
            &targets,
            0,
            None,
            1.0,
            Some(0.5),
            |c, w, _| c * w,
        )
        .map(|(v, cost): (Vec<_>, f64)| (v.into_iter().map(|i| i.index()).collect(), cost))
        .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, vec![0, 3, 2]);
        assert!((results[0].1 - 0.81).abs() < 1e-9);
    }

    #[test]
    fn min_cost_should_yield_empty_when_all_paths_below_threshold() {
        // 0 --0.3--> 1 --0.3--> 2
        // With min_cost = 0.5, the 0->1 edge (cost 0.3) is pruned immediately,
        // so no paths are found.
        let mut graph = DiGraph::<(), f64>::new();
        let n: Vec<_> = (0..3).map(|_| graph.add_node(())).collect();
        graph.extend_with_edges([(n[0], n[1], 0.3), (n[1], n[2], 0.3)]);

        let targets = HashSet::from_iter([n[2]]);
        let results: Vec<(Vec<_>, f64)> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            n[0],
            &targets,
            0,
            None,
            1.0,
            Some(0.5),
            |c, w, _| c * w,
        )
        .map(|(v, cost): (Vec<_>, f64)| (v.into_iter().map(|i| i.index()).collect(), cost))
        .collect();

        assert!(results.is_empty());
    }
}
