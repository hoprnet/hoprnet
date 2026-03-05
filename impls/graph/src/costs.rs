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

    /// Build an `Observations` with only capacity (from on-chain).
    /// No connectivity, no probe data — intermediate_probe is created
    /// by the capacity record but with default (zero-score) link data.
    fn obs_capacity_only() -> Observations {
        let mut obs = Observations::default();
        obs.record(EdgeWeightType::Capacity(Some(1000)));
        obs
    }

    // ── HoprForwardCostFn trait method tests ─────────────────────────────

    #[test]
    fn forward_cost_fn_invariants() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        assert_eq!(cost_fn.initial_cost(), 1.0);
        assert_eq!(cost_fn.min_cost(), Some(0.0));
        Ok(())
    }

    // ── Forward first edge (path_index == 0) ────────────────────────────

    #[test]
    fn forward_first_edge_positive_when_connected_with_capacity() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
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
    fn forward_first_edge_scales_by_immediate_score() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(2.0, &obs, 0);
        // cost = initial_cost * max(immediate_score, intermediate_score); scores in (0, 1]
        assert!(
            cost > 0.0 && cost <= 2.0,
            "cost should be scaled by immediate score, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn forward_first_edge_positive_when_capacity_only_no_intermediate_probe() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
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
    fn forward_first_edge_negative_when_not_connected() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
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
    fn forward_first_edge_negative_when_connected_but_no_intermediate() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
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
    fn forward_first_edge_negative_when_connected_intermediate_but_no_capacity() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
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
    fn forward_first_edge_negative_when_empty() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_empty(), 0);
        assert!(
            cost < 0.0,
            "first edge should be negative with no observations, got {cost}"
        );
        Ok(())
    }

    // ── Forward last edge (path_index == length - 1) ────────────────────
    //
    // The forward last edge (relay -> dest) accepts EITHER:
    //   1. Capacity (on-chain channel) — for edges between other nodes
    //   2. Connectivity + score — for edges where me has direct data (e.g. relay -> me)

    #[test]
    fn forward_last_edge_positive_when_capacity_and_score() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(1.0, &obs, 2);
        assert!(
            cost > 0.0,
            "last edge should have positive cost with capacity and score, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn forward_last_edge_positive_with_capacity_only_no_probes() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        // Capacity creates intermediate_probe with default link data (score 0).
        // The cost function passes through initial_cost as baseline trust.
        let cost = f(1.0, &obs_capacity_only(), 2);
        assert_eq!(
            cost, 1.0,
            "forward last edge with capacity-only should pass through initial_cost, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn forward_last_edge_positive_without_connectivity() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        // Has intermediate + capacity but no connectivity — still positive via capacity
        let obs = obs_not_connected_with_intermediate();

        let cost = f(1.0, &obs, 2);
        assert!(
            cost > 0.0,
            "last edge should be positive with capacity even without connectivity, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn forward_last_edge_positive_with_connectivity_no_capacity() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        // Connected with immediate only — no capacity, but connectivity fallback accepts it
        let obs = obs_connected_only_immediate();

        let cost = f(1.0, &obs, 2);
        assert!(
            cost > 0.0,
            "last edge should be positive via connectivity fallback, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn forward_last_edge_scales_by_intermediate_score() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        let cost = f(2.0, &obs, 2);
        // cost = initial_cost * intermediate_score; intermediate_score is in (0, 1]
        assert!(
            cost > 0.0 && cost <= 2.0,
            "cost should be scaled by intermediate score, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn forward_last_edge_positive_when_intermediate_but_no_capacity() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        // Has intermediate data but no capacity — no channel exists for the last edge,
        // but the forward cost function passes through initial_cost as baseline trust.
        let mut obs = Observations::default();
        obs.record(EdgeWeightType::Intermediate(Ok(std::time::Duration::from_millis(50))));

        let cost = f(1.0, &obs, 2);
        assert_eq!(
            cost, 1.0,
            "last edge should pass through initial_cost without capacity, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn forward_last_edge_positive_when_empty() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_empty(), 2);
        assert_eq!(
            cost, 1.0,
            "last edge should pass through initial_cost with no observations, got {cost}"
        );
        Ok(())
    }

    // ── Forward intermediate edges (0 < path_index < length) ────────────

    #[test]
    fn forward_intermediate_edge_positive_when_capacity_and_score() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
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
    fn forward_intermediate_edge_scales_by_intermediate_score() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
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
    fn forward_intermediate_edge_negative_when_no_intermediate() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
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
    fn forward_intermediate_edge_negative_when_no_capacity() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
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
    fn forward_intermediate_edge_negative_when_empty() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_empty(), 1);
        assert!(
            cost < 0.0,
            "intermediate edge should be negative with no observations, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn forward_intermediate_edge_uses_observations() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        let cost_empty = f(1.0, &obs_empty(), 1);
        let cost_full = f(1.0, &obs_connected_with_capacity(), 1);
        assert_ne!(cost_empty, cost_full, "intermediate edges should use observations");
        Ok(())
    }

    // ── Forward length boundary tests ───────────────────────────────────

    #[test]
    fn forward_length_one_has_only_first_and_last_edge() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(1).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        // path_index 0 = first edge (also last edge for length=1, but first-edge arm catches it)
        let first = f(1.0, &obs, 0);
        assert!(first > 0.0, "index 0 should be first-edge logic");
        Ok(())
    }

    #[test]
    fn forward_length_two_intermediate_at_index_one() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(2).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();
        let obs = obs_connected_with_capacity();

        // index 1 = last edge (length - 1 = 1) — positive when connected
        let cost = f(1.0, &obs, 1);
        assert!(
            cost > 0.0,
            "index 1 should be last-edge logic (positive when connected with score)"
        );

        // index 1 with empty obs — forward last edge passes through initial_cost
        let cost_empty = f(1.0, &obs_empty(), 1);
        assert_eq!(
            cost_empty, 1.0,
            "index 1 (last edge) should pass through initial_cost with empty obs"
        );
        Ok(())
    }

    // ── Forward negative initial cost propagation ───────────────────────

    #[test]
    fn forward_negative_initial_cost_inverts_rejection() -> anyhow::Result<()> {
        let cost_fn =
            HoprForwardCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
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

    // ── HoprReturnCostFn trait method tests ──────────────────────────────

    #[test]
    fn return_cost_fn_invariants() -> anyhow::Result<()> {
        let cost_fn =
            HoprReturnCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(3).context("should be non-zero")?);
        assert_eq!(cost_fn.initial_cost(), 1.0);
        assert_eq!(cost_fn.min_cost(), Some(0.0));
        Ok(())
    }

    // ── Return first edge (path_index == 0) ─────────────────────────────
    //
    // The return first edge (dest -> relay) only requires capacity.
    // No connectivity check. Cost is scaled by intermediate_observation.score().

    #[test]
    fn return_first_edge_positive_with_intermediate_and_capacity() -> anyhow::Result<()> {
        let cost_fn =
            HoprReturnCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(2).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        // intermediate probe (score > 0) + capacity → positive cost
        let obs = obs_not_connected_with_intermediate();
        let cost = f(1.0, &obs, 0);
        assert!(
            cost > 0.0,
            "return first edge should be positive with intermediate + capacity, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn return_first_edge_positive_with_full_data() -> anyhow::Result<()> {
        let cost_fn =
            HoprReturnCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(2).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_connected_with_capacity(), 0);
        assert!(
            cost > 0.0,
            "return first edge should also work with full data, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn return_first_edge_scales_by_intermediate_score() -> anyhow::Result<()> {
        let cost_fn =
            HoprReturnCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(2).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        let obs = obs_not_connected_with_intermediate();
        let cost = f(2.0, &obs, 0);
        // cost = initial_cost * intermediate_score; intermediate_score is in (0, 1]
        assert!(
            cost > 0.0 && cost <= 2.0,
            "return first edge should scale by intermediate score, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn return_first_edge_does_not_require_connectivity() -> anyhow::Result<()> {
        let cost_fn =
            HoprReturnCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(2).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        // Has intermediate + capacity but no connectivity — still positive
        let obs = obs_not_connected_with_intermediate();
        let cost = f(1.0, &obs, 0);
        assert!(
            cost > 0.0,
            "return first edge should not require connectivity, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn return_first_edge_positive_when_capacity_only_no_probes() -> anyhow::Result<()> {
        let cost_fn =
            HoprReturnCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(2).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        // Capacity creates intermediate_probe with default link data (score 0).
        // With no probes the score is 0, but the cost function passes through
        // `initial_cost` as baseline trust when capacity exists.
        let cost = f(1.0, &obs_capacity_only(), 0);
        assert_eq!(
            cost, 1.0,
            "return first edge with capacity-only should pass through initial_cost, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn return_first_edge_negative_when_no_capacity() -> anyhow::Result<()> {
        let cost_fn =
            HoprReturnCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(2).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_connected_only_immediate(), 0);
        assert!(
            cost < 0.0,
            "return first edge should be negative without capacity, got {cost}"
        );
        Ok(())
    }

    #[test]
    fn return_first_edge_negative_when_empty() -> anyhow::Result<()> {
        let cost_fn =
            HoprReturnCostFn::<_, Observations>::new(std::num::NonZeroUsize::new(2).context("should be non-zero")?);
        let f = cost_fn.into_cost_fn();

        let cost = f(1.0, &obs_empty(), 0);
        assert!(
            cost < 0.0,
            "return first edge should be negative with no observations, got {cost}"
        );
        Ok(())
    }

    // ── Return last edge ────────────────────────────────────────────────

    #[test]
    fn return_last_edge_requires_connectivity() -> anyhow::Result<()> {
        let length = std::num::NonZeroUsize::new(2).context("should be non-zero")?;

        let ret = HoprReturnCostFn::<_, Observations>::new(length);
        let ret_fn = ret.into_cost_fn();

        // Return last edge (relay -> me) requires immediate_qos + is_connected
        let obs = obs_connected_with_capacity();
        let cost = ret_fn(1.0, &obs, 1);
        assert!(
            cost > 0.0,
            "return last edge should be positive when connected, got {cost}"
        );

        // Without connectivity → rejected
        let obs_no_conn = obs_not_connected_with_intermediate();
        let cost = ret_fn(1.0, &obs_no_conn, 1);
        assert!(
            cost < 0.0,
            "return last edge should be negative without connectivity, got {cost}"
        );

        Ok(())
    }

    #[test]
    fn forward_last_edge_differs_from_return_last_edge() -> anyhow::Result<()> {
        let length = std::num::NonZeroUsize::new(2).context("should be non-zero")?;

        let fwd = HoprForwardCostFn::<_, Observations>::new(length);
        let ret = HoprReturnCostFn::<_, Observations>::new(length);
        let fwd_fn = fwd.into_cost_fn();
        let ret_fn = ret.into_cost_fn();

        // Edge with capacity but no connectivity: forward accepts, return rejects
        let obs = obs_not_connected_with_intermediate();
        let fwd_cost = fwd_fn(1.0, &obs, 1);
        let ret_cost = ret_fn(1.0, &obs, 1);
        assert!(
            fwd_cost > 0.0,
            "forward last edge accepts capacity-only, got {fwd_cost}"
        );
        assert!(ret_cost < 0.0, "return last edge requires connectivity, got {ret_cost}");

        Ok(())
    }

    // ── Return intermediate edge ────────────────────────────────────────

    #[test]
    fn return_intermediate_edge_same_as_forward() -> anyhow::Result<()> {
        let length = std::num::NonZeroUsize::new(3).context("should be non-zero")?;

        let fwd = HoprForwardCostFn::<_, Observations>::new(length);
        let ret = HoprReturnCostFn::<_, Observations>::new(length);
        let fwd_fn = fwd.into_cost_fn();
        let ret_fn = ret.into_cost_fn();

        let obs = obs_connected_with_capacity();

        let fwd_cost = fwd_fn(1.0, &obs, 1);
        let ret_cost = ret_fn(1.0, &obs, 1);
        assert_eq!(
            fwd_cost, ret_cost,
            "return intermediate edge should behave identically to forward intermediate edge"
        );

        Ok(())
    }

    // ── Symmetrical communication tests ─────────────────────────────────
    //
    // For bidirectional (symmetrical) communication over a 1-hop path,
    // the planner (`me`) must construct both:
    //   forward: me -> relay -> dest   (via HoprForwardCostFn)
    //   return:  dest -> relay -> me   (via HoprReturnCostFn)
    //
    // The planner has full observational data for `me -> relay` (heartbeat
    // probes, intermediate loopback probes, on-chain capacity).  For the
    // reverse first edge `dest -> relay`, the planner typically only has
    // intermediate + capacity data — no connectivity.
    //
    // `HoprForwardCostFn` rejects `dest -> relay` outright because it
    // requires connectivity on the first edge.
    //
    // `HoprReturnCostFn` accepts it: the first edge only needs capacity
    // and scales by the intermediate score.

    #[test]
    fn symmetrical_forward_path_works_with_forward_cost_fn() -> anyhow::Result<()> {
        let length = std::num::NonZeroUsize::new(2).context("should be non-zero")?;
        let cost_fn = HoprForwardCostFn::<_, Observations>::new(length);
        let f = cost_fn.into_cost_fn();

        // Forward path: me -> relay -> dest
        // me->relay has full data; relay->dest only has capacity (me can't see their connectivity)
        let me_to_relay = obs_connected_with_capacity();
        let relay_to_dest = obs_capacity_only();

        let cost_after_first = f(1.0, &me_to_relay, 0);
        assert!(
            cost_after_first > 0.0,
            "forward first edge (me->relay) should be positive, got {cost_after_first}"
        );

        let cost_after_last = f(cost_after_first, &relay_to_dest, 1);
        assert!(
            cost_after_last > 0.0,
            "forward last edge (relay->dest) should be positive with capacity-only, got {cost_after_last}"
        );

        Ok(())
    }

    #[test]
    fn symmetrical_return_path_rejected_by_forward_cost_fn() -> anyhow::Result<()> {
        let length = std::num::NonZeroUsize::new(2).context("should be non-zero")?;
        let cost_fn = HoprForwardCostFn::<_, Observations>::new(length);
        let f = cost_fn.into_cost_fn();

        // Return path: dest -> relay -> me
        // The planner has intermediate + capacity for dest->relay but no
        // connectivity. HoprForwardCostFn rejects this edge outright.
        let dest_to_relay = obs_not_connected_with_intermediate();
        let relay_to_me = obs_connected_with_capacity();

        let cost_after_first = f(1.0, &dest_to_relay, 0);
        assert!(
            cost_after_first < 0.0,
            "HoprForwardCostFn should reject the return first edge without connectivity, got {cost_after_first}"
        );

        // The entire return path is pruned because the first edge is negative.
        let cost_after_last = f(cost_after_first, &relay_to_me, 1);
        assert!(
            cost_after_last < 0.0,
            "HoprForwardCostFn return path should be fully rejected, got {cost_after_last}"
        );

        Ok(())
    }

    #[test]
    fn symmetrical_return_path_works_with_return_cost_fn() -> anyhow::Result<()> {
        let length = std::num::NonZeroUsize::new(2).context("should be non-zero")?;
        let cost_fn = HoprReturnCostFn::<_, Observations>::new(length);
        let f = cost_fn.into_cost_fn();

        // Return path: dest -> relay -> me
        // intermediate + capacity (no connectivity) — accepted by HoprReturnCostFn
        let dest_to_relay = obs_not_connected_with_intermediate();
        let relay_to_me = obs_connected_with_capacity();

        let cost_after_first = f(1.0, &dest_to_relay, 0);
        assert!(
            cost_after_first > 0.0,
            "HoprReturnCostFn first edge should have positive cost, got {cost_after_first}"
        );

        let cost_after_last = f(cost_after_first, &relay_to_me, 1);
        assert!(
            cost_after_last > 0.0,
            "HoprReturnCostFn return path should have positive cost, got {cost_after_last}"
        );

        Ok(())
    }

    #[test]
    fn symmetrical_bidirectional_both_paths_positive() -> anyhow::Result<()> {
        let length = std::num::NonZeroUsize::new(2).context("should be non-zero")?;

        // Forward: me -> relay -> dest (using HoprForwardCostFn)
        // relay->dest only has capacity — me can't observe their connectivity
        let fwd = HoprForwardCostFn::<_, Observations>::new(length);
        let fwd_fn = fwd.into_cost_fn();

        let me_to_relay = obs_connected_with_capacity();
        let relay_to_dest = obs_capacity_only();

        let fwd_cost = fwd_fn(1.0, &me_to_relay, 0);
        let fwd_cost = fwd_fn(fwd_cost, &relay_to_dest, 1);
        assert!(fwd_cost > 0.0, "forward path should have positive cost, got {fwd_cost}");

        // Return: dest -> relay -> me (using HoprReturnCostFn)
        // dest->relay only has capacity — me can't observe their connectivity
        let ret = HoprReturnCostFn::<_, Observations>::new(length);
        let ret_fn = ret.into_cost_fn();

        let dest_to_relay = obs_capacity_only();
        let relay_to_me = obs_connected_with_capacity();

        let ret_cost = ret_fn(1.0, &dest_to_relay, 0);
        let ret_cost = ret_fn(ret_cost, &relay_to_me, 1);
        assert!(ret_cost > 0.0, "return path should have positive cost, got {ret_cost}");

        Ok(())
    }
}
