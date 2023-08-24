import { debug } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { CHECK_TIMEOUT } from './constants.js'

const log = debug('hopr-core:channel-strategy')

// Required to use with Node.js with ES, see https://docs.rs/getrandom/latest/getrandom/#nodejs-es-module-support
import { webcrypto } from 'node:crypto'
// @ts-ignore
globalThis.crypto = webcrypto

import {
  PromiscuousStrategy,
  PassiveStrategy,
  StrategyTickResult,
  Balance,
  BalanceType,
  PeerQuality
} from '../lib/core_hopr.js'

import { ChannelStatus, AcknowledgedTicket, ChannelEntry } from '@hoprnet/hopr-utils'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'

const STRATEGIES = ['passive', 'promiscuous', 'random']
export type Strategy = (typeof STRATEGIES)[number]

export function isStrategy(str: string): str is Strategy {
  return STRATEGIES.includes(str)
}
export interface OutgoingChannelStatus {
  address: string
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

  configure(settings: any): void

  tick(
    balance: BN,
    network_addresses: PeerQuality,
    outgoing_channel: OutgoingChannelStatus[],
  ): StrategyTickResult

  onChannelWillClose(channel: ChannelEntry): Promise<void> // Before a channel closes
  onAckedTicket(t: AcknowledgedTicket): Promise<void>
  shouldCommitToChannel(c: ChannelEntry): boolean

  tickInterval: number
}

/*
 * Saves duplication of 'normal' behaviour.
 *
 * At present this does not take gas into consideration.
 */
export abstract class SaneDefaults {
  protected autoRedeemTickets: boolean = true

  async onAckedTicket(ackTicket: AcknowledgedTicket) {
    if (this.autoRedeemTickets) {
      const counterparty = ackTicket.signer
      log(`auto redeeming tickets in channel to ${counterparty.to_string()}`)
      await HoprCoreEthereum.getInstance().redeemTicketsInChannelByCounterparty(counterparty)
    } else {
      log(`encountered winning ticket, not auto-redeeming`)
    }
  }

  /**
   * When an incoming channel is going to be closed, auto redeem tickets
   * @param channel channel that will be closed
   */
  async onChannelWillClose(channel: ChannelEntry) {
    if (this.autoRedeemTickets) {
      const chain = HoprCoreEthereum.getInstance()
      const counterparty = channel.source
      const selfPubKey = chain.getPublicKey()
      if (!counterparty.eq(selfPubKey.to_address())) {
        log(`auto redeeming tickets in channel to ${counterparty.to_string()}`)
        try {
          await chain.redeemTicketsInChannel(channel)
        } catch (err) {
          log(`Could not redeem tickets in channel ${channel.get_id().to_hex()}`, err)
        }
      }
    } else {
      log(`channel ${channel.get_id().to_hex()} is closing, not auto-redeeming tickets`)
    }
  }

  shouldCommitToChannel(c: ChannelEntry): boolean {
    log(`committing to channel ${c.get_id().to_hex()}`)
    return true
  }

  tickInterval = CHECK_TIMEOUT
}

interface RustStrategyInterface {
  tick: (
    balance: Balance,
    network_addresses: PeerQuality,
    outgoing_channels: OutgoingChannelStatus[],
  ) => StrategyTickResult
  name: string
}

/**
  Temporary wrapper class before we migrate rest of the core to use Rust exported types (before we migrate everything to Rust!)
 */
class RustStrategyWrapper<T extends RustStrategyInterface> extends SaneDefaults implements ChannelStrategyInterface {
  constructor(private strategy: T) {
    super()
  }

  configure(settings: any) {
    for (const [key, value] of Object.entries(settings)) {
      if (key in this.strategy) {
        this.strategy[key] = value
      }
    }
    this.autoRedeemTickets = settings.auto_redeem_tickets ?? false
  }

  tick(
    balance: BN,
    network_addresses: PeerQuality,
    outgoing_channels: OutgoingChannelStatus[],
  ): StrategyTickResult {
    return this.strategy.tick(
      new Balance(balance.toString(), BalanceType.HOPR),
      network_addresses,
      outgoing_channels,
    )
  }

  get name() {
    return this.strategy.name
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
