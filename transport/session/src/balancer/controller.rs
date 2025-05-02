use pid::Pid;
use std::fmt::Display;

use crate::balancer::{SurbFlowController, SurbFlowEstimator};

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
    static ref METRIC_SURBS_CONSUMED: hopr_metrics::metrics::MultiCounter =
        hopr_metrics::metrics::MultiCounter::new(
            "hopr_surb_balancer_surbs_consumed",
            "Estimations of the number of SURBs consumed by the counterparty",
            &["session_id"]
    ).unwrap();
    static ref METRIC_SURBS_PRODUCED: hopr_metrics::metrics::MultiCounter =
        hopr_metrics::metrics::MultiCounter::new(
            "hopr_surb_balancer_surbs_produced",
            "Estimations of the number of SURBs produced for the counterparty",
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
    #[default(5_000)]
    pub target_surb_buffer_size: u64,
    /// Initial outflow of non-organic SURBs.
    ///
    /// The default is 100 (which is 50 packets/second currently)
    #[default(100)]
    pub initial_surbs_per_sec: u64,
    /// Maximum outflow of non-organic surbs.
    ///
    /// The default is 2500 (which is 1250 packets/second currently)
    #[default(2500)]
    pub max_surbs_per_sec: u64,
}

/// Runs a continuous process that attempts to [evaluate](SurbFlowEstimator) and
/// [regulate](SurbFlowController) the flow of non-organic SURBs to the Session counterparty,
/// to keep the number of SURBs at the counterparty at a certain level.
pub struct SurbBalancer<I, O, F, S> {
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
    S: Display,
{
    pub fn new(
        outflow_estimator: O,
        inflow_estimator: I,
        controller: F,
        session_id: S,
        cfg: SurbBalancerConfig,
    ) -> Self {
        let mut pid = Pid::new(cfg.target_surb_buffer_size as f64, cfg.max_surbs_per_sec as f64);
        pid.p(0.6, cfg.max_surbs_per_sec as f64);
        pid.i(0.7, cfg.max_surbs_per_sec as f64);
        pid.d(0.2, cfg.max_surbs_per_sec as f64);

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

    #[tracing::instrument(level = "trace", skip(self), fields(session_id = %self.session_id))]
    pub fn update(&mut self) {
        let dt = self.last_update.elapsed();
        self.last_update = std::time::Instant::now();

        // Number of SURBs sent to the counterparty since the last update
        let current_surbs_delivered = self.outflow_estimator.estimate_surb_turnout();
        let surbs_delivered_delta = current_surbs_delivered - self.last_surbs_delivered;
        self.last_surbs_delivered = current_surbs_delivered;

        // Number of SURBs used by the counterparty since the last update
        let current_surbs_used = self.inflow_estimator.estimate_surb_turnout();
        let surbs_used_delta = current_surbs_used - self.last_surbs_used;
        self.last_surbs_used = current_surbs_used;

        // Estimated change in counteparty's SURB buffer
        let target_buffer_change = surbs_delivered_delta as i64 - surbs_used_delta as i64;
        self.current_target_buffer = self.current_target_buffer.saturating_add_signed(target_buffer_change);
        if self.current_target_buffer == 0 {
            tracing::warn!("target SURB buffer size is 0")
        }

        // Error from the desired target SURB buffer size at counterparty
        let error = self.cfg.target_surb_buffer_size as i64 - self.current_target_buffer as i64;

        tracing::trace!(
            ?dt,
            delta = target_buffer_change,
            current = self.current_target_buffer,
            error,
            rate_up = surbs_delivered_delta as f64 / dt.as_secs_f64(),
            rate_down = surbs_used_delta as f64 / dt.as_secs_f64(),
            "estimated SURB buffer change"
        );

        let output = self.pid.next_control_output(self.current_target_buffer as f64);
        let corrected_output = output.output.clamp(0.0, self.cfg.max_surbs_per_sec as f64);
        self.controller.adjust_surb_flow(corrected_output as usize);

        tracing::trace!(control_output = corrected_output, "next control output");

        #[cfg(feature = "prometheus")]
        {
            let sid = self.session_id.to_string();
            METRIC_TARGET_ERROR_ESTIMATE.set(&[&sid], error as f64);
            METRIC_CONTROL_OUTPUT.set(&[&sid], corrected_output);
            METRIC_SURBS_CONSUMED.increment_by(&[&sid], surbs_used_delta);
            METRIC_SURBS_PRODUCED.increment_by(&[&sid], surbs_delivered_delta);
        }
    }
}
