import type { HoprConnectOptions, Stream, StreamType } from '../types.js'
import { handshake, type Handshake } from 'it-handshake'
import type { PeerId } from '@libp2p/interface-peer-id'
import { unmarshalPublicKey } from '@libp2p/crypto/keys'

import chalk from 'chalk'
import { dial, DialStatus, pubKeyToPeerId, safeCloseConnection } from '@hoprnet/hopr-utils'

import type { RelayState } from '../../lib/connect_relay.js'

import debug from 'debug'
import { DELIVERY_PROTOCOLS } from '../constants.js'
import { Components } from '@libp2p/interfaces/components'

export enum RelayHandshakeMessage {
  OK,
  FAIL,
  FAIL_COULD_NOT_REACH_COUNTERPARTY,
  FAIL_COULD_NOT_IDENTIFY_PEER,
  FAIL_INVALID_PUBLIC_KEY,
  FAIL_LOOPBACKS_ARE_NOT_ALLOWED,
  FAIL_RELAY_FULL
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
    case RelayHandshakeMessage.FAIL_RELAY_FULL:
      return 'FAIL_RELAY_FULL'
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

/**
 * Write a handshake message and then pass the stream
 */
function shakerWrite(shaker: Handshake, msg: RelayHandshakeMessage) {
  try {
    shaker.write(Uint8Array.of(msg))
    shaker.rest()
  } catch (err) {
    log(`Error when writing to the shaker ${err}`)
  }
}
/**
 * Tries to establish a relayed connection to the given destination
 * @param relay relay to use
 * @param destination destination to connect to trough relay
 * @returns a relayed connection to `destination`
 */
export async function initiateRelayHandshake(stream: Stream, relay: PeerId, destination: PeerId): Promise<Response> {
  const shaker = handshake(stream)

  shaker.write(unmarshalPublicKey(destination.publicKey as Uint8Array).marshal())

  let chunk: StreamType | undefined
  try {
    chunk = await shaker.read()
  } catch (err: any) {
    error(`Error while reading answer from ${chalk.green(relay.toString())}.`, err.message)
  }

  if (chunk == null || chunk.length == 0) {
    verbose(`Received empty message. Discarding`)
    shaker.rest()
    return {
      success: false,
      code: 'FAIL'
    }
  }

  const answer = chunk.subarray(0, 1)[0]

  shaker.rest()

  // Anything can happen
  switch (answer as RelayHandshakeMessage) {
    case RelayHandshakeMessage.OK:
      log(
        `Successfully established outbound relayed connection with ${chalk.green(
          destination.toString()
        )} over relay ${chalk.green(relay.toString())}`
      )
      return {
        success: true,
        stream: shaker.stream
      }
    default:
      error(
        `Could not establish relayed connection to ${chalk.green(destination.toString())} over relay ${chalk.green(
          relay.toString()
        )}. Answer was: <${chalk.yellow(handshakeMessageToString(answer))}>`
      )

      return {
        success: false,
        code: 'FAIL'
      }
  }
}

/**
 * Negotiates between initiator and destination whether they can establish
 * a relayed connection.
 * @param source peerId of the initiator
 * @param components libp2p instance components
 * @param state.exists to check if relay state exists
 * @param state.isActive to check if existing relay state can be used
 * @param state.updateExisting to update existing connection with new stream if not active
 * @param state.createNew to establish a whole-new instance
 */
export async function negotiateRelayHandshake(
  stream: Stream,
  source: PeerId,
  components: Components,
  state: RelayState,
  options: HoprConnectOptions
): Promise<void> {
  log(`handling relay request`)
  const shaker = handshake(stream)

  let chunk: StreamType | undefined

  try {
    chunk = await shaker.read()
  } catch (err) {
    error(err)
  }

  if (chunk == null || chunk.length == 0) {
    error(
      `Received empty message from peer ${chalk.yellow(source)}. Ending stream because unable to identify counterparty`
    )
    shakerWrite(shaker, RelayHandshakeMessage.FAIL_INVALID_PUBLIC_KEY)
    return
  }

  let destination: PeerId | undefined

  try {
    destination = pubKeyToPeerId(chunk.subarray())
  } catch (err) {
    error(err)
  }

  if (destination == null) {
    error(`Cannot decode public key of destination.`)
    shakerWrite(shaker, RelayHandshakeMessage.FAIL_INVALID_PUBLIC_KEY)
    return
  }

  log(`counterparty identified as ${destination.toString()}`)

  if (source.equals(destination)) {
    error(`Peer ${source.toString()} is trying to loopback to itself. Dropping connection.`)
    shakerWrite(shaker, RelayHandshakeMessage.FAIL_LOOPBACKS_ARE_NOT_ALLOWED)
    return
  }

  const relayedConnectionExists = state.exists(source, destination)
  log(`checked relay entry existence ${source.toString()} ${destination.toString()}: ${relayedConnectionExists}`)

  if (relayedConnectionExists) {
    // Relayed connection could exist but connection is dead
    const connectionIsActive = await state.isActive(source, destination)

    if (connectionIsActive) {
      shakerWrite(shaker, RelayHandshakeMessage.OK)

      // Relayed connection could have been closed meanwhile
      if (
        state.updateExisting(
          source,
          destination,
          // Some libp2p modules produces Buffer streams but
          // WASM requires Uint8Array streams
          {
            source: (async function* () {
              for await (const maybeBuf of shaker.stream.source) {
                if (Buffer.isBuffer(maybeBuf)) {
                  yield new Uint8Array(maybeBuf.buffer, maybeBuf.byteOffset, maybeBuf.length)
                } else {
                  yield maybeBuf
                }
              }
            })(),
            sink: shaker.stream.sink
          }
        )
      ) {
        // Updated connection, so everything done
        return
      }
    } else {
      state.remove(source, destination)
      log(`deleted inactive relay entry: ${source.toString()} ${destination.toString()}`)
    }
  }

  const result = await dial(
    components,
    destination,
    DELIVERY_PROTOCOLS(options.environment, options.supportedEnvironments),
    false,
    true
  )

  // Anything can happen while attempting to connect
  if (result.status != DialStatus.SUCCESS) {
    error(
      `Failed to create circuit from ${source.toString()} to ${destination.toString()} because destination is not reachable`
    )
    shakerWrite(shaker, RelayHandshakeMessage.FAIL_COULD_NOT_REACH_COUNTERPARTY)
    return
  }

  const destinationShaker = handshake(result.resp.stream)

  let errThrown = false
  try {
    destinationShaker.write(unmarshalPublicKey(source.publicKey as Uint8Array).marshal())
  } catch (err) {
    error(`Error while writing to destination ${destination.toString()}`)
    errThrown = true
  }

  if (errThrown) {
    shakerWrite(shaker, RelayHandshakeMessage.FAIL_COULD_NOT_REACH_COUNTERPARTY)
    destinationShaker.rest()
    await safeCloseConnection(result.resp.conn, components, (err) => {
      error(`Error while closing connection to destination ${destination?.toString()}.`, err)
    })
    return
  }

  let destinationChunk: StreamType | undefined

  try {
    destinationChunk = await destinationShaker.read()
  } catch (err) {
    error(err)
  }

  if (destinationChunk == null || destinationChunk.length == 0) {
    shakerWrite(shaker, RelayHandshakeMessage.FAIL_COULD_NOT_REACH_COUNTERPARTY)

    destinationShaker.rest()
    await safeCloseConnection(result.resp.conn, components, (err) => {
      error(`Error while closing connection to destination ${destination?.toString()}.`, err)
    })
    return
  }

  const destinationAnswer = destinationChunk.subarray(0, 1)[0]

  switch (destinationAnswer as RelayHandshakeMessage) {
    case RelayHandshakeMessage.OK:
      shakerWrite(shaker, RelayHandshakeMessage.OK)
      destinationShaker.rest()

      state.createNew(
        source,
        destination,
        // Some libp2p modules produces Buffer streams but
        // WASM requires Uint8Array streams
        {
          source: (async function* () {
            for await (const maybeBuf of shaker.stream.source) {
              if (Buffer.isBuffer(maybeBuf)) {
                yield new Uint8Array(maybeBuf.buffer, maybeBuf.byteOffset, maybeBuf.length)
              } else {
                yield maybeBuf
              }
            }
          })(),
          sink: shaker.stream.sink
        },
        // Some libp2p modules produces Buffer streams but
        // WASM requires Uint8Array streams
        {
          source: (async function* () {
            for await (const maybeBuf of destinationShaker.stream.source) {
              if (Buffer.isBuffer(maybeBuf)) {
                yield new Uint8Array(maybeBuf.buffer, maybeBuf.byteOffset, maybeBuf.length)
              } else {
                yield maybeBuf
              }
            }
          })(),
          sink: destinationShaker.stream.sink
        }
      )
      break
    default:
      log(`Counterparty replied with ${destinationAnswer} but expected ${RelayHandshakeMessage.OK}`)
      shakerWrite(shaker, RelayHandshakeMessage.FAIL_COULD_NOT_REACH_COUNTERPARTY)

      destinationShaker.rest()
      return
  }
}

export async function abortRelayHandshake(stream: Stream, reason: RelayHandshakeMessage) {
  const shaker = handshake(stream)

  try {
    shaker.write(Uint8Array.of(reason))
    shaker.rest()
  } catch (err) {
    error(`Error while writing to stream`, err)
  }
}

/**
 * Handles an incoming request from a relay
 * @param source peerId of the relay
 * @returns a duplex stream with the initiator
 */
export async function handleRelayHandshake(stream: Stream, source: PeerId): Promise<HandleResponse> {
  const shaker = handshake(stream)

  let chunk: StreamType | undefined
  try {
    chunk = await shaker.read()
  } catch (err) {
    error(err)
  }

  // Anything can happen
  if (chunk == null || chunk.length == 0) {
    error(`Received empty message. Ignoring request`)
    shaker.write(Uint8Array.of(RelayHandshakeMessage.FAIL))
    shaker.rest()

    return {
      success: false,
      code: 'FAIL'
    }
  }

  let initiator: PeerId | undefined

  try {
    initiator = pubKeyToPeerId(chunk.subarray())
  } catch (err: any) {
    error(`Could not decode sender peerId.`, err.message)
  }

  if (initiator == null) {
    shaker.write(Uint8Array.of(RelayHandshakeMessage.FAIL))
    shaker.rest()

    return {
      success: false,
      code: 'FAIL'
    }
  }

  log(
    `Successfully established inbound relayed connection from initiator ${chalk.green(
      initiator.toString()
    )} over relay ${chalk.green(source.toString())}.`
  )

  shaker.write(Uint8Array.of(RelayHandshakeMessage.OK))
  shaker.rest()

  return {
    success: true,
    stream: shaker.stream,
    counterparty: initiator
  }
}
