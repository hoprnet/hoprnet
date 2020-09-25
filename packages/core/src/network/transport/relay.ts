import debug from 'debug'
const log = debug('hopr-core:transport')
const error = debug('hopr-core:transport:error')
const verbose = debug('hopr-core:verbose:transport:error')

import AbortController from 'abort-controller'
import { AbortError } from 'abortable-iterator'
import chalk from 'chalk'
import BL from 'bl'

// @ts-ignore
import handshake = require('it-handshake')

import Multiaddr from 'multiaddr'
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
  RELAY_PAYLOAD_PREFIX,
  RELAY_STATUS_PREFIX,
  STOP,
} from './constants'

import { pubKeyToPeerId } from '../../utils'
import { u8aEquals } from '@hoprnet/hopr-utils'

import { RelayContext } from './relayContext'

import type { Connection, Dialer, DialOptions, Handler, PeerRouting, Registrar, Stream } from './types'

class Relay {
  private _dialer: Dialer
  private _registrar: Registrar
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

    this._streams = new Map<string, Map<string, RelayContext>>()

    this.on = libp2p.on.bind(libp2p)

    libp2p.handle(RELAY_REGISTER, this.handleRelay.bind(this))
    libp2p.handle(DELIVERY_REGISTER, this.handleRelayConnection.bind(this))
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
      let relayConnection: Connection
      try {
        relayConnection = await this.connectToRelay(potentialRelays[i], options)
      } catch (err) {
        error(err)
        continue
      }

      let stream: Stream
      try {
        stream = await this.performHandshake(relayConnection, potentialRelays[i].id, destination)
      } catch (err) {
        error(err)
        continue
      }

      return {
        stream: {
          async sink(source: AsyncIterable<Uint8Array>) {
            stream.sink(
              (async function* () {
                for await (const msg of source) {
                  yield (new BL([
                    (RELAY_PAYLOAD_PREFIX as unknown) as BL,
                    (msg as unknown) as BL,
                  ]) as unknown) as Uint8Array
                }
              })()
            )
          },
          source: stream.source,
        },
        connection: relayConnection,
      }
    }

    throw Error(
      `Unable to establish a connection to any known relay node. Tried ${chalk.yellow(
        potentialRelays.map((potentialRelay: PeerInfo) => potentialRelay.id.toB58String()).join(`, `)
      )}`
    )
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

  private async connectToRelay(relay: PeerInfo, options?: DialOptions): Promise<Connection> {
    let relayConnection = this._registrar.getConnection(relay)

    if (relayConnection == null) {
      try {
        relayConnection = await this._dialer.connectToPeer(relay, { signal: options?.signal })
      } catch (err) {
        log(`Could not reach potential relay ${relay.id.toB58String()}. Error was: ${err}`)
        if (this._dht != null && (options == null || options.signal == null || !options.signal.aborted)) {
          let newAddress = await this._dht.peerRouting.findPeer(relay.id)

          try {
            relayConnection = await this._dialer.connectToPeer(newAddress, { signal: options?.signal })
          } catch (err) {
            log(`Dialling potential relay ${relay.id.toB58String()} after querying DHT failed. Error was ${err}`)
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

    return relayConnection
  }

  private async performHandshake(relayConnection: Connection, relay: PeerId, destination: PeerId): Promise<Stream> {
    let shaker: any
    try {
      shaker = handshake((await relayConnection.newStream([RELAY_REGISTER])).stream)
    } catch (err) {
      throw Error(`failed to establish stream with ${relay.toB58String()}. Error was: ${err}`)
    }

    shaker.write(destination.pubKey.marshal())

    let answer: Buffer | undefined
    try {
      answer = (await shaker.read())?.slice()
    } catch (err) {
      throw Error(`Error while reading answer. Error was ${err}`)
    }

    shaker.rest()

    if (answer == null || !u8aEquals(answer, OK)) {
      throw Error(
        `Could not establish relayed connection to ${chalk.blue(
          destination.toB58String()
        )} over relay ${relay.toB58String()}. Answer was: <${new TextDecoder().decode(answer)}>`
      )
    }

    return shaker.stream
  }

  private async handleRelay({ stream, connection }: Handler) {
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
      toCounterparty.source as any,
      toSender.sink
    )

    this.updateContext(
      counterparty.toB58String(),
      this._peerInfo.id.toB58String(),
      toSender.source as any,
      toCounterparty.sink
    )
  }

  private async establishForwarding(counterparty: PeerId) {
    let timeout: any

    let cParty = new PeerInfo(counterparty)

    let newConn = this._registrar.getConnection(cParty)

    if (!newConn) {
      const abort = new AbortController()

      timeout = setTimeout(() => abort.abort(), RELAY_CIRCUIT_TIMEOUT)

      try {
        newConn = await this._dialer.connectToPeer(cParty, { signal: abort.signal })
      } catch (err) {
        if (this._dht != null && !abort.signal.aborted) {
          try {
            cParty = await this._dht.peerRouting.findPeer(cParty.id)

            newConn = await this._dialer.connectToPeer(cParty, { signal: abort.signal })
          } catch (err) {
            clearTimeout(timeout)

            throw Error(
              `Could not establish forwarding connection to ${counterparty.toB58String()} after querying the DHT. Error was: ${err}`
            )
          }
        }

        clearTimeout(timeout)

        throw Error(`Could not establish forwarding connection to ${counterparty.toB58String()}. Error was: ${err}`)
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
      throw Error(`Error while trying to decode answer. Error was: ${err}`)
    }

    toCounterparty.rest()

    if (answer == null || !u8aEquals(answer, OK)) {
      throw Error(`Could not relay to peer ${counterparty.toB58String()} because we are unable to deliver packets.`)
    }

    return toCounterparty.stream
  }

  private async updateContext(
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
}

export default Relay
