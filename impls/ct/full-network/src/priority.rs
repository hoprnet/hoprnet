use crate::ProberConfig;

/// Maximum staleness (in seconds) used to cap the staleness factor.
const MAX_STALENESS_SECS: f64 = 300.0;

/// Computes the probing priority for an immediate neighbor edge.
///
/// Higher values mean the peer should be probed sooner. Combines:
/// - **Staleness**: time since the edge was last measured (capped at [`MAX_STALENESS_SECS`])
/// - **Inverse quality**: `1.0 - score`, so worse edges get higher priority
/// - **Base**: ensures even well-measured, recently-probed peers get some chance
///
/// Peers with no edge observations receive maximum priority.
pub(crate) fn immediate_probe_priority(
    score: f64,
    last_update: std::time::Duration,
    now: std::time::Duration,
    cfg: &ProberConfig,
) -> f64 {
    let staleness_secs = if last_update.is_zero() {
        MAX_STALENESS_SECS
    } else {
        now.saturating_sub(last_update).as_secs_f64().min(MAX_STALENESS_SECS)
    };
    let normalized_staleness = staleness_secs / MAX_STALENESS_SECS;
    let inverse_quality = 1.0 - score.clamp(0.0, 1.0);

    cfg.staleness_weight * normalized_staleness + cfg.quality_weight * inverse_quality + cfg.base_priority
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maximal_for_unobserved_peers() {
        let cfg = ProberConfig::default();
        let now = std::time::Duration::from_secs(1000);

        let priority = immediate_probe_priority(0.0, std::time::Duration::ZERO, now, &cfg);
        let max_priority = cfg.staleness_weight + cfg.quality_weight + cfg.base_priority;

        assert!(
            (priority - max_priority).abs() < 1e-9,
            "unobserved peer priority {priority} should equal max {max_priority}"
        );
    }

    #[test]
    fn increases_with_staleness() {
        let cfg = ProberConfig::default();
        let now = std::time::Duration::from_secs(10000);
        let score = 0.5;

        let recent = immediate_probe_priority(score, now - std::time::Duration::from_secs(10), now, &cfg);
        let stale = immediate_probe_priority(score, now - std::time::Duration::from_secs(3000), now, &cfg);

        assert!(
            stale > recent,
            "staler peer ({stale}) should have higher priority than recently probed ({recent})"
        );
    }

    #[test]
    fn increases_with_lower_score() {
        let cfg = ProberConfig::default();
        let now = std::time::Duration::from_secs(1000);
        let last_update = now - std::time::Duration::from_secs(100);

        let good = immediate_probe_priority(0.9, last_update, now, &cfg);
        let bad = immediate_probe_priority(0.1, last_update, now, &cfg);

        assert!(
            bad > good,
            "low-score peer ({bad}) should have higher priority than high-score ({good})"
        );
    }
}
