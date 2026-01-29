use std::{sync::Arc, time::Duration};

use futures::channel::mpsc;
use hopr_crypto_random::Randomizable;
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_protocol_session::{FrameInspector, SocketComponents, Stateless};
use hopr_transport_session::{SessionId, SessionStats, StatsState};

#[test]
fn stats_state_run_accepts_inspector() {
    const MTU: usize = 1000;

    let id = SessionId::new(1_u64, HoprPseudonym::random());
    let metrics = Arc::new(SessionStats::new(id, None, 1500, Duration::from_millis(800), 1024));
    let inner = Stateless::<MTU>::new(id.to_string());
    let mut state = StatsState::new(inner, metrics.clone());
    let (ctl_tx, _ctl_rx) = mpsc::channel(1);
    let inspector = FrameInspector::new(1024);
    let components = SocketComponents {
        inspector: Some(inspector),
        ctl_tx,
    };

    state.run(components).expect("expected inspector to be accepted");

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.frame_buffer.incomplete_frames, 0);
}
