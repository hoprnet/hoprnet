pub use hopr_api::graph::costs::*;

#[cfg(test)]
mod tests {
    use anyhow::Context;
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
    fn hopr_cost_fn_invariants() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        assert_eq!(cost_fn.initial_cost(), 1.0);
        assert_eq!(cost_fn.min_cost(), Some(0.0));
        Ok(())
    }

    // ── First edge (path_index == 0) ─────────────────────────────────────

    #[test]
    fn hopr_first_edge_positive_when_connected_with_capacity() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(1.0, &obs, 0);
        assert!(
            cost > 0.0,
            "first edge should have positive cost when connected with capacity, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_first_edge_scales_by_immediate_score() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(2.0, &obs, 0);
        // cost = initial_cost * immediate_score; immediate_score is in (0, 1]
        assert!(
            cost > 0.0 && cost <= 2.0,
            "cost should be scaled by immediate score, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_first_edge_positive_when_capacity_only_no_intermediate_probe() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        // Capacity update creates intermediate_probe with default link data (score 0),
        // but the first edge uses immediate_observation.score() instead.
        let mut obs = Observations::default();
        obs.record(EdgeWeightType::Connected(true));
        obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
        obs.record(EdgeWeightType::Capacity(Some(1000)));

        let cost = f(1.0, &obs, 0);
        assert!(
            cost > 0.0,
            "first edge should be positive when connected with capacity even without intermediate probes, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_first_edge_negative_when_not_connected() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_not_connected_with_intermediate();

        let cost = f(1.0, &obs, 0);
        assert!(
            cost < 0.0,
            "first edge should be negative when not connected, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_first_edge_negative_when_connected_but_no_intermediate() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_only_immediate();

        let cost = f(1.0, &obs, 0);
        assert!(
            cost < 0.0,
            "first edge should be negative without intermediate QoS, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_first_edge_negative_when_connected_intermediate_but_no_capacity() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let mut obs = Observations::default();
        obs.record(EdgeWeightType::Connected(true));
        obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
        obs.record(EdgeWeightType::Intermediate(Ok(std::time::Duration::from_millis(50))));
        // no capacity set

        let cost = f(1.0, &obs, 0);
        assert!(cost < 0.0, "first edge should be negative without capacity, got {cost}");
        Ok(())
    }

    #[test]
    fn hopr_first_edge_negative_when_empty() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_empty(), 0);
        assert!(
            cost < 0.0,
            "first edge should be negative with no observations, got {cost}"
        );
        Ok(())
    }

    // ── Last edge (path_index == length - 1) ───────────────────────────────

    #[test]
    fn hopr_last_edge_positive_when_connected_with_score() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(1.0, &obs, 2);
        assert!(
            cost > 0.0,
            "last edge should have positive cost when connected with score, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_last_edge_positive_when_connected_immediate_only() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_only_immediate();

        let cost = f(1.0, &obs, 2);
        assert!(
            cost > 0.0,
            "last edge should have positive cost with only immediate observation, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_last_edge_scales_by_overall_score() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(2.0, &obs, 2);
        assert!(
            cost > 0.0 && cost <= 2.0,
            "cost should be scaled by overall score, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_last_edge_negative_when_not_connected() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_not_connected_with_intermediate();

        let cost = f(1.0, &obs, 2);
        assert!(
            cost < 0.0,
            "last edge should be negative when not connected, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_last_edge_negative_when_connected_but_zero_score() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        // Connected but only failed probes → score == 0
        let mut obs = Observations::default();
        obs.record(EdgeWeightType::Connected(true));
        obs.record(EdgeWeightType::Immediate(Err(())));

        let cost = f(1.0, &obs, 2);
        assert!(
            cost < 0.0,
            "last edge should be negative when score is zero, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_last_edge_negative_when_empty() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_empty(), 2);
        assert!(
            cost < 0.0,
            "last edge should be negative with no observations, got {cost}"
        );
        Ok(())
    }

    // ── Intermediate edges (0 < path_index < length) ─────────────────────

    #[test]
    fn hopr_intermediate_edge_positive_when_capacity_and_score() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(1.0, &obs, 1);
        assert!(
            cost > 0.0,
            "intermediate edge should have positive cost with capacity and score, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_intermediate_edge_scales_by_intermediate_score() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(2.0, &obs, 1);
        // cost = initial_cost * intermediate_score; intermediate_score is in (0, 1]
        assert!(
            cost > 0.0 && cost <= 2.0,
            "intermediate edge should be scaled by intermediate score, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_intermediate_edge_negative_when_no_intermediate() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_only_immediate();

        let cost = f(1.0, &obs, 1);
        assert!(
            cost < 0.0,
            "intermediate edge should be negative without intermediate QoS, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_intermediate_edge_negative_when_no_capacity() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let mut obs = Observations::default();
        obs.record(EdgeWeightType::Intermediate(Ok(std::time::Duration::from_millis(50))));
        // no capacity set

        let cost = f(1.0, &obs, 1);
        assert!(
            cost < 0.0,
            "intermediate edge should be negative without capacity, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_intermediate_edge_negative_when_empty() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_empty(), 1);
        assert!(
            cost < 0.0,
            "intermediate edge should be negative with no observations, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn hopr_intermediate_edge_uses_observations() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        let cost_empty = f(1.0, &obs_empty(), 1);
        let cost_full = f(1.0, &obs_connected_with_capacity(), 1);
        assert_ne!(cost_empty, cost_full, "intermediate edges should use observations");
        Ok(())
    }

    // ── Length boundary tests ────────────────────────────────────────────

    #[test]
    fn hopr_length_one_has_only_first_and_last_edge() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(1).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        // path_index 0 = first edge (also last edge for length=1, but first-edge arm catches it)
        let first = f(1.0, &obs, 0);
        assert!(first > 0.0, "index 0 should be first-edge logic");
        Ok(())
    }

    #[test]
    fn hopr_length_two_intermediate_at_index_one() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(2).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        // index 1 = last edge (length - 1 = 1) — positive when connected
        let cost = f(1.0, &obs, 1);
        assert!(
            cost > 0.0,
            "index 1 should be last-edge logic (positive when connected with score)"
        );

        // index 1 with empty obs — negative (not connected)
        let cost_empty = f(1.0, &obs_empty(), 1);
        assert!(cost_empty < 0.0, "index 1 should be negative with empty obs");
        Ok(())
    }

    // ── Negative initial cost propagation ────────────────────────────────

    #[test]
    fn hopr_negative_initial_cost_inverts_rejection() -> anyhow::Result<()> {
        let cost_fn = HoprCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        // Normally empty obs at index 0 → -initial_cost = -1.0
        // With initial_cost = -1.0, rejection gives -(-1.0) = 1.0
        let cost = f(-1.0, &obs_empty(), 0);
        assert!(
            cost > 0.0,
            "negative initial cost should invert the rejection, got {cost}"
        );
        Ok(())
    }
}
