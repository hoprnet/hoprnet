import type { Pushable } from 'it-pushable'
import abortable, { AbortError } from 'abortable-iterator'

import type { Socket } from 'net'

import Peer, { Options as SimplePeerOptions } from 'simple-peer'

import debug from 'debug'
const log = debug('hopr-core:transport')

import pipe from 'it-pipe'

// @ts-ignore
import wrtc = require('wrtc')

const _encoder = new TextEncoder()
const _decoder = new TextDecoder()

export default function upgradetoWebRTC(
  sinkBuffer: Pushable<Uint8Array>,
  srcBuffer: Pushable<Uint8Array>,
  options?: {
    initiator?: boolean
    signal?: AbortSignal
  }
): Promise<Socket> {
  if (options?.signal?.aborted) {
    throw new AbortError()
  }

  return new Promise<Socket>(async (resolve, reject) => {
    let webRTCconfig: SimplePeerOptions

    // if (this._useOwnStunServers) {
    //   webRTCconfig = {
    //     wrtc,
    //     initiator: options?.initiator,
    //     trickle: true,
    //     // @ts-ignore
    //     allowHalfTrickle: true,
    //     config: { iceServers: this.stunServers },
    //   }
    // } else {
    webRTCconfig = {
      wrtc,
      initiator: options?.initiator || false,
      trickle: true,
      // @ts-ignore
      allowHalfTrickle: true,
    }
    // }

    const channel = new Peer(webRTCconfig)

    const done = async (err?: Error) => {
      console.log('done called', err)
      channel.removeListener('connect', onConnect)
      channel.removeListener('error', onError)
      channel.removeListener('signal', onSignal)

      if (this._timeoutIntentionallyOnWebRTC !== undefined) {
        await this._timeoutIntentionallyOnWebRTC
      }

      options?.signal?.removeEventListener('abort', onAbort)

      if (!err && !this._failIntentionallyOnWebRTC) {
        setImmediate(resolve, (channel as unknown) as Socket)
      }
    }

    const onAbort = () => {
      channel.destroy()

      setImmediate(reject)
    }

    const onSignal = (data: string): void => {
      if (options?.signal?.aborted) {
        console.log('aborted')
        return
      }

      console.log('sending')
      sinkBuffer.push(_encoder.encode(JSON.stringify(data)))
    }

    const onConnect = (): void => {
      done()
    }

    const onError = (err?: Error) => {
      log(`WebRTC with failed. Error was: ${err}`)
      done(err)
    }

    if (options?.signal?.aborted) {
      return reject(new AbortError())
    }

    channel.on('signal', onSignal)

    channel.once('error', onError)

    channel.once('connect', onConnect)

    options?.signal?.addEventListener('abort', onAbort)

    pipe(
      /* prettier-ignore */
      srcBuffer,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (options?.initiator) {
            console.log('initiator receiving')
          } else {
            console.log('counterparty receiving')
          }

          if (msg == null) {
            continue
          }

          if (options?.signal?.aborted) {
            return
          }

          channel.signal(JSON.parse(_decoder.decode(msg.slice())))
        }
      }
    )
  })
}
