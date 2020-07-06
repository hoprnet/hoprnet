import type { Pushable } from 'it-pushable'
import abortable, { AbortError } from 'abortable-iterator'

import type { Socket } from 'net'

import Peer, { Options as SimplePeerOptions } from 'simple-peer'

import debug from 'debug'
const log = debug('hopr-core:transport')
const error = debug('hopr-core:transport:error')

import pipe from 'it-pipe'

// @ts-ignore
import wrtc = require('wrtc')
import { WEBRTC_TIMEOUT } from './constants'
import { randomBytes } from 'crypto'

const _encoder = new TextEncoder()
const _decoder = new TextDecoder()

export default function upgradetoWebRTC(
  sinkBuffer: Pushable<Uint8Array>,
  srcBuffer: Pushable<Uint8Array>,
  options?: {
    initiator?: boolean
    signal?: AbortSignal
    // ONLY FOR TESTING
    _timeoutIntentionallyOnWebRTC?: Promise<void>
    _failIntentionallyOnWebRTC?: boolean
    _answerIntentionallyWithIncorrectMessages?: boolean
    // END ONLY FOR TESTING
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

    let timeout: any

    const onTimeout = () => {
      clearTimeout(timeout)
      channel.destroy()

      reject(new Error('timeout'))
    }

    timeout = setTimeout(() => onTimeout(), WEBRTC_TIMEOUT)

    const done = async (err?: Error) => {
      clearTimeout(timeout)

      channel.removeListener('iceTimeout', onTimeout)
      channel.removeListener('connect', onConnect)
      channel.removeListener('error', onError)
      channel.removeListener('signal', onSignal)

      if (options?._timeoutIntentionallyOnWebRTC !== undefined) {
        await options?._timeoutIntentionallyOnWebRTC
      }

      options?.signal?.removeEventListener('abort', onAbort)

      if (err || options?._failIntentionallyOnWebRTC) {
        reject(err)
      } else {
        resolve((channel as unknown) as Socket)
      }
    }

    const onAbort = () => {
      channel.destroy()
      clearTimeout(timeout)

      reject()
    }

    const onSignal = (data: string): void => {
      if (options?.signal?.aborted) {
        return
      }

      if (options?._answerIntentionallyWithIncorrectMessages) {
        sinkBuffer.push(randomBytes(31))
      } else {
        sinkBuffer.push(_encoder.encode(JSON.stringify(data)))
      }
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

    channel.once('iceTimeout', onTimeout)

    options?.signal?.addEventListener('abort', onAbort)

    pipe(
      /* prettier-ignore */
      srcBuffer,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (msg == null) {
            continue
          }

          if (options?.signal?.aborted) {
            return
          }

          try {
            channel.signal(JSON.parse(_decoder.decode(msg.slice())))
          } catch (err) {
            error(err)
            continue
          }
        }
      }
    )
  })
}
