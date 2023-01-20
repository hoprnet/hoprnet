import type { MultiaddrConnection } from '@libp2p/interface-connection'
import {
  type Stream,
  type StreamSource,
  type StreamSourceAsync,
  type StreamResult,
  type StreamType,
  type HoprConnectTestingOptions,
  PeerConnectionType
} from '../types.js'
import { randomBytes } from 'crypto'
import { RelayPrefix, ConnectionStatusMessages, StatusMessages, CODE_P2P } from '../constants.js'
import {
  u8aEquals,
  u8aToHex,
  defer,
  createCircuitAddress,
  type DeferType,
  timeout,
  create_counter
} from '@hoprnet/hopr-utils'
import HeapPkg, { type Heap as HeapType } from 'heap-js'

import SimplePeer from 'simple-peer'
import type { PeerId } from '@libp2p/interface-peer-id'

import Debug from 'debug'
import { eagerIterator } from '../utils/index.js'
import assert from 'assert'
import type { ConnectComponents } from '../components.js'

const { Heap } = HeapPkg

const relayedPackets = create_counter(
  'connect_counter_client_relayed_packets',
  'Number of relayed packets (TURN client)'
)

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

export interface RelayConnectionInterface extends MultiaddrConnection {
  source: AsyncIterable<Uint8Array>
  sendUpgraded(): void
  isDestroyed(): boolean
  getCurrentChannel(): SimplePeer.Instance | undefined
  direction: 'inbound' | 'outbound'
  tags: PeerConnectionType[]
  conn: Stream
  counterparty: PeerId
  setOnClose: (closeHandler: () => void) => void
}

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
export function RelayConnection(
  conn: Stream,
  relay: PeerId,
  counterparty: PeerId,
  direction: 'inbound' | 'outbound',
  onClose: (() => void) | undefined,
  connectComponents: ConnectComponents,
  testingOptions: HoprConnectTestingOptions,
  _onReconnect: (newStream: RelayConnectionInterface, counterparty: PeerId) => Promise<void>
): RelayConnectionInterface {
  // Create a unique id to distinguish multiple instances
  const _id = u8aToHex(randomBytes(4), false)
  const state: {
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
    channel?: SimplePeer.Instance
  } = {
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
    _sourceIterator: (conn.source as AsyncIterable<Uint8Array>)[Symbol.asyncIterator](),
    statusMessages: new Heap(statusMessagesCompare)
  }

  const timeline: MultiaddrConnection['timeline'] = {
    open: Date.now()
  }

  const remoteAddr = createCircuitAddress(relay)
    .decapsulateCode(CODE_P2P)
    .encapsulate(`/p2p/${counterparty.toString()}`)

  let upgradeInbound: (_signal?: AbortSignal) => SimplePeer.Instance

  if (!testingOptions.__noWebRTCUpgrade) {
    upgradeInbound = connectComponents.getWebRTCUpgrader().upgradeInbound.bind(connectComponents.getWebRTCUpgrader())

    switch (direction) {
      case 'inbound':
        state.channel = connectComponents.getWebRTCUpgrader().upgradeInbound()

        break
      case 'outbound':
        state.channel = connectComponents.getWebRTCUpgrader().upgradeOutbound()
        break
    }
  }

  /**
   * Log messages and add identity tag to distinguish multiple instances
   */
  const log = (...args: any[]) => {
    _log(`RC [${_id}]`, ...args)
  }

  /**
   * Log verbose messages and add identity tag to distinguish multiple instances
   */
  const verbose = (...args: any[]) => {
    _verbose(`RC [${_id}]`, ...args)
  }

  /**
   * Log errors and add identity tag to distinguish multiple instances
   */
  const error = (...args: any[]) => {
    _error(`RC [${_id}]`, ...args)
  }

  /**
   * Log errors and add identity tag to distinguish multiple instances
   */
  const flow = (...args: any[]) => {
    _flow(`RC [${_id}]`, ...args)
  }

  /**
   * Used to set a handler that is gets called once the connection is closed
   * *after* function got constructed
   * @dev used by reconnect handler
   * @param closeHandler new close handler
   */
  const setOnClose = (closeHandler: () => void) => {
    onClose = closeHandler
  }

  /**
   * Adds a message to the message queue and notifies source
   * that a message is available
   * @param msg message to add
   */
  const queueStatusMessage = (msg: Uint8Array) => {
    state.statusMessages.push(msg)
    state._statusMessagePromise.resolve({
      type: ConnectionEventTypes.STATUS_MESSAGE
    })
  }

  /**
   * Removes the most recent status message from the queue
   * @returns most recent status message
   */
  const unqueueStatusMessage = (): Uint8Array => {
    switch (state.statusMessages.length) {
      case 0:
        throw Error(`No status messages available`)
      case 1:
        state._statusMessagePromise = defer<StatusMessageEvent>()

        return state.statusMessages.pop() as Uint8Array
      default:
        const stopMessage = Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP)

        const nextStatusMessage = state.statusMessages.pop() as Uint8Array

        if (u8aEquals(nextStatusMessage, stopMessage)) {
          state.statusMessages.clear()
        }

        return nextStatusMessage
    }
  }

  /**
   * Marks the stream internally as closed
   */
  const setClosed = () => {
    state._closePromise.resolve({
      type: ConnectionEventTypes.CLOSE
    })
    // Sets the magic *close* property that makes libp2p forget
    // about the connection.
    // @dev this is done implicitly by using meta programming
    timeline.close = Date.now()
    onClose?.()
  }

  const isDestroyed = () => state.destroyed

  const getCurrentChannel = () => state.channel

  /**
   * Send UPGRADED status msg to the relay, so it can free the slot
   */
  const sendUpgraded = () => {
    flow(`FLOW: sending UPGRADED`)
    queueStatusMessage(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.UPGRADED))
  }

  /**
   * Creates a new connection and initiates a handover to the
   * new connection end
   * @returns a new connection end
   */
  const switchConnection = (): RelayConnectionInterface => {
    if (state.channel != undefined) {
      try {
        state.channel.destroy()
      } catch (err) {
        error(`Error while destroying WebRTC instance`, err)
      }
      state.channel = upgradeInbound?.()
    }

    state._migrationDone = defer<void>()
    state._iteration++
    state._sinkSwitchPromise.resolve({
      type: ConnectionEventTypes.SINK_SWITCH
    })
    state._sourceSwitched = true
    state._sourceSwitchPromise.resolve({
      type: ConnectionEventTypes.SOURCE_SWITCH
    })

    // FIXME: The type between iterator/async-iterator cannot be matched in
    // this case easily.
    source = createSource()

    return {
      sendUpgraded,
      direction,
      tags,
      // Set *close* property to notify libp2p that
      // stream was closed
      timeline,
      remoteAddr,
      conn,
      close: close(state._iteration),
      source,
      sink: sink(state._iteration),
      isDestroyed,
      getCurrentChannel,
      counterparty,
      setOnClose
    }
  }

  let onSinkError = (err: any) => {
    error(`Sink threw error before source attach`, err)
  }

  const sink = (_initialIteration: number) => async (source: AsyncIterable<Uint8Array> | Iterable<Uint8Array>) => {
    if (state._migrationDone != undefined) {
      await state._migrationDone.promise
    }

    return new Promise<void>((resolve, reject) => {
      onSinkError = reject
      state._sinkSourceAttachedPromise.resolve({
        type: ConnectionEventTypes.SINK_SOURCE_ATTACHED,
        value: (async function* () {
          try {
            yield* source as AsyncIterable<Uint8Array>
            resolve()
          } catch (err: any) {
            reject(err)
          }
          // finally {
          //   // end connection if there was no reconnect
          //   if (initialIteration == state._iteration) {
          //     queueStatusMessage(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))
          //   }
          // }
        })(),
        ignore: false
      })
    })
  }

  /**
   * Closes the connection
   * @param err Pass an error if necessary
   */
  const close =
    (initialIteration: number) =>
    async (err?: Error): Promise<void> => {
      if (initialIteration != state._iteration) {
        return
      }
      if (err) {
        error(`closed called: Error:`, err)
      } else {
        verbose(`close called. No error`)
      }

      flow(`FLOW: queueing STOP`)
      queueStatusMessage(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))

      if (state.destroyed) {
        flow(`FLOW: connection already destroyed, finish`)
        return
      }

      flow(`FLOW: setting closed`)
      setClosed()

      flow(`FLOW: awaiting destroyed promise / timeout`)
      // @TODO remove timeout once issue with destroyPromise is solved
      await timeout(100, () => state._destroyedPromise.promise)
      flow(`FLOW: close complete, finish`)
    }

  /**
   * Starts the communication with the relay and exchanges status information
   * and control messages.
   * Once a source is attached, forward the messages from the source to the relay.
   */
  async function* sinkFunction(): StreamSource {
    let currentSourceIterator: AsyncIterator<StreamType> | undefined
    let nextMessagePromise: Promise<StreamResult> | undefined

    flow(`sinkFunction called`)
    let leave = false

    while (!leave) {
      let promises: Promise<SinkEvent>[] = []

      // Wait for stream close and stream switch signals
      promises.push(state._closePromise.promise)
      promises.push(state._sinkSwitchPromise.promise)

      // Wait for source being attached to sink,
      // before that happens, there will be only status messages
      if (currentSourceIterator == undefined) {
        promises.push(state._sinkSourceAttachedPromise.promise)
      }

      // Wait for status messages
      promises.push(state._statusMessagePromise.promise)

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
          flow(`FLOW: stream is closed, break`)
          if (state._destroyedPromise != undefined) {
            state._destroyedPromise.resolve()
          }
          if (!state.destroyed) {
            toYield = Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP)
          }
          state.destroyed = true

          leave = true
          break
        // Reconnect happened, cleanup state and reset mutexes
        case ConnectionEventTypes.SINK_SWITCH:
          state._sinkSwitchPromise = defer<SinkSwitchEvent>()

          // Make sure that we don't create hanging promises
          state._sinkSourceAttachedPromise.resolve({
            type: ConnectionEventTypes.SINK_SOURCE_ATTACHED,
            ignore: true
          })
          state._sinkSourceAttachedPromise = defer<SinkSourceAttachedEvent>()
          currentSourceIterator = undefined
          nextMessagePromise = undefined
          state._migrationDone?.resolve()
          flow(`FLOW: stream switched, continue`)
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
          flow(`FLOW: source attached, forwarding`)
          break
        // There is a status message to be sent
        case ConnectionEventTypes.STATUS_MESSAGE:
          const statusMsg = unqueueStatusMessage()

          if (u8aEquals(statusMsg, Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))) {
            state.destroyed = true
            state._destroyedPromise.resolve()

            flow(`FLOW: STOP received, break`)
            toYield = statusMsg
            leave = true
            break
          }
          flow(`FLOW: unrelated status message received, continue`)
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
          flow(`FLOW: loop end`)

          relayedPackets.increment()

          toYield = Uint8Array.from([RelayPrefix.PAYLOAD, ...result.value.value.subarray()])
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
  const createSource = (): AsyncIterable<Uint8Array> => {
    // migration mutex
    let migrationDone = defer<void>()

    const iterator = (async function* (drainIteration: number): AsyncIterable<Uint8Array> {
      let result: SourceEvent

      let streamPromise = state._sourceIterator.next()

      const advanceIterator = () => {
        streamPromise = state._sourceIterator.next()
      }

      if (!testingOptions.__noWebRTCUpgrade) {
        // We're now ready to fetch WebRTC signalling messages
        attachWebRTCListeners(drainIteration)
      }

      let leave = false

      while (
        !leave &&
        // Each reconnect increases `this._iteration` and thereby
        // deprecates previous streams and ends them
        state._iteration == drainIteration
      ) {
        // this.flow(`FLOW: incoming: new loop iteration`)
        const promises: Promise<SourceEvent>[] = []

        // Wait for stream close attempts
        promises.push(state._closePromise.promise)

        // Wait for stream switches
        if (!state._sourceSwitched) {
          promises.push(state._sourceSwitchPromise.promise)
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
        if (state._iteration != drainIteration) {
          // leave loop
          break
        }

        let toYield: Uint8Array | undefined

        switch (result.type) {
          // Stream got destroyed, so end it
          case ConnectionEventTypes.CLOSE:
            flow(`FLOW: stream closed`)
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

            const [PREFIX, SUFFIX] = [result.value.value.subarray(0, 1), result.value.value.subarray(1)]

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
                    log(`STOP received. Ending stream ...`)
                    state.destroyed = true
                    state._destroyedPromise.resolve()
                    setClosed()
                    leave = true
                    break
                  // A reconnect at the other of the relay happened,
                  // so create a new connection endpoint (stream) and
                  // pass it to libp2p
                  // Also create a new WebRTC instance because old one
                  // cannot be used anymore since other party might have
                  // migrated to different port or IP
                  case ConnectionStatusMessages.RESTART:
                    log(`RESTART received. Ending stream ...`)

                    // First switch, then call _onReconnect to make sure
                    // values are set, even if _onReconnect throws
                    let switchedConnection = switchConnection()
                    // We must not await this promise because it resolves once
                    // TLS-alike handshake is done and thus creates a deadlock
                    // since the await blocks this stream
                    _onReconnect(switchedConnection, counterparty)

                    await migrationDone.promise
                    migrationDone = defer<void>()
                    // @TODO resolve first
                    state._sourceSwitchPromise = defer<SourceSwitchEvent>()
                    state._sourceSwitched = false

                    advanceIterator()
                    break
                  default:
                    throw Error(`Invalid suffix. Received ${u8aToHex(SUFFIX)}`)
                }
                break
              // We received a status message. Usually used to send PING / PONG
              // messages to detect if connection works
              case RelayPrefix.STATUS_MESSAGE:
                flow(`Received status message`)
                switch (SUFFIX[0]) {
                  case StatusMessages.PING:
                    verbose(`PING received`)
                    queueStatusMessage(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))
                    break
                  case StatusMessages.PONG:
                    // noop, left for future usage
                    break
                  default:
                    error(
                      `Received invalid status message ${u8aToHex(SUFFIX ?? new Uint8Array([]))}. Dropping message.`
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
                  error(`Error while trying to decode JSON-encoded WebRTC message`)
                }

                if (
                  decoded != undefined &&
                  !testingOptions.__noWebRTCUpgrade &&
                  state.channel != undefined &&
                  !state.channel.connected
                ) {
                  try {
                    state.channel.signal(decoded)
                  } catch (err) {
                    error(`WebRTC error:`, err)
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
    })(state._iteration)

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
  const attachWebRTCListeners = (drainIteration: number) => {
    // Store bound listener instance
    ;(state.channel as SimplePeer.Instance).on('signal', onSignal(drainIteration, state.channel as SimplePeer.Instance))
  }

  const onSignal = (drainIteration: number, currentChannel: SimplePeer.Instance) => (data: Object) => {
    if (state._iteration != drainIteration) {
      return
    }

    queueStatusMessage(
      Uint8Array.from([RelayPrefix.WEBRTC_SIGNALLING, ...new TextEncoder().encode(JSON.stringify(data))])
    )

    return () => {
      currentChannel.removeListener('signal', onSignal)
    }
  }

  const tags = [PeerConnectionType.RELAYED]

  let source = createSource()
  conn.sink(sinkFunction()).catch(onSinkError)

  return {
    sendUpgraded,
    direction,
    tags,
    // Set *close* property to notify libp2p that
    // stream was closed
    timeline,
    remoteAddr,
    conn,
    close: close(state._iteration),
    source,
    sink: sink(state._iteration),
    isDestroyed,
    getCurrentChannel,
    counterparty,
    setOnClose
  }
}
