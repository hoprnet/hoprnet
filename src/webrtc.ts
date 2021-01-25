import SimplePeer from 'simple-peer'
import debug from 'debug'

import type Multiaddr from 'multiaddr'

const wrtc = require('wrtc')

const error = debug('hopr-connect:error')
const verbose = debug('hopr-connect:verbose:webrtc')

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

  upgradeOutbound(_signal?: AbortSignal) {
    return this._connect(true)
  }

  upgradeInbound(_signal?: AbortSignal) {
    return this._connect(false)
  }

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
