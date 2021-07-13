import SimplePeer from 'simple-peer'
import debug from 'debug'

import type { Multiaddr } from 'multiaddr'
import type { EventEmitter } from 'events'

const wrtc = require('wrtc')

const DEBUG_PREFIX = `hopr-connect:webrtc`

const error = debug(DEBUG_PREFIX.concat(':error'))
const verbose = debug(DEBUG_PREFIX.concat(':verbose'))

export function multiaddrToIceServer(ma: Multiaddr): string {
  const options = ma.toOptions()

  return `stun:${options.host}:${options.port}`
}

/**
 * Encapsulate configuration used to create WebRTC instances
 */
class WebRTCUpgrader {
  public rtcConfig?: RTCConfiguration

  constructor(publicNodes?: EventEmitter, initialNodes?: Multiaddr[]) {
    initialNodes?.forEach(this.onNewPublicNode.bind(this))

    publicNodes?.on('publicNode', this.onNewPublicNode.bind(this))
  }

  onNewPublicNode(ma: Multiaddr) {
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
