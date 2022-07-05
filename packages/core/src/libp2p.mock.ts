import { PeerId } from '@libp2p/interface-peer-id'
import { Multiaddr } from '@multiformats/multiaddr'
import type { StreamHandler } from '@libp2p/interface-registrar'
import type { Connection } from '@libp2p/interface-connection'
import { PersistentPeerStore } from '@libp2p/peer-store'
import type { Components } from '@libp2p/interfaces/components'
import { EventEmitter } from '@libp2p/interfaces/events'

import type { Duplex } from 'it-stream-types'

import type { ContentRouting } from '@libp2p/interface-content-routing'
import type { PeerInfo } from '@libp2p/interface-peer-info'
import { CID } from 'multiformats/cid'
import { duplexPair } from 'it-pair/duplex'
import { setImmediate } from 'timers/promises'

import { MemoryDatastore } from 'datastore-core/memory'

import type { Libp2p } from 'libp2p'
import { peerIdFromString } from '@libp2p/peer-id'

interface FakeIncomingStream {
  peer: PeerId
  stream: Duplex<Uint8Array>
}

function createFakeDHT(peer: PeerId, dht: Map<string, string[]>) {
  // Map: relayToken -> peerId
  const fakeDHT = dht ?? new Map<string, string[]>()

  const provide = async (relayToken: CID) => {
    const entries: string[] = fakeDHT.get(relayToken.toString()) ?? []

    if (!entries.includes(peer.toString())) {
      entries.push(peer.toString())
    }

    // Make call asynchronous
    await setImmediate()

    fakeDHT.set(relayToken.toString(), entries)
  }

  const findProviders = async function* (relayToken: CID): AsyncIterable<PeerInfo> {
    const entries = (fakeDHT.get(relayToken.toString()) ?? []).map((idString) => ({
      id: peerIdFromString(idString),
      multiaddrs: [],
      protocols: []
    }))

    // Make call asynchronous
    await setImmediate()

    return entries
  }

  return {
    dht: fakeDHT,
    interface: {
      provide,
      findProviders
    } as ContentRouting
  }
}

function createFakeNetwork(peer: PeerId, network: EventEmitter<any>) {
  const events = network ?? new EventEmitter()

  function protocolName(protocol: string, destination: PeerId) {
    return `${destination.toString()}-${protocol}`
  }

  const handle: Libp2p['handle'] = async (protocol: string, handler: StreamHandler) => {
    await setImmediate()

    events.addEventListener(protocolName(protocol, peer), (event: CustomEvent<FakeIncomingStream>) => {
      handler({
        protocol,
        stream: event.detail.stream as any,
        connection: { remotePeer: event.detail.peer } as Connection
      })
    })
  }

  const dial: Libp2p['dial'] = async (destination: Multiaddr | PeerId) => {
    let destPeerId: PeerId

    if (Multiaddr.isMultiaddr(destination)) {
      destPeerId = peerIdFromString(destination.getPeerId())
    } else {
      destPeerId = destination
    }

    // Make call asynchronous
    await setImmediate()

    return {
      // remotePeer: destPeerId
      async newStream(protocol: string) {
        await setImmediate()
        const stream = duplexPair<Uint8Array>()
        events.dispatchEvent(
          new CustomEvent<FakeIncomingStream>(protocolName(protocol, destPeerId), {
            detail: {
              peer,
              stream: {
                sink: stream[0].sink,
                source: stream[1].source
              }
            }
          })
        )

        return {
          protocol,
          stream: {
            sink: stream[1].sink,
            source: stream[0].source
          }
        }
      }
    } as Connection
  }

  return {
    events,
    dial,
    handle
  }
}

function createLibp2pMock(peer: PeerId, shared?: { dht?: Map<string, string[]>; network?: EventEmitter<any> }) {
  const network = createFakeNetwork(peer, shared?.network)
  const dht = createFakeDHT(peer, shared?.dht)
  const datastore = new MemoryDatastore()
  const peerStore = new PersistentPeerStore()

  const libp2p = {
    components: {
      getContenRouting(): ContentRouting {
        return dht.interface
      },
      getDht() {
        return {
          [Symbol.toStringTag]: 'some nice DHT implementation'
        }
      },
      getRegistrar() {
        return {
          handle: network.handle
        }
      },
      getConnectionManager() {
        return Object.assign(new EventEmitter(), {
          getConnections(_peer: PeerId) {
            return []
          },
          dialer: {
            dial: network.dial
          }
        })
      },
      getDatastore() {
        return datastore
      },
      getPeerStore() {
        return peerStore
      },
      getAddressManager() {
        return {
          getAddresses() {
            return [new Multiaddr(`/ip4/127.0.0.1/tcp/123/p2p/${peer.toString()}`)]
          }
        }
      }
    },
    async start() {
      await setImmediate()
      Promise.resolve()
    },
    async stop() {
      await setImmediate()
      Promise.resolve()
    }
  }

  peerStore.init(libp2p.components as any as Components)

  return libp2p as any as Libp2p
}

export { createLibp2pMock }
