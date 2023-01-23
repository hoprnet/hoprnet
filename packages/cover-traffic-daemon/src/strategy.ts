import { StrategyTickResult, SaneDefaults, type ChannelStrategyInterface } from '@hoprnet/hopr-core'
import Hopr from '@hoprnet/hopr-core'
import BN from 'bn.js'
import { PublicKey, ChannelStatus } from '@hoprnet/hopr-utils'
import type { PersistedState, State } from './state.js'
import { findCtChannelOpenTime, sendCTMessage } from './utils.js'
import {
  CT_INTERMEDIATE_HOPS,
  MESSAGE_FAIL_THRESHOLD,
  MINIMUM_STAKE_BEFORE_CLOSURE,
  CHANNELS_PER_COVER_TRAFFIC_NODE,
  CHANNEL_STAKE,
  CT_NETWORK_QUALITY_THRESHOLD,
  CT_CHANNEL_STALL_TIMEOUT,
  CT_OPEN_CHANNEL_QUALITY_THRESHOLD
} from './constants.js'
import { debug } from '@hoprnet/hopr-utils'
import { OutgoingChannelStatus } from '@hoprnet/hopr-core/lib/core_strategy.js'

const log = debug('hopr:cover-traffic')

type CtChannel = {
  destination: PublicKey
  latestQualityOf: number
  openFrom: number
}

const MAX_CT_AUTO_CHANNELS: number = 100

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
   * @param peerQuality peer quality evaluator
   * @returns
   */
  revisitTopology(
    state: State,
    currentChannels: OutgoingChannelStatus[],
    peerQuality: (string) => number
  ): {
    ctChannels: CtChannel[]
    tickResult: StrategyTickResult
  } {
    let channelsToClose = []
    const ctChannels: CtChannel[] = []

    for (let channel of currentChannels) {
      const peerPubkey = PublicKey.fromPeerIdString(channel.peer_id)
      const quality = peerQuality(channel.peer_id)
      ctChannels.push({
        destination: peerPubkey,
        latestQualityOf: quality,
        openFrom: findCtChannelOpenTime(peerPubkey, state)
      })

      // Cover traffic channels with quality below this threshold will be closed
      if (quality < CT_NETWORK_QUALITY_THRESHOLD) {
        log(`closing channel ${channel.peer_id} with quality < ${CT_NETWORK_QUALITY_THRESHOLD}`)
        channelsToClose.push(channel.peer_id)
      }
      // If the HOPR token balance of the current CT node is no larger than the `MINIMUM_STAKE_BEFORE_CLOSURE`, close all the non-closed channels.
      if (new BN(channel.stake_str).lt(MINIMUM_STAKE_BEFORE_CLOSURE)) {
        log(`closing channel with balance too low ${channel.peer_id}`)
        channelsToClose.push(channel.peer_id)
      }
      // Close the cover-traffic channel when the number of failed messages meets the threshold. Reset the failed message counter.
      if (this.data.messageFails(peerPubkey) > MESSAGE_FAIL_THRESHOLD) {
        log(`closing channel with too many message fails: ${channel.peer_id}`)
        this.data.resetMessageFails(peerPubkey)
        channelsToClose.push(channel.peer_id)
      }
    }

    return {
      tickResult: new StrategyTickResult(MAX_CT_AUTO_CHANNELS, [], channelsToClose),
      ctChannels
    }
  }

  /**
   * Go through network state and get arrays of channels to be opened/closed
   * Called in `tickChannelStrategy` in `hopr-core`
   * @param balance HOPR token balance of the current node
   * @param _peers
   * @param currentChannels All the channels that have ever been opened with the current node as `source`
   * @param peerQuality Peer quality evaluator
   * @returns Array of channels to be opened and channels to be closed.
   */
  tick(
    balance: BN,
    _peers: Iterator<string>,
    currentChannels: OutgoingChannelStatus[],
    peerQuality: (string) => number
  ): StrategyTickResult {
    log(`tick, balance ${balance.toString()}`)
    const state = this.data.get()

    // Refresh open channels.
    const { tickResult, ctChannels } = this.revisitTopology(state, currentChannels, peerQuality)

    this.data.setCTChannels(ctChannels)
    log(
      'channels',
      ctChannels.map((c: CtChannel) => `${c.destination.toString()} - ${c.latestQualityOf}, ${c.openFrom}`).join('; ')
    )

    let toOpen: OutgoingChannelStatus[] = [...tickResult.to_open()]
    let toClose: string[] = [...tickResult.to_close()]

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
              `failed to send to ${openChannel.destination.toString()} fails: ${this.data.messageFails(
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
            log('channel is stalled in WAITING_FOR_COMMITMENT, closing', openChannel.destination.toString())
            toClose.push(openChannel.destination.toPeerId().toString())
          } else {
            log('channel is WAITING_FOR_COMMITMENT, waiting', openChannel.destination.toString())
          }
        } else {
          log(
            `Unknown error with open CT channels. Channel is ${
              channel.status
            }; openChannel is to ${openChannel.destination.toString()} since ${openChannel.openFrom} with quality ${
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
      const quality = peerQuality(choice.toPeerId().toString())
      // Ignore the randomly chosen node, if it's the cover traffic node itself, or a non-closed channel exists
      if (
        ctChannels.find((x: CtChannel) => x.destination.eq(choice)) ||
        choice.eq(this.selfPub) ||
        toOpen.find((x) => x.peer_id == choice.toPeerId().toString()) ||
        toClose.find((x) => x == choice.toPeerId().toString())
      ) {
        //console.error('skipping node', c.toB58String())
        continue
      }
      // It should fulfil the quality threshold
      if (quality < CT_OPEN_CHANNEL_QUALITY_THRESHOLD) {
        //log('low quality node skipped', c.toB58String(), q)
        continue
      }

      log(`opening ${choice.toString()}`)
      currentChannelNum++
      toOpen.push(new OutgoingChannelStatus(choice.toPeerId().toString(), CHANNEL_STAKE.toString()))
    }

    log(
      `strategy tick: ${Date.now()} balance:${balance.toString()} open:${toOpen
        .map((p) => p.peer_id)
        .join(',')} close: ${toClose.join(',')}`.replace('\n', ', ')
    )
    return new StrategyTickResult(MAX_CT_AUTO_CHANNELS, toOpen, toClose)
  }

  async onWinningTicket() {
    log('cover traffic ignores winning ticket.')
  }

  async onChannelWillClose() {
    log('cover traffic doesnt do anything as channel closes')
  }
}
