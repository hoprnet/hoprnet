import debug from 'debug'
const log = debug('hopr-core:transport')
const error = debug('hopr-core:transport:error')
const verbose = debug('hopr-core:verbose:transport:error')

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

import {
  RELAY_CIRCUIT_TIMEOUT,
  RELAY_REGISTER,
  OK,
  FAIL,
  FAIL_COULD_NOT_REACH_COUNTERPARTY,
  DELIVERY_REGISTER,
} from './constants'

import { pubKeyToPeerId } from '../../utils'
import { u8aEquals } from '@hoprnet/hopr-utils'

import { RelayContext } from './relayContext'

import type { DialOptions, Registrar, Dialer, Handler, Stream, PeerRouting } from './types'

type ConnectionContext = {
  deferredPromise: defer.DeferredPromise<AsyncIterable<Uint8Array>>
  sinkDefer: defer.DeferredPromise<void> | undefined
  aborted: boolean
  cache: Uint8Array | undefined
  id: PeerId
  source: Stream['source']
}

class Relay {
  private _dialer: Dialer
  private _registrar: Registrar
  private _handle: (protocols: string[] | string, handler: (connection: Handler) => void) => void
  private _dht: { peerRouting: PeerRouting } | undefined
  private _peerInfo: PeerInfo

  private on: (event: 'peer:connect', handler: (peer: PeerInfo) => void) => void

  private connHandler: (conn: Handler & { counterparty: PeerId }) => void | undefined

  private _streams: Map<string, Map<string, RelayContext>>

  constructor(libp2p: libp2p, _connHandler?: (conn: Handler) => void) {
    this._dialer = libp2p.dialer
    this._registrar = libp2p.registrar
    this._dht = libp2p._dht
    this._peerInfo = libp2p.peerInfo

    this.connHandler = _connHandler

    this.on = libp2p.on.bind(libp2p)

    this._handle = libp2p.handle.bind(libp2p)
    this._handle(RELAY_REGISTER, this.handleRelay.bind(this))
    this._handle(DELIVERY_REGISTER, this.handleRelayConnection.bind(this))
  }

  async handleReRegister() {}

  async handleRelayConnection(conn: Handler): Promise<void> {
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

    const potentialRelays = relays.filter((relay: PeerInfo) => !relay.id.equals(this._peerInfo.id))

    if (potentialRelays.length == 0) {
      throw Error(`Filtered list of relays and there is no one left to establish a connection. `)
    }

    for (let i = 0; i < potentialRelays.length; i++) {
      let relayConnection = this._registrar.getConnection(potentialRelays[i])

      if (relayConnection == null) {
        try {
          relayConnection = await this._dialer.connectToPeer(potentialRelays[i], { signal: options?.signal })
        } catch (err) {
          log(`Could not reach potential relay ${potentialRelays[i].id.toB58String()}. Error was: ${err}`)
          if (this._dht != null && (options == null || options.signal == null || !options.signal.aborted)) {
            let newAddress = await this._dht.peerRouting.findPeer(potentialRelays[i].id)

            try {
              relayConnection = await this._dialer.connectToPeer(newAddress, { signal: options?.signal })
            } catch (err) {
              log(
                `Dialling potential relay ${potentialRelays[
                  i
                ].id.toB58String()} after querying DHT failed. Error was ${err}`
              )
            }
          }
        }

        if (options?.signal?.aborted) {
          if (relayConnection != null) {
            try {
              await relayConnection.close()
            } catch (err) {
              error(err)
            }
          }
          throw new AbortError()
        }
      }

      if (relayConnection == null) {
        continue
      }

      let shaker: any
      try {
        shaker = handshake((await relayConnection.newStream([RELAY_REGISTER])).stream)
      } catch (err) {
        error(`failed to establish stream with ${potentialRelays[i].id.toB58String()}. Error was: ${err}`)
        continue
      }

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
            i
          ].id.toB58String()}. Answer was: <${new TextDecoder().decode(answer)}>`
        )
      }

      return {
        stream: shaker.stream,
        connection: relayConnection,
      }
    }

    throw Error(
      `Unable to establish a connection to any known relay node. Tried ${chalk.yellow(
        potentialRelays.map((potentialRelay: PeerInfo) => potentialRelay.id.toB58String()).join(`, `)
      )}`
    )
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
        `Received empty message from peer ${chalk.yellow(connection?.remotePeer.toB58String())} during connection setup`
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
          connection?.remotePeer.toB58String()
        )} asked to establish relayed connection to invalid counterparty. Error was ${err}. Received message ${pubKeySender}`
      )
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    // @TODO
    if (connection?.remotePeer != null && counterparty.equals(connection.remotePeer)) {
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    const deliveryStream = (await this.establishForwarding(counterparty)) as Stream

    if (deliveryStream == null) {
      // @TODO end deliveryStream
      shaker.write(FAIL_COULD_NOT_REACH_COUNTERPARTY)

      shaker.rest()

      return
    }

    shaker.write(OK)

    shaker.rest()

    const toSender = shaker.stream as Stream
    const toCounterparty = deliveryStream

    this.updateContext(
      this._peerInfo.id.toB58String(),
      counterparty.toB58String(),
      deliveryStream.source as any,
      toSender.sink
    )

    this.updateContext(
      counterparty.toB58String(),
      this._peerInfo.id.toB58String(),
      toSender.source as any,
      deliveryStream.sink
    )

    // let foundCounterparty = this._streams.get(counterparty.toB58String())

    // if (foundCounterparty == null) {
    //   foundCounterparty = new Map<string, RelayContext>()
    // }

    // const ctxCounterparty = new RelayContext(toCounterparty.source as any)

    // foundCounterparty.set(this._peerInfo.id.toB58String(), ctxCounterparty)

    // this._streams.set(counterparty.toB58String(), foundCounterparty)

    // let foundSender = this._streams.get(this._peerInfo.id.toB58String())

    // if (foundSender == null) {
    //   foundSender = new Map<string, RelayContext>()
    // }

    // const ctxCounterparty = new RelayContext(deliveryStream.source as any)

    // foundSender.set(counterparty.toB58String(), ctxCounterparty)

    // this._streams.set(this._peerInfo.id.toB58String(), foundSender)

    // toSender.sink(ctxCounterparty.source)

    // const counterpartyConn: ConnectionContext = {
    //   deferredPromise: defer<AsyncIterable<Uint8Array>>(),
    //   sinkDefer: undefined,
    //   aborted: false,
    //   cache: undefined,
    //   source: (async function* () {
    //     yield* toCounterparty.source

    //     while (true) {
    //       let source = await counterpartyConn.deferredPromise.promise

    //       counterpartyConn.deferredPromise = defer<AsyncIterable<Uint8Array>>()

    //       yield* source
    //     }
    //   })(),
    //   id: counterparty,
    // }

    // const initiatorConn: ConnectionContext = {
    //   deferredPromise: defer<AsyncIterable<Uint8Array>>(),
    //   sinkDefer: undefined,
    //   aborted: false,
    //   cache: undefined,
    //   source: (async function* () {
    //     yield* toSender.source

    //     while (true) {
    //       let source = await initiatorConn.deferredPromise.promise

    //       initiatorConn.deferredPromise = defer<AsyncIterable<Uint8Array>>()

    //       yield* source
    //     }
    //   })(),
    //   id: connection?.remotePeer,
    // }

    // this.on('peer:connect', async (peer: PeerInfo) => {
    //   if (peer.id.equals(counterparty)) {
    //     log(
    //       chalk.yellow(
    //         `overwriting counterparty connection. sender: ${chalk.blue(
    //           initiatorConn.id.toB58String()
    //         )} counterparty: ${chalk.blue(counterpartyConn.id.toB58String())}`
    //       )
    //     )
    //     await this.updateConnection(counterpartyConn, initiatorConn, true)
    //   } else if (peer.id.equals(connection?.remotePeer)) {
    //     log(
    //       chalk.yellow(
    //         `overwriting sender connection. sender: ${chalk.blue(
    //           initiatorConn.id.toB58String()
    //         )} counterparty: ${chalk.blue(counterpartyConn.id.toB58String())}`
    //       )
    //     )
    //     await this.updateConnection(initiatorConn, counterpartyConn, false)
    //   }
    // })

    // toCounterparty.sink(this.forward(counterpartyConn, initiatorConn.source))

    // toSender.sink(this.forward(initiatorConn, counterpartyConn.source))
  }

  async updateContext(
    to: string,
    from: string,
    newSource: AsyncGenerator<Uint8Array>,
    sink: (stream: AsyncIterable<Uint8Array>) => Promise<void>
  ) {
    let found = this._streams.get(to)

    if (found == null) {
      found = new Map<string, RelayContext>()
    }

    const ctx = new RelayContext(newSource)

    found.set(from, ctx)

    this._streams.set(this._peerInfo.id.toB58String(), found)

    sink(ctx.source)
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
        let result = await streamSource[Symbol.asyncIterator]().next()

        if (result.done) {
          break
        }

        // @TODO handle empty messages

        if (obj.aborted) {
          obj.cache = result.value
          break
        } else {
          yield result.value
        }
      }

      obj.sinkDefer.resolve()

      obj.aborted = false
    })()
  }

  async establishForwarding(counterparty: PeerId) {
    let timeout: any

    let cParty = new PeerInfo(counterparty)

    let newConn = this._registrar.getConnection(cParty)

    if (!newConn) {
      const abort = new AbortController()

      timeout = setTimeout(() => abort.abort(), RELAY_CIRCUIT_TIMEOUT)

      try {
        newConn = await this._dialer.connectToPeer(cParty, { signal: abort.signal }).catch(async (err) => {
          if (this._dht != null && !abort.signal.aborted) {
            cParty = await this._dht.peerRouting.findPeer(cParty.id)

            return this._dialer.connectToPeer(cParty, { signal: abort.signal })
          }

          throw Error(`Could not connect to counterparty. Error was: ${err}`)
        })
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
