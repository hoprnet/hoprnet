use crate::balancer::{SurbFlowController, SurbFlowEstimator};

/// Configuration for the [`SurbBalancer`].
#[derive(Clone, Debug, PartialEq, Eq, smart_default::SmartDefault)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SurbBalancerConfig {
    /// The desired number of SURBs to be always kept as a buffer at the Session counterparty.
    ///
    /// The [`SurbBalancer`] will try to maintain approximately this number of SURBs
    /// at the counterparty at all times, by regulating the [flow of non-organic SURBs](SurbFlowController).
    #[default(10_000)]
    pub target_surb_buffer_size: u64,
    /// Initial outflow of non-organic SURBs.
    #[default(2)]
    pub initial_surbs_per_sec: u64,
    /// Maximum outflow of non-organic surbs.
    #[default(2000)]
    pub max_surbs_per_sec: u64,
}

/// Runs a continuous process that attempts to [evaluate](SurbFlowEstimator) and
/// [regulate](SurbFlowController) the flow of non-organic SURBs to the Session counterparty,
/// to keep the number of SURBs at the counterparty at a certain level.
pub struct SurbBalancer<I, O, F> {
    outflow_estimator: O,
    inflow_estimator: I,
    controller: F,
    cfg: SurbBalancerConfig,
    current_target_buffer: u64,
    last_surbs_delivered: u64,
    last_surbs_used: u64,
    last_update: std::time::Instant,
}

impl<I, O, F> SurbBalancer<I, O, F>
where
    O: SurbFlowEstimator,
    I: SurbFlowEstimator,
    F: SurbFlowController,
{
    pub fn new(outflow_estimator: O, inflow_estimator: I, controller: F, cfg: SurbBalancerConfig) -> Self {
        Self {
            outflow_estimator,
            inflow_estimator,
            controller,
            cfg,
            current_target_buffer: 0,
            last_surbs_delivered: 0,
            last_surbs_used: 0,
            last_update: std::time::Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let dt = self.last_update.elapsed();
        let surbs_delivered_delta = self.outflow_estimator.estimate_surb_turnout() - self.last_surbs_delivered;

        let surbs_used_delta = self.inflow_estimator.estimate_surb_turnout() - self.last_surbs_used;

        let target_buffer_change = surbs_delivered_delta as i64 - surbs_used_delta as i64;
        self.current_target_buffer = self.current_target_buffer.saturating_add_signed(target_buffer_change);

        let error = self.cfg.target_surb_buffer_size as i64 - self.current_target_buffer as i64;
        tracing::trace!(
            ?dt,
            change = target_buffer_change,
            current = self.current_target_buffer,
            error,
            up = surbs_delivered_delta as f64 / dt.as_secs_f64(),
            down = surbs_used_delta as f64 / dt.as_secs_f64(),
            "estimated surb buffer change"
        );

        self.last_update = std::time::Instant::now();
    }
}
