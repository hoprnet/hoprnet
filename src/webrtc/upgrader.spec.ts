import { EventEmitter } from 'events'
import { Multiaddr } from 'multiaddr'

import assert from 'assert'

import { MAX_STUN_SERVERS, multiaddrToIceServer, WebRTCUpgrader } from './upgrader'
import { PublicNodesEmitter } from '../types'

describe('webrtc upgrader', function () {
  it('add public nodes', async function () {
    const publicNodeEmitter = new EventEmitter() as PublicNodesEmitter

    const webRTCUpgrader = new WebRTCUpgrader(publicNodeEmitter)

    const testMultiaddr = new Multiaddr(`/ip4/1.2.3.4/udp/12345`)

    publicNodeEmitter.emit(`addPublicNode`, testMultiaddr)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(
      webRTCUpgrader.rtcConfig?.iceServers?.length == 1 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(testMultiaddr)
    )

    const secondTestMultiaddr = new Multiaddr(`/ip4/1.2.3.5/udp/12345`)

    publicNodeEmitter.emit(`addPublicNode`, secondTestMultiaddr)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(
      (webRTCUpgrader.rtcConfig?.iceServers?.length as any) == 2 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(secondTestMultiaddr) &&
        webRTCUpgrader.rtcConfig.iceServers[1].urls === multiaddrToIceServer(testMultiaddr)
    )
  })

  it('add public nodes more than once', async function () {
    const publicNodeEmitter = new EventEmitter() as PublicNodesEmitter

    const webRTCUpgrader = new WebRTCUpgrader(publicNodeEmitter)

    const testMultiaddr = new Multiaddr(`/ip4/1.2.3.4/udp/12345`)

    publicNodeEmitter.emit(`addPublicNode`, testMultiaddr)
    publicNodeEmitter.emit(`addPublicNode`, testMultiaddr)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(
      webRTCUpgrader.rtcConfig?.iceServers?.length == 1 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(testMultiaddr)
    )
  })

  it('add public nodes to initial nodes', async function () {
    const publicNodeEmitter = new EventEmitter() as PublicNodesEmitter

    const initialMultiaddr = new Multiaddr(`/ip4/1.2.3.4/udp/12345`)

    const webRTCUpgrader = new WebRTCUpgrader(publicNodeEmitter, [initialMultiaddr])

    assert(
      webRTCUpgrader.rtcConfig?.iceServers?.length == 1 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(initialMultiaddr)
    )

    const nextMultiaddr = new Multiaddr(`/ip4/1.2.3.5/udp/12345`)

    publicNodeEmitter.emit(`addPublicNode`, nextMultiaddr)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(
      (webRTCUpgrader.rtcConfig?.iceServers?.length as any) == 2 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(nextMultiaddr) &&
        webRTCUpgrader.rtcConfig.iceServers[1].urls === multiaddrToIceServer(initialMultiaddr)
    )
  })

  it('add public nodes - edge cases', async function () {
    const publicNodeEmitter = new EventEmitter() as PublicNodesEmitter

    const webRTCUpgrader = new WebRTCUpgrader(publicNodeEmitter)

    const invalidMultiaddr = new Multiaddr(`/ip4/1.2.3.4/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg`)

    publicNodeEmitter.emit(`addPublicNode`, invalidMultiaddr)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(webRTCUpgrader.rtcConfig?.iceServers == undefined)

    const secondInvalidMultiaddr = new Multiaddr(`/ip6/::/udp/12345`)

    publicNodeEmitter.emit(`addPublicNode`, secondInvalidMultiaddr)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(webRTCUpgrader.rtcConfig?.iceServers == undefined)
  })

  it('limit available STUN servers', async function () {
    const publicNodeEmitter = new EventEmitter() as PublicNodesEmitter

    const webRTCUpgrader = new WebRTCUpgrader(publicNodeEmitter)

    for (let i = 0; i <= MAX_STUN_SERVERS; i++) {
      const multiaddr = new Multiaddr(`/ip4/1.2.3.4/udp/${i + 1}`)

      publicNodeEmitter.emit(`addPublicNode`, multiaddr)

      if (i < MAX_STUN_SERVERS) {
        assert(
          webRTCUpgrader.rtcConfig?.iceServers?.length == i + 1 &&
            webRTCUpgrader.rtcConfig.iceServers[0].urls == multiaddrToIceServer(multiaddr)
        )
      }
    }

    assert(webRTCUpgrader.rtcConfig?.iceServers?.length == MAX_STUN_SERVERS)
  })

  it('remove offline STUN servers', async function () {
    const publicNodeEmitter = new EventEmitter() as PublicNodesEmitter

    const webRTCUpgrader = new WebRTCUpgrader(publicNodeEmitter)

    const ATTEMPTS = Math.min(MAX_STUN_SERVERS, 3)

    const multiaddrs: Multiaddr[] = []
    for (let i = 0; i < ATTEMPTS; i++) {
      const multiaddr = new Multiaddr(`/ip4/1.2.3.4/udp/${i}`)
      multiaddrs.push(multiaddr)

      publicNodeEmitter.emit(`addPublicNode`, multiaddr)

      assert(
        webRTCUpgrader.rtcConfig?.iceServers?.length == i + 1 &&
          webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(multiaddr)
      )
    }

    for (let i = 0; i < ATTEMPTS; i++) {
      publicNodeEmitter.emit(`removePublicNode`, multiaddrs[i])

      assert((webRTCUpgrader.rtcConfig?.iceServers?.length as any) == ATTEMPTS - i - 1)
    }

    assert((webRTCUpgrader.rtcConfig?.iceServers?.length as any) == 0)
  })

  it('remove offline STUN servers - edge cases', async function () {
    const publicNodeEmitter = new EventEmitter() as PublicNodesEmitter

    const webRTCUpgrader = new WebRTCUpgrader(publicNodeEmitter)

    const multiaddr = new Multiaddr(`/ip4/1.2.3.4/udp/123`)

    publicNodeEmitter.emit(`removePublicNode`, multiaddr)

    assert((webRTCUpgrader.rtcConfig?.iceServers?.length as any) == undefined)
  })
})
