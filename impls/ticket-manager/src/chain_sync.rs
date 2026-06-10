use std::sync::Arc;

use futures::StreamExt;
use hopr_api::chain::{ChannelSelector, ChainReadChannelOperations};

use crate::{HoprTicketFactory, HoprTicketManager, RedbStore, RedbTicketQueue, TicketManagerError};

/// Creates a [`HoprTicketFactory`] backed by a temporary [`RedbStore`] and pre-seeds it
/// from the node's current outgoing channels.
///
/// Use this when building an edge (entry/exit) node that issues tickets but does not
/// relay them. For relay nodes that also manage incoming tickets, use
/// [`ticket_manager_from_chain`] instead.
pub async fn ticket_factory_from_chain<C>(
    connector: &C,
) -> Result<Arc<HoprTicketFactory<RedbStore>>, TicketManagerError>
where
    C: ChainReadChannelOperations,
    C::Error: std::fmt::Display,
{
    let backend = RedbStore::new_temp().map_err(TicketManagerError::store)?;
    let factory = Arc::new(HoprTicketFactory::new(backend));

    let me = *connector.me();
    let outgoing: Vec<_> = connector
        .stream_channels(ChannelSelector::default().with_source(me))
        .map_err(|e| TicketManagerError::Other(anyhow::anyhow!("failed to stream outgoing channels: {e}")))?
        .collect()
        .await;

    factory.sync_from_outgoing_channels(&outgoing)?;
    Ok(factory)
}

/// Creates a [`HoprTicketManager`] and its companion [`HoprTicketFactory`], both backed
/// by a temporary [`RedbStore`], and pre-seeds them from the node's current channel state.
///
/// Use this when building a full relay node that both issues and redeems tickets.
/// The factory is pre-seeded from outgoing channels and the manager from incoming channels.
/// For edge nodes that do not redeem tickets, use [`ticket_factory_from_chain`] instead.
pub async fn ticket_manager_from_chain<C>(
    connector: &C,
) -> Result<
    (
        Arc<HoprTicketManager<RedbStore, RedbTicketQueue>>,
        Arc<HoprTicketFactory<RedbStore>>,
    ),
    TicketManagerError,
>
where
    C: ChainReadChannelOperations,
    C::Error: std::fmt::Display,
{
    let backend = RedbStore::new_temp().map_err(TicketManagerError::store)?;
    let (manager, factory) = HoprTicketManager::new_with_factory(backend);
    let manager = Arc::new(manager);
    let factory = Arc::new(factory);

    let me = *connector.me();
    let incoming_stream = connector
        .stream_channels(ChannelSelector::default().with_destination(me))
        .map_err(|e| TicketManagerError::Other(anyhow::anyhow!("failed to stream incoming channels: {e}")))?;
    let outgoing_stream = connector
        .stream_channels(ChannelSelector::default().with_source(me))
        .map_err(|e| TicketManagerError::Other(anyhow::anyhow!("failed to stream outgoing channels: {e}")))?;

    let (incoming, outgoing): (Vec<_>, Vec<_>) =
        futures::join!(incoming_stream.collect(), outgoing_stream.collect());

    manager.sync_from_incoming_channels(&incoming)?;
    factory.sync_from_outgoing_channels(&outgoing)?;

    Ok((manager, factory))
}
