use hopr_api::graph::{
    CostFn, EdgeLinkObservable,
    traits::{EdgeNetworkObservableRead, EdgeObservableRead, EdgeProtocolObservable},
};

use crate::Observations;

/// Build a HOPR cost function for immediate graph traversals.
#[allow(clippy::type_complexity)]
pub struct SimpleHoprCostFn {
    cost_fn: Box<
        dyn Fn(
            <SimpleHoprCostFn as CostFn>::Cost,
            &<SimpleHoprCostFn as CostFn>::Weight,
            usize,
        ) -> <SimpleHoprCostFn as CostFn>::Cost,
    >,
}

impl SimpleHoprCostFn {
    pub fn new(length: usize) -> Self {
        Self {
            cost_fn: Box::new(
                move |initial_cost: f64, observation: &crate::Observations, path_index: usize| {
                    match path_index {
                        0 => {
                            // the first edge should always go to an already connected and measured peer,
                            // otherwise use a negative cost that should remove the edge from consideration
                            if observation.immediate_qos().is_some_and(|o| o.is_connected()) {
                                // TODO(20260217: extend once 1-hop probing verifiably works)
                                // && o.average_latency().is_some_and(|latency| latency)
                                if observation.intermediate_qos().is_some_and(|o| o.capacity().is_some()) {
                                    return initial_cost;
                                }
                            }

                            -initial_cost
                        }
                        v if v == length => {
                            // the last edge should always go from an already connected and measured peer,
                            // otherwise use a negative cost that should remove the edge from consideration
                            if observation.immediate_qos().is_some_and(|o| o.is_connected()) {
                                // TODO(20260217: extend once 1-hop probing verifiably works)
                                // if observation.intermediate_qos().is_some_and(|o| o.capacity().o.score()()) {
                                return initial_cost;
                                // }
                            }

                            -initial_cost
                        }
                        _ => initial_cost,
                    }
                },
            ),
        }
    }
}

impl CostFn for SimpleHoprCostFn {
    type Cost = f64;
    type Weight = Observations;

    fn initial_cost(&self) -> Self::Cost {
        1.0
    }

    fn min_cost(&self) -> Option<Self::Cost> {
        Some(0.0)
    }

    #[warn(clippy::type_complexity)]
    fn into_cost_fn(self) -> Box<dyn Fn(Self::Cost, &Self::Weight, usize) -> Self::Cost> {
        self.cost_fn
    }
}

/// Build a HOPR cost function for full graph traversals.
#[allow(clippy::type_complexity)]
pub struct HoprCostFn {
    cost_fn: Box<
        dyn Fn(<HoprCostFn as CostFn>::Cost, &<HoprCostFn as CostFn>::Weight, usize) -> <HoprCostFn as CostFn>::Cost,
    >,
}

impl HoprCostFn {
    pub fn new(length: usize) -> Self {
        Self {
            cost_fn: Box::new(
                move |initial_cost: f64, observation: &crate::Observations, path_index: usize| {
                    match path_index {
                        0 => {
                            // the first edge should always go to an already connected and measured peer,
                            // otherwise use a negative cost that should remove the edge from consideration
                            if observation.immediate_qos().is_some_and(|o| o.is_connected())
                                && let Some(intermediate_observation) = observation.intermediate_qos()
                                && intermediate_observation.capacity().is_some()
                            {
                                return initial_cost * intermediate_observation.score();
                            }

                            -initial_cost
                        }
                        v if v == length => {
                            // the last edge should always go from an already connected and measured peer,
                            // otherwise use a negative cost that should remove the edge from consideration
                            if observation.immediate_qos().is_some_and(|o| o.is_connected()) {
                                let score = observation.score();
                                if score > 0.0 {
                                    return initial_cost * score;
                                }
                            }

                            -initial_cost
                        }
                        _ => initial_cost,
                    }
                },
            ),
        }
    }
}

impl CostFn for HoprCostFn {
    type Cost = f64;
    type Weight = Observations;

    fn initial_cost(&self) -> Self::Cost {
        1.0
    }

    fn min_cost(&self) -> Option<Self::Cost> {
        Some(0.0)
    }

    #[warn(clippy::type_complexity)]
    fn into_cost_fn(self) -> Box<dyn Fn(Self::Cost, &Self::Weight, usize) -> Self::Cost> {
        self.cost_fn
    }
}

/// Used for finding simple paths without the final loopback in a loopback call.
#[allow(clippy::type_complexity)]
pub struct LoopbackPathCostFn {
    cost_fn: Box<
        dyn Fn(
            <LoopbackPathCostFn as CostFn>::Cost,
            &<LoopbackPathCostFn as CostFn>::Weight,
            usize,
        ) -> <LoopbackPathCostFn as CostFn>::Cost,
    >,
}

impl Default for LoopbackPathCostFn {
    fn default() -> Self {
        Self::new()
    }
}

impl LoopbackPathCostFn {
    pub fn new() -> Self {
        Self {
            cost_fn: Box::new(
                move |initial_cost: f64, observation: &crate::Observations, path_index: usize| {
                    match path_index {
                        0 => {
                            // the first edge should always go to an already connected and measured peer,
                            // otherwise use a negative cost that should remove the edge from consideration
                            if observation.immediate_qos().is_some_and(|o| o.is_connected()) {
                                // TODO(20260217: extend once 1-hop probing verifiably works)
                                // && o.average_latency().is_some_and(|latency| latency)
                                if observation.intermediate_qos().is_some_and(|o| o.capacity().is_some()) {
                                    return initial_cost;
                                }
                            }

                            -initial_cost
                        }
                        _ => {
                            // the last peer is the one before a hop back to ourselves, so it's capacity must exist
                            if observation.intermediate_qos().is_some_and(|o| o.capacity().is_some()) {
                                return initial_cost;
                            }

                            -initial_cost
                        }
                    }
                },
            ),
        }
    }
}

impl CostFn for LoopbackPathCostFn {
    type Cost = f64;
    type Weight = Observations;

    fn initial_cost(&self) -> Self::Cost {
        1.0
    }

    fn min_cost(&self) -> Option<Self::Cost> {
        Some(0.0)
    }

    #[warn(clippy::type_complexity)]
    fn into_cost_fn(self) -> Box<dyn Fn(Self::Cost, &Self::Weight, usize) -> Self::Cost> {
        self.cost_fn
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::graph::traits::{EdgeObservableWrite, EdgeWeightType};

    use super::*;

    /// Build an `Observations` with immediate connected + intermediate with capacity.
    fn obs_connected_with_capacity() -> Observations {
        let mut obs = Observations::default();
        obs.record(EdgeWeightType::Connected(true));
        obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
        obs.record(EdgeWeightType::Intermediate(Ok(std::time::Duration::from_millis(50))));
        obs.record(EdgeWeightType::Capacity(Some(1000)));
        obs
    }

    /// Build an `Observations` with immediate connected but no intermediate data.
    fn obs_connected_only_immediate() -> Observations {
        let mut obs = Observations::default();
        obs.record(EdgeWeightType::Connected(true));
        obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
        obs
    }

    /// Build an `Observations` with intermediate + capacity but not connected.
    fn obs_not_connected_with_intermediate() -> Observations {
        let mut obs = Observations::default();
        obs.record(EdgeWeightType::Intermediate(Ok(std::time::Duration::from_millis(50))));
        obs.record(EdgeWeightType::Capacity(Some(1000)));
        obs
    }

    /// Build a bare `Observations` with no data at all.
    fn obs_empty() -> Observations {
        Observations::default()
    }

    // ── HoprCostFn trait method tests ────────────────────────────────────

    #[test]
    fn hopr_cost_fn_invariants() {
        let cost_fn = HoprCostFn::new(3);
        assert_eq!(cost_fn.initial_cost(), 1.0);
        assert_eq!(cost_fn.min_cost(), Some(0.0));
    }

    // ── First edge (path_index == 0) ─────────────────────────────────────

    #[test]
    fn hopr_first_edge_positive_when_connected_with_capacity() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(1.0, &obs, 0);
        assert!(
            cost > 0.0,
            "first edge should have positive cost when connected with capacity, got {cost}"
        );
    }

    #[test]
    fn hopr_first_edge_scales_by_intermediate_score() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(2.0, &obs, 0);
        // cost = initial_cost * intermediate_score; intermediate_score is in (0, 1]
        assert!(
            cost > 0.0 && cost <= 2.0,
            "cost should be scaled by intermediate score, got {cost}"
        );
    }

    #[test]
    fn hopr_first_edge_negative_when_not_connected() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();
        let obs = obs_not_connected_with_intermediate();

        let cost = f(1.0, &obs, 0);
        assert!(
            cost < 0.0,
            "first edge should be negative when not connected, got {cost}"
        );
    }

    #[test]
    fn hopr_first_edge_negative_when_connected_but_no_intermediate() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_only_immediate();

        let cost = f(1.0, &obs, 0);
        assert!(
            cost < 0.0,
            "first edge should be negative without intermediate QoS, got {cost}"
        );
    }

    #[test]
    fn hopr_first_edge_negative_when_connected_intermediate_but_no_capacity() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();
        let mut obs = Observations::default();
        obs.record(EdgeWeightType::Connected(true));
        obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
        obs.record(EdgeWeightType::Intermediate(Ok(std::time::Duration::from_millis(50))));
        // no capacity set

        let cost = f(1.0, &obs, 0);
        assert!(cost < 0.0, "first edge should be negative without capacity, got {cost}");
    }

    #[test]
    fn hopr_first_edge_negative_when_empty() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_empty(), 0);
        assert!(
            cost < 0.0,
            "first edge should be negative with no observations, got {cost}"
        );
    }

    // ── Last edge (path_index == length) ─────────────────────────────────

    #[test]
    fn hopr_last_edge_positive_when_connected_with_score() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(1.0, &obs, 3);
        assert!(
            cost > 0.0,
            "last edge should have positive cost when connected with score, got {cost}"
        );
    }

    #[test]
    fn hopr_last_edge_positive_when_connected_immediate_only() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_only_immediate();

        let cost = f(1.0, &obs, 3);
        assert!(
            cost > 0.0,
            "last edge should have positive cost with only immediate observation, got {cost}"
        );
    }

    #[test]
    fn hopr_last_edge_scales_by_overall_score() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(2.0, &obs, 3);
        assert!(
            cost > 0.0 && cost <= 2.0,
            "cost should be scaled by overall score, got {cost}"
        );
    }

    #[test]
    fn hopr_last_edge_negative_when_not_connected() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();
        let obs = obs_not_connected_with_intermediate();

        let cost = f(1.0, &obs, 3);
        assert!(
            cost < 0.0,
            "last edge should be negative when not connected, got {cost}"
        );
    }

    #[test]
    fn hopr_last_edge_negative_when_connected_but_zero_score() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();
        // Connected but only failed probes → score == 0
        let mut obs = Observations::default();
        obs.record(EdgeWeightType::Connected(true));
        obs.record(EdgeWeightType::Immediate(Err(())));

        let cost = f(1.0, &obs, 3);
        assert!(
            cost < 0.0,
            "last edge should be negative when score is zero, got {cost}"
        );
    }

    #[test]
    fn hopr_last_edge_negative_when_empty() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_empty(), 3);
        assert!(
            cost < 0.0,
            "last edge should be negative with no observations, got {cost}"
        );
    }

    // ── Intermediate edges (0 < path_index < length) ─────────────────────

    #[test]
    fn hopr_intermediate_edge_passes_through_initial_cost() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();

        assert_eq!(f(1.0, &obs_empty(), 1), 1.0);
        assert_eq!(f(0.5, &obs_connected_with_capacity(), 2), 0.5);
        assert_eq!(f(0.75, &obs_not_connected_with_intermediate(), 1), 0.75);
    }

    #[test]
    fn hopr_intermediate_edge_ignores_observations() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();

        let cost_empty = f(1.0, &obs_empty(), 1);
        let cost_full = f(1.0, &obs_connected_with_capacity(), 1);
        assert_eq!(cost_empty, cost_full, "intermediate edges should ignore observations");
    }

    // ── Length boundary tests ────────────────────────────────────────────

    #[test]
    fn hopr_length_one_has_only_first_and_last_edge() {
        let cost_fn = HoprCostFn::new(1);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        // path_index 0 = first edge
        let first = f(1.0, &obs, 0);
        assert!(first > 0.0, "index 0 should be first-edge logic");

        // path_index 1 = last edge (length == 1)
        let last = f(1.0, &obs, 1);
        assert!(last > 0.0, "index 1 should be last-edge logic");
    }

    #[test]
    fn hopr_length_two_intermediate_at_index_one() {
        let cost_fn = HoprCostFn::new(2);
        let f = cost_fn.into_cost_fn();

        // index 1 is intermediate (not first, not last)
        assert_eq!(
            f(1.0, &obs_empty(), 1),
            1.0,
            "index 1 should be intermediate passthrough"
        );

        // index 2 = last
        let cost = f(1.0, &obs_empty(), 2);
        assert!(cost < 0.0, "index 2 should be last-edge logic (negative for empty obs)");
    }

    // ── Negative initial cost propagation ────────────────────────────────

    #[test]
    fn hopr_negative_initial_cost_inverts_rejection() {
        let cost_fn = HoprCostFn::new(3);
        let f = cost_fn.into_cost_fn();

        // Normally empty obs at index 0 → -initial_cost = -1.0
        // With initial_cost = -1.0, rejection gives -(-1.0) = 1.0
        let cost = f(-1.0, &obs_empty(), 0);
        assert!(
            cost > 0.0,
            "negative initial cost should invert the rejection, got {cost}"
        );
    }
}
