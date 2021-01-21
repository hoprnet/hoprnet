import { u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import { randomBytes } from 'crypto'

import Defer, { DeferredPromise } from 'p-defer'

import type { Stream } from 'libp2p'

import Debug from 'debug'
const _log = Debug(`hopr-connect`)
const _verbose = Debug(`hopr-connect:verbose`)
const _error = Debug(`hopr-connect:error`)

import {
  RELAY_STATUS_PREFIX,
  STOP,
  RESTART,
  RELAY_CONNECTION_STATUS_PREFIX,
  PING,
  PONG,
  VALID_PREFIXES
} from './constants'

const DEFAULT_PING_TIMEOUT = 300

class RelayContext {
  private _switchPromise: DeferredPromise<Stream>
  private _streamSourceSwitchPromise: DeferredPromise<Stream['source']>
  private _streamSinkSwitchPromise: DeferredPromise<Stream['sink']>

  private _id: string

  private _sinkSourceAttached: boolean
  private _sinkSourceAttachedPromise: DeferredPromise<Stream['source']>

  private _statusMessagePromise: DeferredPromise<void>
  private _statusMessages: Uint8Array[] = []
  private _pingResponsePromise?: DeferredPromise<void>
  private _stream: Stream

  private _sourcePromise: Promise<IteratorResult<Uint8Array, void>> | undefined
  private _sourceSwitched: boolean
  private _sinkSwitched: boolean

  public source: Stream['source']
  public sink: Stream['sink']
  public ping: (ms?: number) => Promise<number>
  public update: (newStream: Stream) => void

  constructor(stream: Stream) {
    this._switchPromise = Defer<Stream>()

    this._switchPromise.promise.then(this.switchFunction.bind(this))

    this._id = u8aToHex(randomBytes(4), false)

    this._statusMessagePromise = Defer<void>()
    this._statusMessages = []
    this._stream = stream

    this._sourceSwitched = false
    this._sinkSwitched = false

    this._sinkSourceAttached = false
    this._sinkSourceAttachedPromise = Defer<Stream['source']>()

    this._streamSourceSwitchPromise = Defer<Stream['source']>()
    this._streamSinkSwitchPromise = Defer<Stream['sink']>()

    this.source = this._createSource()
    // this.source.next()

    this.sink = (source: Stream['source']): Promise<void> => {
      // @TODO add support for Iterables such as Arrays
      this._sinkSourceAttached = true
      this._sinkSourceAttachedPromise.resolve(source)

      return Promise.resolve()
    }

    this.ping = async (ms: number = DEFAULT_PING_TIMEOUT) => {
      this.log(`ping`)
      let start = Date.now()
      this._pingResponsePromise = Defer<void>()

      let timeoutDone = false
      let timeout: NodeJS.Timeout

      const timeoutPromise = new Promise<void>((resolve) => {
        timeout = setTimeout(() => {
          this.log(`ping timeout done`)
          timeoutDone = true
          resolve()
        }, ms)
      })

      this._statusMessages.push(Uint8Array.from([...RELAY_STATUS_PREFIX, ...PING]))
      this._statusMessagePromise.resolve()

      await Promise.race([
        // prettier-ignore
        timeoutPromise,
        this._pingResponsePromise.promise
      ])

      if (timeoutDone) {
        return -1
      }

      clearTimeout(timeout!)
      return Date.now() - start
    }

    this.update = (newStream: Stream) => {
      this.log(`updating`)
      let tmpPromise = this._switchPromise
      this._switchPromise = Defer<Stream>()
      tmpPromise.resolve(newStream)
    }

    this._createSink()
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

  private switchFunction(stream: Stream): void {
    this._sinkSwitched = true
    this._sourceSwitched = true

    this._streamSourceSwitchPromise.resolve(stream.source)
    this._streamSinkSwitchPromise.resolve(stream.sink)

    this._switchPromise = Defer<Stream>()

    this._switchPromise.promise.then(this.switchFunction.bind(this))
  }

  private _createSource(): Stream['source'] {
    const iterator: Stream['source'] = async function* (this: RelayContext) {
      this.log(`source called`)
      let result: Stream['source'] | IteratorResult<Uint8Array, void> | undefined

      const next = () => {
        result = undefined

        this._sourcePromise = this._stream.source.next()
      }

      while (true) {
        const promises: Promise<Stream['source'] | IteratorResult<Uint8Array, void>>[] = [
          this._streamSourceSwitchPromise.promise
        ]

        if (result == undefined || (result as IteratorResult<Uint8Array, void>).done != true) {
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
          yield Uint8Array.from([...RELAY_STATUS_PREFIX, ...RESTART])

          this._sourcePromise = this._stream.source.next()
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
        if (u8aEquals(PREFIX, RELAY_STATUS_PREFIX)) {
          if (u8aEquals(SUFFIX, PING)) {
            this.verbose(`PING received`)
            this._statusMessages.push(Uint8Array.from([...RELAY_STATUS_PREFIX, ...PONG]))

            this._statusMessagePromise.resolve()

            // Don't forward ping
          } else if (u8aEquals(SUFFIX, PONG)) {
            this.verbose(`PONG received`)

            this._pingResponsePromise?.resolve()

            // Don't forward pong message
          }

          next()

          continue
          // Interprete connection sub-protocol
        } else if (u8aEquals(PREFIX, RELAY_CONNECTION_STATUS_PREFIX)) {
          if (u8aEquals(SUFFIX, STOP)) {
            this.verbose(`STOP relayed`)

            // forward STOP message
            yield received.value

            // close stream
            break
          } else if (u8aEquals(SUFFIX, RESTART)) {
            this.verbose(`RESTART relayed`)
          }

          this.log(`sourcing`, received.value)
          yield received.value

          next()

          continue
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

    let sourcePromise: Promise<IteratorResult<Uint8Array, void>> | undefined

    let currentSource: Stream['source'] | undefined

    let statusMessageAvailable = false

    const statusSourceFunction = () => {
      statusMessageAvailable = true
    }

    let statusPromise = this._statusMessagePromise.promise.then(statusSourceFunction)

    let switchPromise = Defer<void>()

    async function* drain(this: RelayContext): Stream['source'] {
      type SinkResult = Stream['sink'] | Stream['source'] | IteratorResult<Uint8Array, void> | void

      let result: SinkResult

      while (true) {
        const promises: Promise<SinkResult>[] = []

        if (currentSource == undefined) {
          promises.push(this._sinkSourceAttachedPromise.promise)
        }

        promises.push(this._streamSinkSwitchPromise.promise, statusPromise)

        if (
          currentSource != undefined &&
          (result == undefined || (result as IteratorResult<Uint8Array, void>).done != true)
        ) {
          sourcePromise = sourcePromise ?? currentSource.next()

          promises.push(sourcePromise)
        }

        // (0. Handle source attach)
        // 1. Handle stream switches
        // 2. Handle status messages
        // 3. Handle payload messages
        result = await Promise.race(promises)

        if (this._sinkSourceAttached) {
          this._sinkSourceAttached = false
          currentSource = result as Stream['source']

          result = undefined
          continue
        }

        if (statusMessageAvailable) {
          if (this._statusMessages.length > 0) {
            yield this._statusMessages.shift() as Uint8Array
          }

          if (
            this._statusMessages.length == 0 ||
            (result != undefined && (result as IteratorResult<Uint8Array, void>).done != true)
          ) {
            statusMessageAvailable = false

            this._statusMessagePromise = Defer<void>()

            statusPromise = this._statusMessagePromise.promise.then(statusSourceFunction)
          }
          continue
        }

        if (this._sinkSwitched) {
          currentSink = result as Stream['sink']
          this._streamSinkSwitchPromise = Defer<Stream['sink']>()

          let tmpPromise = switchPromise
          switchPromise = Defer<void>()

          sourcePromise = undefined
          this._sinkSwitched = false

          tmpPromise.resolve()
          break
        }

        let received = result as IteratorResult<Uint8Array>

        if (received.done) {
          continue
        }

        try {
          let [PREFIX, SUFFIX] = [received.value.slice(0, 1), received.value.slice(1)]

          if (u8aEquals(PREFIX, RELAY_STATUS_PREFIX) && u8aEquals(SUFFIX, STOP)) {
            yield received.value
            break
          }

          yield received.value

          result = undefined

          sourcePromise = currentSource?.next()
        } catch (err) {
          this.error(err)
          throw err
        }
      }
    }

    while (true) {
      currentSink(drain.call(this))

      await switchPromise.promise
    }
  }
}

export { RelayContext }
