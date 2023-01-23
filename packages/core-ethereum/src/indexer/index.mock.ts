import EventEmitter from 'events'
import { providers as Providers, Wallet, BigNumber, utils } from 'ethers'
import type { HoprChannels, HoprNetworkRegistry, HoprToken, TypedEvent } from '../utils/index.js'

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

import Indexer from './index.js'
import type { ChainWrapper } from '../ethereum.js'
import type { Event, TokenEvent, RegistryEvent } from './types.js'
import * as fixtures from './fixtures.js'
import { ACCOUNT_A, PARTY_A, PARTY_A_MULTIADDR, PARTY_B } from '../fixtures.js'
import { Multiaddr } from '@multiformats/multiaddr'
import BN from 'bn.js'

//@TODO: Refactor this logger and mock outside of indexer
const chainLogger = debug(`hopr:mocks:indexer-chain`)

const txRequest = {
  to: fixtures.ACCOUNT_B.address,
  data: '0x0',
  value: 0,
  nonce: 0,
  gasLimit: BigNumber.from(400e3),
  maxPriorityFeePerGas: utils.parseUnits('1', 'gwei'),
  maxFeePerGas: utils.parseUnits('1', 'gwei')
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

  const handleEvent = (ev: TypedEvent<any, any>) => {
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
      pastEvents.push(newEvent)
    }

    async queryFilter() {
      return pastEvents
    }

    interface = {
      // Dummy event topic but different for every event
      getEventTopic: (arg: string) => utils.keccak256(utils.toUtf8Bytes(arg)),
      // Events are already correctly formatted
      parseLog: (arg: any) => arg
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

const createHoprTokenMock = (ops: { pastEvents?: Event<any>[] } = {}) => {
  const pastEvents = ops.pastEvents ?? []

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
      pastEvents.push(newEvent)
    }

    async queryFilter() {
      return pastEvents
    }

    interface = {
      // Dummy event topic but different for every event
      getEventTopic: (arg: string) => utils.keccak256(utils.toUtf8Bytes(arg)),
      // Events are already correctly formatted
      parseLog: (arg: any) => arg
    }
  }

  const hoprToken = new FakeToken() as unknown as HoprToken

  return {
    hoprToken,
    newEvent(event: Event<any>) {
      pastEvents.push(event)
    }
  }
}

const createHoprRegistryMock = (ops: { pastEvents?: Event<any>[] } = {}) => {
  const pastEvents = ops.pastEvents ?? []

  class FakeHoprRegistry extends EventEmitter {
    async queryFilter() {
      return pastEvents
    }

    interface = {
      // Dummy event topic but different for every event
      getEventTopic: (arg: string) => utils.keccak256(utils.toUtf8Bytes(arg)),
      // Events are already correctly formatted
      parseLog: (arg: any) => arg
    }
  }

  const hoprRegistry = new FakeHoprRegistry() as unknown as HoprNetworkRegistry

  return {
    hoprRegistry,
    newEvent(event: Event<any>) {
      pastEvents.push(event)
    }
  }
}

const createChainMock = (
  provider: Providers.WebSocketProvider,
  hoprChannels: HoprChannels,
  hoprToken: HoprToken,
  hoprRegistry: HoprNetworkRegistry,
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
      hoprRegistry.on('error', cb)

      return () => {
        provider.off('error', cb)
        hoprChannels.off('error', cb)
        hoprToken.off('error', cb)
        hoprRegistry.off('error', cb)
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
    getNativeTokenTransactionInBlock: (_blockNumber: number, _isOutgoing: boolean = true) =>
      Promise.resolve<string[]>([]),
    updateConfirmedTransaction: (_hash: string) => {},
    getNativeBalance: () => new NativeBalance(SUGGESTED_NATIVE_BALANCE),
    getChannels: () => hoprChannels,
    getToken: () => hoprToken,
    getNetworkRegistry: () => hoprRegistry,
    getWallet: () => account ?? fixtures.ACCOUNT_A,
    getAccount: () => {
      chainLogger('getAccount method was called')
      return Promise.resolve(
        new AccountEntry(
          fixtures.PARTY_A,
          new Multiaddr(`/ip4/127.0.0.1/tcp/124/p2p/${fixtures.PARTY_A.toString()}`),
          new BN('1')
        )
      )
    },
    getPublicKey: () => fixtures.PARTY_A,
    setCommitment: (counterparty: Address, commitment: Hash) =>
      hoprChannels.bumpChannel(counterparty.toHex(), commitment.toHex()),
    getAllQueuingTransactionRequests: () => [txRequest],
    getAllUnconfirmedHash: () => [fixtures.OPENED_EVENT.transactionHash]
  } as unknown as ChainWrapper
}

export class TestingIndexer extends Indexer {
  public restart(): Promise<void> {
    return super.restart()
  }
}

export const useFixtures = async (
  ops: {
    latestBlockNumber?: number
    pastEvents?: Event<any>[]
    pastHoprTokenEvents?: TokenEvent<any>[]
    pastHoprRegistryEvents?: RegistryEvent<any>[]
    id?: PublicKey
  } = {}
) => {
  const latestBlockNumber = ops.latestBlockNumber ?? 0

  const db = HoprDB.createMock(ops.id)
  const { provider, newBlock } = createProviderMock({ latestBlockNumber })
  const { hoprChannels, newEvent } = createHoprChannelsMock({ pastEvents: ops.pastEvents ?? [] })
  const { hoprToken, newEvent: newTokenEvent } = createHoprTokenMock({
    pastEvents: ops.pastHoprTokenEvents ?? []
  })
  const { hoprRegistry, newEvent: newRegistryEvent } = createHoprRegistryMock({
    pastEvents: ops.pastHoprRegistryEvents ?? []
  })
  const chain = createChainMock(provider, hoprChannels, hoprToken, hoprRegistry)

  return {
    db,
    provider,
    newBlock,
    hoprChannels,
    hoprToken,
    hoprRegistry,
    newEvent,
    newTokenEvent,
    newRegistryEvent,
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
