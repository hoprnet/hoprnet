import { AccountEntry, debug, NativeBalance, PublicKey } from '@hoprnet/hopr-utils'
import type HoprCoreEthereum from '.'
import BN from 'bn.js'
import PeerId from 'peer-id'
import { Multiaddr } from 'multiaddr'

const connectorLogger = debug(`hopr:mocks:connector`)
function createConnectorMock(peer: PeerId): HoprCoreEthereum {
  return {
    start: () => {
      connectorLogger('starting connector called.')
      return {} as unknown as HoprCoreEthereum
    },
    stop: () => {
      connectorLogger('stopping connector called.')
      return Promise.resolve()
    },
    getNativeBalance: () => {
      connectorLogger('getNativeBalance method was called')
      return Promise.resolve(new NativeBalance(new BN('10000000000000000000')))
    },
    getPublicKey: () => {
      connectorLogger('getPublicKey method was called')
      return PublicKey.fromPeerId(peer)
    },
    getAccount: () => {
      connectorLogger('getAccount method was called')
      return Promise.resolve(
        new AccountEntry(
          PublicKey.fromPeerId(peer).toAddress(),
          new Multiaddr(`/ip4/127.0.0.1/tcp/124/p2p/${peer.toB58String()}`),
          new BN('1')
        )
      )
    },
    waitForPublicNodes: () => {
      connectorLogger('On-chain request for existing public nodes.')
      return Promise.resolve([])
    },
    announce: () => {
      connectorLogger('On-chain announce request sent')
    },
    on: (event: string) => {
      connectorLogger(`On-chain signal for event "${event}"`)
    },
    indexer: {
      on: (event: string) => connectorLogger(`Indexer on handler top of chain called with event "${event}"`),
      off: (event: string) => connectorLogger(`Indexer off handler top of chain called with event "${event}`)
    }
  } as unknown as HoprCoreEthereum
}
export { createConnectorMock }
