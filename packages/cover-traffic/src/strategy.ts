import { ChannelsToOpen, ChannelsToClose, SaneDefaults } from '@hoprnet/hopr-core'
import type Hopr from '@hoprnet/hopr-core'
import type BN from 'bn.js'
import { PublicKey, ChannelEntry, ChannelStatus } from '@hoprnet/hopr-utils'
import type { PersistedState } from './state'
import { findCtChannelOpenTime, sendCTMessage } from './utils'
import {
  CT_INTERMEDIATE_HOPS,
  MESSAGE_FAIL_THRESHOLD,
  MINIMUM_STAKE_BEFORE_CLOSURE,
  CHANNELS_PER_COVER_TRAFFIC_NODE,
  CHANNEL_STAKE,
  CT_NETWORK_QUALITY_THRESHOLD,
  CT_CHANNEL_STALL_TIMEOUT
} from './constants'

export class CoverTrafficStrategy extends SaneDefaults {
  name = 'covertraffic'

  constructor(private selfPub: PublicKey, private node: Hopr, private data: PersistedState) {
    super()
  }

  tickInterval = 10000

  async tick(
    balance: BN,
    currentChannels: ChannelEntry[],
    peers: any,
    _getRandomChannel: () => Promise<ChannelEntry>
  ): Promise<[ChannelsToOpen[], ChannelsToClose[]]> {
    const toOpen = []
    const toClose = []
    const state = this.data.get()

    // Refresh open channels
    const ctChannels = []
    for (let c of currentChannels) {
      if (c.status === ChannelStatus.Closed) {
        continue
      }
      const q = await peers.qualityOf(c.destination)
      ctChannels.push({ destination: c.destination, latestQualityOf: q, openFrom: findCtChannelOpenTime(c.destination, state)})
      if (q < CT_NETWORK_QUALITY_THRESHOLD) {
        toClose.push(c.destination)
      }
      if (c.balance.toBN().lte(MINIMUM_STAKE_BEFORE_CLOSURE)) {
        toClose.push(c.destination)
      }
      if (this.data.messageFails(c.destination) > MESSAGE_FAIL_THRESHOLD) {
        this.data.resetMessageFails(c.destination)
        toClose.push(c.destination)
      }
    }
    this.data.setCTChannels(ctChannels)

    if (this.data.openChannelCount() > CT_INTERMEDIATE_HOPS + 1) {
      for (let openChannel of state.ctChannels) {
        const channel = this.data.findChannel(this.selfPub, openChannel.destination)
        if (channel && channel.status == ChannelStatus.Open) {
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
        } else if (
          channel && 
          channel.status == ChannelStatus.WaitingForCommitment && 
          Date.now() - openChannel.openFrom >= CT_CHANNEL_STALL_TIMEOUT
        ) {
          // handle waiting for commitment stalls
          toClose.push(openChannel.destination);
        }
      }
    } else {
      this.data.log('aborting send messages - less channels in network than hops required')
    }

    let attempts = 0
    while (
      currentChannels.length < CHANNELS_PER_COVER_TRAFFIC_NODE &&
      Object.keys(state.nodes).length > 0 &&
      attempts < 100
    ) {
      attempts++
      const c = this.data.weightedRandomChoice()
      const q = await peers.qualityOf(c)

      if (
        currentChannels.filter((x) => x.status !== ChannelStatus.Closed).find((x) => x.destination.eq(c)) ||
        c.eq(this.selfPub) ||
        toOpen.find((x) => x[1].eq(c))
      ) {
        console.error('skipping node', c.toB58String())
        continue
      }

      if (q < 0.6) {
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
