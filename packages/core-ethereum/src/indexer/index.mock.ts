import EventEmitter from 'events'
import { providers as Providers, Wallet, BigNumber } from 'ethers'
import type { HoprChannels, HoprToken, TypedEvent } from '@hoprnet/hopr-ethereum'
import {
  Address,
  ChannelEntry,
  Hash,
  HoprDB,
  generateChannelId,
  NativeBalance,
  SUGGESTED_NATIVE_BALANCE,
  debug,
  AccountEntry,
  PublicKey
} from '@hoprnet/hopr-utils'

import Indexer from '.'
import type { ChainWrapper } from '../ethereum'
import type { Event, TokenEvent } from './types'
import * as fixtures from './fixtures'
import { ACCOUNT_A, PARTY_A, PARTY_A_MULTIADDR, PARTY_B } from '../fixtures'
import { Multiaddr } from 'multiaddr'
import BN from 'bn.js'

//@TODO: Refactor this logger and mock outside of indexer
const chainLogger = debug(`hopr:mocks:indexer-chain`)

const txRequest = {
  to: fixtures.ACCOUNT_B.address,
  data: '0x0',
  value: 0,
  nonce: 0,
  gasPrice: 1
}

const createProviderMock = (ops: { latestBlockNumber?: number } = {}) => {
  let latestBlockNumber = ops.latestBlockNumber ?? 0

  const provider = new EventEmitter() as unknown as Providers.WebSocketProvider
  provider.getBlockNumber = async (): Promise<number> => latestBlockNumber

  return {
    provider,
    newBlock() {
      latestBlockNumber++
      provider.emit('block', latestBlockNumber)
    }
  }
}

const createHoprChannelsMock = (ops: { pastEvents?: Event<any>[] } = {}) => {
  const pastEvents = ops.pastEvents ?? []
  const channels: any = {}
  const pubkeys: any = {}

  const handleEvent = (ev) => {
    if (ev.event == 'ChannelUpdated') {
      const updateEvent = ev as Event<'ChannelUpdated'>

      const eventChannelId = generateChannelId(
        Address.fromString(updateEvent.args.source),
        Address.fromString(updateEvent.args.destination)
      )
      channels[eventChannelId.toHex()] = updateEvent.args.newState
    } else if (ev.event == 'Announce') {
      pubkeys[ev.args.account] = ev.args.multiaddr
    } else {
      //throw new Error("MISSING EV HANDLER IN TEST")
    }
  }

  class FakeChannels extends EventEmitter {
    async channels(channelId: string) {
      for (let ev of pastEvents) {
        handleEvent(ev)
      }
      return channels[channelId]
    }

    async bumpChannel(_counterparty: string, _comm: string) {
      let newEvent = {
        event: 'ChannelUpdated',
        transactionHash: '',
        blockNumber: 3,
        transactionIndex: 0,
        logIndex: 0,
        args: {
          source: PARTY_B.toAddress().toHex(),
          destination: PARTY_A.toAddress().toHex(),
          newState: {
            balance: BigNumber.from('3'),
            commitment: Hash.create(new TextEncoder().encode('commA')).toHex(),
            ticketEpoch: BigNumber.from('1'),
            ticketIndex: BigNumber.from('0'),
            status: 2,
            channelEpoch: BigNumber.from('0'),
            closureTime: BigNumber.from('0')
          }
        } as any
      } as Event<'ChannelUpdated'>
      handleEvent(newEvent)
      this.emit('*', newEvent)
    }

    async queryFilter() {
      return pastEvents
    }
  }

  const hoprChannels = new FakeChannels() as unknown as HoprChannels

  return {
    hoprChannels,
    pubkeys,
    newEvent(event: Event<any>) {
      pastEvents.push(event)
      hoprChannels.emit('*', event)
    }
  }
}
const createHoprTokenMock = () => {
  class FakeToken extends EventEmitter {
    async transfer() {
      let newEvent = {
        event: 'Transfer',
        transactionHash: '',
        blockNumber: 8,
        transactionIndex: 0,
        logIndex: 0,
        args: {
          source: PARTY_A.toAddress().toHex(),
          destination: PARTY_B.toAddress().toHex(),
          balance: BigNumber.from('1')
        } as any
      } as TokenEvent<'Transfer'>
      this.emit('*', newEvent)
    }
  }

  const hoprToken = new FakeToken() as unknown as HoprToken

  return {
    hoprToken,
    newEvent(event: Event<any>) {
      hoprToken.emit('*', event)
    }
  }
}

const createChainMock = (
  provider: Providers.WebSocketProvider,
  hoprChannels: HoprChannels,
  hoprToken: HoprToken,
  account?: Wallet
): ChainWrapper => {
  return {
    getLatestBlockNumber: () => provider.getBlockNumber(),
    subscribeBlock: (cb: (blockNumber: number) => void | Promise<void>) => {
      provider.on('block', cb)

      return () => {
        provider.off('block', cb)
      }
    },
    subscribeError: (cb: (err: any) => void | Promise<void>): (() => void) => {
      provider.on('error', cb)
      hoprChannels.on('error', cb)
      hoprToken.on('error', cb)

      return () => {
        provider.off('error', cb)
        hoprChannels.off('error', cb)
        hoprToken.off('error', cb)
      }
    },
    subscribeChannelEvents: (cb: (event: TypedEvent<any, any>) => void | Promise<void>) => {
      hoprChannels.on('*', cb)

      return () => {
        hoprChannels.off('*', cb)
      }
    },
    start: () => {},
    waitUntilReady: () => {
      chainLogger('Await on chain readyness')
      return Promise.resolve()
    },
    getGenesisBlock: () => {
      chainLogger('Genesis log requested')
      return 0
    },
    waitForPublicNodes: () => {
      chainLogger('On-chain request for existing public nodes.')
      return Promise.resolve([])
    },
    announce: () => {
      chainLogger('On-chain announce request sent')
    },
    on: (event: string) => {
      chainLogger(`On-chain signal for event "${event}"`)
    },
    indexer: {
      on: (event: string) => chainLogger(`Indexer on handler top of chain called with event "${event}"`),
      off: (event: string) => chainLogger(`Indexer off handler top of chain called with event "${event}`)
    },
    subscribeTokenEvents: (cb: (event: TypedEvent<any, any>) => void | Promise<void>): (() => void) => {
      hoprToken.on('*', cb)

      return () => {
        hoprToken.off('*', cb)
      }
    },
    getNativeTokenTransactionInBlock: (_blockNumber: number, _isOutgoing: boolean = true) => [],
    updateConfirmedTransaction: (_hash: string) => {},
    getNativeBalance: () => new NativeBalance(SUGGESTED_NATIVE_BALANCE),
    getChannels: () => hoprChannels,
    getWallet: () => account ?? fixtures.ACCOUNT_A,
    getAccount: () => {
      chainLogger('getAccount method was called')
      return Promise.resolve(
        new AccountEntry(
          fixtures.PARTY_A.toAddress(),
          new Multiaddr(`/ip4/127.0.0.1/tcp/124/p2p/${fixtures.PARTY_A.toB58String()}`),
          new BN('1')
        )
      )
    },
    getPublicKey: () => fixtures.PARTY_A,
    setCommitment: (counterparty: Address, commitment: Hash) =>
      hoprChannels.bumpChannel(counterparty.toHex(), commitment.toHex()),
    getAllQueuingTransactionRequests: () => [txRequest]
  } as unknown as ChainWrapper
}

class TestingIndexer extends Indexer {
  public restart(): Promise<void> {
    return super.restart()
  }
}

export const useFixtures = async (
  ops: { latestBlockNumber?: number; pastEvents?: Event<any>[]; id?: PublicKey } = {}
) => {
  const latestBlockNumber = ops.latestBlockNumber ?? 0
  const pastEvents = ops.pastEvents ?? []

  const db = HoprDB.createMock(ops.id)
  const { provider, newBlock } = createProviderMock({ latestBlockNumber })
  const { hoprChannels, newEvent } = createHoprChannelsMock({ pastEvents })
  const { hoprToken } = createHoprTokenMock()
  const chain = createChainMock(provider, hoprChannels, hoprToken)
  return {
    db,
    provider,
    newBlock,
    hoprChannels,
    hoprToken,
    newEvent,
    indexer: new TestingIndexer(!ops.id ? PublicKey.createMock().toAddress() : ops.id.toAddress(), db, 1, 5),
    chain,
    OPENED_CHANNEL: await ChannelEntry.fromSCEvent(fixtures.OPENED_EVENT, (a: Address) =>
      Promise.resolve(a.eq(PARTY_A.toAddress()) ? PARTY_A : PARTY_B)
    ),
    COMMITTED_CHANNEL: await ChannelEntry.fromSCEvent(fixtures.COMMITTED_EVENT, (a: Address) =>
      Promise.resolve(a.eq(PARTY_A.toAddress()) ? PARTY_A : PARTY_B)
    )
  }
}

export { ACCOUNT_A, PARTY_A, PARTY_A_MULTIADDR }
