import { u8aToHex } from '@hoprnet/hopr-utils'
import { randomBytes } from 'crypto'

import Defer, { DeferredPromise } from 'p-defer'
import EventEmitter from 'events'

import type { Stream, StreamResult } from 'libp2p'

import Debug from 'debug'
const _log = Debug(`hopr-connect`)
const _verbose = Debug(`hopr-connect:verbose`)
const _error = Debug(`hopr-connect:error`)

import { RelayPrefix, StatusMessages, VALID_PREFIXES, ConnectionStatusMessages } from '../constants'

export const DEFAULT_PING_TIMEOUT = 300

class RelayContext extends EventEmitter {
  private _streamSourceSwitchPromise: DeferredPromise<Stream['source']>
  private _streamSinkSwitchPromise: DeferredPromise<Stream['sink']>

  private _id: string

  private _sinkSourceAttached: boolean
  private _sinkSourceAttachedPromise: DeferredPromise<Stream['source']>

  private _statusMessagePromise: DeferredPromise<void>
  private _statusMessages: Uint8Array[] = []
  private _pingResponsePromise?: DeferredPromise<void>
  private _stream: Stream

  private _sourcePromise: Promise<StreamResult> | undefined
  private _sourceSwitched: boolean

  public source: Stream['source']
  public sink: Stream['sink']

  constructor(stream: Stream) {
    super()
    this._id = u8aToHex(randomBytes(4), false)

    this._statusMessagePromise = Defer<void>()
    this._statusMessages = []
    this._stream = stream

    this._sourceSwitched = false

    this._sinkSourceAttached = false
    this._sinkSourceAttachedPromise = Defer<Stream['source']>()

    this._streamSourceSwitchPromise = Defer<Stream['source']>()
    this._streamSinkSwitchPromise = Defer<Stream['sink']>()

    this.source = this._createSource()

    this.sink = (source: Stream['source']): Promise<void> => {
      // @TODO add support for Iterables such as Arrays
      this._sinkSourceAttached = true
      this._sinkSourceAttachedPromise.resolve(source)

      return Promise.resolve()
    }

    this._createSink()
  }

  public async ping(ms = DEFAULT_PING_TIMEOUT): Promise<number> {
    let start = Date.now()
    this._pingResponsePromise = Defer<void>()

    let timeoutDone = false

    const timeoutPromise = Defer<void>()
    const timeout = setTimeout(() => {
      this.log(`ping timeout done`)
      timeoutDone = true
      // Make sure that we don't produce any hanging promises
      this._pingResponsePromise?.resolve()
      this._pingResponsePromise = undefined
      timeoutPromise.resolve()
    }, ms)

    this.queueStatusMessage(StatusMessages.PING)

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

  public update(newStream: Stream) {
    this._sourceSwitched = true

    this._streamSourceSwitchPromise.resolve(newStream.source)
    this._streamSinkSwitchPromise.resolve(newStream.sink)

    this.log(`updating`)
  }

  private log(..._: any[]) {
    _log(`RX [${this._id}]`, ...arguments)
  }

  private verbose(..._: any[]) {
    _verbose(`RX [${this._id}]`, ...arguments)
  }

  private error(..._: any[]) {
    _error(`RX [${this._id}]`, ...arguments)
  }

  private _createSource(): Stream['source'] {
    const iterator: Stream['source'] = async function* (this: RelayContext) {
      this.log(`source called`)
      let result: Stream['source'] | StreamResult | undefined

      const next = () => {
        result = undefined

        this._sourcePromise = this._stream.source.next()
      }

      while (true) {
        const promises: Promise<Stream['source'] | StreamResult>[] = [this._streamSourceSwitchPromise.promise]

        if (result == undefined || (result as StreamResult).done != true) {
          this._sourcePromise = this._sourcePromise ?? this._stream.source.next()

          promises.push(this._sourcePromise)
        }

        // 1. Handle Stream switches
        // 2. Handle payload / status messages
        result = await Promise.race(promises)

        if (result == undefined) {
          // @TODO throw Error to make debugging easier
          throw Error(`source result == undefined. Should not happen.`)
        }

        if (this._sourceSwitched) {
          this._sourceSwitched = false

          this._stream.source = result as Stream['source']

          result = undefined

          this._streamSourceSwitchPromise = Defer<Stream['source']>()

          this._sourcePromise = this._stream.source.next()

          yield Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART)

          continue
        }

        const received = result as IteratorYieldResult<Uint8Array>

        if (received.done) {
          continue
        }

        if (received.value.length == 0) {
          this.log(`got empty message`)
          next()

          // Ignore empty messages
          continue
        }

        const [PREFIX, SUFFIX] = [received.value.slice(0, 1), received.value.slice(1)]

        if (!VALID_PREFIXES.includes(PREFIX[0])) {
          this.error(`Invalid prefix: Got <${u8aToHex(PREFIX ?? new Uint8Array())}>. Dropping message in relayContext.`)

          next()

          // Ignore invalid prefixes
          continue
        }

        // Interprete relay sub-protocol
        if (PREFIX[0] == RelayPrefix.STATUS_MESSAGE) {
          if (SUFFIX[0] == StatusMessages.PING) {
            this.verbose(`PING received`)
            this.queueStatusMessage(StatusMessages.PONG)
            // Don't forward ping
          } else if (SUFFIX[0] == StatusMessages.PONG) {
            this.verbose(`PONG received`)

            this._pingResponsePromise?.resolve()
            // Don't forward pong message
          }

          next()

          continue
          // Interprete connection sub-protocol
        } else if (PREFIX[0] == RelayPrefix.CONNECTION_STATUS) {
          if (SUFFIX[0] == ConnectionStatusMessages.STOP) {
            this.verbose(`STOP relayed`)

            this.emit('close')
            // forward STOP message
            yield received.value

            // close stream
            break
          } else if ((SUFFIX[0] = ConnectionStatusMessages.RESTART)) {
            this.verbose(`RESTART relayed`)
          }
        }

        yield received.value

        next()
      }
    }.call(this)

    let result = iterator.next()

    return (async function* () {
      const received = await result
      if (received.done) {
        return
      }
      yield received.value

      yield* iterator
    })()
  }

  private async _createSink(): Promise<void> {
    this.log(`createSink called`)
    let currentSink = this._stream.sink

    let sourcePromise: Promise<StreamResult> | undefined

    let currentSource: Stream['source'] | undefined

    let iteration = 0

    async function* drain(this: RelayContext): Stream['source'] {
      // deep-clone number
      // @TODO make sure that the compiler does not notice
      const drainIteration = parseInt(iteration.toString())
      type SinkResult = Stream['source'] | StreamResult | void

      let result: SinkResult

      const next = () => {
        result = undefined

        sourcePromise = currentSource?.next()
      }

      while (iteration == drainIteration) {
        const promises: Promise<SinkResult>[] = []

        if (currentSource == undefined) {
          promises.push(this._sinkSourceAttachedPromise.promise)
        }

        promises.push(this._statusMessagePromise.promise)

        if (currentSource != undefined && (result == undefined || (result as StreamResult).done != true)) {
          sourcePromise = sourcePromise ?? currentSource.next()

          promises.push(sourcePromise)
        }

        // (0. Handle source attach)
        // 1. Handle stream switches
        // 2. Handle status messages
        // 3. Handle payload messages
        result = await Promise.race(promises)

        // Don't handle incoming messages after migration
        if (iteration != drainIteration) {
          break
        }

        if (this._sinkSourceAttached) {
          this._sinkSourceAttached = false
          currentSource = result as Stream['source']

          result = undefined
          continue
        }

        if (this._statusMessages.length > 0) {
          yield this.unqueueStatusMessage()
          continue
        }

        let received = result as StreamResult

        if (received.done) {
          continue
        }

        if (received.value.length == 0) {
          this.verbose(`Ignoring empty message`)
          next()
          continue
        }

        let [PREFIX, SUFFIX] = [received.value.slice(0, 1), received.value.slice(1)]

        if (SUFFIX.length == 0) {
          this.verbose(`Ignoring empty payload`)
          next()
          continue
        }

        if (PREFIX[0] == RelayPrefix.CONNECTION_STATUS && SUFFIX[0] == ConnectionStatusMessages.STOP) {
          yield received.value
          break
        }

        next()

        yield received.value
      }
    }

    while (true) {
      currentSink(drain.call(this))

      currentSink = await this._streamSinkSwitchPromise.promise
      iteration++
      this._streamSinkSwitchPromise = Defer<Stream['sink']>()
    }
  }

  private queueStatusMessage(msg: StatusMessages) {
    this._statusMessages.push(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, msg))

    this._statusMessagePromise.resolve()
  }

  private unqueueStatusMessage(): Uint8Array {
    switch (this._statusMessages.length) {
      case 0:
        throw Error(`Trying to unqueue empty status message queue`)
      case 1:
        this._statusMessagePromise = Defer<void>()
        return this._statusMessages.pop() as Uint8Array
      default:
        return this._statusMessages.shift() as Uint8Array
    }
  }
}

export { RelayContext }
