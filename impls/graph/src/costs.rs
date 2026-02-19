pub use hopr_api::graph::costs::*;

#[cfg(test)]
mod tests {
    use hopr_api::graph::{
        CostFn,
        traits::{EdgeObservableWrite, EdgeWeightType},
    };

    use super::*;
    use crate::Observations;

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
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
        assert_eq!(cost_fn.initial_cost(), 1.0);
        assert_eq!(cost_fn.min_cost(), Some(0.0));
    }

    // ── First edge (path_index == 0) ─────────────────────────────────────

    #[test]
    fn hopr_first_edge_positive_when_connected_with_capacity() {
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
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
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
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
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
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
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
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
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
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
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
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
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
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
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
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
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
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
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
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
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
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
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_empty(), 3);
        assert!(
            cost < 0.0,
            "last edge should be negative with no observations, got {cost}"
        );
    }

    // ── Intermediate edges (0 < path_index < length) ─────────────────────

    #[test]
    fn hopr_intermediate_edge_positive_when_capacity_and_score() {
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(1.0, &obs, 1);
        assert!(
            cost > 0.0,
            "intermediate edge should have positive cost with capacity and score, got {cost}"
        );
    }

    #[test]
    fn hopr_intermediate_edge_scales_by_intermediate_score() {
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(2.0, &obs, 1);
        // cost = initial_cost * intermediate_score; intermediate_score is in (0, 1]
        assert!(
            cost > 0.0 && cost <= 2.0,
            "intermediate edge should be scaled by intermediate score, got {cost}"
        );
    }

    #[test]
    fn hopr_intermediate_edge_negative_when_no_intermediate() {
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_only_immediate();

        let cost = f(1.0, &obs, 1);
        assert!(
            cost < 0.0,
            "intermediate edge should be negative without intermediate QoS, got {cost}"
        );
    }

    #[test]
    fn hopr_intermediate_edge_negative_when_no_capacity() {
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
        let f = cost_fn.into_cost_fn();
        let mut obs = Observations::default();
        obs.record(EdgeWeightType::Intermediate(Ok(std::time::Duration::from_millis(50))));
        // no capacity set

        let cost = f(1.0, &obs, 1);
        assert!(
            cost < 0.0,
            "intermediate edge should be negative without capacity, got {cost}"
        );
    }

    #[test]
    fn hopr_intermediate_edge_negative_when_empty() {
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_empty(), 1);
        assert!(
            cost < 0.0,
            "intermediate edge should be negative with no observations, got {cost}"
        );
    }

    #[test]
    fn hopr_intermediate_edge_uses_observations() {
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
        let f = cost_fn.into_cost_fn();

        let cost_empty = f(1.0, &obs_empty(), 1);
        let cost_full = f(1.0, &obs_connected_with_capacity(), 1);
        assert_ne!(cost_empty, cost_full, "intermediate edges should use observations");
    }

    // ── Length boundary tests ────────────────────────────────────────────

    #[test]
    fn hopr_length_one_has_only_first_and_last_edge() {
        let cost_fn = HoprCostFn::<_, Observations>::new(1);
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
        let cost_fn = HoprCostFn::<_, Observations>::new(2);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        // index 1 is intermediate (not first, not last) — positive with capacity
        let cost = f(1.0, &obs, 1);
        assert!(
            cost > 0.0,
            "index 1 should be intermediate logic (positive with capacity)"
        );

        // index 1 with empty obs — negative (no intermediate data)
        let cost_empty = f(1.0, &obs_empty(), 1);
        assert!(cost_empty < 0.0, "index 1 should be negative with empty obs");

        // index 2 = last
        let cost_last = f(1.0, &obs_empty(), 2);
        assert!(
            cost_last < 0.0,
            "index 2 should be last-edge logic (negative for empty obs)"
        );
    }

    // ── Negative initial cost propagation ────────────────────────────────

    #[test]
    fn hopr_negative_initial_cost_inverts_rejection() {
        let cost_fn = HoprCostFn::<_, Observations>::new(3);
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
