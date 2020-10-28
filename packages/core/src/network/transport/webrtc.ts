//import { AbortError } from 'abortable-iterator'
import SimplePeer from 'simple-peer'
import debug from 'debug'
// @ts-ignore
import wrtc = require('wrtc')
//import { WEBRTC_TIMEOUT } from './constants'
import type Multiaddr from 'multiaddr'

const log = debug('hopr-core:transport')
//const error = debug('hopr-core:transport:error')
//const verbose = debug('hopr-core:verbose:transport:webrtc')

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

  private _connect(initiator: boolean) {
    log(`inside _connect`)
    const channel = new SimplePeer({
      wrtc,
      initiator,
      trickle: true,
      // @ts-ignore
      allowHalfTrickle: true,
      config: this._stunServers
    })

    // const onTimeout = () => {
    //   verbose('Timeout upgrading to webrtc', channel.address())
    //   channel.destroy()
    // }

    // const done = async (err?: Error) => {
    //   verbose('Completed')

    //   channel.removeListener('iceTimeout', onTimeout)
    //   channel.removeListener('connect', onConnect)
    //   channel.removeListener('error', onError)

    //   //options?.signal?.removeEventListener('abort', onAbort)

    //   if (err) {
    //     verbose('Failed', err)
    //     channel.destroy()
    //   }
    // }

    // // const onAbort = () => {
    // //   channel.destroy()
    // //   verbose('abort')
    // // }

    // const onConnect = (): void => {
    //   verbose('connected')
    //   done()
    // }

    // const onError = (err?: Error) => {
    //   log(`WebRTC with failed. Error was: ${err}`)
    //   done(err)
    // }

    // channel.once('error', onError)
    // channel.once('connect', onConnect)
    // channel.once('iceTimeout', onTimeout)

    // options?.signal?.addEventListener('abort', onAbort)

    return channel
  }
}

export { WebRTCUpgrader }
