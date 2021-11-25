import { AccountEntry, debug, NativeBalance } from '@hoprnet/hopr-utils'
import HoprCoreEthereum, { Indexer } from '@hoprnet/hopr-core-ethereum'
import sinon from 'sinon'
import BN from 'bn.js'
import { NAMESPACE, sampleAddress, sampleMultiaddrs } from './constants'

const chainLogger = debug(`${NAMESPACE}:chain`)

let indexer: Indexer
let chain: HoprCoreEthereum

chain = sinon.createStubInstance(HoprCoreEthereum)
chain.indexer = indexer

// @TODO: Use better (ie typed) way to overload stub
chain.start = sinon.fake(() => {
  chainLogger('On-chain instance start method was called.')
  return {
    getNativeBalance: () => {
      chainLogger('getNativeBalance method was called')
      // Adding a value of >0 to register node has been funded.
      // @TODO: Pick a more relevant value.
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
  }
})

const chainMock = chain
export { chainMock }
