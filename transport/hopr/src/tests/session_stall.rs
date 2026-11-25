//! Reproduces hoprnet#8753 at the Session layer: one Session whose return packets keep finding no
//! SURB must not delay another Session's packet.
//!
//! The Exit stall of the issue builds itself here: two Sessions on the reliable socket write into
//! one resolve stage, which serves one of them a SURB and the other none. That is what a Session
//! owing more return packets than it holds looks like to the resolver, and it is enough to withhold
//! every other Session's traffic when the wait happens inside the ordered stage.

use std::time::{Duration, Instant};

use futures::{AsyncWriteExt, StreamExt};
use futures_time::future::FutureExt as _;
use hopr_api::types::{
    crypto::{crypto_traits::Randomizable, prelude::*},
    internal::{
        path::ValidatedPath,
        prelude::HoprPseudonym,
        routing::{DestinationRouting, ResolvedTransportRouting, SurbMatcher},
    },
};
use hopr_crypto_packet::prelude::HoprSurb;
use hopr_protocol_app::prelude::ApplicationDataOut;
use hopr_transport_session::{Capability, HoprSession, HoprSessionConfig};
use hopr_utils::network_types::crossfire_sink::bounded_sink_channel;

use crate::{
    path::{PathPlannerError, resolve::resolve_routing_stage_with_context},
    trace_context::{ContextSender, WithContext},
};

/// The pseudonym a resolved routing belongs to, which identifies the Session that produced it.
fn pseudonym_of(resolved: &ResolvedTransportRouting<HoprSurb>) -> HoprPseudonym {
    match resolved {
        ResolvedTransportRouting::Forward { pseudonym, .. } => *pseudonym,
        ResolvedTransportRouting::Return(sender_id, _) => sender_id.pseudonym(),
    }
}

/// The wait a starved packet is entitled to. Far above the bound asserted below, so "the bound
/// fired" and "the wait elapsed" cannot be mistaken for one another.
const SURB_WAIT: Duration = Duration::from_millis(500);

/// The bound the served Session's packet is held to: a fraction of the wait it must never serve.
const SERVED_BOUND: Duration = Duration::from_millis(150);

/// A Session that finds no SURB for its return packets must not withhold another Session's packet.
///
/// The Session that starves is written to first, so its packet is ahead in the stage, exactly as a
/// client that keeps sending produces it.
#[test_log::test(tokio::test)]
async fn a_session_without_surbs_should_not_stall_another_sessions_packets() -> anyhow::Result<()> {
    let (starved, served) = (HoprPseudonym::random(), HoprPseudonym::random());

    let (egress, unresolved) = bounded_sink_channel::<WithContext<(DestinationRouting, ApplicationDataOut)>>(8);
    let egress = ContextSender::new(egress);

    let capabilities = Capability::Segmentation | Capability::RetransmissionAck;
    let mut starved_session = HoprSession::new(
        starved,
        DestinationRouting::Return(SurbMatcher::Pseudonym(starved)),
        HoprSessionConfig {
            capabilities,
            ..Default::default()
        },
        (egress.clone(), futures::stream::pending()),
        None,
    )?;
    let mut served_session = HoprSession::new(
        served,
        DestinationRouting::Return(SurbMatcher::Pseudonym(served)),
        HoprSessionConfig {
            capabilities,
            ..Default::default()
        },
        (egress, futures::stream::pending()),
        None,
    )?;

    // The Exit's SURB store: nothing for one Session, a SURB for the other.
    let resolve = move |_size_hint: usize, _max_surbs: usize, routing: DestinationRouting| async move {
        match routing {
            DestinationRouting::Return(matcher) if matcher.pseudonym() == starved => {
                Err(PathPlannerError::Surb("no surb for its pseudonym".into()))
            }
            _ => Ok((
                ResolvedTransportRouting::Forward {
                    pseudonym: served,
                    forward_path: ValidatedPath::direct(
                        *OffchainKeypair::random().public(),
                        ChainKeypair::random().public().to_address(),
                    ),
                    return_paths: vec![],
                },
                Some(1),
                None,
            )),
        }
    };

    starved_session.write_all(b"starved").await?;
    starved_session.flush().await?;
    served_session.write_all(b"served").await?;
    served_session.flush().await?;

    let dropped_before = hopr_transport_session::counters::routing_resolution_surb_timeout_count();
    let started = Instant::now();
    let mut emitted = Box::pin(resolve_routing_stage_with_context(unresolved, resolve, 2, 8, SURB_WAIT));

    let first = emitted
        .next()
        .timeout(futures_time::time::Duration::from(SERVED_BOUND))
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "the Session that found a SURB was withheld for more than {SERVED_BOUND:?} of the {SURB_WAIT:?} its \
                 neighbour is entitled to: this is the head-of-line blocking of hoprnet#8753"
            )
        })?;
    let emission = first.ok_or_else(|| anyhow::anyhow!("the stage emitted nothing"))?;

    assert!(
        matches!(
            emission.value.0,
            ResolvedTransportRouting::Forward { pseudonym, .. } if pseudonym == served
        ),
        "the packet that goes first is the one whose Session found a SURB"
    );
    assert!(
        started.elapsed() < SURB_WAIT,
        "the served Session's packet waited {:?}, which is the starved Session's wait and not its own",
        started.elapsed()
    );

    // The starved Session's packet goes at its deadline, counted and never emitted. What the served
    // Session keeps sending in the meantime (its own resends, since nothing acks them here) passes
    // through, which is the point.
    let mut emitted_pseudonyms = vec![served];
    let give_up = Instant::now() + 2 * SURB_WAIT;
    while hopr_transport_session::counters::routing_resolution_surb_timeout_count() == dropped_before {
        let remaining = give_up.saturating_duration_since(Instant::now());
        match emitted
            .next()
            .timeout(futures_time::time::Duration::from(remaining))
            .await
        {
            Ok(Some(item)) => emitted_pseudonyms.push(pseudonym_of(&item.value.0)),
            _ => break,
        }
    }

    assert!(
        !emitted_pseudonyms.contains(&starved),
        "the packet whose Session holds no SURB must not be emitted"
    );
    assert!(
        hopr_transport_session::counters::routing_resolution_surb_timeout_count() > dropped_before,
        "the starved packet was dropped without being counted, so the drop stays invisible"
    );

    Ok(())
}
