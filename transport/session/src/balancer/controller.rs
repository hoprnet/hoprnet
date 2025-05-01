use std::fmt::Display;
use pid::Pid;
use hopr_crypto_types::crypto_traits::Digest;

use crate::balancer::{SurbFlowController, SurbFlowEstimator};
use crate::SessionId;

#[cfg(feature = "prometheus")]
lazy_static::lazy_static! {
    static ref METRIC_TARGET_ERROR_ESTIMATE: hopr_metrics::metrics::MultiGauge =
        hopr_metrics::metrics::MultiGauge::new(
            "hopr_surb_balancer_target_error_estimate",
            "Estimations of the target error of the SURB balancer",
            &["session_id"]
    ).unwrap();
    static ref METRIC_CONTROL_OUTPUT: hopr_metrics::metrics::MultiGauge =
        hopr_metrics::metrics::MultiGauge::new(
            "hopr_surb_balancer_control_output",
            "Outputs of the SURB balancer PID controller",
            &["session_id"]
    ).unwrap();
    static ref METRIC_SURBS_CONSUMED: hopr_metrics::metrics::MultiGauge =
        hopr_metrics::metrics::MultiGauge::new(
            "hopr_surb_balancer_surbs_consumed",
            "Estimations of the number of SURBs consumed by the counterparty",
            &["session_id"]
    ).unwrap();
    static ref METRIC_SURBS_PRODUCED: hopr_metrics::metrics::MultiGauge =
        hopr_metrics::metrics::MultiGauge::new(
            "hopr_surb_balancer_surbs_produced",
            "Estimations of the number of SURBs produced by the counterparty",
            &["session_id"]
    ).unwrap();
}

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
pub struct SurbBalancer<I, O, F,S> {
    session_id: S,
    pid: Pid<f64>,
    outflow_estimator: O,
    inflow_estimator: I,
    controller: F,
    cfg: SurbBalancerConfig,
    current_target_buffer: u64,
    last_surbs_delivered: u64,
    last_surbs_used: u64,
    last_update: std::time::Instant,
}

impl<I, O, F, S> SurbBalancer<I, O, F, S>
where
    O: SurbFlowEstimator,
    I: SurbFlowEstimator,
    F: SurbFlowController,
    S: Display
{
    pub fn new(outflow_estimator: O, inflow_estimator: I, controller: F, session_id: S, cfg: SurbBalancerConfig) -> Self {
        let pid = Pid::new(cfg.target_surb_buffer_size as f64, 1.0);

        Self {
            outflow_estimator,
            inflow_estimator,
            controller,
            pid,
            cfg,
            session_id,
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
            session = %self.session_id,
            ?dt,
            change = target_buffer_change,
            current = self.current_target_buffer,
            error,
            up = surbs_delivered_delta as f64 / dt.as_secs_f64(),
            down = surbs_used_delta as f64 / dt.as_secs_f64(),
            "estimated surb buffer change"
        );

        let output = self.pid.next_control_output(self.current_target_buffer as f64);

        #[cfg(feature = "prometheus")]
        {
            let sid = self.session_id.to_string();
            METRIC_TARGET_ERROR_ESTIMATE.set(&[&sid], error as f64);
            METRIC_CONTROL_OUTPUT.set(&[&sid], output.output);
            METRIC_SURBS_CONSUMED.set(&[&sid], surbs_used_delta as f64);
            METRIC_SURBS_PRODUCED.set(&[&sid], surbs_delivered_delta as f64);
        }

        self.last_update = std::time::Instant::now();
    }
}
