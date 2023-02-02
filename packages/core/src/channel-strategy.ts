import HoprCoreEthereum, { type ChannelEntry } from '@hoprnet/hopr-core-ethereum'
import { type AcknowledgedTicket, ChannelStatus, debug } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { RS_ChannelEntry } from '@hoprnet/hopr-utils'

const log = debug('hopr-core:channel-strategy')

// Required to use with Node.js with ES, see https://docs.rs/getrandom/latest/getrandom/#nodejs-es-module-support
import { webcrypto } from 'node:crypto'
// @ts-ignore
globalThis.crypto = webcrypto

import {
  PromiscuousStrategy,
  PassiveStrategy,
  RandomStrategy,
  StrategyTickResult,
  Balance,
  BalanceType,
  core_strategy_set_panic_hook
} from '../lib/core_strategy.js'
import { CHECK_TIMEOUT } from './constants.js'

core_strategy_set_panic_hook()

type RustStrategyInterface = PromiscuousStrategy | PassiveStrategy | RandomStrategy;
const STRATEGIES = ['passive', 'promiscuous', 'random']
export type Strategy = typeof STRATEGIES[number]

export function isStrategy(str: string): str is Strategy {
  return STRATEGIES.includes(str)
}

export class OutgoingChannelStatus {
  peer_id: string
  stake_str: string
  status: ChannelStatus
}

/**
 * Staked nodes will likely want to automate opening and closing of channels. By
 * implementing the following interface, they can decide how to allocate their
 * stake to best attract traffic with a useful channel graph.
 *
 * Implementors should bear in mind:
 * - Churn is expensive
 * - Path finding will prefer high stakes, and high availability of nodes.
 */
export interface ChannelStrategyInterface {
  name: string

  configure(settings: any)

  tick(
    balance: BN,
    network_peer_ids: Iterator<string>,
    outgoing_channel: OutgoingChannelStatus[],
    peer_quality: (string) => number
  ): StrategyTickResult

  onChannelWillClose(channel: ChannelEntry)
  onWinningTicket(t: AcknowledgedTicket)
  shouldCommitToChannel(c: ChannelEntry): boolean

  tickInterval: number
}

/*
 * Saves duplication of 'normal' behaviour.
 *
 * At present this does not take gas into consideration.
 */
/*
export abstract class SaneDefaults {
  async onWinningTicket(ackTicket: AcknowledgedTicket, chain: HoprCoreEthereum) {
    const counterparty = ackTicket.signer
    log(`auto redeeming tickets in channel to ${counterparty.toPeerId().toString()}`)
    await chain.redeemTicketsInChannelByCounterparty(counterparty)
  }

  async onChannelWillClose(channel: ChannelEntry, chain: HoprCoreEthereum) {
    const counterparty = channel.source
    const selfPubKey = chain.getPublicKey()
    if (!counterparty.eq(selfPubKey)) {
      log(`auto redeeming tickets in channel to ${counterparty.toPeerId().toString()}`)
      try {
        await chain.redeemTicketsInChannel(channel)
      } catch (err) {
        log(`Could not redeem tickets in channel ${channel.getId().toHex()}`, err)
      }
    }
  }

  shouldCommitToChannel(c: ChannelEntry): boolean {
    log(`committing to channel ${c.getId().toHex()}`)
    return true
  }

  tickInterval = CHECK_TIMEOUT
}
*/

/**
  Temporary wrapper class before we migrate rest of the core to use Rust exported types (before we migrate everything to Rust!)
 */
class RustStrategyWrapper<T extends RustStrategyInterface> implements ChannelStrategyInterface {
  constructor(private strategy: T) {
  }

  configure(settings: any) {
    this.strategy.configure(settings)
  }

  tick(
    balance: BN,
    network_peer_ids: Iterator<string>,
    outgoing_channels: OutgoingChannelStatus[],
    peer_quality: (string) => number
  ): StrategyTickResult {
    return this.strategy.tick(
      new Balance(balance.toString(), BalanceType.HOPR),
      network_peer_ids,
      outgoing_channels,
      peer_quality
    )
  }

  get name() {
    return this.strategy.name
  }

  tickInterval: number = CHECK_TIMEOUT

  onChannelWillClose(channel: ChannelEntry) {
  }

  onWinningTicket(t: AcknowledgedTicket) {
  }

  shouldCommitToChannel(_c: ChannelEntry): boolean {
    let ce: RS_ChannelEntry =  {
      source: Pub
    }

    return this.strategy.should_commit_to_channel(ce)
  }
}

export class StrategyFactory {
  public static getStrategy(strategy: Strategy): ChannelStrategyInterface {
    switch (strategy) {
      case 'promiscuous':
        return new RustStrategyWrapper(new PromiscuousStrategy())
      case 'random':
        log(`error: random strategy not implemented, falling back to 'passive'.`)
      case 'passive':
      default:
        return new RustStrategyWrapper(new PassiveStrategy())
    }
  }
}
