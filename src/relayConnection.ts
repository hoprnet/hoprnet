/// <reference path="./@types/bl.ts" />
/// <reference path="./@types/libp2p.ts" />

import Multiaddr from 'multiaddr'
import BL from 'bl'
import type { MultiaddrConnection, Stream } from 'libp2p'
import { randomBytes } from 'crypto'
import Defer, { DeferredPromise } from 'p-defer'
import { RELAY_PAYLOAD_PREFIX, RELAY_STATUS_PREFIX, RELAY_WEBRTC_PREFIX, RESTART, STOP, PING, PONG } from './constants'
import { u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'

import type { Instance as SimplePeer } from 'simple-peer'

import type PeerId from 'peer-id'

import Debug from 'debug'

const _log = Debug('hopr-connect')
const _verbose = Debug('hopr-connect:verbose')
const _error = Debug('hopr-connect:error')

type WebRTC = {
  channel: SimplePeer
  upgradeInbound: () => SimplePeer
}

class RelayConnection implements MultiaddrConnection {
  private _stream: Stream
  private _destroyed: boolean
  private _sinkTriggered: boolean

  private _switchPromise: DeferredPromise<Stream['source']>
  private _msgPromise: DeferredPromise<void>
  private _msgs: (IteratorResult<Uint8Array, void> & { iteration: number })[]

  private _webRTCStreamResult?: IteratorResult<Uint8Array, void>
  private _webRTCresolved: boolean
  private _webRTCstream?: Stream['source']
  private _webRTCSourceFunction: (arg: IteratorResult<Uint8Array, void>) => IteratorResult<Uint8Array, void>
  private _webRTCPromise?: Promise<IteratorResult<Uint8Array, void>>

  private _destroyedPromise: DeferredPromise<void>

  private _closePromise: DeferredPromise<void>

  private _statusMessagePromise: DeferredPromise<void>
  private _statusMessages: Uint8Array[]

  public _iteration: number

  public _id: string

  private _sinkSourceAttached: boolean
  private _sinkSourceAttachedPromise: DeferredPromise<Stream['source']>

  private _onReconnect: (newStream: RelayConnection, counterparty: PeerId) => Promise<void>

  public webRTC?: WebRTC

  public localAddr: Multiaddr
  public remoteAddr: Multiaddr

  private _counterparty: PeerId

  public source: Stream['source']
  public sink: Stream['sink']
  public close: (err?: Error) => Promise<void>

  public conn: Stream

  public timeline: MultiaddrConnection['timeline']

  constructor(opts: {
    stream: Stream
    self: PeerId
    counterparty: PeerId
    onReconnect: (newStream: RelayConnection, counterparty: PeerId) => Promise<void>
    webRTC?: WebRTC
  }) {
    this.timeline = {
      open: Date.now()
    }

    this._switchPromise = Defer<Stream['source']>()
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

    this._webRTCresolved = false
    this._webRTCStreamResult = opts.webRTC != undefined ? undefined : { done: true, value: undefined }

    this._webRTCSourceFunction = (arg: IteratorResult<Uint8Array, void>) => {
      this._webRTCresolved = true

      return arg
    }

    this._iteration = 0

    this.source = this._createSource.call(this, this._iteration)

    this._sinkSourceAttached = false
    this._sinkSourceAttachedPromise = Defer<Stream['source']>()

    this._stream.sink(this.sinkFunction())

    this.sink = this._createSink.bind(this)

    this.close = (_err?: Error): Promise<void> => {
      this.verbose(`close called`)
      this._statusMessages.unshift(Uint8Array.from([...RELAY_STATUS_PREFIX, ...STOP]))
      this._statusMessagePromise.resolve()
      this._closePromise.resolve()

      this.timeline.close = Date.now()

      return this._destroyedPromise.promise
    }

    this._drainSource()
  }

  private log(..._: any[]) {
    _log(`[${this._id}]`, ...arguments)
  }

  private verbose(..._: any[]) {
    _verbose(`[${this._id}]`, ...arguments)
  }

  private error(..._: any[]) {
    _error(`[${this._id}]`, ...arguments)
  }

  private async _drainSource() {
    type SourceType = IteratorResult<Uint8Array, void> | void

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
          console.log(`sunk`)

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
        this.log(`streamClosed`, this._msgs)

        break
      }

      const received = result as IteratorResult<Uint8Array, void>
      if (received == undefined || received.done) {
        this._msgs.push({ done: true, value: undefined, iteration: this._iteration })
        this.log(`ending stream because 'streamDone' was set to 'true'.`)
        break
      }

      const [PREFIX, SUFFIX] = [received.value.slice(0, 1), received.value.slice(1)]

      if (u8aEquals(PREFIX, RELAY_PAYLOAD_PREFIX)) {
        if (SUFFIX.length > 0) {
          this._msgs.push({ done: false, value: SUFFIX, iteration: this._iteration })
        } else {
          this._msgs.push({ done: true, value: undefined, iteration: this._iteration })
        }
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
            try {
              this.webRTC.channel.destroy()
            } catch {}

            this.webRTC.channel = this.webRTC.upgradeInbound()

            this.log(`resetting WebRTC stream`)
            this._webRTCStreamResult = undefined
            this._webRTCresolved = false
            this._webRTCstream = this._getWebRTCStream()
            this._webRTCPromise = this._webRTCstream.next().then(this._webRTCSourceFunction)
            this.log(`resetting WebRTC stream done`)
          }

          this._iteration++
          this._onReconnect(this.switch(), this._counterparty)
        } else if (u8aEquals(SUFFIX, PING)) {
          this.verbose(`PING received`)
          this._statusMessages.push(Uint8Array.from([...RELAY_STATUS_PREFIX, ...PONG]))

          let tmpPromise = this._statusMessagePromise
          tmpPromise.resolve()

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
              current.value || new Uint8Array()
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

  private _createSink(source: Stream['source']): Promise<void> {
    // @TODO add support for Iterables such as arrays
    this._sinkSourceAttached = true
    this._sinkTriggered = true

    this._sinkSourceAttachedPromise.resolve(source)

    return Promise.resolve()
  }

  private async *_getWebRTCStream(this: RelayConnection): Stream['source'] {
    if (this.webRTC == undefined) {
      throw Error(`Cannot listen to a WebRTC instance because there was none given.`)
    }

    const webRTCmessages: Uint8Array[] = []

    let defer = Defer<void>()
    let waiting = false
    let done = false

    const onSignal = (msg: Object) => {
      if (done) {
        return
      }

      webRTCmessages.push(new TextEncoder().encode(JSON.stringify(msg)))

      if (waiting) {
        waiting = false
        let tmpPromise = defer
        defer = Defer<void>()
        tmpPromise.resolve()
      }
    }

    const endStream = () => {
      done = true
      ;(this.webRTC as WebRTC).channel.removeListener('signal', onSignal)
      ;(this.webRTC as WebRTC).channel.removeListener('error', endStream)
      ;(this.webRTC as WebRTC).channel.removeListener('connect', endStream)

      defer.resolve()
    }

    this.webRTC.channel.on('signal', onSignal)

    this.webRTC.channel.once('error', endStream)
    this.webRTC.channel.once('connect', endStream)

    while (!done) {
      while (webRTCmessages.length > 0) {
        yield webRTCmessages.shift() as Uint8Array
      }

      if (done) {
        break
      }

      waiting = true

      await defer.promise

      if (done) {
        break
      }
    }
  }

  private async *sinkFunction(this: RelayConnection): Stream['source'] {
    type SinkType = Stream['sink'] | Stream['source'] | IteratorResult<Uint8Array, void> | undefined | void
    this.log(`sinkFunction`)
    let currentSource: Stream['source'] | undefined
    let streamPromise: Promise<IteratorResult<Uint8Array, void>> | undefined

    let statusMessageAvailable = this._statusMessages.length > 0

    const statusSourceFunction = () => {
      statusMessageAvailable = true
    }

    let statusPromise: Promise<void> | undefined

    let streamSwitched = false

    let result: SinkType

    const switchFunction = (newSource: Stream['source']) => {
      console.log(`inside switch function`)
      streamSwitched = true

      return newSource
    }

    let switchPromise = this._switchPromise.promise.then(switchFunction)

    while (true) {
      let promises: Promise<SinkType>[] = []

      if (currentSource == undefined) {
        promises.push(this._sinkSourceAttachedPromise.promise)
      }

      promises.push(switchPromise)

      statusPromise = statusPromise ?? this._statusMessagePromise.promise.then(statusSourceFunction)
      promises.push(statusPromise)

      if ((result == undefined || !(result as IteratorResult<Uint8Array, void>).done) && currentSource != undefined) {
        streamPromise = streamPromise ?? currentSource.next()

        promises.push(streamPromise)
      }

      if (this._webRTCStreamResult == undefined || !this._webRTCStreamResult.done) {
        this._webRTCstream = this._webRTCstream ?? this._getWebRTCStream()
        this._webRTCPromise = this._webRTCPromise ?? this._webRTCstream.next().then(this._webRTCSourceFunction)

        promises.push(this._webRTCPromise)
      }

      // (0. Handle source attach)
      // 1. Handle stream switch
      // 2. Handle status messages
      // 3. Handle payload messages
      // 4. Handle WebRTC signalling messages
      result = await Promise.race(promises)

      if (this._sinkSourceAttached) {
        this._sinkSourceAttached = false

        // @TODO do this properly
        currentSource = result as Stream['source']
        result = undefined
        continue
      }

      if (statusMessageAvailable) {
        if (this._statusMessages.length > 0) {
          this.log(`this._statusMessages`, this._statusMessages)
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
        if (
          this._statusMessages.length == 0 ||
          (result != undefined && (result as IteratorResult<Uint8Array, void>).done != true)
        ) {
          statusMessageAvailable = false

          this._statusMessagePromise = Defer<void>()

          statusPromise = this._statusMessagePromise.promise.then(statusSourceFunction)
        }

        continue
      }

      if (this._webRTCresolved) {
        this._webRTCresolved = false

        const received = result as IteratorResult<Uint8Array, void>

        if (received == undefined || received.done) {
          this._webRTCPromise = undefined
          continue
        }

        yield new BL([RELAY_WEBRTC_PREFIX, received.value])

        result = undefined
        this._webRTCPromise = this._webRTCstream!.next().then(this._webRTCSourceFunction)
        continue
      }

      if (streamSwitched) {
        this.log(`RelayConnection: after stream switch sink operation`)
        streamSwitched = false
        // @TODO explain this to Typescript properly
        currentSource = result as Stream['source']
        this._switchPromise = Defer<Stream['source']>()
        console.log(`stream before switch`, result, streamPromise)
        result = undefined
        streamPromise = currentSource.next()
        switchPromise = this._switchPromise.promise.then(switchFunction)

        continue
      }

      const received = result as IteratorResult<Uint8Array, void>

      if (received == undefined || received.done) {
        console.log(`##### EMPTY message #####`, received)
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
    return {
      ...this,
      source: this._createSource(this._iteration),
      sink: this._createSink.bind(this)
    }
  }

  get destroyed(): boolean {
    return this._destroyed
  }
}

export { RelayConnection }
