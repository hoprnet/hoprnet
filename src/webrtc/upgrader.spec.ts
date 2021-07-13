import { EventEmitter } from 'events'
import { Multiaddr } from 'multiaddr'

import assert from 'assert'

import { multiaddrToIceServer, WebRTCUpgrader } from './upgrader'

describe('webrtc upgrader', function () {
  it('add public nodes', async function () {
    const publicNodeEmitter = new EventEmitter()

    const webRTCUpgrader = new WebRTCUpgrader(publicNodeEmitter)

    const testMultiaddr = new Multiaddr(`/ip4/1.2.3.4/udp/12345`)

    publicNodeEmitter.emit(`publicNode`, testMultiaddr)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(
      webRTCUpgrader.rtcConfig?.iceServers?.length == 1 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(testMultiaddr)
    )

    const secondTestMultiaddr = new Multiaddr(`/ip4/1.2.3.5/udp/12345`)

    publicNodeEmitter.emit(`publicNode`, secondTestMultiaddr)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(
      (webRTCUpgrader.rtcConfig?.iceServers?.length as any) == 2 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(secondTestMultiaddr) &&
        webRTCUpgrader.rtcConfig.iceServers[1].urls === multiaddrToIceServer(testMultiaddr)
    )
  })

  it('add public nodes more than once', async function () {
    const publicNodeEmitter = new EventEmitter()

    const webRTCUpgrader = new WebRTCUpgrader(publicNodeEmitter)

    const testMultiaddr = new Multiaddr(`/ip4/1.2.3.4/udp/12345`)

    publicNodeEmitter.emit(`publicNode`, testMultiaddr)
    publicNodeEmitter.emit(`publicNode`, testMultiaddr)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(
      webRTCUpgrader.rtcConfig?.iceServers?.length == 1 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(testMultiaddr)
    )
  })

  it('add public nodes to initial nodes', async function () {
    const publicNodeEmitter = new EventEmitter()

    const initialMultiaddr = new Multiaddr(`/ip4/1.2.3.4/udp/12345`)

    const webRTCUpgrader = new WebRTCUpgrader(publicNodeEmitter, [initialMultiaddr])

    assert(
      webRTCUpgrader.rtcConfig?.iceServers?.length == 1 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(initialMultiaddr)
    )

    const nextMultiaddr = new Multiaddr(`/ip4/1.2.3.5/udp/12345`)

    publicNodeEmitter.emit(`publicNode`, nextMultiaddr)

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(
      (webRTCUpgrader.rtcConfig?.iceServers?.length as any) == 2 &&
        webRTCUpgrader.rtcConfig.iceServers[0].urls === multiaddrToIceServer(nextMultiaddr) &&
        webRTCUpgrader.rtcConfig.iceServers[1].urls === multiaddrToIceServer(initialMultiaddr)
    )
  })
})
