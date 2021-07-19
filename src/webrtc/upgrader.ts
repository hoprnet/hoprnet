import SimplePeer from 'simple-peer'
import debug from 'debug'

import type { Multiaddr } from 'multiaddr'
import type { PublicNodesEmitter } from '../types'
import { CODE_IP4, CODE_TCP, CODE_UDP } from '../constants'

const wrtc = require('wrtc')

const DEBUG_PREFIX = `hopr-connect:webrtc`

const error = debug(DEBUG_PREFIX.concat(':error'))
const verbose = debug(DEBUG_PREFIX.concat(':verbose'))

export function multiaddrToIceServer(ma: Multiaddr): string {
  const options = ma.toOptions()

  return `stun:${options.host}:${options.port}`
}

// @TODO adjust this
export const MAX_STUN_SERVERS = 23

/**
 * Encapsulate configuration used to create WebRTC instances
 */
class WebRTCUpgrader {
  public rtcConfig?: RTCConfiguration

  constructor(publicNodes?: PublicNodesEmitter, initialNodes?: Multiaddr[]) {
    initialNodes?.forEach(this.onNewPublicNode.bind(this))

    publicNodes?.on('addPublicNode', this.onNewPublicNode.bind(this))
  }

  onNewPublicNode(ma: Multiaddr) {
    if (
      this.rtcConfig != undefined &&
      this.rtcConfig.iceServers != undefined &&
      this.rtcConfig.iceServers.length >= MAX_STUN_SERVERS
    ) {
      return
    }

    const tuples = ma.tuples()

    // Also try "TCP addresses" as we expect that node is listening on TCP *and* UDP
    if (tuples[0].length < 2 || tuples[0][0] != CODE_IP4 || ![CODE_UDP, CODE_TCP].includes(tuples[1][0])) {
      verbose(`Dropping potential STUN ${ma.toString()} because format is invalid`)
      return
    }

    const iceServerUrl = multiaddrToIceServer(ma)

    const iceServers = (this.rtcConfig?.iceServers ?? []).filter((iceServer: RTCIceServer) => {
      if (Array.isArray(iceServer.urls)) {
        return !iceServer.urls.some((url: string) => iceServerUrl === url)
      }

      return iceServer.urls !== iceServerUrl
    })

    iceServers.unshift({ urls: iceServerUrl })

    this.rtcConfig = {
      ...this.rtcConfig,
      iceServers
    }
  }

  /**
   * Creates an outbound instance of WebRTC
   * @param _signal @TODO implement this
   * @returns the WebRTC instance
   */
  upgradeOutbound(_signal?: AbortSignal) {
    return this._connect(true)
  }

  /**
   * Creates an inbound instance of WebRTC
   * @param _signal @TODO implement this
   * @returns the WebRTC instance
   */
  upgradeInbound(_signal?: AbortSignal) {
    return this._connect(false)
  }

  /**
   * Creates a configured WebRTC
   * @param initiator true if initiator
   * @param signal abort signal
   * @returns a configured WebRTC instance
   */
  private _connect(initiator: boolean, signal?: AbortSignal) {
    const channel = new SimplePeer({
      wrtc,
      initiator,
      trickle: true,
      // @ts-ignore
      allowHalfTrickle: true,
      config: this.rtcConfig
    })

    const onAbort = () => {
      channel.destroy()
      verbose('abort')
    }

    const done = async (err?: Error) => {
      channel.removeListener('connect', done)
      channel.removeListener('error', done)

      signal?.removeEventListener('abort', onAbort)

      if (err) {
        error(`WebRTC connection update failed. Error was: ${err}`)
        channel.destroy()
      } else {
        verbose('WebRTC execution completed')
      }
    }

    channel.once('error', done)
    channel.once('connect', done)

    signal?.addEventListener('abort', onAbort)

    return channel
  }
}

export { WebRTCUpgrader }
