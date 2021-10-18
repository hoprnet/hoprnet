import { u8aToHex, defer } from '@hoprnet/hopr-utils'
import type { DeferType } from '@hoprnet/hopr-utils'

import { randomBytes } from 'crypto'

import EventEmitter from 'events'

import type { Stream, StreamResult } from '../types'

import Debug from 'debug'

const DEBUG_PREFIX = `hopr-connect`

const _log = Debug(DEBUG_PREFIX)
const _verbose = Debug(`${DEBUG_PREFIX}:verbose`)
const _flow = Debug(`flow:${DEBUG_PREFIX}`)
const _error = Debug(`${DEBUG_PREFIX}:error`)

import { RelayPrefix, StatusMessages, ConnectionStatusMessages, isValidPrefix } from '../constants'
import { eagerIterator } from '../utils'

export const DEFAULT_PING_TIMEOUT = 300

/**
 * Encapsulate the relay-side state management of a relayed connecion
 */
class RelayContext extends EventEmitter {
  private _streamSourceSwitchPromise: DeferType<Stream['source']>
  private _streamSinkSwitchPromise: DeferType<Stream['sink']>

  private _id: string

  private _sinkSourceAttached: boolean
  private _sinkSourceAttachedPromise: DeferType<Stream['source']>

  private _statusMessagePromise: DeferType<void>
  private _statusMessages: Uint8Array[] = []
  private _pingResponsePromise?: DeferType<void>
  private _stream: Stream

  private _sourcePromise: Promise<StreamResult> | undefined
  private _sourceSwitched: boolean

  public source: Stream['source']
  public sink: Stream['sink']

  constructor(stream: Stream, private relayFreeTimeout: number = 0) {
    super()
    this._id = u8aToHex(randomBytes(4), false)

    this._statusMessagePromise = defer<void>()
    this._statusMessages = []
    this._stream = stream

    this._sourceSwitched = false

    this._sinkSourceAttached = false
    this._sinkSourceAttachedPromise = defer<Stream['source']>()

    this._streamSourceSwitchPromise = defer<Stream['source']>()
    this._streamSinkSwitchPromise = defer<Stream['sink']>()

    this.source = this.createSource()

    // Auto-start sink stream and declare variable in advance
    // to make sure we can attach an error handler to it
    let sinkCreator: Promise<void>
    this.sink = (source: Stream['source']): Promise<void> => {
      let deferred = defer<void>()
      // forward sink stream errors
      sinkCreator.catch(deferred.reject)
      this._sinkSourceAttached = true
      this._sinkSourceAttachedPromise.resolve(
        async function* (this: RelayContext) {
          try {
            yield* source
            deferred.resolve()
          } catch (err) {
            // Close stream
            this.queueStatusMessage(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))
            deferred.reject(err)
          }
        }.call(this)
      )

      return deferred.promise
    }

    sinkCreator = this.createSink()

    // Make sure that we catch all errors, even before a sink source has been attached
    sinkCreator.catch((err) => this.error(`Sink has thrown error before attaching source`, err.message))
  }

  /**
   * Sends a low-level ping to the client.
   * Used to test if connection is active.
   * @param ms timeout in miliseconds
   * @returns a Promise that resolves to measured latency
   */
  public async ping(ms = DEFAULT_PING_TIMEOUT): Promise<number> {
    let start = Date.now()
    this._pingResponsePromise = defer<void>()

    let timeoutDone = false

    const timeoutPromise = defer<void>()
    const timeout = setTimeout(() => {
      this.log(`ping timeout done`)
      timeoutDone = true
      // Make sure that we don't produce any hanging promises
      this._pingResponsePromise?.resolve()
      this._pingResponsePromise = undefined
      timeoutPromise.resolve()
    }, ms)

    this.queueStatusMessage(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING))

    await Promise.race([
      // prettier-ignore
      timeoutPromise.promise,
      this._pingResponsePromise.promise
    ])

    if (timeoutDone) {
      return -1
    }

    // Make sure that we don't produce any hanging promises
    timeoutPromise.resolve()
    clearTimeout(timeout)
    return Date.now() - start
  }

  /**
   * Attaches a new stream to an existing context
   * @param newStream the new stream to use
   */
  public update(newStream: Stream) {
    this._sourceSwitched = true

    this._streamSourceSwitchPromise.resolve(newStream.source)
    this._streamSinkSwitchPromise.resolve(newStream.sink)

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
    const iterator: Stream['source'] = async function* (this: RelayContext) {
      this.log(`source called`)
      let result: Stream['source'] | StreamResult | undefined

      const next = () => {
        result = undefined
        this._sourcePromise = this._stream.source.next()
      }

      this.flow(`FLOW: relay_incoming: started loop`)
      while (true) {
        this.flow(`FLOW: relay_incoming: new loop iteration`)

        const promises: Promise<Stream['source'] | StreamResult>[] = []
        let resolvedPromiseName

        const pushPromise = (promise: Promise<any>, name: string) => {
          promises.push(
            promise.then((res) => {
              resolvedPromiseName = name
              return res
            })
          )
        }

        // Wait for stream switches
        pushPromise(this._streamSourceSwitchPromise.promise, 'streamSwitch')

        // Wait for payload messages
        if (result == undefined || (result as StreamResult).done != true) {
          this._sourcePromise = this._sourcePromise ?? this._stream.source.next()

          pushPromise(this._sourcePromise, 'sourcePromise')
        }

        this.flow(`FLOW: relay_incoming: awaiting promises`)
        // 1. Handle Stream switches
        // 2. Handle payload / status messages
        result = await Promise.race(promises)
        this.flow(`FLOW: relay_incoming: promise ${resolvedPromiseName} resolved`)

        if (result == undefined) {
          // @TODO throw Error to make debugging easier
          throw Error(`source result == undefined. Should not happen.`)
        }

        // If source switched, get messages from new source
        if (this._sourceSwitched) {
          this._sourceSwitched = false
          this._stream.source = result as Stream['source']

          this._streamSourceSwitchPromise = defer<Stream['source']>()

          next()

          this.flow(`FLOW: relay_incoming: source switched continue`)
          yield Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART)
          continue
        }

        const received = result as IteratorYieldResult<Uint8Array>

        if (received.done) {
          this.flow(`FLOW: relay_incoming: received done, continue`)
          continue
        }

        // Anything can happen
        if (received.value.length == 0) {
          this.log(`got empty message`)
          next()

          this.flow(`FLOW: relay_incoming: empty message, continue`)
          // Ignore empty messages
          continue
        }

        const [PREFIX, SUFFIX] = [received.value.slice(0, 1), received.value.slice(1)]

        if (!isValidPrefix(PREFIX[0])) {
          this.error(`Invalid prefix: Got <${u8aToHex(PREFIX ?? new Uint8Array())}>. Dropping message in relayContext.`)

          next()

          // Ignore invalid prefixes
          this.flow(`FLOW: relay_incoming: invalid prefix, continue`)
          continue
        }

        // Interprete relay sub-protocol
        if (PREFIX[0] == RelayPrefix.STATUS_MESSAGE) {
          if (SUFFIX[0] == StatusMessages.PING) {
            this.verbose(`PING received`)
            this.queueStatusMessage(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))
            // Don't forward ping
          } else if (SUFFIX[0] == StatusMessages.PONG) {
            this.verbose(`PONG received`)

            this._pingResponsePromise?.resolve()
            // Don't forward pong message
          }

          next()

          this.flow(`FLOW: relay_incoming: got PING or PONG, continue`)
          continue
          // Interprete connection sub-protocol
        } else if (PREFIX[0] == RelayPrefix.CONNECTION_STATUS) {
          if (SUFFIX[0] == ConnectionStatusMessages.STOP) {
            this.verbose(`STOP relayed`)

            this.emit('close')

            this.flow(`FLOW: relay_incoming: STOP relayed, break`)
            // forward STOP message
            yield received.value

            // close stream
            break
          } else if (SUFFIX[0] == ConnectionStatusMessages.UPGRADED) {
            // this is an artificial timeout to test the relay slot being properly freed during the integration test
            this.flow(`FLOW: waiting ${this.relayFreeTimeout}ms before freeing relay`)
            if (this.relayFreeTimeout > 0) {
              await new Promise((resolve) => setTimeout(resolve, this.relayFreeTimeout))
            }
            this.flow(`FLOW: freeing relay`)

            this.emit('upgrade')
            next()
            continue
          } else if ((SUFFIX[0] = ConnectionStatusMessages.RESTART)) {
            this.verbose(`RESTART relayed`)
            this.flow(`FLOW: relay_incoming: RESTART relayed, break`)
          }
        }

        this.flow(`FLOW: relay_incoming: loop iteration end`)

        yield received.value

        next()
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

    let sourcePromise: Promise<StreamResult> | undefined

    let currentSource: Stream['source'] | undefined

    async function* drain(this: RelayContext, end: DeferType<void>): Stream['source'] {
      type SinkResult = Stream['source'] | StreamResult | void

      let result: SinkResult

      const next = () => {
        result = undefined

        sourcePromise = (currentSource as Stream['source']).next()
      }

      let ended = false
      const endPromise = end.promise.then(() => {
        ended = true
      })

      this.flow(`FLOW: relay_outgoing: loop started`)
      while (true) {
        this.flow(`FLOW: relay_outgoing: new loop iteration`)

        const promises: Promise<SinkResult>[] = []

        let resolvedPromiseName

        const pushPromise = (promise: Promise<any>, name: string) => {
          promises.push(
            promise.then((res) => {
              resolvedPromiseName = name
              return res
            })
          )
        }

        pushPromise(endPromise, 'ended')

        if (currentSource == undefined) {
          pushPromise(this._sinkSourceAttachedPromise.promise, 'sinkSourceAttacked')
        }

        pushPromise(this._statusMessagePromise.promise, 'statusMessage')

        if (currentSource != undefined && (result == undefined || (result as StreamResult).done != true)) {
          sourcePromise = sourcePromise ?? currentSource.next()

          pushPromise(sourcePromise, 'payload')
        }

        // (0. Handle source attach)
        // 1. Handle stream switches
        // 2. Handle status messages
        // 3. Handle payload messages
        this.flow(`FLOW: relay_outgoing: awaiting promises`)
        result = await Promise.race(promises)

        this.flow(`FLOW: relay_outgoing: promise ${resolvedPromiseName} resolved`)

        // Don't handle incoming messages after migration
        if (ended) {
          this.flow(`FLOW: relay_outgoing: ended, break`)
          break
        }

        if (this._sinkSourceAttached) {
          this._sinkSourceAttached = false
          currentSource = result as Stream['source']

          result = undefined
          this.flow(`FLOW: relay_outgoing: sinkSource attacked, continue`)
          continue
        }

        if (this._statusMessages.length > 0) {
          yield this.unqueueStatusMessage()
          this.flow(`FLOW: relay_outgoing: unqueuedStatusMsg, continue`)
          continue
        }

        let received = result as StreamResult

        if (received.done) {
          this.flow(`FLOW: relay_outgoing: received done, continue`)
          continue
        }

        if (received.value.length == 0) {
          this.flow(`Ignoring empty message`)
          next()
          this.flow(`FLOW: relay_outgoing: empty msg, continue`)
          continue
        }

        let [PREFIX, SUFFIX] = [received.value.slice(0, 1), received.value.slice(1)]

        if (SUFFIX.length == 0) {
          this.flow(`Ignoring empty payload`)
          next()
          this.flow(`FLOW: relay_outgoing: empty payload, continue`)
          continue
        }

        if (PREFIX[0] == RelayPrefix.CONNECTION_STATUS && SUFFIX[0] == ConnectionStatusMessages.STOP) {
          this.flow(`FLOW: relay_outgoing: STOP, break`)
          yield received.value
          break
        }

        next()
        this.flow(`FLOW: relay_outgoing: end of loop iteration`)
        yield received.value
      }
      this.flow(`FLOW: relay_outgoing: loop ended`)
    }

    while (true) {
      const endPromise = defer<void>()

      let err: any

      const sinkPromise = currentSink(drain.call(this, endPromise)).catch((_err) => {
        err = _err
      })

      await Promise.race([sinkPromise, this._streamSinkSwitchPromise.promise])

      if (err) {
        this.error(`Sink threw error`, err.message)
        throw err
      } else {
        currentSink = await this._streamSinkSwitchPromise.promise
        this._streamSinkSwitchPromise = defer<Stream['sink']>()
        endPromise.resolve()
      }
    }
  }

  /**
   * Add status and control messages to queue
   * @param msg msg to add
   */
  private queueStatusMessage(msg: Uint8Array) {
    this._statusMessages.push(msg)

    this._statusMessagePromise.resolve()
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
        this._statusMessagePromise = defer<void>()
        return this._statusMessages.pop() as Uint8Array
      default:
        return this._statusMessages.shift() as Uint8Array
    }
  }
}

export { RelayContext }
