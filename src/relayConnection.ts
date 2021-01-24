/// <reference path="./@types/bl.ts" />
/// <reference path="./@types/libp2p.ts" />

import Multiaddr from 'multiaddr'
import type { MultiaddrConnection, Stream, StreamResult } from 'libp2p'
import { randomBytes } from 'crypto'
import Defer, { DeferredPromise } from 'p-defer'
import { RELAY_PAYLOAD_PREFIX, RELAY_STATUS_PREFIX, RELAY_WEBRTC_PREFIX, RESTART, STOP, PING, PONG } from './constants'
import { u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import BL from 'bl'

import type { Instance as SimplePeer } from 'simple-peer'

import type PeerId from 'peer-id'

import Debug from 'debug'
import { EventEmitter } from 'events'
import { toU8aStream } from './utils'

const _log = Debug('hopr-connect')
const _verbose = Debug('hopr-connect:verbose')
const _error = Debug('hopr-connect:error')

type WebRTC = {
  channel: SimplePeer
  upgradeInbound: () => SimplePeer
}

class RelayConnection extends EventEmitter implements MultiaddrConnection {
  private _stream: Stream
  private _destroyed: boolean
  private _sinkTriggered: boolean

  private _msgPromise: DeferredPromise<void>
  private _msgs: (StreamResult & { iteration: number })[]

  private _destroyedPromise: DeferredPromise<void>

  private _closePromise: DeferredPromise<void>

  private _statusMessagePromise: DeferredPromise<void>
  private _statusMessages: Uint8Array[]

  public _iteration: number

  public _id: string

  private _sinkSourceAttachedPromise: DeferredPromise<Stream['source']>
  private _switchPromise: DeferredPromise<void>

  private _onReconnect: (newStream: RelayConnection, counterparty: PeerId) => Promise<void>

  public webRTC?: WebRTC

  public localAddr: Multiaddr
  public remoteAddr: Multiaddr

  private _counterparty: PeerId

  public source: Stream['source']
  public sink: Stream['sink']

  public conn: Stream

  public timeline: MultiaddrConnection['timeline']

  constructor(opts: {
    stream: Stream
    self: PeerId
    counterparty: PeerId
    onReconnect: (newStream: RelayConnection, counterparty: PeerId) => Promise<void>
    webRTC?: WebRTC
  }) {
    super()

    this.timeline = {
      open: Date.now()
    }

    this._msgPromise = Defer<void>()

    this._destroyedPromise = Defer<void>()

    this._msgs = []

    this._statusMessagePromise = Defer<void>()
    this._statusMessages = []

    this._destroyed = false
    this._sinkTriggered = false

    this._stream = opts.stream

    this.conn = opts.stream

    this._onReconnect = opts.onReconnect

    this._counterparty = opts.counterparty

    this._closePromise = Defer<void>()

    this._id = u8aToHex(randomBytes(4), false)

    this.localAddr = Multiaddr(`/p2p/${opts.self.toB58String()}`)
    this.remoteAddr = Multiaddr(`/p2p/${opts.counterparty.toB58String()}`)

    this.webRTC = opts.webRTC

    this._iteration = 0

    this.source = this._createSource.call(this, this._iteration)

    this._sinkSourceAttachedPromise = Defer<Stream['source']>()
    this._switchPromise = Defer<void>()

    this.attachWebRTCListeners()

    this._stream.sink(this.sinkFunction())

    this._drainSource()

    this.sink = this._sink.bind(this)
  }

  public close(_err?: Error): Promise<void> {
    this.verbose(`close called`)
    this._statusMessages.unshift(Uint8Array.from([...RELAY_STATUS_PREFIX, ...STOP]))
    this._statusMessagePromise.resolve()
    this._closePromise.resolve()

    this.timeline.close = Date.now()

    return this._destroyedPromise.promise
  }

  private log(..._: any[]) {
    _log(`RC [${this._id}]`, ...arguments)
  }

  private verbose(..._: any[]) {
    _verbose(`RC [${this._id}]`, ...arguments)
  }

  private error(..._: any[]) {
    _error(`RC [${this._id}]`, ...arguments)
  }

  private async _drainSource() {
    type SourceType = StreamResult | void

    let result: SourceType
    let streamClosed = false

    const closePromise = this._closePromise.promise.then(() => {
      streamClosed = true
    })

    let streamPromise = this._stream.source.next()

    while (true) {
      result = await Promise.race([
        // prettier-ignore
        streamPromise,
        closePromise
      ])

      if (streamClosed) {
        if (!this._destroyed) {
          if (!this._sinkTriggered) {
            this._stream.sink(
              (async function* () {
                yield Uint8Array.from([...RELAY_STATUS_PREFIX, ...STOP])
              })()
            )
          }
          this._destroyedPromise.resolve()
          this._destroyed = true
        }
        this._msgs.unshift({ done: true, value: undefined, iteration: this._iteration })
        this._msgPromise.resolve()

        break
      }

      const received = result as StreamResult

      if (received == undefined || received.done) {
        this._msgs.push({ done: true, value: undefined, iteration: this._iteration })
        break
      }

      const [PREFIX, SUFFIX] = [received.value.slice(0, 1), received.value.slice(1)]

      if (u8aEquals(PREFIX, RELAY_PAYLOAD_PREFIX)) {
        this._msgs.push(
          SUFFIX.length > 0
            ? { done: false, value: SUFFIX, iteration: this._iteration }
            : { done: true, value: undefined, iteration: this._iteration }
        )

        this._msgPromise?.resolve()
      } else if (u8aEquals(PREFIX, RELAY_STATUS_PREFIX)) {
        if (u8aEquals(SUFFIX, STOP)) {
          this._destroyed = true
          this._msgs.unshift({ done: true, value: undefined, iteration: this._iteration })
          this._msgPromise.resolve()

          break
        } else if (u8aEquals(SUFFIX, RESTART)) {
          this.log(`RESTART received. Ending stream ...`)

          if (this.webRTC != undefined) {
            this.webRTC?.channel.removeAllListeners('signal')
            this.emit(`restart`)
            try {
              this.webRTC.channel.destroy()
            } catch {}

            this.webRTC.channel = this.webRTC.upgradeInbound()

            this.log(`resetting WebRTC stream`)
            this.attachWebRTCListeners()
            this.log(`resetting WebRTC stream done`)
          }

          this._iteration++
          this._onReconnect(this.switch(), this._counterparty)
        } else if (u8aEquals(SUFFIX, PING)) {
          this.verbose(`PING received`)
          this._statusMessages.push(Uint8Array.from([...RELAY_STATUS_PREFIX, ...PONG]))

          this._statusMessagePromise.resolve()

          // Don't forward ping to receiver
        } else {
          this.error(`Received invalid status message ${u8aToHex(SUFFIX || new Uint8Array([]))}. Dropping message.`)
        }
      } else if (u8aEquals(PREFIX, RELAY_WEBRTC_PREFIX)) {
        try {
          this.webRTC?.channel.signal(JSON.parse(new TextDecoder().decode(SUFFIX)))
        } catch (err) {
          this.error(`WebRTC error:`, err)
        }
      } else {
        this.error(`Received invalid prefix <${u8aToHex(PREFIX || new Uint8Array([]))}. Dropping message.`)
      }

      result = undefined

      streamPromise = this._stream.source.next()
    }
  }

  private async *_createSource(this: RelayConnection, i: number): Stream['source'] {
    while (true) {
      if (i < this._iteration) {
        break
      }

      while (this._msgs.length > 0) {
        let current = this._msgs.shift()

        while (current != undefined && current.iteration < i) {
          this.log(
            `dropping message <${new TextDecoder().decode(
              (current.value || new Uint8Array()).slice()
            )}> from peer ${this.remoteAddr.getPeerId()}`
          )

          current = this._msgs.shift()
        }

        if (current == undefined) {
          break
        }

        if (current.done) {
          return
        }

        yield current.value
      }

      this._msgPromise = Defer<void>()

      await this._msgPromise.promise
    }
  }

  public _sink(_source: Stream['source']): Promise<void> {
    // @TODO add support for Iterables such as arrays
    this._sinkTriggered = true

    this._sinkSourceAttachedPromise.resolve(toU8aStream(_source))

    return Promise.resolve()
  }

  private attachWebRTCListeners() {
    // Cleanup potential previous signalling messages
    for (const [index, msg] of this._statusMessages.entries()) {
      if (msg[0] == RELAY_WEBRTC_PREFIX[0]) {
        let lastElement = this._statusMessages.pop() as Uint8Array

        if (index != this._statusMessages.length - 1) {
          this._statusMessages[index] = lastElement
        }
      }
    }

    const onSignal = (data: Object) => {
      if (this._statusMessages.length == 0) {
        this._statusMessages.push(
          Uint8Array.from([...RELAY_WEBRTC_PREFIX, ...new TextEncoder().encode(JSON.stringify(data))])
        )
        this._statusMessagePromise.resolve()
        return
      } else if (u8aEquals(Uint8Array.from([...RELAY_STATUS_PREFIX, ...STOP]), this._statusMessages[0])) {
        this.log(`Detected Stream close. Ending WebRTC upgrade`)
        this.webRTC?.channel.removeListener('signal', onSignal)
        return
      }

      this._statusMessages.unshift(
        Uint8Array.from([...RELAY_WEBRTC_PREFIX, ...new TextEncoder().encode(JSON.stringify(data))])
      )
      this._statusMessagePromise.resolve()
    }
    this.webRTC?.channel.on('signal', onSignal.bind(this))
  }

  private async *sinkFunction(this: RelayConnection): Stream['source'] {
    type SinkType = Stream['source'] | StreamResult | undefined | void
    this.log(`sinkFunction`)
    let currentSource: Stream['source'] | undefined
    let streamPromise: Promise<StreamResult> | undefined

    let statusMessageAvailable = this._statusMessages.length > 0

    const statusSourceFunction = () => {
      statusMessageAvailable = true
    }

    let sinkSourceAttached = false
    const sinkSourceFunction = (source: Stream['source']): Stream['source'] => {
      sinkSourceAttached = true

      return source
    }

    let sinkSourcePromise: Promise<Stream['source']> | undefined

    let statusPromise: Promise<void> | undefined

    let streamSwitched = false
    const switchFunction = () => {
      streamSwitched = true
    }

    let switchPromise = this._switchPromise.promise.then(switchFunction)

    let result: SinkType

    while (true) {
      let promises: Promise<SinkType>[] = []

      if (currentSource == undefined) {
        sinkSourcePromise = sinkSourcePromise ?? this._sinkSourceAttachedPromise.promise.then(sinkSourceFunction)
        promises.push(sinkSourcePromise)
      }

      promises.push(switchPromise)

      statusPromise = statusPromise ?? this._statusMessagePromise.promise.then(statusSourceFunction)
      promises.push(statusPromise)

      if ((result == undefined || !(result as StreamResult).done) && currentSource != undefined) {
        streamPromise = streamPromise ?? currentSource.next()

        promises.push(streamPromise)
      }

      // (0. Handle source attach)
      // 1. Handle stream switch
      // 2. Handle status messages
      // 3. Handle payload messages
      result = await Promise.race(promises)

      if (sinkSourceAttached) {
        sinkSourceAttached = false
        this._sinkSourceAttachedPromise = Defer<Stream['source']>()

        sinkSourcePromise = undefined
        currentSource = result as Stream['source']
        streamPromise = undefined
        result = undefined
        continue
      }

      if (statusMessageAvailable) {
        if (this._statusMessages.length > 0) {
          // this._destroyed should be true
          if (u8aEquals(Uint8Array.from([...RELAY_STATUS_PREFIX, ...STOP]), this._statusMessages[0])) {
            this._destroyedPromise.resolve()
            this._destroyed = true
            yield this._statusMessages[0]

            break
          } else {
            yield this._statusMessages.shift() as Uint8Array
          }
        }

        // @TODO fix condition
        if (this._statusMessages.length == 0 || (result != undefined && (result as StreamResult).done != true)) {
          statusMessageAvailable = false

          this._statusMessagePromise = Defer<void>()

          statusPromise = this._statusMessagePromise.promise.then(statusSourceFunction)
        }

        continue
      }

      if (streamSwitched) {
        streamSwitched = false

        switchPromise = this._switchPromise.promise.then(switchFunction)
        this.verbose(`Stream switched`)
        currentSource = undefined

        continue
      }

      const received = result as StreamResult

      if (received == undefined || received.done) {
        this.verbose(`##### EMPTY message #####`, received)
        yield new BL([RELAY_PAYLOAD_PREFIX])

        streamPromise = undefined
        continue
      }

      yield new BL([RELAY_PAYLOAD_PREFIX, received.value])

      result = undefined

      streamPromise = (currentSource as Stream['source']).next()
    }
  }

  switch(): RelayConnection {
    let tmpPromise = this._switchPromise
    this._switchPromise = Defer<void>()
    tmpPromise.resolve()

    this.source = this._createSource(++this._iteration)

    return this
  }

  get destroyed(): boolean {
    return this._destroyed
  }
}

export { RelayConnection }
