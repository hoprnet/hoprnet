import { Multiaddr } from 'multiaddr'
import type { MultiaddrConnection } from 'libp2p-interfaces/src/transport/types'
import type { Stream, StreamSink, StreamSource, StreamSourceAsync, StreamResult, StreamType } from '../types'
import { randomBytes } from 'crypto'
import { RelayPrefix, ConnectionStatusMessages, StatusMessages } from '../constants'
import { u8aEquals, u8aToHex, defer, type DeferType } from '@hoprnet/hopr-utils'
import Heap from 'heap-js'

import type { Instance as SimplePeer } from 'simple-peer'
import type PeerId from 'peer-id'

import Debug from 'debug'
import { EventEmitter } from 'events'
import { toU8aStream, eagerIterator } from '../utils'
import assert from 'assert'

const DEBUG_PREFIX = 'hopr-connect'

const _log = Debug(DEBUG_PREFIX)
const _verbose = Debug(`${DEBUG_PREFIX}:verbose`)
const _flow = Debug(`flow:${DEBUG_PREFIX}`)
const _error = Debug(`${DEBUG_PREFIX}:error`)

type WebRTC = {
  channel: SimplePeer
  upgradeInbound: () => SimplePeer
}

export function statusMessagesCompare(a: Uint8Array, b: Uint8Array): -1 | 0 | 1 {
  switch (a[0] as RelayPrefix) {
    case RelayPrefix.CONNECTION_STATUS:
      switch (b[0] as RelayPrefix) {
        case RelayPrefix.CONNECTION_STATUS:
          return 0
        default:
          return -1
      }
    case RelayPrefix.STATUS_MESSAGE:
      switch (b[0] as RelayPrefix) {
        case RelayPrefix.CONNECTION_STATUS:
          return 1
        case RelayPrefix.STATUS_MESSAGE:
          return 0
        default:
          return -1
      }
    case RelayPrefix.WEBRTC_SIGNALLING:
      switch (b[0] as RelayPrefix) {
        case RelayPrefix.CONNECTION_STATUS:
        case RelayPrefix.STATUS_MESSAGE:
          return 1
        case RelayPrefix.WEBRTC_SIGNALLING:
          return 0
        default:
          return -1
      }
    case RelayPrefix.PAYLOAD:
      switch (b[0] as RelayPrefix) {
        case RelayPrefix.CONNECTION_STATUS:
        case RelayPrefix.STATUS_MESSAGE:
        case RelayPrefix.WEBRTC_SIGNALLING:
          return 1
        case RelayPrefix.PAYLOAD:
          return 0
      }
  }
}

/**
 * Encapsulates the client-side state management of a relayed connection
 */
class RelayConnection extends EventEmitter implements MultiaddrConnection {
  private _stream: Stream
  private _sourceIterator: AsyncIterator<StreamType>
  private _sinkSourceAttached: boolean
  private _sinkSourceSwitched: boolean
  private _sourceSwitched: boolean
  private _streamClosed: boolean

  private statusMessages: Heap<Uint8Array>

  public _iteration: number

  private _id: string

  // Mutexes
  private _sinkSourceAttachedPromise: DeferType<StreamSource>
  private _sinkSwitchPromise: DeferType<void>
  private _sourceSwitchPromise: DeferType<void>
  private _migrationDone: DeferType<void> | undefined
  private _destroyedPromise: DeferType<void>
  private _statusMessagePromise: DeferType<void>
  private _closePromise: DeferType<void>

  private _onReconnect: ((newStream: RelayConnection, counterparty: PeerId) => Promise<void>) | undefined

  public destroyed: boolean

  public webRTC?: WebRTC

  public localAddr: Multiaddr
  public remoteAddr: Multiaddr

  private _counterparty: PeerId

  public source: StreamSourceAsync
  public sink: StreamSink

  public conn: Stream

  public timeline: MultiaddrConnection['timeline']

  constructor(opts: {
    stream: Stream
    self: PeerId
    relay: PeerId
    counterparty: PeerId
    onReconnect?: (newStream: RelayConnection, counterparty: PeerId) => Promise<void>
    webRTC?: WebRTC
  }) {
    super()

    this.timeline = {
      open: Date.now()
    }

    this.statusMessages = new Heap(statusMessagesCompare)

    this.destroyed = false

    this._stream = opts.stream

    this.conn = opts.stream

    this._onReconnect = opts.onReconnect

    this._counterparty = opts.counterparty

    this._id = u8aToHex(randomBytes(4), false)

    this.localAddr = new Multiaddr(`/p2p/${opts.relay.toB58String()}/p2p-circuit/p2p/${opts.self.toB58String()}`)
    this.remoteAddr = new Multiaddr(
      `/p2p/${opts.relay.toB58String()}/p2p-circuit/p2p/${opts.counterparty.toB58String()}`
    )

    this.webRTC = opts.webRTC

    this._iteration = 0

    this._sinkSourceAttached = false
    this._sinkSourceSwitched = false
    this._sourceSwitched = false
    this._streamClosed = false

    this._closePromise = defer<void>()
    this._sinkSourceAttachedPromise = defer<StreamSource>()
    this._destroyedPromise = defer<void>()
    this._statusMessagePromise = defer<void>()
    this._sinkSwitchPromise = defer<void>()
    this._sourceSwitchPromise = defer<void>()

    this._sourceIterator = (this._stream.source as AsyncIterable<StreamType>)[Symbol.asyncIterator]()

    // FIXME: The type between iterator/async-iterator cannot be matched in
    // this case easily.
    // @ts-ignore
    this.source = this.createSource()

    // Auto-start sink stream and declare variable in advance
    // to make sure we can attach an error handler to it
    let sinkCreator: Promise<void>
    this.sink = async (source: StreamSource) => {
      if (this._migrationDone != undefined) {
        await this._migrationDone.promise
      }

      let deferred = defer<void>()
      // forward errors
      sinkCreator.catch(deferred.reject)

      this._sinkSourceAttached = true
      this._sinkSourceAttachedPromise.resolve(
        async function* (this: RelayConnection) {
          try {
            yield* toU8aStream(source)
            deferred.resolve()
          } catch (err: any) {
            this.queueStatusMessage(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))
            deferred.reject(err)
          }
        }.call(this)
      )

      return deferred.promise
    }

    sinkCreator = this._stream.sink(this.sinkFunction())

    // catch errors that occur before attaching a sink source stream
    sinkCreator.catch((err) => this.error('sink error thrown before sink attach', err.message))
  }

  /**
   * Closes the connection
   * @param err Pass an error if necessary
   */
  public async close(err?: Error): Promise<void> {
    if (err) {
      this.error(`closed called: Error:`, err)
    } else {
      this.verbose(`close called. No error`)
    }

    this.flow(`FLOW: queueing STOP`)
    this.queueStatusMessage(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))

    if (this.destroyed) {
      this.flow(`FLOW: connection already destroyed, finish`)
      return
    }

    this.flow(`FLOW: setting closed`)
    this.setClosed()

    this.flow(`FLOW: awaiting destroyed promise / timeout`)
    // @TODO remove timeout once issue with destroyPromise is solved
    await Promise.race([new Promise((resolve) => setTimeout(resolve, 100)), this._destroyedPromise.promise])
    this.flow(`FLOW: close complete, finish`)
  }

  /**
   * Send UPGRADED status msg to the relay, so it can free the slot
   */
  public sendUpgraded() {
    this.flow(`FLOW: sending UPGRADED`)
    this.queueStatusMessage(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.UPGRADED))
  }

  /**
   * Log messages and add identity tag to distinguish multiple instances
   */
  private log(..._: any[]) {
    _log(`RC [${this._id}]`, ...arguments)
  }

  /**
   * Log verbose messages and add identity tag to distinguish multiple instances
   */
  private verbose(..._: any[]) {
    _verbose(`RC [${this._id}]`, ...arguments)
  }

  /**
   * Log errors and add identity tag to distinguish multiple instances
   */
  private error(..._: any[]) {
    _error(`RC [${this._id}]`, ...arguments)
  }

  /**
   * Log errors and add identity tag to distinguish multiple instances
   */
  private flow(..._: any[]) {
    _flow(`RC [${this._id}]`, ...arguments)
  }

  /**
   * Creates a new connection and initiates a handover to the
   * new connection end
   * @returns a new connection end
   */
  public switch(): RelayConnection {
    if (this.webRTC != undefined) {
      try {
        this.webRTC.channel.destroy()
      } catch {}
    }

    this._migrationDone = defer<void>()
    this._iteration++
    this._sinkSourceSwitched = true
    this._sinkSwitchPromise.resolve()
    this._sourceSwitched = true
    this._sourceSwitchPromise.resolve()

    if (this.webRTC != undefined) {
      this.webRTC.channel = this.webRTC.upgradeInbound()
    }

    // FIXME: The type between iterator/async-iterator cannot be matched in
    // this case easily.
    // @ts-ignore
    this.source = this.createSource()

    return this
  }

  /**
   * Marks the stream internally as closed
   */
  private setClosed() {
    this._streamClosed = true
    this._closePromise.resolve()
    this.timeline.close = Date.now()
  }

  /**
   * Starts the communication with the relay and exchanges status information
   * and control messages.
   * Once a source is attached, forward the messages from the source to the relay.
   */
  private async *sinkFunction(): StreamSource {
    type SinkType = StreamSource | StreamResult | undefined | void

    let currentSource: AsyncIterator<StreamType> | undefined
    let streamPromise: Promise<StreamResult> | undefined

    let result: SinkType

    while (true) {
      let promises: Promise<SinkType>[] = []

      let resolvedPromiseName

      const pushPromise = (promise: Promise<SinkType>, name: string) => {
        promises.push(
          promise.then((res) => {
            resolvedPromiseName = name
            return res
          })
        )
      }

      // Wait for stream close and stream switch signals
      pushPromise(this._closePromise.promise, 'close')
      pushPromise(this._sinkSwitchPromise.promise, 'sinkSwitch')

      // Wait for source being attached to sink
      if (currentSource == undefined) {
        pushPromise(this._sinkSourceAttachedPromise.promise, 'sinkSourceAttached')
      }

      // Wait for status messages
      pushPromise(this._statusMessagePromise.promise, 'statusMessage')

      // Wait for payload messages
      if (currentSource != undefined) {
        streamPromise = streamPromise ?? currentSource.next()

        pushPromise(streamPromise, 'payload')
      }

      // 1. Handle stream close
      // 2. Handle stream switch
      // 3. Handle source attach
      // 4. Handle status messages
      // 5. Handle payload messages
      this.flow(`FLOW: outgoing: awaiting promises`)
      result = await Promise.race(promises)
      this.flow(`FLOW: outgoing: promise ${resolvedPromiseName} resolved`)

      // Stream is done, nothing to do
      if (this._streamClosed && this.destroyed) {
        this.flow(`FLOW: stream is closed, break`)
        break
      }

      // Stream switched and currently no source available,
      // wait until new source gets attached
      if (this._sinkSourceSwitched) {
        this._sinkSourceSwitched = false
        this._sinkSwitchPromise = defer<void>()

        // Make sure that we don't create hanging promises
        this._sinkSourceAttachedPromise.resolve()
        this._sinkSourceAttachedPromise = defer<StreamSource>()
        result = undefined
        currentSource = undefined
        streamPromise = undefined
        this._migrationDone?.resolve()
        this.flow(`FLOW: stream switched, continue`)
        continue
      }

      // Source got attached, start forwarding messages
      if (this._sinkSourceAttached) {
        this._sinkSourceAttached = false

        currentSource = (result as AsyncIterable<StreamType>)[Symbol.asyncIterator]()

        streamPromise = undefined
        result = undefined
        this.flow(`FLOW: source attached, forwarding`)
        continue
      }

      // Status messages are available, take the first one and forward it
      if (this.statusMessages.length > 0) {
        const statusMsg = this.unqueueStatusMessage()

        if (u8aEquals(statusMsg, Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))) {
          this.destroyed = true
          this._destroyedPromise.resolve()

          this.flow(`FLOW: STOP received, break`)
          yield statusMsg
          break
        }

        this.flow(`FLOW: unrelated status message received, continue`)
        yield statusMsg

        continue
      }

      const received = result as StreamResult

      if (received == undefined) {
        throw Error(`Received must not be undefined`)
      }

      if (received.done) {
        currentSource = undefined
        streamPromise = undefined
        this.flow(`FLOW: received.done == true, break`)
        break
      }

      result = undefined
      assert(currentSource != undefined)
      streamPromise = currentSource.next()

      this.flow(`FLOW: loop end`)

      yield Uint8Array.from([RelayPrefix.PAYLOAD, ...received.value.slice()])
    }
    this.flow(`FLOW: breaked out the loop`)
  }

  /**
   * Returns incoming payload messages and handles status and control messages.
   * @returns an async iterator yielding incoming payload messages
   */
  private createSource() {
    // migration mutex
    let migrationDone = defer<void>()

    const iterator = async function* (this: RelayConnection) {
      // deep-clone number
      // @TOOD make sure that the compiler does not notice
      const drainIteration = parseInt(this._iteration.toString())

      let result: StreamResult | undefined

      let streamPromise = this._sourceIterator.next()

      const next = () => {
        result = undefined
        streamPromise = this._sourceIterator.next()
      }

      if (this.webRTC != undefined) {
        this.attachWebRTCListeners(drainIteration)
      }

      while (this._iteration == drainIteration) {
        this.flow(`FLOW: incoming: new loop iteration`)
        const promises: Promise<any>[] = []

        let resolvedPromiseName

        const pushPromise = (promise: Promise<any>, name: string) => {
          promises.push(
            promise.then((res) => {
              resolvedPromiseName = name
              return res
            })
          )
        }
        // Wait for stream close attempts
        pushPromise(this._closePromise.promise, 'close')

        // Wait for stream switches
        if (!this._sourceSwitched) {
          pushPromise(this._sourceSwitchPromise.promise, 'sourceSwitch')
        }

        // Wait for payload messages
        pushPromise(streamPromise, 'payload')

        result = (await Promise.race(promises)) as any

        this.flow(`FLOW: incoming: promise ${resolvedPromiseName} resolved`)

        // End stream once new instance is used
        if (this._iteration != drainIteration) {
          await migrationDone.promise
          migrationDone = defer<void>()
          this._sourceSwitchPromise = defer<void>()
          this._sourceSwitched = false
          break
        }

        if (this._streamClosed) {
          this.flow(`FLOW: stream closed`)
          break
        }

        if (result == undefined && this._sourceSwitched) {
          migrationDone.resolve()
          this.flow(`FLOW: migration done`)
          continue
        }

        const received = result as StreamResult

        if (received.done) {
          // @TODO how to proceed ???
          this.flow(`FLOW: received done`)
          break
        }

        if (received.value.length == 0) {
          next()
          this.verbose(`Ignoring empty message`)
          continue
        }

        const [PREFIX, SUFFIX] = [received.value.slice(0, 1), received.value.slice(1)]

        if (SUFFIX.length == 0) {
          next()
          this.verbose(`Ignoring empty payload`)
          continue
        }

        // Handle relay sub-protocol
        if (PREFIX[0] == RelayPrefix.CONNECTION_STATUS) {
          if (SUFFIX[0] == ConnectionStatusMessages.STOP) {
            this.log(`STOP received. Ending stream ...`)
            this.destroyed = true
            this._destroyedPromise.resolve()
            this.setClosed()
            break
          } else if (SUFFIX[0] == ConnectionStatusMessages.RESTART) {
            this.log(`RESTART received. Ending stream ...`)
            this.emit(`restart`)

            // First switch, then call _onReconnect to make sure
            // values are set, even if _onReconnect throws
            let switchedConnection = this.switch()

            this._onReconnect?.(switchedConnection, this._counterparty)

            continue
          }
        } else if (PREFIX[0] == RelayPrefix.STATUS_MESSAGE) {
          if (SUFFIX[0] == StatusMessages.PING) {
            this.verbose(`PING received`)
            this.queueStatusMessage(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))
            // Don't forward ping to receiver
          } else if (SUFFIX[0] == StatusMessages.PONG) {
            // noop
          } else {
            this.error(`Received invalid status message ${u8aToHex(SUFFIX || new Uint8Array([]))}. Dropping message.`)
          }

          next()
          continue
        } else if (PREFIX[0] == RelayPrefix.WEBRTC_SIGNALLING) {
          let decoded: Object | undefined
          try {
            decoded = JSON.parse(new TextDecoder().decode(SUFFIX))
          } catch {
            this.error(`Error while trying to decode JSON-encoded WebRTC message`)
          }

          if (decoded != undefined && this.webRTC != undefined && !this.webRTC.channel.connected) {
            try {
              this.webRTC?.channel.signal(decoded as any)
            } catch (err) {
              this.error(`WebRTC error:`, err)
            }
          }

          next()
          continue
        }

        // Forward payload message
        next()
        yield SUFFIX
      }
    }.call(this)

    return eagerIterator(iterator)
  }

  /**
   * Attaches a listener to the WebRTC 'signal' events
   * and removes it once class iteration increases
   * @param drainIteration index of current iteration
   */
  private attachWebRTCListeners(drainIteration: number) {
    let currentChannel: SimplePeer
    let onSignalListener: (data: Object) => void

    const onSignal = (data: Object) => {
      if (this._iteration != drainIteration) {
        currentChannel.removeListener('signal', onSignalListener)

        return
      }

      this.queueStatusMessage(
        Uint8Array.from([RelayPrefix.WEBRTC_SIGNALLING, ...new TextEncoder().encode(JSON.stringify(data))])
      )
    }
    // Store bound listener instance
    onSignalListener = onSignal.bind(this)
    currentChannel = this.webRTC?.channel.on('signal', onSignalListener) as SimplePeer
  }

  /**
   * Adds a message to the message queue and notifies source
   * that a message is available
   * @param msg message to add
   */
  private queueStatusMessage(msg: Uint8Array) {
    this.statusMessages.push(msg)
    this._statusMessagePromise.resolve()
  }

  /**
   * Removes the most recent status message from the queue
   * @returns most recent status message
   */
  private unqueueStatusMessage(): Uint8Array {
    switch (this.statusMessages.length) {
      case 0:
        throw Error(`No status messages available`)
      case 1:
        this._statusMessagePromise = defer<void>()

        return this.statusMessages.pop() as Uint8Array
      default:
        const stopMessage = Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP)

        if (u8aEquals(this.statusMessages.top(0)[0], stopMessage)) {
          this.statusMessages.clear()
          return stopMessage
        }

        return this.statusMessages.pop() as Uint8Array
    }
  }
}

export { RelayConnection }
