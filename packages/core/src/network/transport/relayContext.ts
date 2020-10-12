import { u8aConcat, u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import Defer, { DeferredPromise } from 'p-defer'
import BL from 'bl'

import { Stream } from './types'

import Debug from 'debug'
const log = Debug(`hopr-core:transport`)
const verbose = Debug(`hopr-core:verbose:transport`)

const error = Debug(`hopr-core:transport:error`)

import { RELAY_STATUS_PREFIX, STOP, RESTART } from './constants'

class RelayContext {
  private _switchPromise: DeferredPromise<Stream>
  private _stream: Stream

  public source: Stream['source']
  public sink: Stream['sink']

  constructor(stream: Stream) {
    this._switchPromise = Defer<Stream>()
    this._stream = stream

    this.source = this._createSource.call(this)

    this.sink = this._createSink.bind(this)
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
          if (u8aEquals(PREFIX, RELAY_STATUS_PREFIX)) {
            if (u8aEquals(SUFFIX, STOP)) {
              verbose(`STOP relayed`)
              break
            } else if (u8aEquals(SUFFIX, RESTART)) {
              verbose(`RESTART relayed`)
            } else {
              error(`Invalid status message. Got <${u8aToHex(SUFFIX)}>`)
            }
          }

          verbose(`relaying ${new TextDecoder().decode(sourceMsg.slice(1))}`, u8aToHex(sourceMsg.slice()))

          yield sourceMsg
        }

        if (!sourceDone) {
          sourcePromise = currentSource.next().then(sourceFunction)
        }
      } else if (streamSwitched) {
        streamSwitched = false
        sourceDone = false
        currentSource = tmpSource
        switchPromise = this._switchPromise.promise.then(switchFunction)
        yield new BL([(RELAY_STATUS_PREFIX as unknown) as BL, (RESTART as unknown) as BL])
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

      let switchPromise = this._switchPromise.promise.then(() => {
        streamSwitched = true
      })

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
  update(newStream: Stream) {
    log(`updating`)
    let tmpPromise = this._switchPromise
    this._switchPromise = Defer<Stream>()
    tmpPromise.resolve(newStream)
  }
}

export { RelayContext }
