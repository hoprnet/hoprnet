/// Network health represented with colors, where green is the best and red
/// is the worst possible observed network quality.
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

pub enum Measurement {
    Probe(std::result::Result<std::time::Duration, ()>),
}

pub trait Observable {
    /// Record a new result of the probe towards the measured peer.
    fn record_probe(&mut self, latency: std::result::Result<std::time::Duration, ()>);

    /// The timestamp of the last update.
    fn last_update(&self) -> std::time::Duration;

    /// Return average latency observed for the measured peer.
    fn average_latency(&self) -> Option<std::time::Duration>;

    /// A value representing the average success rate of probes.
    ///
    /// It is from the range [0.0, 1.0]. The higher the value, the better the score.
    fn average_probe_rate(&self) -> f64;

    /// A value scoring the observed peer.
    ///
    /// It is from the range [0.0, 1.0]. The higher the value, the better the score.
    fn score(&self) -> f64;
}

#[cfg(test)]
mod tests {
    use super::Health;

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
}
