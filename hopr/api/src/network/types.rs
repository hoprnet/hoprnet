/// Network health represented with colors, where green is the best and red
/// is the worst possible observed nework quality.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, strum::Display, strum::EnumString)]
pub enum Health {
    /// Unknown health, on application startup
    Unknown = 0,
    /// No connection, default
    Red = 1,
    /// Low-quality connection to at least 1 public relay
    Orange = 2,
    /// High-quality connection to at least 1 public relay
    Yellow = 3,
    /// High-quality connection to at least 1 public relay and 1 NAT node
    Green = 4,
}

use super::utils::ExponentialMovingAverage;

/// Observations related to a specific peer in the network.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Observations {
    pub msg_sent: u64,
    pub ack_received: u64,
    pub last_update: std::time::Duration,
    latency_average: ExponentialMovingAverage<3>,
    probe_success_rate: ExponentialMovingAverage<5>,
}

impl Observations {
    /// Record a new result of the probe towards the measured peer.
    pub fn record_probe(&mut self, latency: std::result::Result<std::time::Duration, ()>) {
        self.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        if let Ok(latency) = latency {
            self.latency_average.update(latency.as_millis() as f64);
            self.probe_success_rate.update(1.0);
        } else {
            self.probe_success_rate.update(0.0);
        }
    }

    /// Return average latency observed for the measured peer.
    pub fn average_latency(&self) -> Option<std::time::Duration> {
        if self.latency_average.get() <= 0.0 {
            None
        } else {
            Some(std::time::Duration::from_millis(self.latency_average.get() as u64))
        }
    }

    /// A value between 0.0 and 1.0 scoring the observed peer.
    ///
    /// The higher the value, the better the score.
    pub fn score(&self) -> f64 {
        self.probe_success_rate.get()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use assertables::{assert_gt, assert_in_delta, assert_lt};

    use super::Health;
    use crate::network::Observations;
    #[test]
    fn network_health_should_be_ordered_numerically_for_hopr_metrics_output() {
        assert_eq!(Health::Unknown as i32, 0);
        assert_eq!(Health::Red as i32, 1);
        assert_eq!(Health::Orange as i32, 2);
        assert_eq!(Health::Yellow as i32, 3);
        assert_eq!(Health::Green as i32, 4);
    }

    #[test]
    fn network_health_should_serialize_to_a_proper_string() {
        assert_eq!(format!("{}", Health::Orange), "Orange".to_owned())
    }

    #[test]
    fn network_health_should_deserialize_from_proper_string() -> anyhow::Result<()> {
        let parsed: Health = "Orange".parse()?;
        assert_eq!(parsed, Health::Orange);

        Ok(())
    }

    #[test]
    fn observations_should_update_the_timestamp_on_latency_update() {
        let mut observation = Observations::default();

        assert_eq!(observation.last_update, std::time::Duration::default());

        observation.record_probe(Ok(std::time::Duration::from_millis(50)));

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
            observation.record_probe(Ok(small_latency));
        }

        assert_eq!(
            observation.average_latency().context("should contain a value")?,
            small_latency
        );

        observation.record_probe(Ok(big_latency));

        assert_gt!(
            observation.average_latency().context("should contain a value")?,
            small_latency
        );
        assert_lt!(
            observation.average_latency().context("should contain a value")?,
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
                observation.record_probe(Err(()));
            } else {
                observation.record_probe(Ok(small_latency));
            }
        }

        assert_in_delta!(observation.score(), 0.5, 0.05);
    }
}
