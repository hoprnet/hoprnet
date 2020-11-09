import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { ChannelData } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { u8aEquals } from '@hoprnet/hopr-utils'
import { pubKeyToPeerId } from '@hoprnet/hopr-core/lib/utils'
import PeerId from 'peer-id'
import { isBootstrapNode } from './isBootstrapNode'

/**
 * Get node's peers.
 * @returns an array of peer ids
 */
export function getPeers(
  node: Hopr<HoprCoreConnector>,
  ops: {
    noBootstrapNodes: boolean
  } = {
    noBootstrapNodes: false
  }
): PeerId[] {
  let peers = node.getConnectedPeers()

  if (ops.noBootstrapNodes) {
    peers = peers.filter((peerId: PeerId) => !isBootstrapNode(node, peerId))
  }

  return peers
}

/**
 * Get node's peer ids in string.
 * @returns an array of peer ids
 */
export function getPeersIdsAsString(
  node: Hopr<HoprCoreConnector>,
  ops: {
    noBootstrapNodes: boolean
  } = {
    noBootstrapNodes: false
  }
): string[] {
  return getPeers(node, ops).map((peerId) => peerId.toB58String())
}

/**
 * Get node's counterParties by looking into the open channels stored in the DB.
 * @returns a promise that resolves to an array of peer ids
 */
export async function getMyOpenChannels(node: Hopr<HoprCoreConnector>): Promise<PeerId[]> {
  return Promise.all(
      (await node.getAllOpenChannels()).map(async (channel: ChannelData) => {
    return await pubKeyToPeerId(channel.offChainCounterparty)
  }))
}

/**
 * Get node's open channels and a counterParty's using connector's indexer.
 * @returns a promise that resolves to an array of peer ids
 */
export async function getPartyOpenChannels(node: Hopr<HoprCoreConnector>, party: PeerId): Promise<PeerId[]> {
  const { indexer, utils, types } = node.paymentChannels
  const partyPubKey = new types.Public(party.pubKey.marshal())
  if (!indexer) {
    throw new Error('Indexer is required')
  }

  // get indexed open channels
  const channels = await indexer.get({
    partyA: partyPubKey
  })
  // get the counterparty of each channel
  const channelAccountIds = channels.map((channel: any) => {
    return u8aEquals(channel.partyA, partyPubKey) ? channel.partyB : channel.partyA
  })

  // get available nodes
  const peers = await Promise.all(
    getPeers(node, {
      noBootstrapNodes: true
    }).map(async (peer) => {
      return {
        peer,
        accountId: await utils.pubKeyToAccountId(peer.pubKey.marshal())
      }
    })
  )

  return peers.reduce((acc: PeerId[], { peer, accountId }) => {
    if (
      channelAccountIds.find((channelAccountId: Uint8Array) => {
        return u8aEquals(accountId, channelAccountId)
      })
    ) {
      acc.push(peer)
    }

    return acc
  }, [])
}

/**
 * Get node's open channels with a counterParty using connector's DB or indexer if supported.
 * @returns a promise that resolves to an array of peer ids
 */
export async function getOpenChannels(node: Hopr<HoprCoreConnector>, partyPeerId: PeerId): Promise<PeerId[]> {
  const supportsIndexer = typeof node.paymentChannels.indexer !== 'undefined'
  const partyIsSelf = node.getId().equals(partyPeerId)

  if (partyIsSelf) {
    // if party is self, and indexer not supported
    return getMyOpenChannels(node)
  } else if (supportsIndexer) {
    // if connector supports indexeer
    return getPartyOpenChannels(node, partyPeerId)
  } else {
    // return an emptry array if connector does not support indexer
    return []
  }
}
