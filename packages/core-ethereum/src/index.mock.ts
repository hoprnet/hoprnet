import { AccountEntry, Address, debug, NativeBalance } from '@hoprnet/hopr-utils'
import HoprCoreEthereum, { Indexer } from '.'
import BN from 'bn.js'
import PeerId from 'peer-id'
import { Multiaddr } from 'multiaddr'

export const sampleAddress = Address.fromString('0x55CfF15a5159239002D57C591eF4ACA7f2ACAfE6')
export const samplePeerId = PeerId.createFromB58String('16Uiu2HAmThyWP5YWutPmYk9yUZ48ryWyZ7Cf6pMTQduvHUS9sGE7')
export const sampleMultiaddrs = new Multiaddr(`/ip4/127.0.0.1/tcp/124/p2p/${samplePeerId.toB58String()}`)

const chainLogger = debug(`hopr:mocks:chain`)
const chainMock = {} as unknown as HoprCoreEthereum
chainMock.indexer = {} as unknown as Indexer
chainMock.stop = () => {
  chainLogger('On-chain stop instance method was called.')
  return Promise.resolve()
}
chainMock.start = () => {
  chainLogger('On-chain instance start method was called.')
  return Promise.resolve({
    getNativeBalance: () => {
      chainLogger('getNativeBalance method was called')
      return Promise.resolve(new NativeBalance(new BN('10000000000000000000')))
    },
    getPublicKey: () => {
      chainLogger('getPublicKey method was called')
      return {
        toAddress: () => Promise.resolve(sampleAddress)
      }
    },
    getAccount: () => {
      chainLogger('getAccount method was called')
      return Promise.resolve(new AccountEntry(sampleAddress, sampleMultiaddrs, new BN('1')))
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
    }
  } as unknown as HoprCoreEthereum)
}

export { chainMock }
