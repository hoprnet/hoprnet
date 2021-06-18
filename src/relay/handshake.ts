/// <reference path="../@types/it-handshake.ts" />
/// <reference path="../@types/libp2p.ts" />
/// <reference path="../@types/libp2p-interfaces.ts" />

import { Stream } from 'libp2p'
import { BLInterface } from 'bl'
import handshake, { Handshake } from 'it-handshake'
import PeerId from 'peer-id'

import { blue, yellow } from 'chalk'

import { pubKeyToPeerId } from '@hoprnet/hopr-utils'

import { RelayState } from './state'

import debug from 'debug'

enum RelayHandshakeMessage {
  OK,
  FAIL,
  FAIL_COULD_NOT_REACH_COUNTERPARTY,
  FAIL_COULD_NOT_IDENTIFY_PEER,
  FAIL_INVALID_PUBLIC_KEY,
  FAIL_LOOPBACKS_ARE_NOT_ALLOWED
}

function handshakeMessageToString(handshakeMessage: RelayHandshakeMessage): string {
  switch (handshakeMessage) {
    case RelayHandshakeMessage.OK:
      return 'OK'
    case RelayHandshakeMessage.FAIL_COULD_NOT_REACH_COUNTERPARTY:
      return 'FAIL_COULD_NOT_REACH_COUNTERPARTY'
    case RelayHandshakeMessage.FAIL_COULD_NOT_IDENTIFY_PEER:
      return 'FAIL_COULD_NOT_IDENTIFY_PEER'
    case RelayHandshakeMessage.FAIL_INVALID_PUBLIC_KEY:
      return 'FAIL_INVALID_PUBLIC_KEY'
    case RelayHandshakeMessage.FAIL_LOOPBACKS_ARE_NOT_ALLOWED:
      return 'FAIL_LOOPBACKS_ARE_NOT_ALLOWED'
    default:
      throw Error(`Invalid state. Got ${handshakeMessage}`)
  }
}

const DEBUG_PREFIX = 'hopr-connect:relay:handshake'

const log = debug(DEBUG_PREFIX)
const error = debug(DEBUG_PREFIX.concat(':error'))
const verbose = debug(DEBUG_PREFIX.concat(':verbose'))

type Response =
  | {
      success: false
      code: 'FAIL'
    }
  | {
      success: true
      stream: Stream
    }

type HandleResponse =
  | {
      success: false
      code: 'FAIL'
    }
  | {
      success: true
      stream: Stream
      counterparty: PeerId
    }

type StreamResult = Buffer | Uint8Array | BLInterface

class RelayHandshake {
  private shaker: Handshake<StreamResult>

  constructor(stream: Stream) {
    this.shaker = handshake<StreamResult>(stream)
  }

  async initiate(relay: PeerId, destination: PeerId): Promise<Response> {
    this.shaker.write(destination.pubKey.marshal())

    let chunk: StreamResult | undefined
    try {
      chunk = await this.shaker.read()
    } catch (err) {
      error(`Error while reading answer ${blue(relay.toB58String())}. ${err.message}`)
    }

    if (chunk == null) {
      verbose(`Received empty message. Discarding`)
      this.shaker.rest()
      return {
        success: false,
        code: 'FAIL'
      }
    }

    const answer = chunk.slice(0, 1)[0]

    this.shaker.rest()

    if (answer == RelayHandshakeMessage.OK) {
      verbose(`Relay handshake with ${blue(destination.toB58String())} successful`)
      return {
        success: true,
        stream: this.shaker.stream
      }
    }

    error(
      `Could not establish relayed connection to ${blue(
        destination.toB58String()
      )} over relay ${relay.toB58String()}. Answer was: <${yellow(handshakeMessageToString(answer))}>`
    )

    return {
      success: false,
      code: 'FAIL'
    }
  }

  async negotiate(
    source: PeerId,
    getStreamToCounterparty: (peerId: PeerId) => Promise<Stream | undefined>,
    exists: InstanceType<typeof RelayState>['exists'],
    isActive: InstanceType<typeof RelayState>['isActive'],
    updateExisting: InstanceType<typeof RelayState>['updateExisting'],
    createNew: InstanceType<typeof RelayState>['createNew']
  ): Promise<void> {
    log(`handle relay request`)

    let chunk: StreamResult | undefined

    try {
      chunk = await this.shaker.read()
    } catch (err) {
      error(err)
    }

    if (chunk == null) {
      this.shaker.write(Uint8Array.of(RelayHandshakeMessage.FAIL_INVALID_PUBLIC_KEY))
      this.shaker.rest()
      error(`Received empty message from peer ${yellow(source)}. Ending stream because unable to identify counterparty`)
      return
    }

    let destination: PeerId | undefined

    try {
      destination = PeerId.createFromBytes(chunk.slice())
    } catch (err) {
      error(err)
    }

    if (destination == null) {
      error(`Cannot decode public key of destination.`)
      this.shaker.write(Uint8Array.of(RelayHandshakeMessage.FAIL_INVALID_PUBLIC_KEY))
      this.shaker.rest()
      return
    }

    log(`counterparty identified as ${destination.toB58String()}`)

    if (source.equals(destination)) {
      error(`Peer ${source.toB58String()} is trying to loopback to itself. Dropping connection.`)
      this.shaker.write(Uint8Array.of(RelayHandshakeMessage.FAIL_LOOPBACKS_ARE_NOT_ALLOWED))
      this.shaker.rest()
      return
    }

    const relayedConnectionExists = exists(source, destination)

    if (relayedConnectionExists) {
      const connectionIsActive = await isActive(source, destination)

      if (connectionIsActive) {
        this.shaker.write(Uint8Array.of(RelayHandshakeMessage.OK))
        this.shaker.rest()

        updateExisting(source, destination, this.shaker.stream)

        return
      }
    }

    let toDestination: Stream | undefined
    try {
      toDestination = await getStreamToCounterparty(destination)
    } catch (err) {
      error(err)
    }

    if (toDestination == null) {
      error(`Cannot establish a relayed connection from ${source.toB58String()} to ${destination.toB58String()}`)
      this.shaker.write(Uint8Array.of(RelayHandshakeMessage.FAIL_COULD_NOT_REACH_COUNTERPARTY))
      this.shaker.rest()
      return
    }

    const destinationShaker = handshake<StreamResult>(toDestination)

    destinationShaker.write(source.pubKey.marshal())

    let destinationChunk: StreamResult | undefined

    try {
      destinationChunk = await destinationShaker.read()
    } catch (err) {
      error(err)
    }

    if (destinationChunk == null) {
      this.shaker.write(Uint8Array.of(RelayHandshakeMessage.FAIL_COULD_NOT_REACH_COUNTERPARTY))
      this.shaker.rest()

      destinationShaker.rest()
      return
    }

    const destinationAnswer = destinationChunk.slice(0, 1)[0]

    if (destinationAnswer != RelayHandshakeMessage.OK) {
      this.shaker.write(Uint8Array.of(RelayHandshakeMessage.FAIL_COULD_NOT_REACH_COUNTERPARTY))
      this.shaker.rest()

      destinationShaker.rest()
      return
    }

    destinationShaker.rest()

    createNew(source, destination, this.shaker.stream, destinationShaker.stream)
  }

  async handle(): Promise<HandleResponse> {
    let chunk: Uint8Array | BLInterface | undefined
    try {
      chunk = await this.shaker.read()
    } catch (err) {
      error(err)
    }

    if (chunk == null) {
      error(`Received empty message. Ignoring request`)
      this.shaker.write(Uint8Array.of(RelayHandshakeMessage.FAIL))
      this.shaker.rest()

      return {
        success: false,
        code: 'FAIL'
      }
    }

    let initiator: PeerId | undefined

    try {
      initiator = pubKeyToPeerId(chunk.slice())
    } catch (err) {
      error(`Could not decode sender peerId. Error was: ${err.message}`)
    }

    if (initiator == null) {
      this.shaker.write(Uint8Array.of(RelayHandshakeMessage.FAIL))
      this.shaker.rest()

      return {
        success: false,
        code: 'FAIL'
      }
    }

    this.shaker.write(Uint8Array.of(RelayHandshakeMessage.OK))
    this.shaker.rest()

    return {
      success: true,
      stream: this.shaker.stream,
      counterparty: initiator
    }
  }
}

export { RelayHandshake }
