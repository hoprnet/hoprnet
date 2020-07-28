import debug from 'debug'
const log = debug('hopr-core:transport')
const error = debug('hopr-core:transport:error')

import AbortController from 'abort-controller'
import { AbortError } from 'abortable-iterator'
import chalk from 'chalk'
import defer from 'p-defer'

// @ts-ignore
import handshake = require('it-handshake')

import type Multiaddr from 'multiaddr'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

// @ts-ignore
import libp2p = require('libp2p')

import { RELAY_CIRCUIT_TIMEOUT, RELAY_REGISTER, OK, FAIL, DELIVERY_REGISTER } from './constants'

import { pubKeyToPeerId } from '../../utils'
import { u8aEquals } from '@hoprnet/hopr-utils'

import type { Connection, DialOptions, Registrar, Dialer, Handler, Stream } from './types'

type ConnectionContext = {
  deferredPromise: defer.DeferredPromise<AsyncIterable<Uint8Array>>
  sinkDefer: defer.DeferredPromise<void> | undefined
  aborted: boolean
  cache: Uint8Array
  id: PeerId
  source: Stream['source']
}

class Relay {
  private _dialer: Dialer
  private _registrar: Registrar
  private _handle: (protocols: string[] | string, handler: (connection: Handler) => void) => void
  private on: (event: 'peer:connect', handler: (peer: PeerInfo) => void) => void

  private connHandler: (conn: Handler & { counterparty: PeerId }) => void

  constructor(libp2p: libp2p, _connHandler: (conn: Handler) => void) {
    this._dialer = libp2p.dialer
    this._registrar = libp2p.registrar

    this.connHandler = _connHandler

    this.on = libp2p.on.bind(libp2p)

    this._handle = libp2p.handle.bind(libp2p)
    this._handle(RELAY_REGISTER, this.handleRelay.bind(this))
    this._handle(DELIVERY_REGISTER, this.handleRelayConnection.bind(this))
  }

  async handleRelayConnection(conn: Handler): Promise<{ stream: Stream; counterparty: PeerId; conn: Connection }> {
    let shaker = handshake(conn.stream)

    let sender: PeerId

    let pubKeySender: Buffer | undefined
    try {
      pubKeySender = (await shaker.read())?.slice()
    } catch (err) {
      error(err)
    }

    if (pubKeySender == null) {
      error(`Received empty message. Ignoring connection ...`)
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    try {
      sender = await pubKeyToPeerId(pubKeySender)
    } catch (err) {
      error(`Could not decode sender peerId. Error was: ${err}`)
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    shaker.write(OK)
    shaker.rest()

    this.connHandler?.({ stream: shaker.stream, connection: conn.connection, counterparty: sender })
  }

  async establishRelayedConnection(ma: Multiaddr, relays: PeerInfo[], options?: DialOptions): Promise<Handler> {
    const destination = PeerId.createFromCID(ma.getPeerId())

    if (options?.signal?.aborted) {
      throw new AbortError()
    }

    let [relayConnection, index] = await Promise.race(
      relays.map(
        async (relay: PeerInfo, index: number): Promise<[Connection, number]> => {
          let relayConnection = this._registrar.getConnection(relay)

          if (!relayConnection) {
            relayConnection = await this._dialer.connectToPeer(relay, { signal: options?.signal })
          }

          return [relayConnection, index]
        }
      )
    )

    if (!relayConnection) {
      throw Error(
        `Unable to establish a connection to any known relay node. Tried ${chalk.yellow(
          relays.map((relay: PeerInfo) => relay.id.toB58String()).join(`, `)
        )}`
      )
    }

    if (options?.signal?.aborted) {
      try {
        await relayConnection.close()
      } catch (err) {
        error(err)
      }
      throw new AbortError()
    }

    let { stream } = await relayConnection.newStream([RELAY_REGISTER])

    const shaker = handshake(stream)

    shaker.write(destination.pubKey.marshal())

    let answer: Buffer | undefined
    try {
      answer = (await shaker.read())?.slice()
    } catch (err) {
      error(err)
    }

    shaker.rest()

    if (answer == null || !u8aEquals(answer, OK)) {
      throw Error(
        `Could not establish relayed connection to ${chalk.blue(destination.toB58String())} over relay ${relays[
          index
        ].id.toB58String()}. Answer was: <${new TextDecoder().decode(answer)}>`
      )
    }

    return {
      stream: shaker.stream,
      connection: relayConnection,
    }
  }

  async handleRelay({ stream, connection }: Handler) {
    const shaker = handshake(stream)

    let counterparty: PeerId
    let pubKeySender: Buffer | undefined

    try {
      pubKeySender = (await shaker.read())?.slice()
    } catch (err) {
      error(err)
    }

    if (pubKeySender == null) {
      error(
        `Received empty message from peer ${chalk.yellow(connection.remotePeer.toB58String())} during connection setup`
      )
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    try {
      counterparty = await pubKeyToPeerId(pubKeySender)
    } catch (err) {
      error(
        `Peer ${chalk.yellow(
          connection.remotePeer.toB58String()
        )} asked to establish relayed connection to invalid counterparty. Error was ${err}. Received message ${pubKeySender}`
      )
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    // @TODO
    if (connection.remotePeer != null && counterparty.isEqual(connection.remotePeer)) {
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    const deliveryStream = await this.establishForwarding(counterparty)

    if (!deliveryStream) {
      shaker.write(FAIL)

      shaker.rest()

      return
    }

    shaker.write(OK)

    shaker.rest()

    const toSender = shaker.stream
    const toCounterparty = deliveryStream

    const counterpartyConn: ConnectionContext = {
      deferredPromise: defer<AsyncIterable<Uint8Array>>(),
      sinkDefer: undefined,
      aborted: false,
      cache: undefined,
      source: (async function* () {
        yield* toCounterparty.source

        while (true) {
          let source = await counterpartyConn.deferredPromise.promise

          counterpartyConn.deferredPromise = defer<AsyncIterable<Uint8Array>>()

          yield* source
        }
      })(),
      id: counterparty,
    }

    const initiatorConn: ConnectionContext = {
      deferredPromise: defer<AsyncIterable<Uint8Array>>(),
      sinkDefer: undefined,
      aborted: false,
      cache: undefined,
      source: (async function* () {
        yield* toSender.source

        while (true) {
          let source = await initiatorConn.deferredPromise.promise

          initiatorConn.deferredPromise = defer<AsyncIterable<Uint8Array>>()

          yield* source
        }
      })(),
      id: connection.remotePeer,
    }

    this.on('peer:connect', async (peer: PeerInfo) => {
      if (peer.id.isEqual(counterparty)) {
        log(
          chalk.yellow(
            `overwriting counterparty connection. sender: ${chalk.blue(
              initiatorConn.id.toB58String()
            )} counterparty: ${chalk.blue(counterpartyConn.id.toB58String())}`
          )
        )
        await this.updateConnection(counterpartyConn, initiatorConn, true)
      } else if (peer.id.isEqual(connection.remotePeer)) {
        log(
          chalk.yellow(
            `overwriting sender connection. sender: ${chalk.blue(
              initiatorConn.id.toB58String()
            )} counterparty: ${chalk.blue(counterpartyConn.id.toB58String())}`
          )
        )
        await this.updateConnection(initiatorConn, counterpartyConn, false)
      }
    })

    toCounterparty.sink(this.forward(counterpartyConn, initiatorConn.source))

    toSender.sink(this.forward(initiatorConn, counterpartyConn.source))
  }

  async updateConnection(reconnected: ConnectionContext, existing: ConnectionContext, senderToCounterparty: boolean) {
    reconnected.aborted = true

    const newStream = await this.establishForwarding(reconnected.id)

    if (newStream == null) {
      error(`Could not establish relay delivery stream to node ${reconnected.id.toB58String()}`)
      return
    }

    newStream.sink(this.forward(reconnected, existing.source))

    reconnected.deferredPromise.resolve(newStream.source)
  }

  forward(obj: ConnectionContext, streamSource: AsyncIterable<Uint8Array>): AsyncIterable<Uint8Array> {
    return (async function* () {
      obj.sinkDefer && (await obj.sinkDefer.promise)

      obj.sinkDefer = defer<void>()

      if (obj.cache != null) {
        let cacheResult = obj.cache
        obj.cache = undefined

        yield cacheResult
      }

      while (!obj.aborted) {
        const msg = (await streamSource[Symbol.asyncIterator]().next()).value

        // @TODO handle empty messages

        if (obj.aborted) {
          obj.cache = msg
          break
        } else {
          yield msg
        }
      }

      obj.sinkDefer.resolve()

      obj.aborted = false
    })()
  }

  async establishForwarding(counterparty: PeerId) {
    let timeout: any

    let newConn = this._registrar.getConnection(new PeerInfo(counterparty))

    if (!newConn) {
      const abort = new AbortController()

      timeout = setTimeout(() => abort.abort(), RELAY_CIRCUIT_TIMEOUT)

      try {
        newConn = await this._dialer.connectToPeer(new PeerInfo(counterparty), { signal: abort.signal })
      } catch (err) {
        clearTimeout(timeout)
        error(err)
        return
      }
    }

    const { stream: newStream } = await newConn.newStream([DELIVERY_REGISTER])

    timeout && clearTimeout(timeout)

    const toCounterparty = handshake(newStream)

    toCounterparty.write(counterparty.pubKey.marshal())

    let answer: Buffer | undefined
    try {
      answer = (await toCounterparty.read())?.slice()
    } catch (err) {
      error(err)
      return
    }

    toCounterparty.rest()

    if (answer == null || !u8aEquals(answer, OK)) {
      error(`Could not relay to peer ${counterparty.toB58String()} because we are unable to deliver packets.`)

      return
    }

    return toCounterparty.stream
  }
}

export default Relay
