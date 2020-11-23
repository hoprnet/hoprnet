import Multiaddr from 'multiaddr'
import BL from 'bl'
import type { MultiaddrConnection, Stream } from 'libp2p'
import Defer, { DeferredPromise } from 'p-defer'
import {
  RELAY_PAYLOAD_PREFIX,
  RELAY_STATUS_PREFIX,
  RELAY_WEBRTC_PREFIX,
  RESTART,
  STOP,
  PING,
  PING_RESPONSE
} from './constants'
import { u8aConcat, u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'

import type { Instance as SimplePeer } from 'simple-peer'

import type PeerId from 'peer-id'

import Debug from 'debug'

const log = Debug('hopr-core:transport')
const error = Debug('hopr-core:transport:error')

class RelayConnection implements MultiaddrConnection {
  private _closePromise: DeferredPromise<void>
  private _stream: Stream
  private _destroyed: boolean
  private _sinkTriggered: boolean

  private _switchPromise: DeferredPromise<Stream['source']>
  private _msgPromise: DeferredPromise<void>
  private _msgs: (IteratorResult<Uint8Array, void> & { iteration: number })[]

  private _webRTCdone: boolean
  private _webRTCmsg?: Uint8Array
  private _webRTCresolved: boolean
  private _webRTCstream: Stream['source']
  private _webRTCSourceFunction: (arg: IteratorResult<Uint8Array, void>) => void
  private _webRTCPromise: Promise<void>

  private _statusMessagePromise: DeferredPromise<void>
  private _statusMessages: Uint8Array[]

  public _iteration: number

  private _onReconnect: (newStream: MultiaddrConnection, counterparty: PeerId) => Promise<void>
  private _webRTCUpgradeInbound: () => SimplePeer

  public webRTC: SimplePeer
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
    webRTC?: SimplePeer
    onReconnect: (newStream: MultiaddrConnection, counterparty: PeerId) => Promise<void>
    webRTCUpgradeInbound?: () => SimplePeer
  }) {
    this.timeline = {
      open: Date.now()
    }

    this._closePromise = Defer()

    this._switchPromise = Defer<Stream['source']>()
    this._msgPromise = Defer<void>()

    this._msgs = []

    this._statusMessagePromise = Defer<void>()
    this._statusMessages = []

    this._destroyed = false
    this._sinkTriggered = false

    this._stream = opts.stream

    this._onReconnect = opts.onReconnect
    this._webRTCUpgradeInbound = opts.webRTCUpgradeInbound

    this._counterparty = opts.counterparty

    this.localAddr = Multiaddr(`/p2p/${opts.self.toB58String()}`)
    this.remoteAddr = Multiaddr(`/p2p/${opts.counterparty.toB58String()}`)

    this.webRTC = opts.webRTC

    this._webRTCresolved = false
    this._webRTCdone = this.webRTC == null

    this._webRTCSourceFunction = (arg: IteratorResult<Uint8Array, void>) => {
      this._webRTCresolved = true
      this._webRTCdone = arg.done

      if (!arg.done) {
        this._webRTCmsg = arg.value as Uint8Array
      }
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

    let streamResolved = false
    let streamMsg: Uint8Array
    let streamDone = false

    function sourceFunction(arg: IteratorResult<Uint8Array, void>) {
      streamResolved = true
      streamDone = arg.done

      if (!arg.done) {
        streamMsg = arg.value as Uint8Array
      }
    }

    let streamPromise = this._stream.source.next().then(sourceFunction)

    while (!streamDone) {
      await Promise.race([
        // prettier-ignore
        streamPromise,
        closePromise
      ])

      if (streamResolved) {
        streamResolved = false

        if (streamDone || streamClosed) {
          this._msgs.push({ done: true, value: undefined, iteration: this._iteration })
          log(`ending stream because 'streamDone' was set to 'true'.`)
          break
        }

        const received = (streamMsg as Uint8Array).slice()

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

            console.log(`this.webRTC`, this.webRTC)
            if (this.webRTC != null) {
              try {
                await new Promise((resolve) => this.webRTC._destroy(undefined, resolve))
              } catch {}

              this.webRTC = this._webRTCUpgradeInbound()
              log(`resetting WebRTC stream`)
              this._webRTCdone = false
              this._webRTCresolved = false
              this._webRTCmsg = undefined
              this._webRTCstream = this._getWebRTCStream()
              this._webRTCPromise = this._webRTCstream.next().then(this._webRTCSourceFunction)
              log(`resetting WebRTC stream done`)
            }

            this._iteration++
            this._onReconnect(this.switch(), this._counterparty)
          } else if (u8aEquals(SUFFIX, PING)) {
            log(`PING received`)
            this._statusMessages.push(u8aConcat(RELAY_STATUS_PREFIX, PING_RESPONSE))

            let tmpPromise = this._statusMessagePromise
            tmpPromise.resolve()

            // Don't forward ping to receiver
          } else {
            error(`Received invalid status message ${u8aToHex(SUFFIX || new Uint8Array([]))}. Dropping message.`)
          }
        } else if (u8aEquals(PREFIX, RELAY_WEBRTC_PREFIX)) {
          try {
            this.webRTC?.signal(JSON.parse(new TextDecoder().decode(received.slice(1))))
          } catch (err) {
            error(`WebRTC error:`, err)
          }
        } else {
          error(`Received invalid prefix <${u8aToHex(PREFIX || new Uint8Array([]))}. Dropping message.`)
        }

        streamPromise = this._stream.source.next().then(sourceFunction)
      } else if (streamClosed || streamDone) {
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
    }
  }

  private async *_createSource(this: RelayConnection, i: number): Stream['source'] {
    while (true) {
      if (i < this._iteration) {
        break
      }

      while (this._msgs.length > 0) {
        let current = this._msgs[0]

        while (current != null && current.iteration < i) {
          log(
            `dropping message <${new TextDecoder().decode(
              this._msgs.shift()?.value || new Uint8Array([])
            )}> from peer ${this.remoteAddr.getPeerId()}`
          )

          current = this._msgs[0]
        }

        if (current == null) {
          break
        }

        if (current.done) {
          return
        }

        yield this._msgs.shift().value as Uint8Array
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
    let defer = Defer<void>()
    let waiting = false
    const webRTCmessages: Uint8Array[] = []
    let done = false
    function onSignal(msg: any) {
      webRTCmessages.push(new TextEncoder().encode(JSON.stringify(msg)))
      if (waiting) {
        waiting = false
        let tmpPromise = defer
        defer = Defer<void>()
        tmpPromise.resolve()
      }
    }
    this.webRTC.on('signal', onSignal)

    this.webRTC.once('connect', () => {
      done = true
      this.webRTC.removeListener('signal', onSignal)
      defer.resolve()
    })

    while (!done) {
      while (webRTCmessages.length > 0) {
        yield webRTCmessages.shift()
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
    let streamResolved = false
    let streamMsg: Uint8Array
    let streamDone = true

    let streamClosed = false

    const closePromise = this._closePromise.promise.then(() => {
      streamClosed = true
    })

    let currentSource: Stream['source']
    let tmpSource: Stream['source']
    let streamPromise: Promise<void>

    let iteration = 0

    const streamSourceFunction = (_iteration: number) => (arg: IteratorResult<Uint8Array, void>) => {
      if (iteration == _iteration) {
        streamResolved = true
        streamDone = arg.done

        if (!arg.done) {
          streamMsg = arg.value as Uint8Array
        }
      }
    }

    let statusMessageAvailable = this._statusMessages.length > 0

    const statusSourceFunction = () => {
      statusMessageAvailable = true
    }

    let statusPromise =
      this._statusMessages.length > 0
        ? Promise.resolve()
        : this._statusMessagePromise.promise.then(statusSourceFunction)

    let streamSwitched = false

    const switchFunction = (newSource: Stream['source']) => {
      streamSwitched = true
      tmpSource = newSource
      iteration++
    }

    let switchPromise = this._switchPromise.promise.then(switchFunction)

    while (true) {
      if (!this._webRTCdone && !streamDone) {
        if (streamPromise == null) {
          streamPromise = currentSource.next().then(streamSourceFunction(iteration))
        }
        if (this._webRTCstream == null) {
          this._webRTCstream = this._getWebRTCStream()
        }
        if (this._webRTCPromise == null) {
          this._webRTCPromise = this._webRTCstream.next().then(this._webRTCSourceFunction)
        }
        await Promise.race([
          // prettier-ignore
          streamPromise,
          statusPromise,
          this._webRTCPromise,
          switchPromise,
          closePromise
        ])
      } else if (!this._webRTCdone) {
        if (this._webRTCstream == null) {
          this._webRTCstream = this._getWebRTCStream.call(this)
        }
        if (this._webRTCPromise == null) {
          this._webRTCPromise = this._webRTCstream.next().then(this._webRTCSourceFunction)
        }
        await Promise.race([
          // prettier-ignore
          statusPromise,
          this._webRTCPromise,
          switchPromise,
          closePromise
        ])
      } else if (!streamDone) {
        if (streamPromise == null) {
          streamPromise = currentSource.next().then(streamSourceFunction(iteration))
        }

        await Promise.race([
          // prettier-ignore
          streamPromise,
          statusPromise,
          switchPromise,
          closePromise
        ])
      } else {
        await Promise.race([
          // prettier-ignore
          statusPromise,
          switchPromise,
          closePromise
        ])
      }

      if (streamResolved) {
        streamResolved = false

        if (streamDone) {
          yield (new BL([(RELAY_PAYLOAD_PREFIX as unknown) as BL]) as unknown) as Uint8Array

          streamPromise = undefined
          continue
        }

        yield (new BL([(RELAY_PAYLOAD_PREFIX as unknown) as BL, (streamMsg as unknown) as BL]) as unknown) as Uint8Array

        if (streamClosed) {
          this._destroyed = true

          yield u8aConcat(RELAY_STATUS_PREFIX, STOP)
          break
        }

        streamPromise = currentSource.next().then(streamSourceFunction(iteration))
      } else if (this._webRTCresolved) {
        this._webRTCresolved = false

        if (this._webRTCdone) {
          this._webRTCPromise = undefined
          continue
        }

        yield (new BL([
          (RELAY_WEBRTC_PREFIX as unknown) as BL,
          (this._webRTCmsg as unknown) as BL
        ]) as unknown) as Uint8Array

        if (streamClosed) {
          this._destroyed = true

          yield u8aConcat(RELAY_STATUS_PREFIX, STOP)
          break
        }

        this._webRTCPromise = this._webRTCstream.next().then(this._webRTCSourceFunction)
      } else if (streamClosed) {
        if (!this._destroyed) {
          this._destroyed = true

          yield u8aConcat(RELAY_STATUS_PREFIX, STOP)
        }

        break
      } else if (statusMessageAvailable) {
        statusMessageAvailable = false
        while (this._statusMessages.length > 0) {
          yield this._statusMessages.shift()
        }

        this._statusMessagePromise = Defer<void>()

        statusPromise = this._statusMessagePromise.promise.then(statusSourceFunction)
      } else if (streamSwitched) {
        log(`RelayConnection: after stream switch sink operation`, iteration)
        streamSwitched = false
        currentSource = tmpSource
        this._switchPromise = Defer<Stream['source']>()
        streamDone = false
        streamPromise = currentSource.next().then(streamSourceFunction(iteration))
        switchPromise = this._switchPromise.promise.then(switchFunction)
      }
    }
  }

  switch(): MultiaddrConnection {
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
