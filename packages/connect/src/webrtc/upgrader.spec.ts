import { EventEmitter } from 'events'
import { Multiaddr } from 'multiaddr'

import assert from 'assert'

import { MAX_STUN_SERVERS, multiaddrToIceServer, WebRTCUpgrader } from './upgrader'
import type { PublicNodesEmitter, PeerStoreType } from '../types'
import { createPeerId } from '../base/utils.spec'
import type PeerId from 'peer-id'

async function getPeerStoreEntry(addr: string): Promise<PeerStoreType> {
  return {
    id: createPeerId(),
    multiaddrs: [new Multiaddr(addr)]
  }
}

describe('webrtc upgrader', function () {
  it('add public nodes', async function () {
    const publicNodes = new EventEmitter() as PublicNodesEmitter

    const webRTCUpgrader = new WebRTCUpgrader({ publicNodes })

    webRTCUpgrader.start()

    const testPeer = await getPeerStoreEntry(`/ip4/1.2.3.4/udp/12345`)

    publicNodes.emit(`addPublicNode`, testPeer)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(
      webRTCUpgrader.rtcConfig?.iceServers?.length == 1 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(testPeer.multiaddrs[0])
    )

    const secondPeer = await getPeerStoreEntry(`/ip4/1.2.3.5/udp/12345`)

    publicNodes.emit(`addPublicNode`, secondPeer)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(
      (webRTCUpgrader.rtcConfig?.iceServers?.length as any) == 2 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(secondPeer.multiaddrs[0]) &&
        webRTCUpgrader.rtcConfig.iceServers[1].urls === multiaddrToIceServer(testPeer.multiaddrs[0])
    )

    webRTCUpgrader.stop()
  })

  it('add public nodes more than once', async function () {
    const publicNodes = new EventEmitter() as PublicNodesEmitter

    const webRTCUpgrader = new WebRTCUpgrader({ publicNodes })

    webRTCUpgrader.start()

    const testPeer = await getPeerStoreEntry(`/ip4/1.2.3.4/udp/12345`)

    publicNodes.emit(`addPublicNode`, testPeer)
    publicNodes.emit(`addPublicNode`, testPeer)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(
      webRTCUpgrader.rtcConfig?.iceServers?.length == 1 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(testPeer.multiaddrs[0])
    )

    webRTCUpgrader.stop()
  })

  it('add public nodes to initial nodes', async function () {
    const publicNodes = new EventEmitter() as PublicNodesEmitter

    const initialPeer = await getPeerStoreEntry(`/ip4/1.2.3.4/udp/12345`)

    const webRTCUpgrader = new WebRTCUpgrader({ publicNodes, initialNodes: [initialPeer] })

    webRTCUpgrader.start()

    assert(
      webRTCUpgrader.rtcConfig?.iceServers?.length == 1 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(initialPeer.multiaddrs[0])
    )

    const nextPeer = await getPeerStoreEntry(`/ip4/1.2.3.5/udp/12345`)

    publicNodes.emit(`addPublicNode`, nextPeer)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(
      (webRTCUpgrader.rtcConfig?.iceServers?.length as any) == 2 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(nextPeer.multiaddrs[0]) &&
        webRTCUpgrader.rtcConfig.iceServers[1].urls === multiaddrToIceServer(initialPeer.multiaddrs[0])
    )

    webRTCUpgrader.stop()
  })

  it('add public nodes - edge cases', async function () {
    const publicNodes = new EventEmitter() as PublicNodesEmitter

    const webRTCUpgrader = new WebRTCUpgrader({ publicNodes })

    webRTCUpgrader.start()

    const peerId = createPeerId()
    const invalidMultiaddr = new Multiaddr(`/ip4/1.2.3.4/p2p/${peerId.toB58String()}`)

    publicNodes.emit(`addPublicNode`, { id: peerId, multiaddrs: [invalidMultiaddr] })

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(webRTCUpgrader.rtcConfig?.iceServers?.length == 0)

    const secondInvalidMultiaddr = new Multiaddr(`/ip6/::/udp/12345`)

    publicNodes.emit(`addPublicNode`, { id: peerId, multiaddrs: [secondInvalidMultiaddr] })

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(webRTCUpgrader.rtcConfig?.iceServers.length == 0)

    webRTCUpgrader.stop()
  })

  it('limit available STUN servers', async function () {
    const publicNodes = new EventEmitter() as PublicNodesEmitter

    const webRTCUpgrader = new WebRTCUpgrader({ publicNodes })

    webRTCUpgrader.start()

    for (let i = 0; i <= MAX_STUN_SERVERS; i++) {
      const peer = await getPeerStoreEntry(`/ip4/1.2.3.4/udp/${i + 1}`)

      publicNodes.emit(`addPublicNode`, peer)

      if (i < MAX_STUN_SERVERS) {
        assert(
          webRTCUpgrader.rtcConfig?.iceServers?.length == i + 1 &&
            webRTCUpgrader.rtcConfig.iceServers[0].urls == multiaddrToIceServer(peer.multiaddrs[0])
        )
      }
    }

    assert(webRTCUpgrader.rtcConfig?.iceServers?.length == MAX_STUN_SERVERS)

    webRTCUpgrader.stop()
  })

  it('remove offline STUN servers', async function () {
    const publicNodes = new EventEmitter() as PublicNodesEmitter

    const webRTCUpgrader = new WebRTCUpgrader({ publicNodes })

    webRTCUpgrader.start()

    const ATTEMPTS = Math.min(MAX_STUN_SERVERS, 3)

    const peerIds: PeerId[] = []
    for (let i = 0; i < ATTEMPTS; i++) {
      const peerId = createPeerId()
      const multiaddr = new Multiaddr(`/ip4/1.2.3.4/udp/${i}/p2p/${peerId.toB58String()}`)
      peerIds.push(peerId)

      publicNodes.emit(`addPublicNode`, { id: peerId, multiaddrs: [multiaddr] })

      assert(
        webRTCUpgrader.rtcConfig?.iceServers?.length == i + 1 &&
          webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(multiaddr)
      )
    }

    for (let i = 0; i < ATTEMPTS; i++) {
      publicNodes.emit(`removePublicNode`, peerIds[i])

      assert((webRTCUpgrader.rtcConfig?.iceServers?.length as any) == ATTEMPTS - i - 1)
    }

    assert((webRTCUpgrader.rtcConfig?.iceServers?.length as any) == 0)

    webRTCUpgrader.stop()
  })

  it('remove offline STUN servers - edge cases', function () {
    const publicNodes = new EventEmitter() as PublicNodesEmitter

    const webRTCUpgrader = new WebRTCUpgrader({ publicNodes })

    webRTCUpgrader.start()

    const peerId = createPeerId()

    publicNodes.emit(`removePublicNode`, peerId)

    assert((webRTCUpgrader.rtcConfig?.iceServers?.length as any) == undefined)

    webRTCUpgrader.stop()
  })
})
