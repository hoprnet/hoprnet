//! Jemalloc statistics logging - single module implementation.

use std::time::{Duration, Instant};

use tikv_jemalloc_ctl::{arenas, epoch, opt, stats, version};
use tokio::time;
use tracing::{info, warn};

/// Jemalloc statistics collector with automatic lifecycle management
pub struct JemallocStats {
    _handle: Option<tokio::task::JoinHandle<()>>,
}

impl JemallocStats {
    /// Initialize and start jemalloc statistics logging
    pub async fn start() -> Self {
        // Log startup configuration immediately
        log_startup_config();

        // Log stats once on startup
        log_periodic_stats(Instant::now());

        // Start periodic statistics collection
        let handle = tokio::spawn(async {
            let mut interval = time::interval(Duration::from_secs(60));
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
            let start_time = Instant::now();

            loop {
                interval.tick().await;
                log_periodic_stats(start_time);
            }
        });

        Self { _handle: Some(handle) }
    }
}

impl Drop for JemallocStats {
    fn drop(&mut self) {
        if let Some(handle) = self._handle.take() {
            handle.abort();
        }
    }
}

/// Log jemalloc configuration at startup
fn log_startup_config() {
    match read_config() {
        Ok(config_str) => info!("{}", config_str),
        Err(_) => info!("jemalloc_config: status=error reason=\"failed to read configuration\""),
    }
}

fn read_config() -> Result<String, String> {
    // Trigger epoch update to get current stats
    epoch::advance().map_err(|e| format!("epoch advance failed: {}", e))?;

    let version_str = version::read().map_err(|e| format!("version read failed: {}", e))?;
    let narenas = opt::narenas::read().unwrap_or(0);
    let tcache_max = opt::tcache_max::read().unwrap_or(0);
    let background_thread = opt::background_thread::read().unwrap_or(false);

    Ok(format!(
        "jemalloc_config: version=\"{}\" narenas={} tcache_max={} background_thread={}",
        version_str, narenas, tcache_max, background_thread
    ))
}

/// Log periodic jemalloc statistics
fn log_periodic_stats(start_time: Instant) {
    match read_stats(start_time) {
        Ok(stats_str) => info!("{}", stats_str),
        Err(e) => warn!("Failed to collect jemalloc statistics: {}", e),
    }
}

fn read_stats(start_time: Instant) -> Result<String, String> {
    // Trigger epoch update to get current stats
    epoch::advance().map_err(|e| format!("epoch advance failed: {}", e))?;

    let allocated = stats::allocated::read().map_err(|e| format!("allocated read failed: {}", e))?;
    let active = stats::active::read().map_err(|e| format!("active read failed: {}", e))?;
    let mapped = stats::mapped::read().map_err(|e| format!("mapped read failed: {}", e))?;
    let retained = stats::retained::read().map_err(|e| format!("retained read failed: {}", e))?;
    let narenas = arenas::narenas::read().map_err(|e| format!("narenas read failed: {}", e))?;
    let uptime = start_time.elapsed().as_secs();

    // Calculate thread cache efficiency (allocated/active ratio)
    let cache_efficiency = if active > 0 {
        allocated as f64 / active as f64
    } else {
        0.0
    };

    Ok(format!(
        "jemalloc_stats: allocated={} active={} mapped={} retained={} arenas_active={} cache_efficiency={:.3} \
         uptime_secs={}",
        humansize::SizeFormatter::new(allocated, humansize::DECIMAL),
        humansize::SizeFormatter::new(active, humansize::DECIMAL),
        humansize::SizeFormatter::new(mapped, humansize::DECIMAL),
        humansize::SizeFormatter::new(retained, humansize::DECIMAL),
        narenas,
        cache_efficiency,
        uptime
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn test_stats_lifecycle() {
        let _stats = JemallocStats::start().await;
        // Collector automatically stops when dropped
    }
}
