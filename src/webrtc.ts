import SimplePeer from 'simple-peer'
import debug from 'debug'

// @ts-ignore
import wrtc = require('wrtc')
import type Multiaddr from 'multiaddr'

const error = debug('hopr-connect:error')
const verbose = debug('hopr-connect:verbose:webrtc')

class WebRTCUpgrader {
  private _stunServers?: {
    iceServers?: {
      urls: string
    }[]
  }
  constructor({ stunServers }: { stunServers?: Multiaddr[] }) {
    this._stunServers = {
      iceServers: stunServers?.map((ma: Multiaddr) => {
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
      channel.removeListener('error', done)

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
