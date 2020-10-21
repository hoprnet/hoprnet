import { u8aConcat, u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import Defer, { DeferredPromise } from 'p-defer'

import { Stream } from '../../@types/transport'

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
  private _statusMessages: Uint8Array[]
  private _pingResponsePromise: DeferredPromise<void>
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

      const timeoutPromise = new Promise((resolve) => {
        timeout = (setTimeout(() => {
          console.log(timeoutDone)
          timeoutDone = true
          resolve()
        }, ms) as unknown) as NodeJS.Timeout
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
      } else {
        clearTimeout(timeout)
        return Date.now() - start
      }
    }

    this.update = (newStream: Stream) => {
      log(`updating`)
      let tmpPromise = this._switchPromise
      this._switchPromise = Defer<Stream>()
      tmpPromise.resolve(newStream)
    }
  }

  private async *_createSource() {
    let sourceReceived = false
    let sourceMsg: Uint8Array
    let sourceDone = false

    function sourceFunction({ value, done }: { value?: Uint8Array | void; done?: boolean | void }) {
      sourceReceived = true
      sourceMsg = value as Uint8Array

      if (done) {
        sourceDone = true
      }
    }

    let tmpSource: Stream['source']
    let currentSource = this._stream.source

    let sourcePromise = currentSource.next().then(sourceFunction)

    let streamSwitched = false

    function switchFunction(stream: Stream) {
      tmpSource = stream.source
      streamSwitched = true
    }
    let switchPromise = this._switchPromise.promise.then(switchFunction)

    while (true) {
      if (!sourceDone) {
        await Promise.race([
          // prettier-ignore
          sourcePromise,
          switchPromise
        ])
      } else {
        await switchPromise
      }

      if (sourceReceived) {
        sourceReceived = false
        // Does not forward empty messages
        if (sourceMsg != null) {
          const received = sourceMsg.slice()

          const [PREFIX, SUFFIX] = [received.subarray(0, 1), received.subarray(1)]

          if (![RELAY_STATUS_PREFIX[0], RELAY_WEBRTC_PREFIX[0], RELAY_PAYLOAD_PREFIX[0]].includes(PREFIX[0])) {
            error(`Invalid prefix: Got <${u8aToHex(PREFIX)}>. Dropping message in relayContext.`)
            if (!sourceDone) {
              sourcePromise = currentSource.next().then(sourceFunction)
            }
            continue
          }

          if (u8aEquals(PREFIX, RELAY_STATUS_PREFIX)) {
            if (u8aEquals(SUFFIX, STOP)) {
              verbose(`STOP relayed`)
              break
            } else if (u8aEquals(SUFFIX, RESTART)) {
              verbose(`RESTART relayed`)
            } else if (u8aEquals(SUFFIX, PING)) {
              verbose(`PING received`)
              this._statusMessages.push(u8aConcat(RELAY_STATUS_PREFIX, PING_RESPONSE))

              let tmpPromise = this._statusMessagePromise
              this._statusMessagePromise = Defer<void>()
              tmpPromise.resolve()

              if (!sourceDone) {
                sourcePromise = currentSource.next().then(sourceFunction)
              }

              // Don't forward ping to receiver
              continue
            } else if (u8aEquals(SUFFIX, PING_RESPONSE)) {
              verbose(`PONG received`)

              this._pingResponsePromise?.resolve()

              if (!sourceDone) {
                sourcePromise = currentSource.next().then(sourceFunction)
              }

              // Don't forward pong message to receiver
              continue
            } else {
              error(`received status message`, SUFFIX)
              //error(`Invalid status message. Got <${u8aToHex(SUFFIX)}>`)
            }
          }

          //verbose(`relaying ${new TextDecoder().decode(SUFFIX)}`, u8aToHex(received))
          verbose(`relaying`, SUFFIX)
          yield received
        } else {
          verbose(`empty message dropped`)
        }

        if (!sourceDone) {
          sourcePromise = currentSource.next().then(sourceFunction)
        }
      } else if (streamSwitched) {
        streamSwitched = false
        sourceDone = false
        currentSource = tmpSource
        switchPromise = this._switchPromise.promise.then(switchFunction)
        verbose(`################### streamSwitched ###################`)
        // @TODO replace this by a mutex
        await new Promise((resolve) => setTimeout(resolve, 100))
        yield u8aConcat(RELAY_STATUS_PREFIX, RESTART)

        sourcePromise = currentSource.next().then(sourceFunction)
      }
    }
  }

  private async _createSink(source: Stream['source']) {
    let currentSink = this._stream.sink

    async function* drain(this: RelayContext) {
      let sourceReceived = false
      let sourceMsg: Uint8Array | void
      let sourceDone = false

      function sourceFunction({ value, done }: { value?: Uint8Array | void; done?: boolean | void }) {
        sourceReceived = true
        sourceMsg = value

        if (done) {
          sourceDone = true
        }
      }

      let sourcePromise = source.next().then(sourceFunction)

      let streamSwitched = false
      let statusMessageAvailable = false

      let switchPromise = this._switchPromise.promise.then(() => {
        streamSwitched = true
      })

      function statusSourceFunction() {
        statusMessageAvailable = true
      }

      let statusPromise = this._statusMessagePromise.promise.then(statusSourceFunction)

      while (true) {
        if (!sourceDone) {
          await Promise.race([
            // prettier-ignore
            sourcePromise,
            statusPromise,
            switchPromise
          ])
        } else {
          await switchPromise
        }

        if (sourceReceived) {
          sourceReceived = false

          // Ignoring empty messages
          if (sourceMsg != null) {
            if (sourceDone) {
              return sourceMsg
            } else {
              yield sourceMsg
            }
          } else {
            if (sourceDone) {
              return
            }
          }

          sourcePromise = source.next().then(sourceFunction)
        } else if (statusMessageAvailable) {
          statusMessageAvailable = false

          while (this._statusMessages.length > 0) {
            yield this._statusMessages.shift()
          }

          statusPromise = this._statusMessagePromise.promise.then(statusSourceFunction)
        } else if (streamSwitched) {
          break
        }
      }
    }

    while (true) {
      currentSink(drain.call(this))

      currentSink = (await this._switchPromise.promise).sink
    }
  }
}

export { RelayContext }
