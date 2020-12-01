import { u8aConcat, u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import Defer, { DeferredPromise } from 'p-defer'

import type { Stream } from 'libp2p'

import Debug from 'debug'
const log = Debug(`hopr-core:transport`)
const verbose = Debug(`hopr-core:verbose:transport`)
const error = Debug(`hopr-core:transport:error`)

import {
  RELAY_STATUS_PREFIX,
  STOP,
  RESTART,
  RELAY_WEBRTC_PREFIX,
  RELAY_PAYLOAD_PREFIX,
  PING,
  PING_RESPONSE
} from './constants'

const DEFAULT_PING_TIMEOUT = 300

class RelayContext {
  private _switchPromise: DeferredPromise<Stream>
  private _statusMessagePromise: DeferredPromise<void>
  private _statusMessages: Uint8Array[] = []
  private _pingResponsePromise?: DeferredPromise<void>
  private _stream: Stream

  public source: Stream['source']
  public sink: Stream['sink']
  public ping: (ms?: number) => Promise<number>
  public update: (newStream: Stream) => void

  constructor(stream: Stream) {
    this._switchPromise = Defer<Stream>()
    this._statusMessagePromise = Defer<void>()
    this._statusMessages = []
    this._stream = stream

    this.source = this._createSource.call(this)

    this.sink = this._createSink.bind(this)

    this.ping = async (ms: number = DEFAULT_PING_TIMEOUT) => {
      let start = Date.now()
      this._pingResponsePromise = Defer<void>()

      let timeoutDone = false
      let timeout: NodeJS.Timeout

      const timeoutPromise = new Promise<void>((resolve) => {
        timeout = setTimeout(() => {
          log(`ping timeout done`)
          timeoutDone = true
          resolve()
        }, ms)
      })

      let tmpPromise = this._statusMessagePromise

      this._statusMessagePromise = Defer<void>()
      this._statusMessages.push(u8aConcat(RELAY_STATUS_PREFIX, PING))
      tmpPromise.resolve()

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
      log(`updating`)
      let tmpPromise = this._switchPromise
      this._switchPromise = Defer<Stream>()
      tmpPromise.resolve(newStream)
    }
  }

  private async *_createSource(): Stream['source'] {
    let result: IteratorResult<Uint8Array, void> = { done: false, value: new Uint8Array() }

    let iteration = 0
    const sourceFunction = (_iteration: number) => (arg: IteratorResult<Uint8Array, void>) => {
      result = arg
    }

    let tmpSource: Stream['source']
    let currentSource = this._stream.source

    let sourcePromise = currentSource.next().then(sourceFunction(iteration))

    let streamSwitched = false

    function switchFunction(stream: Stream) {
      tmpSource = stream.source
      streamSwitched = true
    }

    let switchPromise = this._switchPromise.promise.then(switchFunction)

    while (true) {
      if (!result.done) {
        await Promise.race([
          // prettier-ignore
          sourcePromise,
          switchPromise
        ])
      } else {
        await switchPromise
      }

      if (streamSwitched) {
        streamSwitched = false
        result.done = false
        currentSource = tmpSource!
        switchPromise = this._switchPromise.promise.then(switchFunction)
        yield u8aConcat(RELAY_STATUS_PREFIX, RESTART)

        sourcePromise = currentSource.next().then(sourceFunction(++iteration))
        continue
      }

      if (result.done) {
        continue
      }

      if (result.value == undefined || result.value.length == 0) {
        sourcePromise = currentSource.next().then(sourceFunction(iteration))

        continue
      }

      const received = result.value.slice()

      const [PREFIX, SUFFIX] = [received.subarray(0, 1), received.subarray(1)]

      if (
        !u8aEquals(RELAY_STATUS_PREFIX, PREFIX) &&
        !u8aEquals(RELAY_WEBRTC_PREFIX, PREFIX) &&
        !u8aEquals(RELAY_PAYLOAD_PREFIX, PREFIX)
      ) {
        error(`Invalid prefix: Got <${u8aToHex(PREFIX ?? new Uint8Array())}>. Dropping message in relayContext.`)

        sourcePromise = currentSource.next().then(sourceFunction(iteration))

        continue
      }

      if (u8aEquals(PREFIX, RELAY_STATUS_PREFIX)) {
        if (u8aEquals(SUFFIX, STOP)) {
          verbose(`STOP relayed`)
          yield received
          break
        } else if (u8aEquals(SUFFIX, RESTART)) {
          verbose(`RESTART relayed`)
        } else if (u8aEquals(SUFFIX, PING)) {
          verbose(`PING received`)
          this._statusMessages.push(u8aConcat(RELAY_STATUS_PREFIX, PING_RESPONSE))

          this._statusMessagePromise.resolve()

          sourcePromise = currentSource.next().then(sourceFunction(iteration))

          // Don't forward ping
          continue
        } else if (u8aEquals(SUFFIX, PING_RESPONSE)) {
          verbose(`PONG received`)

          this._pingResponsePromise?.resolve()

          sourcePromise = currentSource.next().then(sourceFunction(iteration))

          // Don't forward pong message
          continue
        } else {
          error(`Invalid status message. Got <${u8aToHex(SUFFIX || new Uint8Array([]))}>`)
        }
      }

      yield received

      sourcePromise = currentSource.next().then(sourceFunction(iteration))
    }
  }

  private async _createSink(source: Stream['source']): Promise<void> {
    let currentSink = this._stream.sink

    let result: IteratorResult<Uint8Array> = { done: false, value: new Uint8Array() }

    let iteration = 0
    const sourceFunction = (arg: IteratorResult<Uint8Array, void>) => {
      result = arg
    }

    let sourcePromise = source.next().then(sourceFunction)

    let streamSwitched = false
    let statusMessageAvailable = false

    const switchFunction = () => {
      streamSwitched = true
    }

    const statusSourceFunction = () => {
      statusMessageAvailable = true
    }
    let statusPromise = this._statusMessagePromise.promise.then(statusSourceFunction)

    async function* drain(this: RelayContext, _iteration: number): Stream['source'] {
      streamSwitched = false

      let switchPromise = this._switchPromise.promise.then(switchFunction)

      while (!result.done) {
        if (iteration != _iteration) {
          break
        }

        await Promise.race([
          // prettier-ignore
          sourcePromise,
          statusPromise,
          switchPromise
        ])

        if (iteration != _iteration) {
          break
        }

        if (statusMessageAvailable) {
          statusMessageAvailable = false

          while (this._statusMessages.length > 0) {
            yield this._statusMessages.shift()!
          }

          if (!result.done) {
            this._statusMessagePromise = Defer<void>()

            statusPromise = this._statusMessagePromise.promise.then(statusSourceFunction)
          }
          continue
        }

        if (streamSwitched) {
          break
        }

        if (result.done) {
          break
        }

        let received = result.value.slice()

        let [PREFIX, SUFFIX] = [received.slice(0, 1), received.slice(1)]

        if (u8aEquals(PREFIX, RELAY_STATUS_PREFIX) && u8aEquals(SUFFIX, STOP)) {
          yield received
          break
        }

        yield received

        sourcePromise = source.next().then(sourceFunction)
      }
    }

    while (!result.done) {
      currentSink(drain.call(this, iteration))

      currentSink = (await this._switchPromise.promise).sink

      iteration++
    }
  }
}

export { RelayContext }
