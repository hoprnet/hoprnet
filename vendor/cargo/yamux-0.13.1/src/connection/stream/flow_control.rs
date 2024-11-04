use std::{cmp, sync::Arc};

use instant::Instant;
use parking_lot::Mutex;

use crate::{connection::rtt::Rtt, Config, DEFAULT_CREDIT};

#[derive(Debug)]
pub(crate) struct FlowController {
    config: Arc<Config>,
    last_window_update: Instant,
    /// See [`Connection::rtt`].
    rtt: Rtt,
    /// See [`Connection::accumulated_max_stream_windows`].
    accumulated_max_stream_windows: Arc<Mutex<usize>>,
    receive_window: u32,
    max_receive_window: u32,
    send_window: u32,
}

impl FlowController {
    pub(crate) fn new(
        receive_window: u32,
        send_window: u32,
        accumulated_max_stream_windows: Arc<Mutex<usize>>,
        rtt: Rtt,
        config: Arc<Config>,
    ) -> Self {
        Self {
            receive_window,
            send_window,
            config,
            rtt,
            accumulated_max_stream_windows,
            max_receive_window: DEFAULT_CREDIT,
            last_window_update: Instant::now(),
        }
    }

    /// Calculate the number of additional window bytes the receiving side (local) should grant the
    /// sending side (remote) via a window update message.
    ///
    /// Returns `None` if too small to justify a window update message.
    pub(crate) fn next_window_update(&mut self, buffer_len: usize) -> Option<u32> {
        self.assert_invariants(buffer_len);

        let bytes_received = self.max_receive_window - self.receive_window;
        let mut next_window_update =
            bytes_received.saturating_sub(buffer_len.try_into().unwrap_or(u32::MAX));

        // Don't send an update in case half or more of the window is still available to the sender.
        if next_window_update < self.max_receive_window / 2 {
            return None;
        }

        log::trace!(
            "received {} mb in {} seconds ({} mbit/s)",
            next_window_update as f64 / crate::MIB as f64,
            self.last_window_update.elapsed().as_secs_f64(),
            next_window_update as f64 / crate::MIB as f64 * 8.0
                / self.last_window_update.elapsed().as_secs_f64()
        );

        // Auto-tuning `max_receive_window`
        //
        // The ideal `max_receive_window` is equal to the bandwidth-delay-product (BDP), thus
        // allowing the remote sender to exhaust the entire available bandwidth on a single stream.
        // Choosing `max_receive_window` too small prevents the remote sender from exhausting the
        // available bandwidth. Choosing `max_receive_window` to large is wasteful and delays
        // backpressure from the receiver to the sender on the stream.
        //
        // In case the remote sender has exhausted half or more of its credit in less than 2
        // round-trips, try to double `max_receive_window`.
        //
        // For simplicity `max_receive_window` is never decreased.
        //
        // This implementation is heavily influenced by QUIC. See document below for rational on the
        // above strategy.
        //
        // https://docs.google.com/document/d/1F2YfdDXKpy20WVKJueEf4abn_LVZHhMUMS5gX6Pgjl4/edit?usp=sharing
        if self
            .rtt
            .get()
            .map(|rtt| self.last_window_update.elapsed() < rtt * 2)
            .unwrap_or(false)
        {
            let mut accumulated_max_stream_windows = self.accumulated_max_stream_windows.lock();

            // Ideally one can just double it:
            let new_max = self.max_receive_window.saturating_mul(2);

            // But one has to consider the configured connection limit:
            let new_max = {
                let connection_limit: usize = self.max_receive_window as usize +
                    // the overall configured conneciton limit
                    (self.config.max_connection_receive_window.unwrap_or(usize::MAX)
                    // minus the minimum amount of window guaranteed to each stream
                    - self.config.max_num_streams * DEFAULT_CREDIT as usize
                    // minus the amount of bytes beyond the minimum amount (`DEFAULT_CREDIT`)
                    // already allocated by this and other streams on the connection.
                    - *accumulated_max_stream_windows);

                cmp::min(new_max, connection_limit.try_into().unwrap_or(u32::MAX))
            };

            // Account for the additional credit on the accumulated connection counter.
            *accumulated_max_stream_windows += (new_max - self.max_receive_window) as usize;
            drop(accumulated_max_stream_windows);

            log::debug!(
                "old window_max: {} mb, new window_max: {} mb",
                self.max_receive_window as f64 / crate::MIB as f64,
                new_max as f64 / crate::MIB as f64
            );

            self.max_receive_window = new_max;

            // Recalculate `next_window_update` with the new `max_receive_window`.
            let bytes_received = self.max_receive_window - self.receive_window;
            next_window_update =
                bytes_received.saturating_sub(buffer_len.try_into().unwrap_or(u32::MAX));
        }

        self.last_window_update = Instant::now();
        self.receive_window += next_window_update;

        self.assert_invariants(buffer_len);

        return Some(next_window_update);
    }

    fn assert_invariants(&self, buffer_len: usize) {
        if !cfg!(debug_assertions) {
            return;
        }

        let config = &self.config;
        let rtt = self.rtt.get();
        let accumulated_max_stream_windows = *self.accumulated_max_stream_windows.lock();

        assert!(
            buffer_len <= self.max_receive_window as usize,
            "The current buffer size never exceeds the maximum stream receive window."
        );
        assert!(
            self.receive_window <= self.max_receive_window,
            "The current window never exceeds the maximum."
        );
        assert!(
            (self.max_receive_window - DEFAULT_CREDIT) as usize
                <= config.max_connection_receive_window.unwrap_or(usize::MAX)
                    - config.max_num_streams * DEFAULT_CREDIT as usize,
            "The maximum never exceeds its maximum portion of the configured connection limit."
        );
        assert!(
            (self.max_receive_window - DEFAULT_CREDIT) as usize
                <= accumulated_max_stream_windows,
            "The amount by which the stream maximum exceeds DEFAULT_CREDIT is tracked in accumulated_max_stream_windows."
        );
        if rtt.is_none() {
            assert_eq!(
                self.max_receive_window, DEFAULT_CREDIT,
                "The maximum is only increased iff an rtt measurement is available."
            );
        }
    }

    pub(crate) fn send_window(&self) -> u32 {
        self.send_window
    }

    pub(crate) fn consume_send_window(&mut self, i: u32) {
        self.send_window = self
            .send_window
            .checked_sub(i)
            .expect("not exceed send window");
    }

    pub(crate) fn increase_send_window_by(&mut self, i: u32) {
        self.send_window = self
            .send_window
            .checked_add(i)
            .expect("send window not to exceed u32");
    }

    pub(crate) fn receive_window(&self) -> u32 {
        self.receive_window
    }

    pub(crate) fn consume_receive_window(&mut self, i: u32) {
        self.receive_window = self
            .receive_window
            .checked_sub(i)
            .expect("not exceed receive window");
    }
}

impl Drop for FlowController {
    fn drop(&mut self) {
        let mut accumulated_max_stream_windows = self.accumulated_max_stream_windows.lock();

        debug_assert!(
            *accumulated_max_stream_windows >= (self.max_receive_window - DEFAULT_CREDIT) as usize,
            "{accumulated_max_stream_windows} {}",
            self.max_receive_window
        );

        *accumulated_max_stream_windows -= (self.max_receive_window - DEFAULT_CREDIT) as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use instant::Duration;
    use quickcheck::{GenRange, QuickCheck};

    #[derive(Debug)]
    struct Input {
        controller: FlowController,
        buffer_len: usize,
    }

    #[cfg(test)]
    impl Clone for Input {
        fn clone(&self) -> Self {
            Self {
                controller: FlowController {
                    config: self.controller.config.clone(),
                    accumulated_max_stream_windows: Arc::new(Mutex::new(
                        self.controller
                            .accumulated_max_stream_windows
                            .lock()
                            .clone(),
                    )),
                    rtt: self.controller.rtt.clone(),
                    last_window_update: self.controller.last_window_update.clone(),
                    receive_window: self.controller.receive_window,
                    max_receive_window: self.controller.max_receive_window,
                    send_window: self.controller.send_window,
                },
                buffer_len: self.buffer_len,
            }
        }
    }

    impl quickcheck::Arbitrary for Input {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let config = Arc::new(Config::arbitrary(g));
            let rtt = Rtt::arbitrary(g);

            let max_connection_minus_default =
                config.max_connection_receive_window.unwrap_or(usize::MAX)
                    - (config.max_num_streams * (DEFAULT_CREDIT as usize));

            let max_receive_window = if rtt.get().is_none() {
                DEFAULT_CREDIT
            } else {
                g.gen_range(
                    DEFAULT_CREDIT
                        ..(DEFAULT_CREDIT as usize)
                            .saturating_add(max_connection_minus_default)
                            .try_into()
                            .unwrap_or(u32::MAX)
                            .saturating_add(1),
                )
            };
            let receive_window = g.gen_range(0..max_receive_window);
            let buffer_len = g.gen_range(0..max_receive_window as usize);
            let accumulated_max_stream_windows = Arc::new(Mutex::new(g.gen_range(
                (max_receive_window - DEFAULT_CREDIT) as usize
                    ..max_connection_minus_default.saturating_add(1),
            )));
            let last_window_update =
                Instant::now() - Duration::from_secs(g.gen_range(0..(60 * 60 * 24)));
            let send_window = g.gen_range(0..u32::MAX);

            Self {
                controller: FlowController {
                    accumulated_max_stream_windows,
                    rtt,
                    last_window_update,
                    config,
                    receive_window,
                    max_receive_window,
                    send_window,
                },
                buffer_len,
            }
        }
    }

    #[test]
    fn next_window_update() {
        fn property(
            Input {
                mut controller,
                buffer_len,
            }: Input,
        ) {
            controller.next_window_update(buffer_len);
        }

        QuickCheck::new().quickcheck(property as fn(_))
    }
}
