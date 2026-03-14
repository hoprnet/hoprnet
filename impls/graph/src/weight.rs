use hopr_api::graph::{
    EdgeImmediateProtocolObservable, EdgeLinkObservable,
    traits::{
        EdgeNetworkObservableRead, EdgeObservableRead, EdgeObservableWrite, EdgeProtocolObservable,
        EdgeTransportMeasurement, EdgeWeightType,
    },
};
use hopr_statistics::ExponentialMovingAverage;

/// A representation of a individual neighbor link measurement
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct TransportLinkMeasurement {
    latency_average: ExponentialMovingAverage<3>,
    probe_success_rate: ExponentialMovingAverage<5>,
}

impl EdgeLinkObservable for TransportLinkMeasurement {
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
        self.average_probe_rate() * latency_score(self.average_latency())
    }
}

/// Aid in calculation of the overall transport link score.
///
/// The smaller the latency over the channel, the more useful the link might
/// be for routing complext traffic.
fn latency_score(latency: Option<std::time::Duration>) -> f64 {
    if let Some(latency) = latency {
        match latency.as_millis() {
            0..=75 => 1.0,
            76..=125 => 0.7,
            126..=200 => 0.3,
            _ => 0.15,
        }
    } else {
        0.05
    }
}

/// Observations related to a specific peer in the network.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Observations {
    last_update: std::time::Duration,
    immediate_probe: Option<TransportImmediates>,
    intermediate_probe: Option<TransportIntermediates>,
}

impl EdgeObservableWrite for Observations {
    #[tracing::instrument(level = "trace", skip(self), name = "record_observation")]
    fn record(&mut self, measurement: EdgeWeightType) {
        self.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        match measurement {
            EdgeWeightType::Immediate(result) => self.immediate_probe.get_or_insert_default().record(result),
            EdgeWeightType::Intermediate(result) => self.intermediate_probe.get_or_insert_default().record(result),
            EdgeWeightType::Capacity(capacity) => self.intermediate_probe.get_or_insert_default().capacity = capacity,
            EdgeWeightType::Connected(is_connected) => {
                self.immediate_probe.get_or_insert_default().is_connected = is_connected
            }
            EdgeWeightType::ImmediateProtocolConformance { num_packets, num_acks } => {
                let imm = self.immediate_probe.get_or_insert_default();
                imm.messages_sent += num_packets;
                imm.acks_received += num_acks;
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct TransportImmediates {
    link: TransportLinkMeasurement,
    is_connected: bool,
    messages_sent: u64,
    acks_received: u64,
}

impl EdgeNetworkObservableRead for TransportImmediates {
    fn is_connected(&self) -> bool {
        self.is_connected
    }
}

impl EdgeImmediateProtocolObservable for TransportImmediates {
    fn ack_rate(&self) -> Option<f64> {
        if self.messages_sent == 0 {
            None
        } else {
            Some(self.acks_received as f64 / self.messages_sent as f64)
        }
    }
}

impl EdgeLinkObservable for TransportImmediates {
    fn record(&mut self, measurement: EdgeTransportMeasurement) {
        self.link.record(measurement)
    }

    fn average_latency(&self) -> Option<std::time::Duration> {
        self.link.average_latency()
    }

    fn average_probe_rate(&self) -> f64 {
        self.link.average_probe_rate()
    }

    fn score(&self) -> f64 {
        self.link.score()
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct TransportIntermediates {
    link: TransportLinkMeasurement,
    capacity: Option<u128>,
}

impl EdgeProtocolObservable for TransportIntermediates {
    fn capacity(&self) -> Option<u128> {
        self.capacity
    }
}

impl EdgeLinkObservable for TransportIntermediates {
    fn record(&mut self, measurement: EdgeTransportMeasurement) {
        self.link.record(measurement);
    }

    fn average_latency(&self) -> Option<std::time::Duration> {
        self.link.average_latency()
    }

    fn average_probe_rate(&self) -> f64 {
        self.link.average_probe_rate()
    }

    fn score(&self) -> f64 {
        self.link.score()
    }
}

impl EdgeObservableRead for Observations {
    type ImmediateMeasurement = TransportImmediates;
    type IntermediateMeasurement = TransportIntermediates;

    #[inline]
    fn last_update(&self) -> std::time::Duration {
        self.last_update
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
    ///
    /// TODO: find a better way to do this or completely remove this score function,
    /// as it is not clear how to combine the different observations in a meaningful way.
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
    fn ack_rate_should_be_none_when_no_messages_sent() -> anyhow::Result<()> {
        let mut observation = Observations::default();
        observation.record(EdgeWeightType::Connected(true));

        let imm = observation.immediate_qos().context("should have immediate QoS")?;
        assert_eq!(imm.ack_rate(), None);
        Ok(())
    }

    #[test]
    fn ack_rate_should_be_one_when_all_messages_acked() -> anyhow::Result<()> {
        let mut observation = Observations::default();
        observation.record(EdgeWeightType::ImmediateProtocolConformance {
            num_packets: 10,
            num_acks: 10,
        });

        let imm = observation.immediate_qos().context("should have immediate QoS")?;
        assert_eq!(imm.ack_rate(), Some(1.0));
        Ok(())
    }

    #[test]
    fn ack_rate_should_reflect_partial_acknowledgment() -> anyhow::Result<()> {
        let mut observation = Observations::default();
        observation.record(EdgeWeightType::ImmediateProtocolConformance {
            num_packets: 10,
            num_acks: 7,
        });

        let imm = observation.immediate_qos().context("should have immediate QoS")?;
        let rate = imm.ack_rate().context("should have ack rate")?;
        assert_in_delta!(rate, 0.7, 0.001);
        Ok(())
    }

    #[test]
    fn ack_rate_should_accumulate_across_multiple_records() -> anyhow::Result<()> {
        let mut observation = Observations::default();
        observation.record(EdgeWeightType::ImmediateProtocolConformance {
            num_packets: 5,
            num_acks: 5,
        });
        observation.record(EdgeWeightType::ImmediateProtocolConformance {
            num_packets: 5,
            num_acks: 0,
        });

        let imm = observation.immediate_qos().context("should have immediate QoS")?;
        let rate = imm.ack_rate().context("should have ack rate")?;
        assert_in_delta!(rate, 0.5, 0.001);
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
