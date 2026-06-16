//! Adaptation of the algorithm for `petgraph::algo::simple_path::all_simple_paths_multi` to accept
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
use rand::seq::SliceRandom;
use smallvec::SmallVec;

/// Calculate all simple paths from a source node to any of several target nodes.
///
/// This function is a variant of `all_simple_paths` that accepts a `HashSet` of
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
/// to the source, the lookup overhead could make repeated calls to `all_simple_paths`
/// a faster alternative.
///
/// **Note**: If security is not a concern, a faster hasher (e.g., `FxBuildHasher`)
/// can be specified to minimize the `HashSet` lookup overhead.
///
/// # Arguments
/// * `graph`: an input graph.
/// * `from`: an initial node of desired paths.
/// * `to`: a `HashSet` of target nodes. A path is yielded as soon as it reaches any node in this set.
/// * `excluded_nodes`: an optional set of nodes to exclude from all returned paths. Excluded nodes are pre-seeded into
///   the visited set so the DFS never enters a branch containing them. Passing `Some(&empty)` or `None` is equivalent
///   to the default behavior (no exclusions). If `from` is in the excluded set, it is ignored — the source is always
///   reachable.
/// * `min_intermediate_nodes`: the minimum number of nodes in the desired paths.
/// * `max_intermediate_nodes`: the maximum number of nodes in the desired paths (optional).
/// * `initial_cost`: the starting cost value before any edges are traversed.
/// * `min_cost`: an optional threshold. If the accumulated cost drops below this value (via `PartialOrd`), the branch
///   is pruned — it is neither yielded nor explored further.
/// * `cost_fn`: an accumulator function `(accumulated_cost, &edge_weight, edge_count) -> new_cost` applied at each
///   edge. The `edge_count` is the 0-based hop number from the source (i.e., 0 for the first edge, 1 for the second,
///   etc.).
///
/// # Returns
/// Returns an iterator that produces `(path, cost)` tuples for all simple paths from `from` node to any node in the
/// `to` set, which contains at least `min_intermediate_nodes` and at most `max_intermediate_nodes` intermediate nodes,
/// if given, or limited by the graph's order otherwise. The cost is the result of folding `cost_fn` over the edge
/// weights along the path. Paths whose accumulated cost falls below `min_cost` at any point are excluded.
///
/// # Complexity
/// * Time complexity: for computing the first **k** paths, the running time will be **O(k|V| + k|E|)**.
/// * Auxiliary space: **O(|V|)**.
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
///     &graph, a, &targets, None, 0, None, 0i32, None, |cost, weight, _| cost + weight,
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
    excluded_nodes: Option<&'a HashSet<G::NodeId, S>>,
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
    // last elem is a shuffled vec of edges of last visited node
    let mut rng = rand::rng();
    let mut initial: SmallVec<[_; 16]> = graph.edges_directed(from, Outgoing).collect();
    initial.shuffle(&mut rng);
    let mut stack = Vec::with_capacity(max_nodes);
    stack.push(initial.into_iter());
    // accumulated cost at each depth level, parallel to visited
    let mut costs: Vec<C> = Vec::with_capacity(max_nodes);
    costs.push(initial_cost);

    from_fn(move || {
        while let Some(edges) = stack.last_mut() {
            if let Some(edge) = edges.next() {
                let child = edge.target();

                // Excluded nodes checked separately — not inserted into `visited` — so they
                // never appear in the yielded path. Excluding `from` is a no-op since it's
                // already in `visited` at position 0.
                if visited.contains(&child) || excluded_nodes.is_some_and(|excl| excl.contains(&child)) {
                    continue;
                }

                // initialized by `from` so is always at least len 1
                let current_nodes = visited.len();
                let new_cost = cost_fn(costs.last().unwrap().clone(), edge.weight(), current_nodes - 1);

                // Prune branch if cost drops below threshold
                if let Some(ref min) = min_cost
                    && new_cost < *min
                {
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
                if (current_nodes < (max_nodes - 1)) && to.iter().any(|n| *n != child && !visited.contains(n)) {
                    visited.insert(child);
                    let mut child_edges: SmallVec<[_; 16]> = graph.edges_directed(child, Outgoing).collect();
                    child_edges.shuffle(&mut rng);
                    stack.push(child_edges.into_iter());
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

    /// Collect paths as sorted Vec<Vec<usize>> for deterministic snapshots.
    fn sorted_paths<T, I: Iterator<Item = (Vec<petgraph::graph::NodeIndex>, T)>>(iter: I) -> Vec<Vec<usize>> {
        let mut paths: Vec<Vec<usize>> = iter.map(|(v, _)| v.into_iter().map(|i| i.index()).collect()).collect();
        paths.sort();
        paths
    }

    #[test]
    fn undirected_graph_should_find_all_paths_to_multiple_targets() {
        let graph = UnGraph::<i32, i32>::from_edges([(0, 1), (1, 2), (2, 3), (2, 4)]);
        let targets = HashSet::from_iter([3.into(), 4.into()]);
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            None,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        ));
        insta::assert_yaml_snapshot!(paths);
    }

    #[test]
    fn directed_graph_should_find_all_paths_to_multiple_targets() {
        let graph = DiGraph::<i32, ()>::from_edges([(0, 1), (1, 2), (2, 3), (2, 4)]);
        let targets = HashSet::from_iter([3.into(), 4.into()]);
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            None,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        ));
        insta::assert_yaml_snapshot!(paths);
    }

    #[test]
    fn undirected_graph_should_respect_max_intermediate_nodes() {
        let graph = UnGraph::<i32, ()>::from_edges([(0, 1), (1, 2), (2, 3), (2, 4)]);
        let targets = HashSet::from_iter([3.into(), 4.into()]);
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            None,
            0,
            Some(2),
            0,
            None,
            |c, _, _| c,
        ));
        insta::assert_yaml_snapshot!(paths);
    }

    #[test]
    fn max_intermediate_nodes_should_not_be_exceeded_when_target_connects_to_target() {
        // Chain: 0->1->2->3, targets={2,3}, max_intermediate_nodes=1
        // Only [0,1,2] is valid (1 intermediate node).
        // Bug: the algorithm also yields [0,1,2,3] (2 intermediate nodes) because
        // it expands through target 2 to reach target 3, pushing visited to max_nodes,
        // then yields the grandchild path without checking the max length.
        let graph = DiGraph::<i32, ()>::from_edges([(0, 1), (1, 2), (2, 3)]);
        let targets = HashSet::from_iter([2.into(), 3.into()]);
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            None,
            0,
            Some(1),
            0,
            None,
            |c, _, _| c,
        ));
        insta::assert_yaml_snapshot!(paths);
    }

    #[test]
    fn directed_graph_should_respect_max_intermediate_nodes() {
        let graph = DiGraph::<i32, ()>::from_edges([(0, 1), (1, 2), (2, 3), (2, 4)]);
        let targets = HashSet::from_iter([3.into(), 4.into()]);
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            None,
            0,
            Some(2),
            0,
            None,
            |c, _, _| c,
        ));
        insta::assert_yaml_snapshot!(paths);
    }

    #[test]
    fn inline_targets_should_yield_both_short_and_long_paths() {
        let graph = UnGraph::<i32, ()>::from_edges([(0, 1), (1, 2), (2, 3)]);
        let targets = HashSet::from_iter([2.into(), 3.into()]);
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            None,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        ));
        insta::assert_yaml_snapshot!(paths);
    }

    #[test]
    fn cyclic_graph_should_yield_only_simple_paths() {
        let graph = DiGraph::<i32, ()>::from_edges([(0, 1), (1, 2), (2, 0), (1, 3)]);
        let targets = HashSet::from_iter([2.into(), 3.into()]);
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            None,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        ));
        insta::assert_yaml_snapshot!(paths);
    }

    #[test]
    fn source_in_target_set_should_not_yield_zero_length_path() {
        let graph = UnGraph::<i32, ()>::from_edges([(0, 1), (1, 2)]);
        let targets = HashSet::from_iter([0.into(), 1.into(), 2.into()]);
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            None,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        ));
        insta::assert_yaml_snapshot!(paths);
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
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            1.into(),
            &targets,
            None,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        ));
        insta::assert_yaml_snapshot!(paths);
    }

    #[test]
    fn min_intermediate_nodes_should_exclude_shorter_paths() {
        let graph = UnGraph::<i32, ()>::from_edges([(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
        let targets = HashSet::from_iter([1.into(), 3.into()]);
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            None,
            2,
            None,
            0,
            None,
            |c, _, _| c,
        ));
        insta::assert_yaml_snapshot!(paths);
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
        let results: Vec<(Vec<_>, f64)> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            n[0],
            &targets,
            None,
            0,
            None,
            1.0,
            None,
            |c, w, _| c * w,
        )
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
            None,
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
            None,
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
            None,
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

    #[test]
    fn excluded_nodes_should_prune_branches_containing_them() {
        // 0 → 1 → 2 → 3
        //      ↘ 4 → 3
        // Excluding node 2 forces the DFS to take only the 0→1→4→3 branch.
        let graph = DiGraph::<i32, ()>::from_edges([(0, 1), (1, 2), (2, 3), (1, 4), (4, 3)]);
        let targets = HashSet::from_iter([3.into()]);
        let excluded: HashSet<petgraph::graph::NodeIndex, RandomState> = HashSet::from_iter([2.into()]);
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            Some(&excluded),
            0,
            None,
            0,
            None,
            |c, _, _| c,
        ));
        assert_eq!(paths, vec![vec![0, 1, 4, 3]]);
    }

    #[test]
    fn excluded_target_yields_no_paths() {
        // If the only target is excluded, no paths should be returned.
        let graph = DiGraph::<i32, ()>::from_edges([(0, 1), (1, 2)]);
        let targets = HashSet::from_iter([2.into()]);
        let excluded: HashSet<petgraph::graph::NodeIndex, RandomState> = HashSet::from_iter([2.into()]);
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            Some(&excluded),
            0,
            None,
            0,
            None,
            |c, _, _| c,
        ));
        assert!(paths.is_empty());
    }

    #[test]
    fn complete_graph_should_yield_all_and_only_simple_paths() {
        use petgraph::graph::NodeIndex;

        // Build K5: complete directed graph on 5 nodes (every ordered pair gets an edge).
        let edges: Vec<(u32, u32)> = (0u32..5)
            .flat_map(|a| (0u32..5).filter(move |&b| b != a).map(move |b| (a, b)))
            .collect();
        let graph = DiGraph::<i32, ()>::from_edges(edges);

        let src: NodeIndex = 0.into();
        let dst: NodeIndex = 4.into();
        let targets = HashSet::from_iter([dst]);

        // ── Without exclusions ─────────────────────────────────────────────
        let all_paths: Vec<(Vec<NodeIndex>, i32)> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            src,
            &targets,
            None,
            0,
            None,
            0,
            None,
            |c, _, _| c,
        )
        .collect();

        // Intermediate pool = {1, 2, 3} (3 nodes).
        // Simple paths = P(3,0) + P(3,1) + P(3,2) + P(3,3) = 1 + 3 + 6 + 6 = 16.
        assert_eq!(all_paths.len(), 16, "expected 16 simple paths in K5 from 0 to 4");

        for (path, _) in &all_paths {
            assert_eq!(path.first(), Some(&src), "path must start at src: {path:?}");
            assert_eq!(path.last(), Some(&dst), "path must end at dst: {path:?}");
            // src and dst must not appear anywhere in the interior of the path.
            let interior = &path[1..path.len() - 1];
            assert!(!interior.contains(&src), "src repeated inside path: {path:?}");
            assert!(!interior.contains(&dst), "dst repeated inside path: {path:?}");
            // No node anywhere in the path (including src and dst) appears more than once.
            let unique: HashSet<&NodeIndex, RandomState> = path.iter().collect();
            assert_eq!(unique.len(), path.len(), "duplicate node in path: {path:?}");
        }

        // ── With excluded_nodes = {2} ──────────────────────────────────────
        let excluded: HashSet<NodeIndex, RandomState> = HashSet::from_iter([NodeIndex::from(2u32)]);
        let restricted: Vec<(Vec<NodeIndex>, i32)> = all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            src,
            &targets,
            Some(&excluded),
            0,
            None,
            0,
            None,
            |c, _, _| c,
        )
        .collect();

        // Intermediate pool shrinks to {1, 3} (2 nodes).
        // P(2,0) + P(2,1) + P(2,2) = 1 + 2 + 2 = 5.
        assert_eq!(restricted.len(), 5, "expected 5 paths when node 2 is excluded");

        for (path, _) in &restricted {
            assert!(
                !path.contains(&NodeIndex::from(2u32)),
                "excluded node 2 in path: {path:?}"
            );
            let unique: HashSet<&NodeIndex, RandomState> = path.iter().collect();
            assert_eq!(unique.len(), path.len(), "duplicate node in path: {path:?}");
        }

        // Restricted paths are a strict subset: every restricted path also appears in all_paths.
        let all_set: Vec<Vec<usize>> = all_paths
            .iter()
            .map(|(p, _)| p.iter().map(|n| n.index()).collect())
            .collect();
        for (path, _) in &restricted {
            let as_usize: Vec<usize> = path.iter().map(|n| n.index()).collect();
            assert!(
                all_set.contains(&as_usize),
                "restricted path not in all-paths: {path:?}"
            );
        }
    }

    #[test]
    fn excluded_from_should_be_ignored() {
        // Excluding the source itself must not prevent paths from starting.
        let graph = DiGraph::<i32, ()>::from_edges([(0, 1), (1, 2)]);
        let targets = HashSet::from_iter([2.into()]);
        let excluded: HashSet<petgraph::graph::NodeIndex, RandomState> = HashSet::from_iter([0.into()]);
        let paths = sorted_paths(all_simple_paths_multi::<_, _, RandomState, _, _>(
            &graph,
            0.into(),
            &targets,
            Some(&excluded),
            0,
            None,
            0,
            None,
            |c, _, _| c,
        ));
        assert_eq!(paths, vec![vec![0, 1, 2]]);
    }
}
