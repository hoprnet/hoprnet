import { type StrategyTickResult, SaneDefaults, type ChannelStrategyInterface } from '@hoprnet/hopr-core'
import type Hopr from '@hoprnet/hopr-core'
import type BN from 'bn.js'
import { type PublicKey, type ChannelEntry, ChannelStatus } from '@hoprnet/hopr-utils'
import type { PersistedState, State } from './state'
import { findCtChannelOpenTime, sendCTMessage } from './utils'
import {
  CT_INTERMEDIATE_HOPS,
  MESSAGE_FAIL_THRESHOLD,
  MINIMUM_STAKE_BEFORE_CLOSURE,
  CHANNELS_PER_COVER_TRAFFIC_NODE,
  CHANNEL_STAKE,
  CT_NETWORK_QUALITY_THRESHOLD,
  CT_CHANNEL_STALL_TIMEOUT,
  CT_OPEN_CHANNEL_QUALITY_THRESHOLD
} from './constants'
import { debug } from '@hoprnet/hopr-utils'

const log = debug('hopr:cover-traffic')

type CtChannel = {
  destination: PublicKey
  latestQualityOf: number
  openFrom: number
}

export class CoverTrafficStrategy extends SaneDefaults implements ChannelStrategyInterface {
  name = 'covertraffic'

  constructor(private selfPub: PublicKey, private node: Hopr, private data: PersistedState) {
    super()
  }

  // Interval of the `periodicCheck` in hopr-core
  tickInterval = 10000

  /**
   * Iterates through the persisted state and the current payment channel
   * graph and marks channels that should be closed and those that should
   * be opened.
   * Also returns a list of ct channels.
   * @param state current persisted state
   * @param currentChannels recent payment channel graph
   * @param peers known peers in the network
   * @returns
   */
  revisitTopology(
    state: State,
    currentChannels: ChannelEntry[],
    peers: Hopr['networkPeers']
  ): {
    ctChannels: CtChannel[]
    tickResult: StrategyTickResult
  } {
    const tickResult: StrategyTickResult = { toOpen: [], toClose: [] }
    const ctChannels: CtChannel[] = []

    for (let channel of currentChannels) {
      if (channel.status === ChannelStatus.Closed) {
        continue
      }
      const quality = peers.qualityOf(channel.destination.toPeerId())
      ctChannels.push({
        destination: channel.destination,
        latestQualityOf: quality,
        openFrom: findCtChannelOpenTime(channel.destination, state)
      })

      // Cover traffic channels with quality below this threshold will be closed
      if (quality < CT_NETWORK_QUALITY_THRESHOLD) {
        log(`closing channel ${channel.destination.toB58String()} with quality < ${CT_NETWORK_QUALITY_THRESHOLD}`)
        tickResult.toClose.push({
          destination: channel.destination
        })
      }
      // If the HOPR token balance of the current CT node is no larger than the `MINIMUM_STAKE_BEFORE_CLOSURE`, close all the non-closed channels.
      if (channel.balance.toBN().lt(MINIMUM_STAKE_BEFORE_CLOSURE)) {
        log(`closing channel with balance too low ${channel.destination.toB58String()}`)
        tickResult.toClose.push({
          destination: channel.destination
        })
      }
      // Close the cover-traffic channel when the number of failed messages meets the threshold. Reset the failed message counter.
      if (this.data.messageFails(channel.destination) > MESSAGE_FAIL_THRESHOLD) {
        log(`closing channel with too many message fails: ${channel.destination.toB58String()}`)
        this.data.resetMessageFails(channel.destination)
        tickResult.toClose.push({
          destination: channel.destination
        })
      }
    }

    return {
      tickResult,
      ctChannels
    }
  }

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
    peers: Hopr['networkPeers'],
    _getRandomChannel: () => Promise<ChannelEntry>
  ): Promise<StrategyTickResult> {
    log(`tick, balance ${balance.toString()}`)
    const state = this.data.get()

    // Refresh open channels.
    const { tickResult, ctChannels } = this.revisitTopology(state, currentChannels, peers)

    this.data.setCTChannels(ctChannels)
    log(
      'channels',
      ctChannels
        .map((c: CtChannel) => `${c.destination.toB58String()} - ${c.latestQualityOf}, ${c.openFrom}`)
        .join('; ')
    )

    // Network must have at least some channels to create a full cover-traffic loop.
    if (this.data.openChannelCount() > CT_INTERMEDIATE_HOPS + 1) {
      for (let openChannel of state.ctChannels) {
        // all the non-closed channels from this cover-traffic node.
        const channel = this.data.findChannel(this.selfPub, openChannel.destination)
        if (channel && channel.status == ChannelStatus.Open) {
          // send messages for open channels

          const success = await sendCTMessage(
            openChannel.destination,
            this.selfPub,
            async (message: Uint8Array, path: PublicKey[]) => {
              await this.node.sendMessage(message, this.selfPub.toPeerId(), path)
            },
            this.data
          )

          if (!success) {
            log(
              `failed to send to ${openChannel.destination.toB58String()} fails: ${this.data.messageFails(
                openChannel.destination
              )}`
            )
            this.data.incrementMessageFails(openChannel.destination)
          } else {
            this.data.incrementMessageTotalSuccess()
          }
        } else if (channel && channel.status == ChannelStatus.WaitingForCommitment) {
          if (Date.now() - openChannel.openFrom >= CT_CHANNEL_STALL_TIMEOUT) {
            // handle waiting for commitment stalls
            log('channel is stalled in WAITING_FOR_COMMITMENT, closing', openChannel.destination.toB58String())
            tickResult.toClose.push({
              destination: openChannel.destination
            })
          } else {
            log('channel is WAITING_FOR_COMMITMENT, waiting', openChannel.destination.toB58String())
          }
        } else {
          log(
            `Unknown error with open CT channels. Channel is ${
              channel.status
            }; openChannel is to ${openChannel.destination.toB58String()} since ${openChannel.openFrom} with quality ${
              openChannel.latestQualityOf
            }`
          )
        }
      }
    } else {
      log('aborting send messages - less channels in network than hops required')
    }
    log(`message send phase complete for ${state.ctChannels.length} ctChannels`)

    let attempts = 0
    let currentChannelNum = currentChannels.length
    // When there is no enough cover traffic channels, providing node exists and adequete past attempts, the node will open some channels.
    while (
      currentChannelNum < CHANNELS_PER_COVER_TRAFFIC_NODE &&
      Object.keys(state.nodes).length > 0 &&
      attempts < 100
    ) {
      attempts++
      const choice = this.data.weightedRandomChoice()
      const quality = peers.qualityOf(choice.toPeerId())
      // Ignore the randomly chosen node, if it's the cover traffic node itself, or a non-closed channel exists
      if (
        ctChannels.find((x: CtChannel) => x.destination.eq(choice)) ||
        choice.eq(this.selfPub) ||
        tickResult.toOpen.find((x: StrategyTickResult['toOpen'][number]) => x.destination.eq(choice)) ||
        tickResult.toClose.find((x: StrategyTickResult['toClose'][number]) => x.destination.eq(choice))
      ) {
        //console.error('skipping node', c.toB58String())
        continue
      }
      // It should fulfil the quality threshold
      if (quality < CT_OPEN_CHANNEL_QUALITY_THRESHOLD) {
        //log('low quality node skipped', c.toB58String(), q)
        continue
      }

      log(`opening ${choice.toB58String()}`)
      currentChannelNum++
      tickResult.toOpen.push({
        destination: choice,
        stake: CHANNEL_STAKE
      })
    }

    log(
      `strategy tick: ${Date.now()} balance:${balance.toString()} open:${tickResult.toOpen
        .map((p: StrategyTickResult['toOpen'][number]) => p.destination.toPeerId().toB58String())
        .join(',')} close: ${tickResult.toClose
        .map((p: StrategyTickResult['toClose'][number]) => p.destination.toPeerId().toB58String())
        .join(',')}`.replace('\n', ', ')
    )
    return tickResult
  }

  async onWinningTicket() {
    log('cover traffic ignores winning ticket.')
  }

  async onChannelWillClose() {
    log('cover traffic doesnt do anything as channel closes')
  }
}
