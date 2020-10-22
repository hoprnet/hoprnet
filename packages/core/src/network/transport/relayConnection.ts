import Multiaddr from 'multiaddr'
import BL from 'bl'
import { MultiaddrConnection, Stream } from '../../@types/transport'
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
  private _defer: DeferredPromise<void>
  private _stream: Stream
  private _destroyed: boolean
  private _sinkTriggered: boolean

  private _switchPromise: DeferredPromise<Stream['source']>

  private _statusMessagePromise: DeferredPromise<void>
  private _statusMessages: Uint8Array[]

  private _onReconnect: (relayConn: MultiaddrConnection) => Promise<void>

  private webRTC: SimplePeer
  public localAddr: Multiaddr
  public remoteAddr: Multiaddr

  public source: Stream['source']
  public sink: Stream['sink']
  public close: (err?: Error) => Promise<void>

  public conn: Stream

  private _sinkSourceDone: boolean

  public timeline: {
    open: number
    close?: number
  }

  constructor({
    stream,
    self,
    counterparty,
    webRTC,
    onReconnect
  }: {
    stream: Stream
    self: PeerId
    counterparty: PeerId
    onReconnect: (relayConn: MultiaddrConnection) => Promise<void>
    webRTC?: SimplePeer
  }) {
    this.timeline = {
      open: Date.now()
    }

    this._defer = Defer()

    this._switchPromise = Defer<Stream['source']>()

    this._statusMessagePromise = Defer<void>()
    this._statusMessages = []

    this._destroyed = false
    this._sinkTriggered = false

    this._stream = stream

    this._onReconnect = onReconnect

    this.localAddr = Multiaddr(`/p2p/${self.toB58String()}`)
    this.remoteAddr = Multiaddr(`/p2p/${counterparty.toB58String()}`)

    this.webRTC = webRTC

    this._sinkSourceDone = true

    this.source = this._createSource.call(this)

    this._stream.sink(this.sinkFunction())

    this.sink = this._createSink.bind(this)

    this.close = (err?: Error): Promise<void> => {
      this._defer.resolve()

      this.timeline.close = Date.now()

      return Promise.resolve()
    }
  }

  private async *_createSource() {
    log(`source called`)
    let streamClosed = false

    const closePromise = this._defer.promise.then(() => {
      streamClosed = true
    })

    let streamResolved = false
    let streamMsg: Uint8Array | void
    let streamDone = false

    function sourceFunction({ done, value }: { done?: boolean; value?: Uint8Array | void }) {
      streamResolved = true
      streamMsg = value

      if (done) {
        streamDone = done
      }
    }

    let __reconnectCounter = 0

    let streamPromise = this._stream.source.next().then(sourceFunction)

    while (true) {
      log(`source iteration`)
      if (!streamDone) {
        await Promise.race([
          // prettier-ignore
          streamPromise,
          //switchPromise,
          closePromise
        ])
      } else {
        //await switchPromise
      }

      if (streamResolved) {
        log(`handling message`)
        streamResolved = false

        if (streamMsg != null) {
          const received = (streamMsg as Uint8Array).slice()

          log(`received`, received)

          const [PREFIX, SUFFIX] = [received.subarray(0, 1), received.subarray(1)]

          if (u8aEquals(PREFIX, RELAY_PAYLOAD_PREFIX)) {
            log(`RelayConnection: Delivering`, SUFFIX)
            if (streamDone || streamClosed) {
              this._destroyed = true
              return SUFFIX
            } else {
              yield SUFFIX
            }
          } else if (u8aEquals(PREFIX, RELAY_STATUS_PREFIX)) {
            if (u8aEquals(SUFFIX, STOP)) {
              this._destroyed = true
              break
            } else if (u8aEquals(SUFFIX, RESTART)) {
              log(`RESTART received, reconnectReceived: ${__reconnectCounter++}. Ending stream ...`)

              this._sinkSourceDone = true

              this._onReconnect(this)

              // log(`after reconnect`, (await this._stream.source.next()))

              // end stream
              break
              // @TODO replace timeout by something more meaningful
              // await new Promise((resolve) => setTimeout(resolve, 100))
            } else if (u8aEquals(SUFFIX, PING)) {
              log(`PING received`)
              this._statusMessages.push(u8aConcat(RELAY_STATUS_PREFIX, PING_RESPONSE))

              let tmpPromise = this._statusMessagePromise
              tmpPromise.resolve()

              if (!streamDone) {
                streamPromise = this._stream.source.next().then(sourceFunction)
              }

              // Don't forward ping to receiver
              continue
            } else {
              error(`Received invalid status message ${u8aToHex(SUFFIX || new Uint8Array([]))}. Dropping message.`)
            }
          } else if (u8aEquals(PREFIX, RELAY_WEBRTC_PREFIX)) {
            // console.log(`Receiving fancy WebRTC message`, JSON.parse(new TextDecoder().decode(received.slice(1))))
            this.webRTC?.signal(JSON.parse(new TextDecoder().decode(received.slice(1))))
          } else {
            error(`Received invalid prefix <${u8aToHex(PREFIX || new Uint8Array([]))}. Dropping message.`)
          }
        } else {
          log(`dropping empty message in source function`)
        }

        streamPromise = this._stream.source.next().then(sourceFunction)

        // } else if (streamSwitched) {
        //   log(`################### streamSwitched relayConnection ###################`)
        //   break
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
        return
      }
    }

    console.log(`returned`)
  }

  private async _createSink(source: Stream['source']) {
    this._switchPromise.resolve(source)
    log(`RelayConnection: sink triggered`)

    this._sinkTriggered = true
  }

  private async *_getWebRTCStream(this: RelayConnection) {
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

  private async *sinkFunction(this: RelayConnection) {
    let streamResolved = false
    let streamMsg: Uint8Array | void

    let webRTCresolved = false
    let webRTCdone = this.webRTC == null
    let webRTCmsg: Uint8Array | void

    let streamClosed = false

    const closePromise = this._defer.promise.then(() => {
      streamClosed = true
    })

    let currentSource: Stream['source']
    let tmpSource: Stream['source']
    let streamPromise: Promise<void>

    const streamSourceFunction = ({ done, value }: { done?: boolean; value?: Uint8Array | void }) => {
      streamResolved = true
      streamMsg = value

      if (done) {
        this._sinkSourceDone = true
      }
    }

    const webRTCSourceFunction = ({ done, value }: { done?: boolean; value: Uint8Array | void }) => {
      webRTCresolved = true
      webRTCmsg = value

      if (done) {
        webRTCdone = true
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
      log(`RelayConnection: source received`)
      streamSwitched = true
      tmpSource = newSource
    }

    let switchPromise = this._switchPromise.promise.then(switchFunction)

    while (true) {
      log(`RelayConnection: sink iteration`, webRTCdone, this._sinkSourceDone)
      if (!webRTCdone && !this._sinkSourceDone) {
        log(`RelayConnection: !webRTCdone && !streamDone`)
        if (streamPromise == null) {
          streamPromise = currentSource.next().then(streamSourceFunction)
        }
        if (webRTCstream == null) {
          webRTCstream = this._getWebRTCStream.call(this)
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
        log(`RelayConnection: !webRTCdone`)

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
      } else if (!this._sinkSourceDone) {
        log(`RelayConnection: !streamDone`, currentSource)

        if (streamPromise == null) {
          streamPromise = currentSource.next().then(streamSourceFunction)
        }

        await Promise.race([
          // prettier-ignore
          streamPromise,
          statusPromise,
          switchPromise,
          closePromise
        ])
      } else {
        log(`RelayConnection: streamDone && webRTCdone`, currentSource)
        await Promise.race([
          // prettier-ignore
          statusPromise,
          switchPromise,
          closePromise
        ])
      }

      log(`RelayConnection: sink after await`, streamResolved, webRTCresolved, statusMessageAvailable, streamSwitched)

      if (streamResolved) {
        streamResolved = false

        // Drop empty messages
        if (streamMsg != null) {
          log(`RelayConnection: sink yielding`, streamMsg)
          yield (new BL([
            (RELAY_PAYLOAD_PREFIX as unknown) as BL,
            (streamMsg as unknown) as BL
          ]) as unknown) as Uint8Array
        } else {
          log(`dropping empty message in relayConnection [in sinkFunction]`)
        }

        if (streamClosed || (this._sinkSourceDone && webRTCdone)) {
          this._destroyed = true

          return u8aConcat(RELAY_STATUS_PREFIX, STOP)
        } else {
          streamPromise = currentSource.next().then(streamSourceFunction)
          log(`RelayConnection: sink -> currentSource.next()`, streamPromise)
        }
      } else if (webRTCresolved) {
        webRTCresolved = false

        // Drop empty WebRTC messages
        if (webRTCmsg != null) {
          yield (new BL([
            (RELAY_WEBRTC_PREFIX as unknown) as BL,
            (webRTCmsg as unknown) as BL
          ]) as unknown) as Uint8Array
        }

        if (!webRTCdone) {
          webRTCPromise = webRTCstream.next().then(webRTCSourceFunction)
        }
      } else if (streamClosed) {
        if (!this._destroyed) {
          this._destroyed = true

          return u8aConcat(RELAY_STATUS_PREFIX, STOP)
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
        streamSwitched = false
        currentSource = tmpSource
        this._switchPromise = Defer<Stream['source']>()
        this._sinkSourceDone = false
        streamPromise = currentSource.next().then(streamSourceFunction)
        switchPromise = this._switchPromise.promise.then(switchFunction)
        log(`RelayConnection: sink migrated`, currentSource)
      }
    }
  }

  switch(): MultiaddrConnection {
    log(`updating`)

    return {
      ...this,
      source: this._createSource(),
      sink: this._createSink.bind(this)
    }
  }

  get destroyed(): boolean {
    return this._destroyed
  }
}

export { RelayConnection }
