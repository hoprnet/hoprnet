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

  private _statusMessagePromise: DeferredPromise<void>
  private _statusMessages: Uint8Array[]

  private _iteration: number

  private _onReconnect: (newStream: MultiaddrConnection, counterparty: PeerId) => Promise<void>
  private _webRTCUpgradeInbound: () => SimplePeer

  public webRTC: SimplePeer
  public localAddr: Multiaddr
  public remoteAddr: Multiaddr

  private _counterparty: PeerId

  public _tmpWebRTC: SimplePeer

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

            // this._tmpWebRTC = this._webRTCUpgradeInbound()
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
          // console.log(`Receiving fancy WebRTC message`, JSON.parse(new TextDecoder().decode(received.slice(1))))
          try {
            this.webRTC?.signal(JSON.parse(new TextDecoder().decode(received.slice(1))))
          } catch (err) {
            console.log(`WebRTC error:`, err)
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
          this._msgs.shift()
          current = this._msgs[0]
        }

        if (current == null || current.done) {
          console.log(`exiting`)
          return
        }

        yield this._msgs.shift().value as Uint8Array
      }

      this._msgPromise = Defer<void>()

      await this._msgPromise.promise
    }
    console.log(`done`)
  }

  private _createSink(source: Stream['source']): Promise<void> {
    this._switchPromise.resolve(
      (async function* () {
        let result: IteratorResult<Uint8Array> = { done: false, value: undefined }

        while (!result.done) {
          result = await source.next()

          console.log(`in relay connection`, result)

          yield result.value
        }
      })()
    )

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

    let webRTCresolved = false
    let webRTCdone = this.webRTC == null
    let webRTCmsg: Uint8Array

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

    const webRTCSourceFunction = (arg: IteratorResult<Uint8Array, void>) => {
      webRTCresolved = true
      webRTCdone = arg.done

      if (!arg.done) {
        webRTCmsg = arg.value as Uint8Array
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

    let webRTCstream: Stream['source']

    let webRTCPromise: Promise<void>

    let streamSwitched = false

    const switchFunction = (newSource: Stream['source']) => {
      streamSwitched = true
      tmpSource = newSource
      iteration++
    }

    let switchPromise = this._switchPromise.promise.then(switchFunction)

    while (true) {
      if (!webRTCdone && !streamDone) {
        if (streamPromise == null) {
          streamPromise = currentSource.next().then(streamSourceFunction(iteration))
        }
        if (webRTCstream == null) {
          webRTCstream = this._getWebRTCStream()
        }
        if (webRTCPromise == null) {
          webRTCPromise = webRTCstream.next().then(webRTCSourceFunction)
        }
        await Promise.race([
          // prettier-ignore
          streamPromise,
          statusPromise,
          webRTCPromise,
          switchPromise,
          closePromise
        ])
      } else if (!webRTCdone) {
        if (webRTCstream == null) {
          webRTCstream = this._getWebRTCStream.call(this)
        }
        if (webRTCPromise == null) {
          webRTCPromise = webRTCstream.next().then(webRTCSourceFunction)
        }
        await Promise.race([
          // prettier-ignore
          statusPromise,
          webRTCPromise,
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
      } else if (webRTCresolved) {
        webRTCresolved = false

        if (webRTCdone) {
          webRTCPromise = undefined
          continue
        }

        yield (new BL([(RELAY_WEBRTC_PREFIX as unknown) as BL, (webRTCmsg as unknown) as BL]) as unknown) as Uint8Array

        if (streamClosed) {
          this._destroyed = true

          yield u8aConcat(RELAY_STATUS_PREFIX, STOP)
          break
        }

        webRTCPromise = webRTCstream.next().then(webRTCSourceFunction)
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

        if (iteration > 1 && this.webRTC != null) {
          log(`resetting WebRTC`)
          try {
            this.webRTC.destroy()
          } catch (err) {
            err(`WebRTC error:`, err)
          }
          if (this._tmpWebRTC == null) {
            this.webRTC = this._webRTCUpgradeInbound()
          } else {
            this.webRTC = this._tmpWebRTC
            this._tmpWebRTC = undefined
          }
          webRTCdone = false
          webRTCstream = this._getWebRTCStream()
          webRTCPromise = webRTCstream.next().then(webRTCSourceFunction)
          log(`resetting WebRTC done`)
        }
      }
    }
  }

  switch(): MultiaddrConnection {
    return {
      ...this,
      source: this._createSource(++this._iteration),
      sink: this._createSink.bind(this)
    }
  }

  get destroyed(): boolean {
    return this._destroyed
  }
}

export { RelayConnection }
