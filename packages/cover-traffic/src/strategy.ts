import { ChannelsToOpen, ChannelsToClose, SaneDefaults } from '@hoprnet/hopr-core'
import type Hopr from '@hoprnet/hopr-core'
import type BN from 'bn.js'
import { PublicKey, ChannelEntry, ChannelStatus } from '@hoprnet/hopr-utils'
import type { PersistedState } from './state'
import { sendCTMessage } from './utils'
import {
  CT_INTERMEDIATE_HOPS,
  MESSAGE_FAIL_THRESHOLD,
  MINIMUM_STAKE_BEFORE_CLOSURE,
  CHANNELS_PER_COVER_TRAFFIC_NODE,
  CHANNEL_STAKE,
  CT_NETWORK_QUALITY_THRESHOLD,
  CT_OPEN_CHANNEL_QUALITY_THRESHOLD
} from './constants'

export class CoverTrafficStrategy extends SaneDefaults {
  name = 'covertraffic'

  constructor(private selfPub: PublicKey, private node: Hopr, private data: PersistedState) {
    super()
  }

  // Interval of the `periodicCheck` in hopr-core
  tickInterval = 10000

  /**
   * Go through network state and get arrays of channels to be opened/closed
   * Called in `tickChannelStrategy` in `hopr-core`
   * @param balance HOPR token balance of the current node
   * @param currentChannels All the channels that have ever been opened with the current node as `source`
   * @param peers All the peers detected (destination of channels ever existed, destination of channels to be created, and peers connected through libp2p)
   * @param _getRandomChannel Method to get a random open channel
   * @returns Array of channels to be opened and channels to be closed.
   */
  async tick(
    balance: BN,
    currentChannels: ChannelEntry[],
    peers: any,
    _getRandomChannel: () => Promise<ChannelEntry>
  ): Promise<[ChannelsToOpen[], ChannelsToClose[]]> {
    const toOpen = []
    const toClose = []
    const state = this.data.get()

    // Refresh open channels.
    const ctChannels = []
    for (let c of currentChannels) {
      if (c.status === ChannelStatus.Closed) {
        continue
      }
      const q = await peers.qualityOf(c.destination)
      ctChannels.push({ destination: c.destination, latestQualityOf: q })
      // Cover traffic channels with quality below this threshold will be closed
      if (q < CT_NETWORK_QUALITY_THRESHOLD) {
        toClose.push(c.destination)
      }
      // If the HOPR token balance of the current CT node is no larger than the `MINIMUM_STAKE_BEFORE_CLOSURE`, close all the non-closed channels.
      if (c.balance.toBN().lte(MINIMUM_STAKE_BEFORE_CLOSURE)) {
        toClose.push(c.destination)
      }
      // Close the cover-traffic channel when the number of failed messages meets the threshold. Reset the failed message counter.
      if (this.data.messageFails(c.destinationPubKey) > MESSAGE_FAIL_THRESHOLD) {
        this.data.resetMessageFails(c.destinationPubKey)
        toClose.push(c.destination)
      }
    }
    this.data.setCTChannels(ctChannels)

    // Network must have at least some channels to create a full cover-traffic loop.
    if (this.data.openChannelCount() > CT_INTERMEDIATE_HOPS + 1) {
      for (let openChannel of state.ctChannels) {
        // all the non-closed channels from this cover-traffic node.
        const channel = this.data.findChannel(this.selfPub, openChannel.destination)
        if (channel && channel.status == ChannelStatus.Open) {
          // send messages for open channels
          const success = sendCTMessage(
            openChannel.destination,
            this.selfPub,
            async (path: PublicKey[]) => {
              await this.node.sendMessage(new Uint8Array(1), openChannel.destination.toPeerId(), path)
            },
            this.data
          )
          if (!success) {
            this.data.incrementMessageFails(openChannel.destination)
          }
        }
        // TODO: handle waiting for commitment stalls
      }
    } else {
      this.data.log('aborting send messages - less channels in network than hops required')
    }

    let attempts = 0
    // When there is no enough cover traffic channels, providing node exists and adequete past attempts, the node will open some channels.
    while (
      currentChannels.length < CHANNELS_PER_COVER_TRAFFIC_NODE &&
      Object.keys(state.nodes).length > 0 &&
      attempts < 100
    ) {
      attempts++
      const c = this.data.weightedRandomChoice()
      const q = await peers.qualityOf(c)
      // Ignore the randomly chosen node, if it's the cover traffic node itself, or a non-closed channel exists
      if (
        currentChannels.filter((x) => x.status !== ChannelStatus.Closed).find((x) => x.destinationPubKey.eq(c)) ||
        c.eq(this.selfPub) ||
        toOpen.find((x) => x[1].eq(c))
      ) {
        console.error('skipping node', c.toB58String())
        continue
      }
      // It should fulfil the quality threshold
      if (q < CT_OPEN_CHANNEL_QUALITY_THRESHOLD) {
        console.error('low quality node skipped', c.toB58String(), q)
        continue
      }

      toOpen.push([c, CHANNEL_STAKE])
    }

    this.data.log(
      `strategy tick: ${Date.now()} balance:${balance.toString()} open:${toOpen
        .map((p) => p[0].toPeerId().toB58String())
        .join(',')} close: ${toClose.map((p) => p.toPeerId().toB58String()).join(',')}`.replace('\n', ', ')
    )
    return [toOpen, toClose]
  }
}
