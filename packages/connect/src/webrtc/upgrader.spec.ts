import { Multiaddr } from '@multiformats/multiaddr'
import { setTimeout } from 'timers/promises'

// import assert from 'assert'

import { WebRTCUpgrader } from './upgrader.js'
import { EntryNodes, EntryNodeData } from '../entry.js'
import type { PeerStoreType } from '../types.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import { startStunServer, createPeerId } from '../base/utils.spec.js'
import { ConnectComponents } from '../components.js'
import { once } from 'events'
// import { u8aEquals } from '@hoprnet/hopr-utils'

function getFakeConnectComponents(
  lastUpdate: number,
  availableEntryNodes: EntryNodeData[] = [],
  uncheckedEntryNodes: PeerStoreType[] = []
): ConnectComponents {
  return {
    getEntryNodes() {
      return {
        lastUpdate,
        getAvailableEntryNodes() {
          console.log(`returning`)
          return availableEntryNodes.values()
        },
        getUncheckedEntryNodes() {
          return uncheckedEntryNodes.values()
        }
      } as any as EntryNodes
    }
  } as ConnectComponents
}

function getPeerStoreEntry(ip: string, port: number, id: PeerId = createPeerId()): PeerStoreType {
  return {
    id,
    multiaddrs: [new Multiaddr(`/ip4/${ip}/tcp/${port}/p2p/${id.toString()}`)]
  }
}

describe.only('webrtc upgrader', function () {
  it('base functionality', async function () {
    this.timeout(10e3)
    // If this test fails, either simple-peer library or WebRTC binary is broken
    const stunServer = await startStunServer()

    const initiator = new WebRTCUpgrader()
    const receiver = new WebRTCUpgrader()

    const fakedComponents = getFakeConnectComponents(-1, undefined, [
      getPeerStoreEntry('127.0.0.1', stunServer.address().port)
    ])

    initiator.initConnect(fakedComponents)
    receiver.initConnect(fakedComponents)

    const initiatorPeer = initiator.upgradeOutbound()
    const receiverPeer = receiver.upgradeInbound()

    const initiatorConnect = once(initiatorPeer, 'connect')
    const receiverConnect = once(receiverPeer, 'connect')

    initiatorPeer.on('signal', receiverPeer.signal.bind(receiverPeer))
    receiverPeer.on('signal', initiatorPeer.signal.bind(initiatorPeer))

    await Promise.all([initiatorConnect, receiverConnect])

    // const pingInitiator = new TextEncoder().encode('PING initiator')
    // const pingReceiver = new TextEncoder().encode('PING receiver')

    // const pongInitiator = new TextEncoder().encode('PONG initiator')
    // const pongReceiver = new TextEncoder().encode('PONG receiver')

    // initiatorPeer.write(pingInitiator)
    // receiverPeer.write(pingReceiver)

    // const initiatorIterator = initiatorPeer[Symbol.asyncIterator]()
    // const receiverIterator = receiverPeer[Symbol.asyncIterator]()

    // assert(u8aEquals((await initiatorIterator.next()).value, pingReceiver))
    // assert(u8aEquals((await receiverIterator.next()).value, pingInitiator))

    // initiatorPeer.write(pongInitiator)
    // receiverPeer.write(pongReceiver)

    // assert(u8aEquals((await initiatorIterator.next()).value, pongReceiver))
    // assert(u8aEquals((await receiverIterator.next()).value, pongInitiator))

    // await new Promise<void>((resolve) => initiatorPeer.end(resolve))
    // await new Promise<void>((resolve) => receiverPeer.end(resolve))

    stunServer.close()

    initiatorPeer.destroy()
    receiverPeer.destroy()

    await setTimeout(4e3)

    await setTimeout(4e3)
  })
})
