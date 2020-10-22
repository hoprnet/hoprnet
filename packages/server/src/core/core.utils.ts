import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Channel } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type { CoreService } from './core.service'
import PeerId from 'peer-id'
import { u8aEquals } from '@hoprnet/hopr-utils'
import { pubKeyToPeerId } from '@hoprnet/hopr-core/lib/utils'

/*
  A decorator function to check whether node is started,
  if not it will throw an error
*/
export function mustBeStarted(): MethodDecorator {
  return (
    _target: CoreService,
    _key: string,
    descriptor: TypedPropertyDescriptor<any>,
  ): TypedPropertyDescriptor<any> => {
    const originalFn = descriptor.value

    descriptor.value = function (...args: any[]) {
      if (!this.started) {
        throw Error('HOPR node is not started')
      }

      return originalFn.bind(this)(...args)
    }

    return descriptor
  }
}

/**
 * Checks whether the given PeerId belongs to any known bootstrap node.
 *
 * @param peerId
 */
export function isBootstrapNode(node: Hopr<HoprCoreConnector>, peerId: PeerId): boolean {
  for (let i = 0; i < node.bootstrapServers.length; i++) {
    if (peerId.isEqual(node.bootstrapServers[i].id)) {
      return true
    }
  }
  return false
}

/**
 * Get node's peers.
 * @returns an array of peer ids
 */
export function getPeers(
  node: Hopr<HoprCoreConnector>,
  ops: {
    noBootstrapNodes: boolean
  } = {
    noBootstrapNodes: false,
  },
): PeerId[] {
  let peers = node.getConnectedPeers()

  if (ops.noBootstrapNodes) {
    peers = peers.filter((peerId) => {
      return !isBootstrapNode(node, peerId)
    })
  }

  return peers
}

/**
 * Get node's open channels by looking into connector's DB.
 * @returns a promise that resolves to an array of channel instances in which we have open channels with
 */
export function getMyOpenChannels(node: Hopr<HoprCoreConnector>): Promise<Channel[]> {
  return new Promise<Channel[]>((resolve, reject) => {
    try {
      const channels: Channel[] = []

      node.paymentChannels.channel.getAll(
        async (channel: Channel) => {
          channels.push(channel)
        },
        async (promises: Promise<void>[]) => {
          await Promise.all(promises)
          return resolve(channels)
        },
      )
    } catch (err) {
      return reject(err)
    }
  })
}

/**
 * Get node's open channels and a counterParty's using connector's indexer.
 * @returns a promise that resolves to an array of peer ids in which we have open channels with
 */
export async function getPartyOpenChannels(node: Hopr<HoprCoreConnector>, party: PeerId): Promise<PeerId[]> {
  const { indexer, utils, types } = node.paymentChannels
  const partyPublicKey = new types.Public(party.pubKey.marshal())

  // get indexed open channels
  const channels = await indexer.get({
    partyA: partyPublicKey,
  })
  // get the counterparty of each channel
  const channelAccountIds = channels.map((channel) => {
    return u8aEquals(channel.partyA, partyPublicKey) ? channel.partyB : channel.partyA
  })

  // get available nodes
  const peers = await Promise.all(
    getPeers(node, {
      noBootstrapNodes: true,
    }).map(async (peer) => {
      return {
        peer,
        accountId: await utils.pubKeyToAccountId(peer.pubKey.marshal()),
      }
    }),
  )

  return peers.reduce((acc: PeerId[], { peer, accountId }) => {
    if (
      channelAccountIds.find((channelAccountId) => {
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
  const partyIfSelf = node.getId().equals(partyPeerId)

  if (partyIfSelf) {
    // if party is self
    return getMyOpenChannels(node).then((channels) => {
      return Promise.all(
        channels.map(async (channel) => {
          const pubKey = await channel.offChainCounterparty
          const peerId = await pubKeyToPeerId(pubKey)

          return peerId
        }),
      )
    })
  } else if (supportsIndexer) {
    // if connector supports indexeer
    return getPartyOpenChannels(node, partyPeerId)
  } else {
    // return an emptry array if connector does not support indexer
    return []
  }
}
