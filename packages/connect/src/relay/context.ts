import { u8aToHex, defer } from '@hoprnet/hopr-utils'
import type { DeferType } from '@hoprnet/hopr-utils'

import { randomBytes } from 'crypto'
import EventEmitter from 'events'

import type { Stream, StreamResult, StreamType, StreamSourceAsync, StreamSink } from '../types.js'

import Debug from 'debug'

const DEBUG_PREFIX = `hopr-connect`

const _log = Debug(DEBUG_PREFIX)
const _verbose = Debug(`${DEBUG_PREFIX}:verbose`)
const _flow = Debug(`flow:${DEBUG_PREFIX}`)
const _error = Debug(`${DEBUG_PREFIX}:error`)

import { RelayPrefix, StatusMessages, ConnectionStatusMessages, isValidPrefix } from '../constants.js'
import { eagerIterator } from '../utils/index.js'
import assert from 'assert'

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
 * Encapsulate the relay-side state management of a relayed connecion
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
    this._id = u8aToHex(randomBytes(4), false)

    this._statusMessagePromise = defer<StatusMessageEvent>()
    this._statusMessages = []
    this._stream = stream

    this._sinkSourceAttachedPromise = defer<SinkSourceAttachedEvent>()

    this._streamSourceSwitchPromise = defer<StreamSourceSwitchEvent>()
    this._streamSinkSwitchPromise = defer<StreamSinkSwitchEvent>()

    this.source = this.createSource()

    // Auto-start sink stream and declare variable in advance
    // to make sure we can attach an error handler to it

    this.sinkCreator = this.createSink()
      // Make sure that we catch all errors, even before a sink source has been attached
      .catch((err) => this.error(`Sink has thrown error before attaching source`, err.message))

    // Passed as a function handle so we need to bind it explicitly
    this.sink = this._sink.bind(this)
  }

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

    const pingTimeoutPromise = new Promise<PingTimeoutEvent>((resolve) =>
      setTimeout(() => {
        if (timeoutDone) {
          return
        }
        timeoutDone = true
        this.log(`ping timeout done`)

        resolve({
          type: ConnectionEventTypes.PING_TIMEOUT
        })
      }, ms)
    )
    const result = await Promise.race([pingTimeoutPromise, this._pingResponsePromise.promise])

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
   * @param newStream the new stream to use
   */
  public update(newStream: Stream) {
    console.log(`############## updating`)
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

  private flow(..._: any[]) {
    _flow(`RX [${this._id}]`, ...arguments)
  }

  /**
   * Forwards incoming messages from current incoming stream
   * @returns an async iterator
   */
  private createSource(): Stream['source'] {
    let sourceIterator: AsyncIterator<Uint8Array> | undefined
    let sourcePromise: Promise<PayloadEvent> | undefined

    try {
      sourceIterator = (this._stream.source as AsyncIterable<StreamType>)[Symbol.asyncIterator]()
    } catch (err) {
      this.error(`Error while starting source iterator`, err)
    }

    const nextMessage = () => {
      if (sourceIterator == null) {
        throw Error(`Source not yet set`)
      }
      sourcePromise = sourceIterator.next().then((res) => ({
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
          if (sourcePromise == null) {
            nextMessage()
          }

          this.flow(`FLOW: relay_incoming: waiting for payload`)
          promises.push(sourcePromise as Promise<PayloadEvent>)
        }

        // 1. Handle Stream switches
        // 2. Handle payload / status messages
        const result = await Promise.race(promises)
        console.log(`incoming`, result)

        let toYield: Uint8Array | undefined

        switch (result.type) {
          case ConnectionEventTypes.STREAM_SOURCE_SWITCH:
            sourceIterator = result.value[Symbol.asyncIterator]()

            this._streamSourceSwitchPromise = defer<StreamSourceSwitchEvent>()
            nextMessage()

            this.flow(`FLOW: relay_incoming: source switched continue`)
            toYield = Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART)
            break
          case ConnectionEventTypes.PAYLOAD:
            if (result.value.done) {
              this.flow(`FLOW: relay_incoming: received done, continue`)
              sourceIterator = undefined
              break
            }

            // Anything can happen
            if (result.value.value.length == 0) {
              this.log(`got empty message`)
              nextMessage()

              this.flow(`FLOW: relay_incoming: empty message, continue`)
              // Ignore empty messages
              break
            }

            const [PREFIX, SUFFIX] = [result.value.value.slice(0, 1), result.value.value.slice(1)]

            if (!isValidPrefix(PREFIX[0])) {
              this.error(
                `Invalid prefix: Got <${u8aToHex(PREFIX ?? new Uint8Array())}>. Dropping message in relayContext.`
              )

              nextMessage()

              // Ignore invalid prefixes
              this.flow(`FLOW: relay_incoming: invalid prefix, continue`)
              break
            }

            switch (PREFIX[0]) {
              case RelayPrefix.STATUS_MESSAGE:
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
                nextMessage()
                break
              case RelayPrefix.CONNECTION_STATUS:
                switch (SUFFIX[0]) {
                  case ConnectionStatusMessages.STOP:
                    this.verbose(`STOP relayed`)

                    this.emit('close')

                    this.flow(`FLOW: relay_incoming: STOP relayed, break`)
                    // forward STOP message
                    toYield = result.value.value
                    // close stream
                    leave = true
                    break
                  case ConnectionStatusMessages.RESTART:
                    this.verbose(`RESTART relayed`)
                    this.flow(`FLOW: relay_incoming: RESTART relayed, break`)

                    // Unclear
                    toYield = result.value.value

                    break
                  case ConnectionStatusMessages.UPGRADED:
                    // this is an artificial timeout to test the relay slot being properly freed during the integration test
                    this.flow(`FLOW: waiting ${this.relayFreeTimeout}ms before freeing relay`)
                    // @TODO remove this
                    if (this.relayFreeTimeout > 0) {
                      await new Promise((resolve) => setTimeout(resolve, this.relayFreeTimeout))
                    }
                    this.flow(`FLOW: freeing relay`)

                    this.emit('upgrade')
                    nextMessage()
                    break
                  default:
                    throw Error(`Invalid connection status prefix. Received ${u8aToHex(SUFFIX.slice(0, 1))}`)
                }
                break
              case RelayPrefix.STATUS_MESSAGE:
                this.flow(`FLOW: relay_incoming: got PING or PONG, continue`)

                // Unclear
                toYield = result.value.value
                nextMessage()
                break
              case RelayPrefix.PAYLOAD:
                toYield = result.value.value
                nextMessage()
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
   * Passes messages from source into current outgoing stream
   */
  private async createSink(): Promise<void> {
    this.log(`createSink called`)
    let currentSink = this._stream.sink

    let sourcePromise: Promise<PayloadEvent> | undefined

    let currentSource: AsyncIterator<StreamType> | undefined

    let iteration = 0

    async function* drain(this: RelayContext, internalIteration: number, endPromise: DeferType<SinkEndedEvent>) {
      console.log(`drain called`, internalIteration)
      const nextMessage = () => {
        assert(currentSource != undefined)
        sourcePromise = currentSource.next().then((res) => ({
          type: ConnectionEventTypes.PAYLOAD,
          value: res
        }))
      }

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
          sourcePromise =
            sourcePromise ??
            currentSource.next().then((res) => ({
              type: ConnectionEventTypes.PAYLOAD,
              value: res
            }))

          promises.push(sourcePromise)
        }

        this.flow(`FLOW: relay_outgoing: awaiting promises`)
        const result = await Promise.race(promises)

        console.log(`iteration`, iteration, `internalIterarion`, internalIteration)
        if (iteration != internalIteration) {
          leave = true
          break
        }

        console.log(`outgoing`, result, internalIteration)

        let toYield: Uint8Array | undefined

        switch (result.type) {
          case ConnectionEventTypes.SINK_SOURCE_ATTACHED:
            currentSource = result.value[Symbol.asyncIterator]()
            nextMessage()
            this.flow(`FLOW: relay_outgoing: sinkSource attacked, continue`)
            break
          case ConnectionEventTypes.STATUS_MESSAGE:
            if (this._statusMessages.length > 0) {
              toYield = this.unqueueStatusMessage()
              this.flow(`FLOW: relay_outgoing: unqueuedStatusMsg, continue`)
            }
            break
          case ConnectionEventTypes.PAYLOAD:
            if (result.value.done) {
              this.flow(`FLOW: relay_outgoing: received done, continue`)
              leave = true
              currentSource = undefined
              break
            }

            if (result.value.value.length == 0) {
              this.flow(`Ignoring empty message`)
              nextMessage()
              this.flow(`FLOW: relay_outgoing: empty msg, continue`)
              break
            }

            let [PREFIX, SUFFIX] = [result.value.value.slice(0, 1), result.value.value.slice(1)]

            if (SUFFIX.length == 0) {
              this.flow(`Ignoring empty payload`)
              nextMessage()
              this.flow(`FLOW: relay_outgoing: empty payload, continue`)
              break
            }

            if (PREFIX[0] == RelayPrefix.CONNECTION_STATUS && SUFFIX[0] == ConnectionStatusMessages.STOP) {
              this.flow(`FLOW: relay_outgoing: STOP, break`)
              toYield = result.value.value
              leave = true
              break
            }

            toYield = result.value.value
            nextMessage()

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
        type: ConnectionEventTypes.STREAM_ENDED
      })
      this.flow(`FLOW: relay_outgoing: loop ended`, internalIteration)
    }

    while (true) {
      console.log(`sink iteration`)
      const endPromise = defer<SinkEndedEvent>()
      try {
        await currentSink(drain.call(this, iteration, endPromise))
      } catch (err) {
        endPromise.resolve({
          type: ConnectionEventTypes.STREAM_ENDED,
          err
        })
      }

      const result = await Promise.race([endPromise.promise, this._streamSinkSwitchPromise.promise])

      console.log(`sink result`, result)

      switch (result.type) {
        case ConnectionEventTypes.STREAM_ENDED:
          // Wait for next sink
          if (result.err) {
            // be bit more verbose to enhance debugging
            this.error(`Sink threw error`, result.err)
            currentSink = (await this._streamSinkSwitchPromise.promise).value
            iteration++
            console.log(`after error`, currentSink)
          } else {
            currentSink = (await this._streamSinkSwitchPromise.promise).value
            iteration++
          }
          // sink call might return earlier, so wait for new stream
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
