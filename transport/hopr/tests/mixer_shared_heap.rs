/// Regression test for the mixer wiring change introduced in this PR.
///
/// # Background
///
/// `start_outgoing_ack_pipeline` clones the wire sink once per destination inside
/// `for_each_concurrent`. Before this fix the wire sink was a `MixerSink`, where every
/// clone owns a *separate* `BinaryHeap`. Items pushed via clone A are delayed only
/// against each other; items from clone B form their own independent delay queue. This
/// means cross-destination packets are never mixed together, silently undermining the
/// mixer's anonymity goal.
///
/// The fix replaces the `MixerSink` with `hopr_transport_mixer::channel`. All
/// `Sender` clones share one `Arc<Mutex<BinaryHeap>>`, so cross-destination packets
/// interleave in the single forwarder's output stream.
///
/// # What this test checks
///
/// Given two `Sender` clones, A and B:
/// - Push N items tagged with 'A' on clone A, then N items tagged with 'B' on clone B.
/// - With per-clone heaps (the old, broken topology): the first N items in the output
///   are all 'A'-tagged (clone A's heap drained first), then all 'B'-tagged.
///   `bs_in_first_half == 0`.
/// - With a shared heap (the new, correct topology): 'A' and 'B' items are interleaved
///   because they compete in the same heap under random delays.
///   `bs_in_first_half >> 0`.
use std::time::Duration;

use anyhow::Context;
use futures::StreamExt;
use hopr_transport_mixer::{MixerConfig, channel};
use tokio::time::timeout;

const PROCESSING_LEEWAY: Duration = Duration::from_millis(500);

/// Items from different `Sender` clones appear interleaved in the receiver output.
#[tokio::test(flavor = "current_thread")]
async fn wire_mixer_interleaves_across_clones() -> anyhow::Result<()> {
    let cfg = MixerConfig::default(); // 10..210ms uniform random delay
    let (tx_a, rx) = channel::<(u8, u32)>(cfg);
    let tx_b = tx_a.clone();

    const N: u32 = 200;
    for i in 0..N {
        tx_a.send((b'A', i)).context("send A")?;
    }
    for i in 0..N {
        tx_b.send((b'B', i)).context("send B")?;
    }
    drop(tx_a);
    drop(tx_b);

    // Collect all 2N items. With the default delay range, each item takes up to
    // ~210ms; N=200 items can all complete well within 2 * 210ms + leeway since
    // they are queued at the same time and the heap is drained concurrently.
    let max_wait = Duration::from_millis(210) * 2 + PROCESSING_LEEWAY;
    let out: Vec<(u8, u32)> = timeout(max_wait, rx.take(2 * N as usize).collect())
        .await
        .context("timed out collecting mixed output")?;

    assert_eq!(out.len(), 2 * N as usize, "not all items received");

    // Count 'B'-tagged items in the first half of the output.
    // With a shared heap these should be thoroughly interleaved; at minimum at
    // least one 'B' must appear before index N (the seam that a per-clone heap
    // would preserve perfectly). We use a conservative threshold of N/10 to
    // avoid a flaky failure on a heavily loaded CI runner.
    let bs_in_first_half = out[..N as usize].iter().filter(|(c, _)| *c == b'B').count();
    assert!(
        bs_in_first_half > N as usize / 10,
        "only {bs_in_first_half}/{N} 'B'-tagged items in the first half — \
         heap is not shared across Sender clones (first 20: {:?})",
        &out[..20.min(out.len())]
    );
    Ok(())
}
