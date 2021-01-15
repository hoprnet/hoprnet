/// <reference path="./@types/bl.ts" />

import Multiaddr from 'multiaddr'
import BL from 'bl'
import type { MultiaddrConnection, Stream } from 'libp2p'
import Defer, { DeferredPromise } from 'p-defer'
import { RELAY_PAYLOAD_PREFIX, RELAY_STATUS_PREFIX, RELAY_WEBRTC_PREFIX, RESTART, STOP, PING, PONG } from './constants'
import { u8aConcat, u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'

import type { Instance as SimplePeer } from 'simple-peer'

import type PeerId from 'peer-id'

import Debug from 'debug'

const log = Debug('hopr-connect')
const verbose = Debug('hopr-connect:verbose')
const error = Debug('hopr-connect:error')

type WebRTC = {
  channel: SimplePeer
  upgradeInbound: () => SimplePeer
}

class RelayConnection implements MultiaddrConnection {
  private _closePromise: DeferredPromise<void>
  private _stream: Stream
  private _destroyed: boolean
  private _sinkTriggered: boolean

  private _switchPromise: DeferredPromise<Stream['source']>
  private _msgPromise: DeferredPromise<void>
  private _msgs: (IteratorResult<Uint8Array, void> & { iteration: number })[]

  private _webRTCStreamResult?: IteratorResult<Uint8Array, void>
  private _webRTCresolved: boolean
  private _webRTCstream?: Stream['source']
  private _webRTCSourceFunction: (arg: IteratorResult<Uint8Array, void>) => void
  private _webRTCPromise?: Promise<void>

  private _statusMessagePromise: DeferredPromise<void>
  private _statusMessages: Uint8Array[]

  public _iteration: number

  private _onReconnect: (newStream: RelayConnection, counterparty: PeerId) => Promise<void>

  public webRTC?: WebRTC

  public localAddr: Multiaddr
  public remoteAddr: Multiaddr

  private _counterparty: PeerId

  public source: Stream['source']
  public sink: Stream['sink']
  public close: (err?: Error) => Promise<void>

  public conn: Stream

  public timeline: {
    open: number
    close?: number
  }

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

    this._closePromise = Defer<void>()

    this._switchPromise = Defer<Stream['source']>()
    this._msgPromise = Defer<void>()

    this._msgs = []

    this._statusMessagePromise = Defer<void>()
    this._statusMessages = []

    this._destroyed = false
    this._sinkTriggered = false

    this._stream = opts.stream

    this.conn = opts.stream

    this._onReconnect = opts.onReconnect

    this._counterparty = opts.counterparty

    this.localAddr = Multiaddr(`/p2p/${opts.self.toB58String()}`)
    this.remoteAddr = Multiaddr(`/p2p/${opts.counterparty.toB58String()}`)

    this.webRTC = opts.webRTC

    this._webRTCresolved = false
    this._webRTCStreamResult = opts.webRTC != undefined ? undefined : { done: true, value: undefined }

    this._webRTCSourceFunction = (arg: IteratorResult<Uint8Array, void>) => {
      this._webRTCresolved = true

      this._webRTCStreamResult = arg
    }

    this._iteration = 0

    this.source = this._createSource.call(this, this._iteration)

    this._stream.sink(this.sinkFunction())

    this.sink = this._createSink.bind(this)

    this.close = (_err?: Error): Promise<void> => {
      this._closePromise.resolve()

      this.timeline.close = Date.now()

      return Promise.resolve()
    }

    this._drainSource()
  }

  private async _drainSource() {
    let streamClosed = false

    const closePromise = this._closePromise.promise.then(() => {
      streamClosed = true
    })

    let streamResult: IteratorResult<Uint8Array, void> | undefined

    function sourceFunction(arg: IteratorResult<Uint8Array, void>) {
      streamResult = arg
    }

    let streamPromise = this._stream.source.next().then(sourceFunction)

    while (streamResult == undefined || !streamResult.done) {
      await Promise.race([
        // prettier-ignore
        streamPromise,
        closePromise
      ])

      if (streamClosed || streamResult?.done) {
        if (!this._destroyed) {
          if (!this._sinkTriggered) {
            this._stream.sink(
              (async function* () {
                yield u8aConcat(RELAY_STATUS_PREFIX, STOP)
              })()
            )
          }
          this._destroyed = true
        }
        this._msgs.push({ done: true, value: undefined, iteration: this._iteration })
        break
      }

      if (streamResult == undefined || streamResult.done || streamClosed) {
        this._msgs.push({ done: true, value: undefined, iteration: this._iteration })
        log(`ending stream because 'streamDone' was set to 'true'.`)
        break
      }

      const received = streamResult.value.slice()

      const [PREFIX, SUFFIX] = [received.subarray(0, 1), received.subarray(1)]

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
          this._msgs.push({ done: true, value: undefined, iteration: this._iteration })
          this._msgPromise?.resolve()

          break
        } else if (u8aEquals(SUFFIX, RESTART)) {
          log(`RESTART received. Ending stream ...`)

          if (this.webRTC != undefined) {
            try {
              this.webRTC.channel.destroy()
            } catch {}

            this.webRTC.channel = this.webRTC.upgradeInbound()

            log(`resetting WebRTC stream`)
            this._webRTCStreamResult = undefined
            this._webRTCresolved = false
            this._webRTCstream = this._getWebRTCStream()
            this._webRTCPromise = this._webRTCstream.next().then(this._webRTCSourceFunction)
            log(`resetting WebRTC stream done`)
          }

          this._iteration++
          this._onReconnect(this.switch(), this._counterparty)
        } else if (u8aEquals(SUFFIX, PING)) {
          verbose(`PING received`)
          this._statusMessages.push(Uint8Array.from([...RELAY_STATUS_PREFIX, ...PONG]))

          let tmpPromise = this._statusMessagePromise
          tmpPromise.resolve()

          // Don't forward ping to receiver
        } else {
          error(`Received invalid status message ${u8aToHex(SUFFIX || new Uint8Array([]))}. Dropping message.`)
        }
      } else if (u8aEquals(PREFIX, RELAY_WEBRTC_PREFIX)) {
        try {
          this.webRTC?.channel.signal(JSON.parse(new TextDecoder().decode(received.slice(1))))
        } catch (err) {
          error(`WebRTC error:`, err)
        }
      } else {
        error(`Received invalid prefix <${u8aToHex(PREFIX || new Uint8Array([]))}. Dropping message.`)
      }

      streamPromise = this._stream.source.next().then(sourceFunction)
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
          log(
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
    this._switchPromise.resolve(source)

    this._sinkTriggered = true

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
    log(`sinkFunction`)
    let streamResolved = false
    let skippedMsg = false
    let streamResult: IteratorResult<Uint8Array, void> | undefined

    let streamClosed = false

    const closePromise = this._closePromise.promise.then(() => {
      streamClosed = true
    })

    let currentSource: Stream['source'] | undefined
    let tmpSource: Stream['source']
    let streamPromise: Promise<void> | undefined

    let iteration = 0

    const streamSourceFunction = (_iteration: number) => (arg: IteratorResult<Uint8Array, void>) => {
      // console.log(`sinking to relay`, arg.done ? undefined : new TextDecoder().decode(arg.value.slice()), iteration, _iteration)
      streamResolved = true

      if (iteration == _iteration) {
        streamResult = arg
      } else {
        skippedMsg = true
      }
    }

    let statusMessageAvailable = this._statusMessages.length > 0

    const statusSourceFunction = () => {
      statusMessageAvailable = true
    }

    let statusPromise: Promise<void> | undefined

    let streamSwitched = false

    const switchFunction = (newSource: Stream['source']) => {
      console.log(`inside switch function`)
      streamSwitched = true
      tmpSource = newSource
      iteration++
    }

    let switchPromise = this._switchPromise.promise.then(switchFunction)

    while (true) {
      let promises: Promise<void>[] = [switchPromise, closePromise]

      statusPromise = statusPromise ?? this._statusMessagePromise.promise.then(statusSourceFunction)

      promises.push(statusPromise)
      if ((streamResult == undefined || !streamResult.done) && currentSource != undefined) {
        streamPromise = streamPromise ?? currentSource.next().then(streamSourceFunction(iteration))

        promises.push(streamPromise)
      }

      if (this._webRTCStreamResult == undefined || !this._webRTCStreamResult.done) {
        this._webRTCstream = this._webRTCstream ?? this._getWebRTCStream()
        this._webRTCPromise = this._webRTCPromise ?? this._webRTCstream.next().then(this._webRTCSourceFunction)

        promises.push(this._webRTCPromise)
      }

      // 1. Handle stream switch
      // 2. Handle stream close
      // 3. Handle status messages
      // 4. Handle payload messages
      // 5. Handle WebRTC signalling messages
      await Promise.race(promises)

      log(`sink promise`)

      if (this._webRTCresolved) {
        this._webRTCresolved = false

        if (this._webRTCStreamResult == undefined || this._webRTCStreamResult.done) {
          this._webRTCPromise = undefined
          continue
        }

        yield new BL([RELAY_WEBRTC_PREFIX, this._webRTCStreamResult.value])

        if (streamClosed) {
          this._destroyed = true

          yield u8aConcat(RELAY_STATUS_PREFIX, STOP)
          break
        }

        this._webRTCPromise = this._webRTCstream!.next().then(this._webRTCSourceFunction)
        continue
      } else if (streamClosed) {
        if (!this._destroyed) {
          this._destroyed = true

          yield u8aConcat(RELAY_STATUS_PREFIX, STOP)
        }

        break
      } else if (statusMessageAvailable) {
        statusMessageAvailable = false
        while (this._statusMessages.length > 0) {
          yield this._statusMessages.shift() as Uint8Array
        }

        this._statusMessagePromise = Defer<void>()

        statusPromise = this._statusMessagePromise.promise.then(statusSourceFunction)
        continue
      } else if (streamSwitched) {
        log(`RelayConnection: after stream switch sink operation`, iteration)
        streamSwitched = false
        // @TODO explain this to Typescript properly
        currentSource = tmpSource!
        this._switchPromise = Defer<Stream['source']>()
        console.log(`stream before switch`, streamResult, streamResolved)
        streamResult = undefined
        streamPromise = currentSource.next().then(streamSourceFunction(iteration))
        switchPromise = this._switchPromise.promise.then(switchFunction)
        continue
      }

      if (skippedMsg) {
        skippedMsg = false
        streamPromise = (currentSource as Stream['source']).next().then(streamSourceFunction(iteration))

        continue
      }

      if (streamResult == undefined || streamResult.done) {
        console.log(`##### EMPTY message #####`, streamResult, streamResult)
        yield new BL([RELAY_PAYLOAD_PREFIX])

        streamPromise = undefined
        continue
      }

      yield new BL([RELAY_PAYLOAD_PREFIX, streamResult.value])

      if (streamClosed) {
        this._destroyed = true

        yield u8aConcat(RELAY_STATUS_PREFIX, STOP)
        break
      }

      streamPromise = (currentSource as Stream['source']).next().then(streamSourceFunction(iteration))
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
