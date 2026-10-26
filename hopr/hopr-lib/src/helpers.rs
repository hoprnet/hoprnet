use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use hopr_api::{
    chain::ChainValues,
    types::primitive::{
        balance::{Balance, Currency},
        prelude::Address,
    },
};

use crate::constants;

/// Polls the on-chain balance of `address` for currency `C` with exponential backoff,
/// returning `Ok` as soon as it reaches `min_balance`, or `Err` once the per-iteration
/// backoff grows past `max_delay`.
pub(crate) async fn wait_for_balance<C: Currency, R: ChainValues>(
    min_balance: Balance<C>,
    max_delay: Duration,
    address: Address,
    resolver: &R,
) -> Result<(), ()> {
    let multiplier = 1.05;
    let mut current_delay = Duration::from_secs(2).min(max_delay);

    while current_delay <= max_delay {
        match resolver.balance::<C, _>(address).await {
            Ok(current_balance) => {
                tracing::info!(%address, balance = %current_balance, "balance status");
                if current_balance.ge(&min_balance) {
                    return Ok(());
                } else {
                    tracing::warn!(%address, "still underfunded, trying again soon");
                }
            }
            Err(error) => tracing::error!(%address, %error, "failed to fetch balance from the chain"),
        }

        hopr_utils::runtime::prelude::sleep(current_delay).await;
        current_delay = current_delay.mul_f64(multiplier);
    }

    Err(())
}

/// Kernel UDP socket buffer defaults and caps, as reported by the OS (Linux only; see
/// <https://github.com/hoprnet/hoprnet/issues/8503>).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct UdpBufferLimits {
    pub rmem_default: Option<u64>,
    pub rmem_max: Option<u64>,
    pub wmem_default: Option<u64>,
    pub wmem_max: Option<u64>,
}

/// Current and available TCP congestion-control algorithms, as reported by the OS (Linux only).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TcpCongestionControl {
    /// Currently active algorithm (`net.ipv4.tcp_congestion_control`), e.g. `"cubic"` or `"bbr"`.
    pub current: Option<String>,
    /// Whether `"bbr"` is listed in `net.ipv4.tcp_available_congestion_control`.
    ///
    /// `None` when the kernel doesn't expose that sysctl at all.
    pub bbr_available: Option<bool>,
}

#[cfg(any(target_os = "linux", test))]
fn parse_sysctl_u64(raw: &str) -> Option<u64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse().ok()
}

#[cfg(any(target_os = "linux", test))]
fn parse_sysctl_string(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(any(target_os = "linux", test))]
fn read_udp_buffer_limits_from(proc_sys_root: &std::path::Path) -> UdpBufferLimits {
    let read_u64 = |relative: &str| {
        std::fs::read_to_string(proc_sys_root.join(relative))
            .ok()
            .and_then(|raw| parse_sysctl_u64(&raw))
    };

    UdpBufferLimits {
        rmem_default: read_u64("net/core/rmem_default"),
        rmem_max: read_u64("net/core/rmem_max"),
        wmem_default: read_u64("net/core/wmem_default"),
        wmem_max: read_u64("net/core/wmem_max"),
    }
}

#[cfg(any(target_os = "linux", test))]
fn read_tcp_congestion_control_from(proc_sys_root: &std::path::Path) -> TcpCongestionControl {
    let current = std::fs::read_to_string(proc_sys_root.join("net/ipv4/tcp_congestion_control"))
        .ok()
        .and_then(|raw| parse_sysctl_string(&raw));

    let bbr_available = std::fs::read_to_string(proc_sys_root.join("net/ipv4/tcp_available_congestion_control"))
        .ok()
        .and_then(|raw| parse_sysctl_string(&raw))
        .map(|available| available.split_whitespace().any(|algo| algo == "bbr"));

    TcpCongestionControl { current, bbr_available }
}

/// Reads kernel UDP socket buffer defaults and caps. Only supported on Linux (reads `/proc/sys`);
/// returns all-`None` elsewhere.
#[cfg(target_os = "linux")]
pub(crate) fn read_udp_buffer_limits() -> UdpBufferLimits {
    read_udp_buffer_limits_from(std::path::Path::new("/proc/sys"))
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn read_udp_buffer_limits() -> UdpBufferLimits {
    UdpBufferLimits {
        rmem_default: None,
        rmem_max: None,
        wmem_default: None,
        wmem_max: None,
    }
}

/// Reads the current and available TCP congestion-control algorithms. Only supported on Linux
/// (reads `/proc/sys`); returns all-`None` elsewhere.
#[cfg(target_os = "linux")]
pub(crate) fn read_tcp_congestion_control() -> TcpCongestionControl {
    read_tcp_congestion_control_from(std::path::Path::new("/proc/sys"))
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn read_tcp_congestion_control() -> TcpCongestionControl {
    TcpCongestionControl {
        current: None,
        bbr_available: None,
    }
}

/// Host OS network tuning state relevant to node throughput (UDP buffer sizing, TCP congestion
/// control), plus the recommendations derived from it. See
/// <https://github.com/hoprnet/hoprnet/issues/8503>.
#[derive(Debug, Clone, Default)]
pub(crate) struct NetworkTuningReport {
    pub udp: UdpBufferLimits,
    pub congestion_control: TcpCongestionControl,
}

impl NetworkTuningReport {
    /// Reads the current host's network tuning state (Linux only; all-`None` elsewhere).
    pub(crate) fn collect() -> Self {
        Self {
            udp: read_udp_buffer_limits(),
            congestion_control: read_tcp_congestion_control(),
        }
    }

    /// Actionable tuning recommendations, one per issue found; empty when nothing needs attention.
    pub(crate) fn recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self
            .udp
            .rmem_default
            .is_some_and(|v| v < constants::RECOMMENDED_MIN_UDP_BUFFER_BYTES)
            || self
                .udp
                .wmem_default
                .is_some_and(|v| v < constants::RECOMMENDED_MIN_UDP_BUFFER_BYTES)
        {
            recommendations.push(format!(
                "kernel UDP socket buffer defaults are below the recommended minimum ({} bytes) and can cause packet \
                 drops under load; raise them with: sysctl -w net.core.rmem_default={0} net.core.wmem_default={0} \
                 (persist in /etc/sysctl.conf)",
                constants::RECOMMENDED_MIN_UDP_BUFFER_BYTES
            ));
        }

        if let Some(current) = self.congestion_control.current.as_deref()
            && current != constants::RECOMMENDED_CONGESTION_CONTROL
            && self.congestion_control.bbr_available == Some(true)
        {
            recommendations.push(format!(
                "TCP congestion control is set to {current}, not {rec}; {rec} generally gives better throughput on \
                 lossy/high-latency links; switch with: sysctl -w net.ipv4.tcp_congestion_control={rec} (persist in \
                 /etc/sysctl.conf)",
                rec = constants::RECOMMENDED_CONGESTION_CONTROL
            ));
        }

        recommendations
    }
}

impl std::fmt::Display for NetworkTuningReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let recommendations = self.recommendations();
        if recommendations.is_empty() {
            write!(f, "OS network tuning looks fine")
        } else {
            write!(f, "{}", recommendations.join("; "))
        }
    }
}

#[cfg(test)]
mod sysctl_tests {
    use anyhow::Context;

    use super::{
        NetworkTuningReport, TcpCongestionControl, UdpBufferLimits, parse_sysctl_string, parse_sysctl_u64,
        read_tcp_congestion_control_from, read_udp_buffer_limits_from,
    };

    #[test]
    fn parse_sysctl_u64_accepts_plain_digits() {
        assert_eq!(parse_sysctl_u64("4194304"), Some(4194304));
    }

    #[test]
    fn parse_sysctl_u64_trims_trailing_newline() {
        assert_eq!(parse_sysctl_u64("212992\n"), Some(212992));
    }

    #[test]
    fn parse_sysctl_u64_trims_surrounding_whitespace() {
        assert_eq!(parse_sysctl_u64("  212992  \n"), Some(212992));
    }

    #[test]
    fn parse_sysctl_u64_rejects_empty_string() {
        assert_eq!(parse_sysctl_u64(""), None);
        assert_eq!(parse_sysctl_u64("\n"), None);
    }

    #[test]
    fn parse_sysctl_u64_rejects_non_numeric_garbage() {
        assert_eq!(parse_sysctl_u64("not-a-number"), None);
    }

    #[test]
    fn parse_sysctl_string_trims_trailing_newline() {
        assert_eq!(parse_sysctl_string("bbr\n"), Some("bbr".to_string()));
    }

    #[test]
    fn parse_sysctl_string_rejects_empty_string() {
        assert_eq!(parse_sysctl_string(""), None);
        assert_eq!(parse_sysctl_string("\n"), None);
    }

    fn write_sysctl_file(root: &std::path::Path, relative: &str, contents: &str) -> anyhow::Result<()> {
        let path = root.join(relative);
        let parent = path.parent().context("sysctl file path has no parent directory")?;
        std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        std::fs::write(&path, contents).with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }

    #[test]
    fn read_udp_buffer_limits_reads_all_present_values() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().context("creating tempdir")?;
        write_sysctl_file(dir.path(), "net/core/rmem_default", "212992\n")?;
        write_sysctl_file(dir.path(), "net/core/rmem_max", "4194304\n")?;
        write_sysctl_file(dir.path(), "net/core/wmem_default", "212992\n")?;
        write_sysctl_file(dir.path(), "net/core/wmem_max", "4194304\n")?;

        let limits = read_udp_buffer_limits_from(dir.path());

        assert_eq!(limits.rmem_default, Some(212992));
        assert_eq!(limits.rmem_max, Some(4194304));
        assert_eq!(limits.wmem_default, Some(212992));
        assert_eq!(limits.wmem_max, Some(4194304));
        Ok(())
    }

    #[test]
    fn read_udp_buffer_limits_leaves_missing_files_as_none() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().context("creating tempdir")?;
        write_sysctl_file(dir.path(), "net/core/rmem_default", "212992\n")?;
        // rmem_max, wmem_default, wmem_max intentionally absent.

        let limits = read_udp_buffer_limits_from(dir.path());

        assert_eq!(limits.rmem_default, Some(212992));
        assert_eq!(limits.rmem_max, None);
        assert_eq!(limits.wmem_default, None);
        assert_eq!(limits.wmem_max, None);
        Ok(())
    }

    #[test]
    fn read_udp_buffer_limits_leaves_garbage_content_as_none() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().context("creating tempdir")?;
        write_sysctl_file(dir.path(), "net/core/rmem_default", "not-a-number\n")?;

        let limits = read_udp_buffer_limits_from(dir.path());

        assert_eq!(limits.rmem_default, None);
        Ok(())
    }

    #[test]
    fn read_tcp_congestion_control_reports_bbr_already_active() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().context("creating tempdir")?;
        write_sysctl_file(dir.path(), "net/ipv4/tcp_congestion_control", "bbr\n")?;
        write_sysctl_file(dir.path(), "net/ipv4/tcp_available_congestion_control", "cubic bbr\n")?;

        let cc = read_tcp_congestion_control_from(dir.path());

        assert_eq!(cc.current.as_deref(), Some("bbr"));
        assert_eq!(cc.bbr_available, Some(true));
        Ok(())
    }

    #[test]
    fn read_tcp_congestion_control_reports_bbr_available_but_unused() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().context("creating tempdir")?;
        write_sysctl_file(dir.path(), "net/ipv4/tcp_congestion_control", "cubic\n")?;
        write_sysctl_file(
            dir.path(),
            "net/ipv4/tcp_available_congestion_control",
            "cubic bbr reno\n",
        )?;

        let cc = read_tcp_congestion_control_from(dir.path());

        assert_eq!(cc.current.as_deref(), Some("cubic"));
        assert_eq!(cc.bbr_available, Some(true));
        Ok(())
    }

    #[test]
    fn read_tcp_congestion_control_reports_bbr_unavailable() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().context("creating tempdir")?;
        write_sysctl_file(dir.path(), "net/ipv4/tcp_congestion_control", "cubic\n")?;
        write_sysctl_file(dir.path(), "net/ipv4/tcp_available_congestion_control", "cubic reno\n")?;

        let cc = read_tcp_congestion_control_from(dir.path());

        assert_eq!(cc.current.as_deref(), Some("cubic"));
        assert_eq!(cc.bbr_available, Some(false));
        Ok(())
    }

    #[test]
    fn read_tcp_congestion_control_leaves_missing_available_list_as_none() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().context("creating tempdir")?;
        write_sysctl_file(dir.path(), "net/ipv4/tcp_congestion_control", "cubic\n")?;
        // tcp_available_congestion_control intentionally absent.

        let cc = read_tcp_congestion_control_from(dir.path());

        assert_eq!(cc.current.as_deref(), Some("cubic"));
        assert_eq!(cc.bbr_available, None);
        Ok(())
    }

    #[test]
    fn recommendations_empty_when_tuned_well() {
        let report = NetworkTuningReport {
            udp: UdpBufferLimits {
                rmem_default: Some(4 * 1024 * 1024),
                rmem_max: Some(4 * 1024 * 1024),
                wmem_default: Some(4 * 1024 * 1024),
                wmem_max: Some(4 * 1024 * 1024),
            },
            congestion_control: TcpCongestionControl {
                current: Some("bbr".to_string()),
                bbr_available: Some(true),
            },
        };

        assert!(report.recommendations().is_empty());
        assert_eq!(report.to_string(), "OS network tuning looks fine");
    }

    #[test]
    fn recommendations_flag_low_udp_buffer_default() {
        let report = NetworkTuningReport {
            udp: UdpBufferLimits {
                rmem_default: Some(212992),
                wmem_default: Some(212992),
                ..Default::default()
            },
            congestion_control: TcpCongestionControl::default(),
        };

        let recommendations = report.recommendations();
        assert_eq!(recommendations.len(), 1);
        assert!(recommendations[0].contains("sysctl -w net.core.rmem_default=4194304 net.core.wmem_default=4194304"));
    }

    #[test]
    fn recommendations_ignore_low_max_when_default_is_fine() {
        let report = NetworkTuningReport {
            udp: UdpBufferLimits {
                rmem_default: Some(4 * 1024 * 1024),
                rmem_max: Some(212992),
                wmem_default: Some(4 * 1024 * 1024),
                wmem_max: Some(212992),
            },
            congestion_control: TcpCongestionControl::default(),
        };

        assert!(report.recommendations().is_empty());
    }

    #[test]
    fn recommendations_flag_bbr_available_but_unused() {
        let report = NetworkTuningReport {
            udp: UdpBufferLimits::default(),
            congestion_control: TcpCongestionControl {
                current: Some("cubic".to_string()),
                bbr_available: Some(true),
            },
        };

        let recommendations = report.recommendations();
        assert_eq!(recommendations.len(), 1);
        assert!(recommendations[0].contains("sysctl -w net.ipv4.tcp_congestion_control=bbr"));
    }

    #[test]
    fn recommendations_skip_bbr_suggestion_when_unavailable() {
        let report = NetworkTuningReport {
            udp: UdpBufferLimits::default(),
            congestion_control: TcpCongestionControl {
                current: Some("cubic".to_string()),
                bbr_available: Some(false),
            },
        };

        assert!(report.recommendations().is_empty());
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };

    use hopr_api::{
        chain::{ChainInfo, ChainValues, DomainSeparators, RedemptionStats, WinningProbability},
        types::primitive::{
            balance::{Balance, Currency},
            prelude::{Address, HoprBalance},
        },
    };

    use super::wait_for_balance;

    #[derive(Debug, thiserror::Error)]
    #[error("mock chain error")]
    struct MockError;

    /// Mock [`ChainValues`] resolver that scripts a sequence of `balance()` outcomes.
    ///
    /// The first `fail_first` calls return an error (simulating a flaky RPC), after which
    /// the reported balance is `initial_base` until `bump_after` successful reads have
    /// happened, at which point it jumps to `funded_base` (simulating the safe being topped up).
    #[derive(Default)]
    struct MockChain {
        calls: AtomicUsize,
        fail_first: usize,
        bump_after: usize,
        initial_base: u64,
        funded_base: u64,
    }

    impl MockChain {
        /// Number of times `balance()` was invoked.
        fn call_count(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl ChainValues for MockChain {
        type Error = MockError;

        async fn balance<C: Currency, A: Into<Address> + Send>(&self, _address: A) -> Result<Balance<C>, Self::Error> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            if n < self.fail_first {
                return Err(MockError);
            }
            let base = if n >= self.bump_after {
                self.funded_base
            } else {
                self.initial_base
            };
            Ok(Balance::<C>::new_base(base))
        }

        async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn key_binding_fee(&self) -> Result<HoprBalance, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn chain_info(&self) -> Result<ChainInfo, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn redemption_stats<A: Into<Address> + Send>(
            &self,
            _safe_addr: A,
        ) -> Result<RedemptionStats, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn typical_resolution_time(&self) -> Result<Duration, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }
    }

    #[tokio::test(start_paused = true)]
    async fn returns_immediately_when_safe_is_already_funded() {
        let chain = MockChain {
            initial_base: 100,
            funded_base: 100,
            ..Default::default()
        };

        let result = wait_for_balance(
            HoprBalance::new_base(50),
            Duration::from_secs(200),
            Address::default(),
            &chain,
        )
        .await;

        assert!(result.is_ok());
        // A single read is enough since the safe already holds the required balance.
        assert_eq!(chain.call_count(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn errors_when_safe_stays_underfunded_until_deadline() {
        let chain = MockChain {
            initial_base: 10,
            funded_base: 10,
            ..Default::default()
        };

        let result = wait_for_balance(
            HoprBalance::new_base(50),
            Duration::from_secs(200),
            Address::default(),
            &chain,
        )
        .await;

        assert_eq!(result, Err(()), "expected the wait to time out, got {result:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn succeeds_after_the_safe_gets_topped_up() {
        // Underfunded for the first two reads, then funded.
        let chain = MockChain {
            initial_base: 10,
            funded_base: 100,
            bump_after: 2,
            ..Default::default()
        };

        let result = wait_for_balance(
            HoprBalance::new_base(50),
            Duration::from_secs(200),
            Address::default(),
            &chain,
        )
        .await;

        assert!(result.is_ok());
        // Two underfunded reads plus the successful third one.
        assert_eq!(chain.call_count(), chain.bump_after + 1);
    }

    #[tokio::test(start_paused = true)]
    async fn tolerates_transient_fetch_errors_then_succeeds() {
        // First read errors out, the safe is funded from the second read onwards.
        let chain = MockChain {
            initial_base: 100,
            funded_base: 100,
            fail_first: 1,
            ..Default::default()
        };

        let result = wait_for_balance(
            HoprBalance::new_base(50),
            Duration::from_secs(200),
            Address::default(),
            &chain,
        )
        .await;

        assert!(result.is_ok());
        // One failed read followed by a successful one.
        assert_eq!(chain.call_count(), chain.fail_first + 1);
    }
}

#[derive(Clone)]
pub struct BroadcastSenderSink<T>(pub async_broadcast::Sender<T>);

impl<T: Clone> futures::Sink<T> for BroadcastSenderSink<T> {
    type Error = async_broadcast::TrySendError<T>;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.0.try_broadcast(item).map(|_| ())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.close();
        Poll::Ready(Ok(()))
    }
}
