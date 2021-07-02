import SimplePeer from 'simple-peer'
import debug from 'debug'

import type { Multiaddr } from 'multiaddr'

const wrtc = require('wrtc')

const error = debug('hopr-connect:error')
const verbose = debug('hopr-connect:verbose:webrtc')

/**
 * Encapsulate configuration used to create WebRTC instances
 */
class WebRTCUpgrader {
  private _stunServers?: {
    iceServers?: {
      urls: string
    }[]
  }
  constructor(opts: { stunServers?: Multiaddr[] }) {
    this._stunServers = {
      iceServers: opts.stunServers?.map((ma: Multiaddr) => {
        const options = ma.toOptions()

        return { urls: `stun:${options.host}:${options.port}` }
      })
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
      config: this._stunServers
    })

    const onAbort = () => {
      channel.destroy()
      verbose('abort')
    }

    const done = async (err?: Error) => {
      channel.removeListener('connect', done)
      // do not remove error listener
      //channel.removeListener('error', done)

      signal?.removeEventListener('abort', onAbort)

      if (err) {
        error(`WebRTC connection update failed. Error was: ${err}`)
        channel.destroy()
      } else {
        verbose('WebRTC execution completed')
      }
    }

    channel.on('error', done)
    channel.once('connect', done)

    signal?.addEventListener('abort', onAbort)

    return channel
  }
}

export { WebRTCUpgrader }
