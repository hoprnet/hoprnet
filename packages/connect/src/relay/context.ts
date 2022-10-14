import { u8aToHex, defer } from '@hoprnet/hopr-utils'
import type { DeferType } from '@hoprnet/hopr-utils'

import { randomBytes } from 'crypto'
import EventEmitter from 'events'

import type { Stream, StreamResult, StreamType, StreamSourceAsync, StreamSink } from '../types.js'

import Debug from 'debug'

// @ts-ignore untyped library
import retimer from 'retimer'

const DEBUG_PREFIX = `hopr-connect`

const _log = Debug(DEBUG_PREFIX)
const _verbose = Debug(`${DEBUG_PREFIX}:verbose`)
const _flow = Debug(`flow:${DEBUG_PREFIX}`)
const _error = Debug(`${DEBUG_PREFIX}:error`)

import { RelayPrefix, StatusMessages, ConnectionStatusMessages } from '../constants.js'
import { eagerIterator } from '../utils/index.js'

export const DEFAULT_PING_TIMEOUT = 300

enum ConnectionEventTypes {
  STREAM_SOURCE_SWITCH,
  ENDED,
  SINK_SOURCE_ATTACHED,
  STATUS_MESSAGE,
  PAYLOAD,
  STREAM_ENDED,
  STREAM_SINK_SWITCH,
  PING_TIMEOUT,
  PING_RESPONE
}

type StreamSourceSwitchEvent = {
  type: ConnectionEventTypes.STREAM_SOURCE_SWITCH
  value: StreamSourceAsync
}

type EndedEvent = {
  type: ConnectionEventTypes.ENDED
}

type SinkSourceAttachedEvent = {
  type: ConnectionEventTypes.SINK_SOURCE_ATTACHED
  value: StreamSourceAsync
}

type StatusMessageEvent = {
  type: ConnectionEventTypes.STATUS_MESSAGE
}

type PayloadEvent = {
  type: ConnectionEventTypes.PAYLOAD
  value: StreamResult
}

type SinkEndedEvent = {
  type: ConnectionEventTypes.STREAM_ENDED
  err?: any
}

type StreamSinkSwitchEvent = {
  type: ConnectionEventTypes.STREAM_SINK_SWITCH
  value: StreamSink
}

type PingTimeoutEvent = {
  type: ConnectionEventTypes.PING_TIMEOUT
}

type PingResponseEvent = {
  type: ConnectionEventTypes.PING_RESPONE
}

type SinkEvent = EndedEvent | SinkSourceAttachedEvent | StatusMessageEvent | PayloadEvent
type SourceEvent = StreamSourceSwitchEvent | PayloadEvent

/**
 * Encapsulates the relay-side stream state management.
 *
 * ┌────┐   stream    ┌────────┐
 * │ A  ├────────────►│        │ stream
 * └────┘             │Context ├───────►
 *                ┌──►│        │
 * ┌────┐         │   └────────┘
 * │ A' ├─────────┘
 * └────┘  new stream
 *
 * Initialized with a stream which gets overwritten
 * once the node reconnects.
 *
 * Upon reconnects, the stream handler issues a status
 * message `RESTART` to notify the other end such that it
 * can reinitialize the connection, i.e. it will redo the
 * TLS-alike handshake
 */
class RelayContext extends EventEmitter {
  private _streamSourceSwitchPromise: DeferType<StreamSourceSwitchEvent>
  private _streamSinkSwitchPromise: DeferType<StreamSinkSwitchEvent>

  private _id: string

  private _sinkSourceAttachedPromise: DeferType<SinkSourceAttachedEvent>

  private _statusMessagePromise: DeferType<StatusMessageEvent>
  private _statusMessages: Uint8Array[] = []
  private _pingResponsePromise?: DeferType<PingResponseEvent>
  private _stream: Stream

  private sinkCreator: Promise<void>

  public source: Stream['source']
  public sink: Stream['sink']

  constructor(stream: Stream, private relayFreeTimeout: number = 0) {
    super()

    // Assign a unique id to distinguish instances
    this._id = u8aToHex(randomBytes(4), false)

    // Internal status message buffer, promise resolves once
    // there is a new status message
    this._statusMessagePromise = defer<StatusMessageEvent>()
    this._statusMessages = []

    this._stream = stream

    this._sinkSourceAttachedPromise = defer<SinkSourceAttachedEvent>()

    // Resolves once there is a new stream
    this._streamSourceSwitchPromise = defer<StreamSourceSwitchEvent>()
    this._streamSinkSwitchPromise = defer<StreamSinkSwitchEvent>()

    this.source = this.createSource()

    // Initializes the sink and stores the handle to assign
    // an error handler
    this.sinkCreator = this.createSink()

    // Make sure that we catch all errors, even before a sink source has been attached
    this.sinkCreator.catch((err) => this.error(`Sink has thrown error before attaching source`, err.message))

    // Passed as a function handle so we need to bind it explicitly
    this.sink = this._sink.bind(this)
  }

  /**
   * Sends a low-level ping to the client.
   * Used to test if connection is active.
   * @param ms timeout in miliseconds
   * @returns a Promise that resolves to measured latency
   */
  public async ping(ms = DEFAULT_PING_TIMEOUT): Promise<number> {
    let start = Date.now()
    this._pingResponsePromise = defer<PingResponseEvent>()
    let timeoutDone = false

    this.queueStatusMessage(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING))

    let timer: any
    const pingTimeoutPromise = new Promise<PingTimeoutEvent>((resolve) => {
      timer = retimer(() => {
        if (timeoutDone) {
          return
        }
        timeoutDone = true
        this.log(`ping timeout done`)

        resolve({
          type: ConnectionEventTypes.PING_TIMEOUT
        })
      }, ms)
    })

    const result = await Promise.race([pingTimeoutPromise, this._pingResponsePromise.promise])
    timer.clear()

    switch (result.type) {
      case ConnectionEventTypes.PING_RESPONE:
        timeoutDone = true

        return Date.now() - start
      case ConnectionEventTypes.PING_TIMEOUT:
        // Make sure we don't produce any hanging promises
        this._pingResponsePromise.resolve(undefined as any)
        this._pingResponsePromise = undefined

        return -1
    }
  }

  /**
   * Attaches a new stream to an existing context
   * and thereby overwrites the previous stream
   * @param newStream the new stream to use
   */
  public update(newStream: Stream) {
    this._streamSourceSwitchPromise.resolve({
      type: ConnectionEventTypes.STREAM_SOURCE_SWITCH,
      value: newStream.source as StreamSourceAsync
    })
    this._streamSinkSwitchPromise.resolve({
      type: ConnectionEventTypes.STREAM_SINK_SWITCH,
      value: newStream.sink
    })

    this.log(`updating`)
  }

  /**
   * Log messages and add identity tag to distinguish multiple instances
   */
  private log(..._: any[]) {
    _log(`RX [${this._id}]`, ...arguments)
  }

  /**
   * Log verbose messages and add identity tag to distinguish multiple instances
   */
  private verbose(..._: any[]) {
    _verbose(`RX [${this._id}]`, ...arguments)
  }

  /**
   * Log errors and add identity tag to distinguish multiple instances
   */
  private error(..._: any[]) {
    _error(`RX [${this._id}]`, ...arguments)
  }

  /**
   * Log control flow and add identity tag to distinguish multiple instances
   */
  private flow(..._: any[]) {
    _flow(`RX [${this._id}]`, ...arguments)
  }

  /**
   * Called with a stream of messages to be sent. This resolves
   * the sinkSourcePromise such that the sinkFunction can fetch
   * messages and forward them.
   *
   * @param source stream of messages to be sent
   * @returns a Promise that resovles once the source stream ends
   */
  public async _sink(source: Stream['source']): Promise<void> {
    let deferred = defer<void>()
    // forward sink stream errors
    this.sinkCreator.catch(deferred.reject)
    this._sinkSourceAttachedPromise.resolve({
      type: ConnectionEventTypes.SINK_SOURCE_ATTACHED,
      value: async function* (this: RelayContext) {
        try {
          yield* source
          deferred.resolve()
        } catch (err) {
          // Close stream
          this.queueStatusMessage(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))
          deferred.reject(err)
        }
      }.call(this)
    })

    return deferred.promise
  }

  /**
   * Creates an AsyncIterable that resolves to the messages received
   * from the current source.
   *
   * @returns an async iterator
   */
  private createSource(): Stream['source'] {
    let sourceIterator: AsyncIterator<Uint8Array> | undefined
    let nextMessagePromise: Promise<PayloadEvent> | undefined

    // Anything can happen ...
    try {
      sourceIterator = (this._stream.source as AsyncIterable<StreamType>)[Symbol.asyncIterator]()
    } catch (err) {
      this.error(`Error while starting source iterator`, err)
    }

    // Advances the source iterator
    const advanceIterator = () => {
      if (sourceIterator == null) {
        throw Error(`Source not yet set`)
      }
      nextMessagePromise = sourceIterator.next().then((res) => ({
        type: ConnectionEventTypes.PAYLOAD,
        value: res
      }))
    }

    const iterator: Stream['source'] = async function* (this: RelayContext) {
      this.log(`source called`)

      let leave = false
      this.flow(`FLOW: relay_incoming: started loop`)
      while (!leave) {
        this.flow(`FLOW: relay_incoming: new loop iteration`)

        const promises: Promise<SourceEvent>[] = []

        // Wait for stream switches
        this.flow(`FLOW: relay_incoming: waiting for streamSourceSwitch`)
        promises.push(this._streamSourceSwitchPromise.promise)

        // Wait for payload messages
        if (sourceIterator != undefined) {
          if (nextMessagePromise == null) {
            advanceIterator()
          }

          this.flow(`FLOW: relay_incoming: waiting for payload`)
          promises.push(nextMessagePromise as Promise<PayloadEvent>)
        }

        const result = await Promise.race(promises)

        let toYield: Uint8Array | undefined

        switch (result.type) {
          // Reconnect happened, attach new source and notify counterparty
          case ConnectionEventTypes.STREAM_SOURCE_SWITCH:
            sourceIterator = result.value[Symbol.asyncIterator]()

            this._streamSourceSwitchPromise = defer<StreamSourceSwitchEvent>()
            advanceIterator()

            this.flow(`FLOW: relay_incoming: source switched continue`)
            toYield = Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART)
            break
          // Forward payload data
          case ConnectionEventTypes.PAYLOAD:
            // Stream ended, but there might eventually be a new stream
            // after a reconnect
            if (result.value.done) {
              this.flow(`FLOW: relay_incoming: received done, continue`)
              sourceIterator = undefined
              break
            }

            // Anything can happen
            if (result.value.value.length == 0) {
              this.log(`got empty message`)
              advanceIterator()

              this.flow(`FLOW: relay_incoming: empty message, continue`)
              // Ignore empty messages
              break
            }

            const [PREFIX, SUFFIX] = [result.value.value.subarray(0, 1), result.value.value.subarray(1)]

            switch (PREFIX[0]) {
              case RelayPrefix.STATUS_MESSAGE:
                this.flow(`FLOW: relay_incoming: got PING or PONG, continue`)
                switch (SUFFIX[0]) {
                  case StatusMessages.PING:
                    this.verbose(`PING received`)
                    this.queueStatusMessage(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))
                    // Don't forward ping
                    break
                  case StatusMessages.PONG:
                    this.verbose(`PONG received`)

                    this._pingResponsePromise?.resolve({
                      type: ConnectionEventTypes.PING_RESPONE
                    })
                    // Don't forward pong message
                    break
                  default:
                    throw Error(`Invalid status message. Received ${u8aToHex(SUFFIX)}`)
                }
                advanceIterator()
                break
              case RelayPrefix.CONNECTION_STATUS:
                switch (SUFFIX[0]) {
                  // STOP = we're done, no further reconnect attempts
                  case ConnectionStatusMessages.STOP:
                    this.verbose(`STOP relayed`)

                    this.emit('close')

                    this.flow(`FLOW: relay_incoming: STOP relayed, break`)
                    // forward STOP message
                    toYield = result.value.value
                    // close stream
                    leave = true
                    break
                  // Reconnect at the other end of the relay
                  case ConnectionStatusMessages.RESTART:
                    this.verbose(`RESTART relayed`)
                    this.flow(`FLOW: relay_incoming: RESTART relayed, break`)

                    // Unclear, probably wrong
                    toYield = result.value.value

                    break
                  // WebRTC upgrade has happened
                  case ConnectionStatusMessages.UPGRADED:
                    // this is an artificial timeout to test the relay slot being properly freed during the integration test
                    this.flow(`FLOW: waiting ${this.relayFreeTimeout}ms before freeing relay`)
                    // @TODO remove this
                    if (this.relayFreeTimeout > 0) {
                      await new Promise((resolve) => setTimeout(resolve, this.relayFreeTimeout))
                    }
                    this.flow(`FLOW: freeing relay`)

                    this.emit('upgrade')
                    advanceIterator()
                    break
                  default:
                    throw Error(`Invalid connection status prefix. Received ${u8aToHex(SUFFIX.subarray(0, 1))}`)
                }
                break
              // Forward any WebRTC signalling
              case RelayPrefix.WEBRTC_SIGNALLING:
              // Forward any payload message
              case RelayPrefix.PAYLOAD:
                toYield = result.value.value
                advanceIterator()
                break
              default:
                throw Error(`Invalid prefix. Received ${u8aToHex(PREFIX)}`)
            }
            break
          default:
            throw Error(`Invalid result ${JSON.stringify(result)}`)
        }

        if (toYield != undefined) {
          yield toYield
        }

        this.flow(`FLOW: relay_incoming: loop iteration end`)
      }

      this.flow(`FLOW: relay_incoming: loop ended`)
    }.call(this)

    return eagerIterator(iterator)
  }

  /**
   * Starts the sink. Whenever there is a STATUS message,
   * sink it to the currently attached sink, even if no stream
   * is attached yet.
   */
  private async createSink(): Promise<void> {
    this.log(`createSink called`)
    let currentSink = this._stream.sink

    let nextMessagePromise: Promise<PayloadEvent> | undefined

    let currentSource: AsyncIterator<StreamType> | undefined

    let iteration = 0

    // On every reconnect, create a new asyncIterable to pass into
    async function* drain(
      this: RelayContext,
      internalIteration: number,
      endPromise: DeferType<SinkEndedEvent | EndedEvent>
    ) {
      const advanceIterator = () => {
        if (currentSource == null) {
          throw Error(`Source is not yet set`)
        }
        nextMessagePromise = currentSource.next().then((res) => ({
          type: ConnectionEventTypes.PAYLOAD,
          value: res
        }))
      }

      // Notify *why* stream has ended
      let reasonToLeave: ConnectionEventTypes.STREAM_ENDED | ConnectionEventTypes.ENDED =
        ConnectionEventTypes.STREAM_ENDED
      let leave = false
      this.flow(`FLOW: relay_outgoing: loop started`)

      while (!leave) {
        this.flow(`FLOW: relay_outgoing: new loop iteration`)

        const promises: Promise<SinkEvent>[] = []

        if (currentSource == undefined) {
          promises.push(this._sinkSourceAttachedPromise.promise)
        }

        promises.push(this._statusMessagePromise.promise)

        if (currentSource != undefined) {
          nextMessagePromise =
            nextMessagePromise ??
            currentSource.next().then((res) => ({
              type: ConnectionEventTypes.PAYLOAD,
              value: res
            }))

          promises.push(nextMessagePromise)
        }

        this.flow(`FLOW: relay_outgoing: awaiting promises`)
        const result = await Promise.race(promises)

        if (iteration != internalIteration) {
          break
        }

        let toYield: Uint8Array | undefined

        switch (result.type) {
          // There is a source, so let's drain it
          case ConnectionEventTypes.SINK_SOURCE_ATTACHED:
            currentSource = result.value[Symbol.asyncIterator]()
            advanceIterator()
            this.flow(`FLOW: relay_outgoing: sinkSource attacked, continue`)
            break
          // There is a status message, i.e. PING / PONG
          // ready to be sent.
          case ConnectionEventTypes.STATUS_MESSAGE:
            if (this._statusMessages.length > 0) {
              toYield = this.unqueueStatusMessage()
              this.flow(`FLOW: relay_outgoing: unqueuedStatusMsg, continue`)
            }
            break
          // There is a payload message to be forwarded
          case ConnectionEventTypes.PAYLOAD:
            // Source stream ended, wait for next reconnect
            if (result.value.done) {
              this.flow(`FLOW: relay_outgoing: received done, continue`)
              leave = true
              currentSource = undefined
              break
            }

            // Anything can happen
            if (result.value.value.length == 0) {
              this.flow(`Ignoring empty message`)
              advanceIterator()
              this.flow(`FLOW: relay_outgoing: empty msg, continue`)
              break
            }

            let [PREFIX, SUFFIX] = [result.value.value.subarray(0, 1), result.value.value.subarray(1)]

            // Anything can happen
            if (SUFFIX.length == 0) {
              this.flow(`Ignoring empty payload`)
              advanceIterator()
              this.flow(`FLOW: relay_outgoing: empty payload, continue`)
              break
            }

            // STOP received == we no longer need the relay connection
            if (PREFIX[0] == RelayPrefix.CONNECTION_STATUS && SUFFIX[0] == ConnectionStatusMessages.STOP) {
              this.flow(`FLOW: relay_outgoing: STOP, break`)
              toYield = result.value.value
              leave = true
              reasonToLeave = ConnectionEventTypes.ENDED
              break
            }

            toYield = result.value.value
            advanceIterator()

            break
          default:
            throw Error(`Invalid result. Received ${JSON.stringify(result)}`)
        }

        if (toYield != undefined) {
          yield toYield
        }

        this.flow(`FLOW: relay_outgoing: end of loop iteration`)
      }

      endPromise.resolve({
        type: reasonToLeave
      })
      this.flow(`FLOW: relay_outgoing: loop ended`, internalIteration)
    }

    // Set to true once STOP signal received
    let leaveSinkLoop = false

    // Attach a new sink on every reconnect
    while (!leaveSinkLoop) {
      const endPromise = defer<SinkEndedEvent | EndedEvent>()

      try {
        await currentSink(drain.call(this, iteration, endPromise))
      } catch (err) {
        endPromise.resolve({
          type: ConnectionEventTypes.STREAM_ENDED,
          err
        })
      }

      const result = await Promise.race([endPromise.promise, this._streamSinkSwitchPromise.promise])

      switch (result.type) {
        case ConnectionEventTypes.STREAM_ENDED:
          // Wait for next sink
          if (result.err) {
            // be bit more verbose to enhance debugging
            this.error(`Sink threw error`, result.err)
            currentSink = (await this._streamSinkSwitchPromise.promise).value
            iteration++
          } else {
            currentSink = (await this._streamSinkSwitchPromise.promise).value
            iteration++
          }
          // sink call might return earlier, so wait for new stream
          break
        case ConnectionEventTypes.ENDED:
          // Beta, needs more testing
          leaveSinkLoop = true
          break
        case ConnectionEventTypes.STREAM_SINK_SWITCH:
          iteration++
          currentSink = result.value

          break
        default:
          throw Error(`Invalid return type. Received ${result}`)
      }

      this._streamSinkSwitchPromise = defer<StreamSinkSwitchEvent>()
    }
  }

  /**
   * Add status and control messages to queue
   * @param msg msg to add
   */
  private queueStatusMessage(msg: Uint8Array) {
    this._statusMessages.push(msg)

    this._statusMessagePromise.resolve({
      type: ConnectionEventTypes.STATUS_MESSAGE
    })
  }

  /**
   * Removes latest message from queue and returns it.
   * Resets the waiting mutex if queue is empty.
   * @returns latest status or control message
   */
  private unqueueStatusMessage(): Uint8Array {
    switch (this._statusMessages.length) {
      case 0:
        throw Error(`Trying to unqueue empty status message queue`)
      case 1:
        this._statusMessagePromise = defer<StatusMessageEvent>()
        return this._statusMessages.pop() as Uint8Array
      default:
        return this._statusMessages.shift() as Uint8Array
    }
  }
}

export { RelayContext }
