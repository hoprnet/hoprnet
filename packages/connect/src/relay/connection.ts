import type { MultiaddrConnection } from '@libp2p/interface-connection'
import type {
  Stream,
  StreamSink,
  StreamSource,
  StreamSourceAsync,
  StreamResult,
  StreamType,
  HoprConnectTestingOptions
} from '../types.js'
import { randomBytes } from 'crypto'
import { RelayPrefix, ConnectionStatusMessages, StatusMessages } from '../constants.js'
import { u8aEquals, u8aToHex, defer, createCircuitAddress, type DeferType } from '@hoprnet/hopr-utils'
import HeapPkg, { type Heap as HeapType } from 'heap-js'

import SimplePeer from 'simple-peer'
import type { PeerId } from '@libp2p/interface-peer-id'

import Debug from 'debug'
import { EventEmitter } from 'events'
import { toU8aStream, eagerIterator } from '../utils/index.js'
import assert from 'assert'
import type { ConnectComponents } from '../components.js'
import type { WebRTCUpgrader } from '../webrtc/upgrader.js'

const { Heap } = HeapPkg

const DEBUG_PREFIX = 'hopr-connect'

const _log = Debug(DEBUG_PREFIX)
const _verbose = Debug(`${DEBUG_PREFIX}:verbose`)
const _flow = Debug(`flow:${DEBUG_PREFIX}`)
const _error = Debug(`${DEBUG_PREFIX}:error`)

// Sort status messsages according to importance
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

enum ConnectionEventTypes {
  CLOSE,
  SINK_SOURCE_ATTACHED,
  STATUS_MESSAGE,
  PAYLOAD,
  SINK_SWITCH,
  SOURCE_SWITCH
}

type CloseEvent = {
  type: ConnectionEventTypes.CLOSE
}

type SinkSourceAttachedEvent = {
  type: ConnectionEventTypes.SINK_SOURCE_ATTACHED
} & (
  | {
      value: StreamSourceAsync
      ignore: false
    }
  | {
      ignore: true
    }
)

type StatusMessageEvent = {
  type: ConnectionEventTypes.STATUS_MESSAGE
}

type PayloadEvent = {
  type: ConnectionEventTypes.PAYLOAD
  value: StreamResult
}

type SinkSwitchEvent = {
  type: ConnectionEventTypes.SINK_SWITCH
}

type SourceSwitchEvent = {
  type: ConnectionEventTypes.SOURCE_SWITCH
}

type SinkEvent = CloseEvent | SinkSourceAttachedEvent | StatusMessageEvent | PayloadEvent | SinkSwitchEvent
type SourceEvent = CloseEvent | SourceSwitchEvent | PayloadEvent

const RELAYED_CONNECTION_RESTART = 'restart'

// Extracts function type but ignores type of `this`
// Used as type for context-bounded functions
type FunctionType<K extends keyof T, T> = T[K] extends (...args: infer P) => infer R ? (...args: P) => R : never

/**
 * Encapsulates the client-side stream state management of a relayed connection
 *
 *          ┌───────────┐       ┌─────────┐
 *  Stream  │Connection ├─────┐►│Stream   │
 * ────────►│           │     │ └─────────┘
 *          │           ├──┐  │
 *          └───────────┘  │  │ ┌─────────┐
 *                         │  └►│WebRTC   │
 *                         │    └─────────┘
 *                         │
 *                         │     after reconnect
 *                         │    ┌─────────┐
 *                         └──┐►│Stream'  │
 *                            │ └─────────┘
 *                            │
 *                            │ ┌─────────┐
 *                            └►│WebRTC'  │
 *                              └─────────┘
 *
 * Listens to protocol messages and multiplexes WebRTC signalling
 * into WebRTC instance.
 *
 * Once there was a reconnect at the relay, create a new WebRTC
 * instance and a *new* connection endpoint which get passed to
 * libp2p as *new* stream
 */
class RelayConnection extends EventEmitter implements MultiaddrConnection {
  private readonly _id: string

  public readonly state: {
    _sinkSourceAttachedPromise: DeferType<SinkSourceAttachedEvent>
    _sinkSwitchPromise: DeferType<SinkSwitchEvent>
    _sourceSwitchPromise: DeferType<SourceSwitchEvent>
    _migrationDone: DeferType<void> | undefined
    _destroyedPromise: DeferType<void>
    _statusMessagePromise: DeferType<StatusMessageEvent>
    _closePromise: DeferType<CloseEvent>
    destroyed: boolean
    _sourceIterator: AsyncIterator<StreamType>
    _sourceSwitched: boolean
    _iteration: number
    statusMessages: HeapType<Uint8Array>
    channel: SimplePeer.Instance | undefined
    source: AsyncIterable<StreamType>
  }

  public logging: {
    log: (...args: any[]) => void
    verbose: (...args: any[]) => void
    error: (...args: any[]) => void
    flow: (...args: any[]) => void
  }

  public _queueStatusMessage: FunctionType<'queueStatusMessage', RelayConnection>
  public _unqueueStatusMessage: FunctionType<'unqueueStatusMessage', RelayConnection>

  public _setClosed: FunctionType<'setClosed', RelayConnection>
  public _switch: FunctionType<'switch', RelayConnection>
  public _attachWebRTCListeners: FunctionType<'attachWebRTCListeners', RelayConnection>
  public _upgradeInbound: WebRTCUpgrader['upgradeInbound'] | undefined
  public _emitRestart: () => void

  public readonly remoteAddr: MultiaddrConnection['remoteAddr']

  public readonly _counterparty: PeerId

  // Current connection endpoint to be used by libp2p
  public readonly sink: StreamSink

  public conn: Stream

  public readonly timeline: MultiaddrConnection['timeline']

  constructor(
    private _stream: Stream,
    relay: PeerId,
    counterparty: PeerId,
    direction: 'inbound' | 'outbound',
    public connectComponents: ConnectComponents,
    public testingOptions: HoprConnectTestingOptions,
    public _onReconnect: (newStream: RelayConnection, counterparty: PeerId) => Promise<void>,
    state?: RelayConnection['state']
  ) {
    super()

    // Create a unique id to distinguish multiple instances
    this._id = u8aToHex(randomBytes(4), false)

    this.logging = {
      log: this.log.bind({ _id: this._id }),
      verbose: this.verbose.bind({ _id: this._id }),
      error: this.error.bind({ _id: this._id }),
      flow: this.flow.bind({ _id: this._id })
    }

    // Set *close* property to notify libp2p that
    // stream was closed
    this.timeline = {
      open: Date.now()
    }

    // Internal status message buffer
    this.conn = _stream

    this._counterparty = counterparty

    this.remoteAddr = createCircuitAddress(relay, counterparty)

    // Pre-generate object to attach to function pointers
    this.state = state ?? ({} as any)

    this._queueStatusMessage = this.queueStatusMessage.bind({
      state: this.state
    })

    this._unqueueStatusMessage = this.unqueueStatusMessage.bind({
      state: this.state
    })

    this._setClosed = this.setClosed.bind({
      timeline: this.timeline,
      state: this.state
    })

    this._attachWebRTCListeners = this.attachWebRTCListeners.bind({
      _queueStatusMessage: this._queueStatusMessage,
      state: this.state
    })

    this._switch = this.switch.bind(this)

    // For testing fallback relayed connection, disable WebRTC upgrade attempts
    if (!this.testingOptions.__noWebRTCUpgrade) {
      this._upgradeInbound = this.connectComponents
        .getWebRTCUpgrader()
        .upgradeInbound.bind(this.connectComponents.getWebRTCUpgrader())

      switch (direction) {
        case 'inbound':
          Object.assign(this.state, {
            channel: this.connectComponents.getWebRTCUpgrader().upgradeInbound()
          })
          break
        case 'outbound':
          Object.assign(this.state, {
            channel: this.connectComponents.getWebRTCUpgrader().upgradeOutbound()
          })
      }
    }

    this._emitRestart = () => this.emit(RELAYED_CONNECTION_RESTART)

    Object.assign(this.state, {
      // After reconnect, deprecate old stream
      _iteration: 0,
      destroyed: false,
      // Set to true during stream migration
      _sourceSwitched: false,
      _closePromise: defer<CloseEvent>(),
      _sinkSourceAttachedPromise: defer<SinkSourceAttachedEvent>(),
      _destroyedPromise: defer<void>(),
      _statusMessagePromise: defer<StatusMessageEvent>(),
      _sinkSwitchPromise: defer<SinkSwitchEvent>(),
      _sourceSwitchPromise: defer<SourceSwitchEvent>(),
      _migrationDone: undefined,
      _sourceIterator: (this._stream.source as AsyncIterable<Uint8Array>)[Symbol.asyncIterator](),
      statusMessages: new Heap(statusMessagesCompare)
    })

    Object.assign(this.state, {
      source: this.createSource()
    })

    // Auto-start sink stream and declare variable in advance
    // to make sure we can attach an error handler to it
    const sinkCreator = this._stream.sink(
      this.sinkFunction.call({
        unqueueStatusMessage: this._unqueueStatusMessage,
        state: this.state,
        logging: this.logging
      })
    )

    // catch errors that occur before attaching a sink source stream
    sinkCreator.catch((err) => this.error('sink error thrown before sink attach', err.message))

    // Stream sink gets passed as function handle, so we
    // need to explicitly bind it to an environment
    this.sink = this._sink.bind({
      state: this.state,
      _queueStatusMessage: this._queueStatusMessage,
      sinkCreator
    })
  }

  public async _sink(
    this: Pick<RelayConnection, 'state' | '_queueStatusMessage'> & { sinkCreator: Promise<void> },
    source: StreamSource
  ): Promise<void> {
    if (this.state._migrationDone != undefined) {
      await this.state._migrationDone.promise
    }

    let deferred = defer<void>()
    // forward errors
    this.sinkCreator.catch(deferred.reject)

    this.state._sinkSourceAttachedPromise.resolve({
      type: ConnectionEventTypes.SINK_SOURCE_ATTACHED,
      value: async function* (this: Pick<RelayConnection, '_queueStatusMessage'>) {
        try {
          yield* toU8aStream(source) as StreamSourceAsync
          deferred.resolve()
        } catch (err: any) {
          this._queueStatusMessage(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))
          deferred.reject(err)
        }
      }.call({
        _queueStatusMessage: this._queueStatusMessage
      }),
      ignore: false
    })

    return deferred.promise
  }

  public get source() {
    return this.state.source
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

    if (this.state.destroyed) {
      this.flow(`FLOW: connection already destroyed, finish`)
      return
    }

    this.flow(`FLOW: setting closed`)
    this.setClosed()

    this.flow(`FLOW: awaiting destroyed promise / timeout`)
    // @TODO remove timeout once issue with destroyPromise is solved
    await Promise.race([new Promise((resolve) => setTimeout(resolve, 100)), this.state._destroyedPromise.promise])
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
  public switch(this: RelayConnection): RelayConnection {
    if (this.state.channel != undefined) {
      try {
        this.state.channel.destroy()
      } catch (err) {
        this.logging.error(`Error while destroying WebRTC instance`, err)
      }
      this.state.channel = this._upgradeInbound?.()
    }

    this.state._migrationDone = defer<void>()
    this.state._iteration++
    this.state._sinkSwitchPromise.resolve({
      type: ConnectionEventTypes.SINK_SWITCH
    })
    this.state._sourceSwitched = true
    this.state._sourceSwitchPromise.resolve({
      type: ConnectionEventTypes.SOURCE_SWITCH
    })

    // FIXME: The type between iterator/async-iterator cannot be matched in
    // this case easily.
    this.state.source = this.createSource() as AsyncIterable<Uint8Array>

    return this
  }

  /**
   * Marks the stream internally as closed
   */
  public setClosed() {
    this.state._closePromise.resolve({
      type: ConnectionEventTypes.CLOSE
    })
    // Sets the magic *close* property that makes libp2p forget
    // about the connection.
    // @dev this is done implicitly by using meta programming
    this.timeline.close = Date.now()
  }

  /**
   * Starts the communication with the relay and exchanges status information
   * and control messages.
   * Once a source is attached, forward the messages from the source to the relay.
   */
  private async *sinkFunction(this: Pick<RelayConnection, 'state' | 'unqueueStatusMessage' | 'logging'>): StreamSource {
    let currentSourceIterator: AsyncIterator<StreamType> | undefined
    let nextMessagePromise: Promise<StreamResult> | undefined

    this.logging.flow(`sinkFunction called`)
    let leave = false

    while (!leave) {
      let promises: Promise<SinkEvent>[] = []

      // Wait for stream close and stream switch signals
      promises.push(this.state._closePromise.promise)
      promises.push(this.state._sinkSwitchPromise.promise)

      // Wait for source being attached to sink,
      // before that happens, there will be only status messages
      if (currentSourceIterator == undefined) {
        promises.push(this.state._sinkSourceAttachedPromise.promise)
      }

      // Wait for status messages
      promises.push(this.state._statusMessagePromise.promise)

      // Wait for payload messages
      if (currentSourceIterator != undefined) {
        // Advances the iterator if not yet happened
        nextMessagePromise = nextMessagePromise ?? currentSourceIterator.next()

        promises.push(
          nextMessagePromise.then((res: StreamResult) => ({
            type: ConnectionEventTypes.PAYLOAD,
            value: res
          }))
        )
      }

      let toYield: Uint8Array | undefined

      const result = await Promise.race(promises)

      // Something happened, let's find out what
      switch (result.type) {
        // Destroy called, so notify relay first and then tear down rest
        case ConnectionEventTypes.CLOSE:
          this.logging.flow(`FLOW: stream is closed, break`)
          if (this.state._destroyedPromise != undefined) {
            this.state._destroyedPromise.resolve()
          }
          if (!this.state.destroyed) {
            toYield = Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP)
          }
          this.state.destroyed = true

          leave = true
          break
        // Reconnect happened, cleanup state and reset mutexes
        case ConnectionEventTypes.SINK_SWITCH:
          this.state._sinkSwitchPromise = defer<SinkSwitchEvent>()

          // Make sure that we don't create hanging promises
          this.state._sinkSourceAttachedPromise.resolve({
            type: ConnectionEventTypes.SINK_SOURCE_ATTACHED,
            ignore: true
          })
          this.state._sinkSourceAttachedPromise = defer<SinkSourceAttachedEvent>()
          currentSourceIterator = undefined
          nextMessagePromise = undefined
          this.state._migrationDone?.resolve()
          this.logging.flow(`FLOW: stream switched, continue`)
          break
        // A sink stream got attached, either after initialization
        // or after a reconnect
        case ConnectionEventTypes.SINK_SOURCE_ATTACHED:
          if (result.ignore) {
            break
          }

          // Start the iterator
          currentSourceIterator = result.value[Symbol.asyncIterator]()

          nextMessagePromise = undefined
          this.logging.flow(`FLOW: source attached, forwarding`)
          break
        // There is a status message to be sent
        case ConnectionEventTypes.STATUS_MESSAGE:
          const statusMsg = this.unqueueStatusMessage()

          if (u8aEquals(statusMsg, Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))) {
            this.state.destroyed = true
            this.state._destroyedPromise.resolve()

            this.logging.flow(`FLOW: STOP received, break`)
            toYield = statusMsg
            leave = true
            break
          }
          this.logging.flow(`FLOW: unrelated status message received, continue`)
          toYield = statusMsg

          break
        // There is a payload message that we need to forward
        case ConnectionEventTypes.PAYLOAD:
          if (result.value == undefined) {
            throw Error(`Received must not be undefined`)
          }

          // No more messages to send by libp2p, so end stream
          if (result.value.done) {
            currentSourceIterator = undefined
            nextMessagePromise = undefined
            // this.flow(`FLOW: received.done == true, break`)
            leave = true
            break
          }
          assert(currentSourceIterator != undefined)

          // Advance iterator
          nextMessagePromise = currentSourceIterator.next()
          this.logging.flow(`FLOW: loop end`)

          toYield = Uint8Array.from([RelayPrefix.PAYLOAD, ...result.value.value.slice()])
          break
        default:
          throw Error(`Invalid result. Received ${result}`)
      }

      if (toYield != undefined) {
        yield toYield
      }
    }
    // this.flow(`FLOW: breaked out the loop`)
  }

  /**
   * Creates an async iterable that resolves to incoming messages.
   * Streams ends once there is a reconnect.
   * @returns an async iterator yielding incoming payload messages
   */
  private createSource() {
    // migration mutex
    let migrationDone = defer<void>()

    const iterator = async function* (
      this: Pick<
        RelayConnection,
        | 'state'
        | 'queueStatusMessage'
        | 'unqueueStatusMessage'
        | 'setClosed'
        | '_attachWebRTCListeners'
        | 'testingOptions'
        | '_onReconnect'
        | '_switch'
        | 'logging'
        | '_emitRestart'
        | '_counterparty'
      >,
      drainIteration: number
    ) {
      let result: SourceEvent

      let streamPromise = this.state._sourceIterator.next()

      const advanceIterator = () => {
        streamPromise = this.state._sourceIterator.next()
      }

      if (!this.testingOptions.__noWebRTCUpgrade) {
        // We're now ready to fetch WebRTC signalling messages
        this._attachWebRTCListeners(drainIteration)
      }

      let leave = false

      while (
        !leave &&
        // Each reconnect increases `this._iteration` and thereby
        // deprecates previous streams and ends them
        this.state._iteration == drainIteration
      ) {
        // this.flow(`FLOW: incoming: new loop iteration`)
        const promises: Promise<SourceEvent>[] = []

        // Wait for stream close attempts
        promises.push(this.state._closePromise.promise)

        // Wait for stream switches
        if (!this.state._sourceSwitched) {
          promises.push(this.state._sourceSwitchPromise.promise)
        }

        // Wait for payload messages
        promises.push(
          streamPromise.then((res) => ({
            type: ConnectionEventTypes.PAYLOAD,
            value: res
          }))
        )

        result = await Promise.race(promises)

        // End stream once new instance is used
        if (this.state._iteration != drainIteration) {
          // leave loop
          break
        }

        let toYield: Uint8Array | undefined

        switch (result.type) {
          // Stream got destroyed, so end it
          case ConnectionEventTypes.CLOSE:
            this.logging.flow(`FLOW: stream closed`)
            leave = true
            // leave loop
            break
          // A reconnect happened a source got attached
          // in next iteration, attach new source
          case ConnectionEventTypes.SOURCE_SWITCH:
            migrationDone.resolve()
            // this.flow(`FLOW: migration done`)
            break
          // We received a payload message, if it is a
          // status message, interprete it, otherwise
          // forward it to libp2p
          case ConnectionEventTypes.PAYLOAD:
            // Stream ended, so we're done here
            if (result.value.done) {
              // @TODO how to proceed ???
              // this.flow(`FLOW: received done`)
              leave = true
              // leave loop
              break
            }

            // Anything can happen
            if (result.value.value.length == 0) {
              // this.verbose(`Ignoring empty message`)
              advanceIterator()
              break
            }

            const [PREFIX, SUFFIX] = [result.value.value.slice(0, 1), result.value.value.slice(1)]

            // Anything can happen
            if (SUFFIX.length == 0) {
              advanceIterator()
              // this.verbose(`Ignoring empty payload`)
              break
            }

            // Handle relay sub-protocol
            switch (PREFIX[0]) {
              // Something on the connection happened
              case RelayPrefix.CONNECTION_STATUS:
                switch (SUFFIX[0]) {
                  // Relay asks us to stop stream
                  case ConnectionStatusMessages.STOP:
                    this.logging.log(`STOP received. Ending stream ...`)
                    this.state.destroyed = true
                    this.state._destroyedPromise.resolve()
                    this.setClosed()
                    leave = true
                    break
                  // A reconnect at the other of the relay happened,
                  // so create a new connection endpoint (stream) and
                  // pass it to libp2p
                  // Also create a new WebRTC instance because old one
                  // cannot be used anymore since other party might have
                  // migrated to different port or IP
                  case ConnectionStatusMessages.RESTART:
                    this.logging.log(`RESTART received. Ending stream ...`)
                    this._emitRestart()

                    // First switch, then call _onReconnect to make sure
                    // values are set, even if _onReconnect throws
                    let switchedConnection = this._switch()
                    // We must not await this promise because it resolves once
                    // TLS-alike handshake is done and thus creates a deadlock
                    // since the await blocks this stream
                    this._onReconnect(switchedConnection, this._counterparty)

                    await migrationDone.promise
                    migrationDone = defer<void>()
                    // @TODO resolve first
                    this.state._sourceSwitchPromise = defer<SourceSwitchEvent>()
                    this.state._sourceSwitched = false

                    advanceIterator()
                    break
                  default:
                    throw Error(`Invalid suffix. Received ${u8aToHex(SUFFIX)}`)
                }
                break
              // We received a status message. Usually used to send PING / PONG
              // messages to detect if connection works
              case RelayPrefix.STATUS_MESSAGE:
                this.logging.flow(`Received status message`)
                switch (SUFFIX[0]) {
                  case StatusMessages.PING:
                    this.logging.verbose(`PING received`)
                    this.queueStatusMessage(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))
                    break
                  case StatusMessages.PONG:
                    // noop, left for future usage
                    break
                  default:
                    this.logging.error(
                      `Received invalid status message ${u8aToHex(SUFFIX || new Uint8Array([]))}. Dropping message.`
                    )
                    break
                }
                advanceIterator()
                break
              // Upgrade to direct WebRTC is ongoing, forward ICE signalling
              // messages to WebRTC instance
              case RelayPrefix.WEBRTC_SIGNALLING:
                let decoded: SimplePeer.SignalData | undefined
                try {
                  decoded = JSON.parse(new TextDecoder().decode(SUFFIX)) as SimplePeer.SignalData
                } catch {
                  this.logging.error(`Error while trying to decode JSON-encoded WebRTC message`)
                }

                if (
                  decoded != undefined &&
                  !this.testingOptions.__noWebRTCUpgrade &&
                  this.state.channel != undefined &&
                  !this.state.channel.connected
                ) {
                  try {
                    this.state.channel.signal(decoded)
                  } catch (err) {
                    this.logging.error(`WebRTC error:`, err)
                  }
                }
                advanceIterator()
                break
              // Forward message to libp2p
              case RelayPrefix.PAYLOAD:
                toYield = SUFFIX
                advanceIterator()
                break
              default:
                throw Error(`Invalid prefix. Received ${u8aToHex(PREFIX)}`)
            }
            break
          default:
            throw Error(`Invalid result. Received ${result}`)
        }

        if (toYield != undefined) {
          yield toYield
        }
      }
    }.call(
      {
        state: this.state,
        queueStatusMessage: this._queueStatusMessage,
        unqueueStatusMessage: this._unqueueStatusMessage,
        setClosed: this._setClosed,
        _attachWebRTCListeners: this._attachWebRTCListeners,
        _switch: this._switch,
        _onReconnect: this._onReconnect,
        testingOptions: this.testingOptions,
        logging: this.logging,
        _emitRestart: this._emitRestart,
        _counterparty: this._counterparty
      },
      this.state._iteration
    )

    // We need to eagerly drain the iterator to make sure it fetches
    // status messages and WebRTC signallign messages - even before
    // libp2p decides to drain the stream
    return eagerIterator(iterator)
  }

  /**
   * Attaches a listener to the WebRTC 'signal' events
   * and removes it once there is a reconnect
   * @param drainIteration index of current iteration
   */
  public attachWebRTCListeners(this: Pick<RelayConnection, 'state' | '_queueStatusMessage'>, drainIteration: number) {
    let currentChannel: SimplePeer.Instance
    function onSignal(this: Pick<RelayConnection, 'state' | '_queueStatusMessage'>, data: Object) {
      if (this.state._iteration != drainIteration) {
        currentChannel.removeListener('signal', onSignal)

        return
      }

      this._queueStatusMessage(
        Uint8Array.from([RelayPrefix.WEBRTC_SIGNALLING, ...new TextEncoder().encode(JSON.stringify(data))])
      )
    }

    // Store bound listener instance
    currentChannel = (this.state.channel as SimplePeer.Instance).on(
      'signal',
      onSignal.bind({
        _queueStatusMessage: this._queueStatusMessage,
        state: this.state
      })
    )
  }

  /**
   * Adds a message to the message queue and notifies source
   * that a message is available
   * @param msg message to add
   */
  public queueStatusMessage(this: Pick<RelayConnection, 'state'>, msg: Uint8Array) {
    this.state.statusMessages.push(msg)
    this.state._statusMessagePromise.resolve({
      type: ConnectionEventTypes.STATUS_MESSAGE
    })
  }

  /**
   * Removes the most recent status message from the queue
   * @returns most recent status message
   */
  public unqueueStatusMessage(): Uint8Array {
    switch (this.state.statusMessages.length) {
      case 0:
        throw Error(`No status messages available`)
      case 1:
        this.state._statusMessagePromise = defer<StatusMessageEvent>()

        return this.state.statusMessages.pop() as Uint8Array
      default:
        const stopMessage = Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP)

        const nextStatusMessage = this.state.statusMessages.pop() as Uint8Array

        if (u8aEquals(nextStatusMessage, stopMessage)) {
          this.state.statusMessages.clear()
        }

        return nextStatusMessage
    }
  }
}

export { RelayConnection }
