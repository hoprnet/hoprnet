use hopr_api::graph::{
    EdgeTransportObservable,
    traits::{EdgeObservable, EdgeTransportMeasurement, EdgeWeightType},
};
use hopr_primitive_types::prelude::{Balance, WxHOPR};
use hopr_statistics::ExponentialMovingAverage;

/// A representation of a individual edge measurement
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct TransportLinkMeasurement {
    latency_average: ExponentialMovingAverage<3>,
    probe_success_rate: ExponentialMovingAverage<5>,
}

impl EdgeTransportObservable for TransportLinkMeasurement {
    fn record(&mut self, measurement: EdgeTransportMeasurement) {
        if let Ok(latency) = measurement {
            self.latency_average.update(latency.as_millis() as f64);
            self.probe_success_rate.update(1.0);
        } else {
            self.probe_success_rate.update(0.0);
        }
    }

    fn average_latency(&self) -> Option<std::time::Duration> {
        if self.latency_average.get() <= 0.0 {
            None
        } else {
            Some(std::time::Duration::from_millis(self.latency_average.get() as u64))
        }
    }

    fn average_probe_rate(&self) -> f64 {
        self.probe_success_rate.get()
    }

    fn score(&self) -> f64 {
        self.probe_success_rate.get()
    }
}

/// Observations related to a specific peer in the network.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Observations {
    last_update: std::time::Duration,
    balance: Option<Balance<WxHOPR>>,
    immediate_probe: Option<TransportLinkMeasurement>,
    intermediate_probe: Option<TransportLinkMeasurement>,
}

impl EdgeObservable for Observations {
    type ImmediateMeasurement = TransportLinkMeasurement;
    type IntermediateMeasurement = TransportLinkMeasurement;

    fn record(&mut self, measurement: EdgeWeightType) {
        self.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        match measurement {
            EdgeWeightType::Immediate(result) => {
                if self.immediate_probe.is_none() {
                    self.immediate_probe = Some(TransportLinkMeasurement::default());
                }

                if let Some(probe) = self.immediate_probe.as_mut() { probe.record(result) }
            }

            EdgeWeightType::Intermediate(result) => {
                if self.intermediate_probe.is_none() {
                    self.intermediate_probe = Some(TransportLinkMeasurement::default());
                }

                if let Some(probe) = self.intermediate_probe.as_mut() { probe.record(result) }
            }
        }
    }

    #[inline]
    fn last_update(&self) -> std::time::Duration {
        self.last_update
    }

    fn balance(&self) -> Option<&Balance<WxHOPR>> {
        self.balance.as_ref()
    }

    fn immediate_qos(&self) -> Option<&Self::ImmediateMeasurement> {
        self.immediate_probe.as_ref()
    }

    fn intermediate_qos(&self) -> Option<&Self::IntermediateMeasurement> {
        self.intermediate_probe.as_ref()
    }

    /// The score is calculated based on the available observations, with priority order:
    /// 1. intermediate probe
    /// 2. immediate ones
    fn score(&self) -> f64 {
        if let Some(qos) = &self.intermediate_probe {
            qos.score()
        } else if let Some(qos) = &self.immediate_probe {
            qos.score()
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use assertables::{assert_gt, assert_in_delta, assert_lt};

    use super::*;

    #[test]
    fn observations_should_update_the_timestamp_on_latency_update() {
        let mut observation = Observations::default();

        assert_eq!(observation.last_update, std::time::Duration::default());

        observation.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));

        std::thread::sleep(std::time::Duration::from_millis(10));

        let after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        assert_gt!(observation.last_update, std::time::Duration::default());
        assert_lt!(observation.last_update, after);
    }

    #[test]
    fn observations_should_store_an_average_latency_value_after_multiple_updates() -> anyhow::Result<()> {
        let big_latency = std::time::Duration::from_millis(300);
        let small_latency = std::time::Duration::from_millis(10);

        let mut observation = Observations::default();

        for _ in 0..10 {
            observation.record(EdgeWeightType::Immediate(Ok(small_latency)));
        }

        assert_eq!(
            observation
                .immediate_qos()
                .ok_or_else(|| anyhow::anyhow!("should contain a value"))?
                .average_latency()
                .context("should contain a value")?,
            small_latency
        );

        observation.record(EdgeWeightType::Immediate(Ok(big_latency)));

        assert_gt!(
            observation
                .immediate_qos()
                .ok_or_else(|| anyhow::anyhow!("should contain a value"))?
                .average_latency()
                .context("should contain a value")?,
            small_latency
        );
        assert_lt!(
            observation
                .immediate_qos()
                .ok_or_else(|| anyhow::anyhow!("should contain a value"))?
                .average_latency()
                .context("should contain a value")?,
            big_latency
        );

        Ok(())
    }

    #[test]
    fn observations_should_store_the_averaged_success_rate_of_the_probes() {
        let small_latency = std::time::Duration::from_millis(10);

        let mut observation = Observations::default();

        for i in 0..10 {
            if i % 2 == 0 {
                observation.record(EdgeWeightType::Immediate(Err(())));
            } else {
                observation.record(EdgeWeightType::Immediate(Ok(small_latency)));
            }
        }

        assert_in_delta!(observation.score(), 0.5, 0.05);
    }
}
