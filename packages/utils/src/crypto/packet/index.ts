import type PeerId from 'peer-id'

import { COMPRESSED_PUBLIC_KEY_LENGTH, MAC_LENGTH, END_PREFIX_LENGTH } from './constants'
import { createRoutingInfo, forwardTransform as routingInfoTransform } from './routingInfo'
import { generateKeyShares, forwardTransform as keyShareTransform } from './keyShares'
import { PRP } from '../prp'
import { PAYLOAD_SIZE } from './constants'
import { derivePRPParameters } from './keyDerivation'
import { addPadding, removePadding } from './padding'

/**
 * Encrypts the plaintext in the reverse order of the path
 * @param text plaintext of the payload
 * @param secrets shared secrets with the creator of the packet
 * @returns
 */
function onionEncrypt(text: Uint8Array, secrets: Uint8Array[]): Uint8Array {
  for (let i = 0; i < secrets.length; i++) {
    const prp = PRP.createPRP(derivePRPParameters(secrets[secrets.length - i - 1]))

    prp.permutate(text)
  }

  return text
}

export function getHeaderLength(
  maxHops: number,
  additionalDataRelayerLength: number,
  additionalDataLastHopLength: number
) {
  const perHop = COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH + additionalDataRelayerLength
  const lastHop = END_PREFIX_LENGTH + additionalDataLastHopLength

  return lastHop + (maxHops - 1) * perHop
}

export function getPacketLength(
  maxHops: number,
  additionalDataRelayerLength: number,
  additionalDataLastHopLength: number
) {
  return getHeaderLength(maxHops, additionalDataRelayerLength, additionalDataLastHopLength) + PAYLOAD_SIZE
}

export { generateKeyShares }

/**
 * Creates a mixnet packet
 * @param msg payload to send
 * @param path nodes to use for relaying, including the final
 * destination
 * @param maxHops maximal number of hops to use
 * @param additionalDataRelayer additional data to put next to
 * each node's routing information
 * @param additionalDataLastHop additional data for the final destination
 * @returns the packet as u8a
 */
export function createPacket(
  secrets: Uint8Array[],
  alpha: Uint8Array,
  msg: Uint8Array,
  path: PeerId[],
  maxHops: number,
  additionalDataRelayer: Uint8Array[],
  additionalDataLastHop: Uint8Array
): Uint8Array {
  if (msg.length > PAYLOAD_SIZE) {
    throw Error(`Invalid arguments. Messages greater than ${PAYLOAD_SIZE} are not yet supported`)
  }

  const paddedMsg = addPadding(msg)

  const { routingInformation, mac } = createRoutingInfo(
    maxHops,
    path,
    secrets,
    additionalDataRelayer,
    additionalDataLastHop
  )

  const ciphertext = onionEncrypt(paddedMsg, secrets)

  return Uint8Array.from([...alpha, ...routingInformation, ...mac, ...ciphertext])
}

type LastNodeOutput = {
  lastNode: true
  plaintext: Uint8Array
  additionalData: Uint8Array
}
type RelayNodeOutput = {
  lastNode: false
  packet: Uint8Array
  nextHop: Uint8Array
  additionalRelayData: Uint8Array
}

/**
 * Applies the transformation to the header to forward
 * it to the next node or deliver it to the user
 * @param privKey private key of the relayer
 * @param packet incoming packet as u8a
 * @param additionalDataRelayerLength length of the additional
 * data next the routing information of each hop
 * @param additionalDataLastHopLength lenght of the additional
 * data for the last hop
 * @param maxHops maximal amount of hops
 * @returns whether the packet is valid, if yes returns
 * the transformed packet, the public key of the next hop
 * and the data next to the routing information. If current
 * hop is the final recipient, it returns the plaintext
 */
export function forwardTransform(
  privKey: PeerId,
  packet: Uint8Array,
  additionalDataRelayerLength: number,
  additionalDataLastHopLength: number,
  maxHops: number
): LastNodeOutput | RelayNodeOutput {
  if (privKey.privKey == null) {
    throw Error(`Invalid arguments`)
  }

  const headerLength = getHeaderLength(maxHops, additionalDataRelayerLength, additionalDataLastHopLength)

  let decoded = decodePacket(packet, headerLength)

  const { alpha, secret } = keyShareTransform(decoded.alpha, privKey)

  let header = routingInfoTransform(
    secret,
    decoded.routingInformation,
    decoded.mac,
    maxHops,
    additionalDataRelayerLength,
    additionalDataLastHopLength
  )

  const prp = PRP.createPRP(derivePRPParameters(secret))

  prp.inverse(decoded.ciphertext)

  if (header.lastNode == true) {
    return { lastNode: true, plaintext: removePadding(decoded.ciphertext), additionalData: header.additionalData }
  } else {
    const packet = Uint8Array.from([...alpha, ...header.header, ...header.mac, ...decoded.ciphertext])

    return { lastNode: false, packet, nextHop: header.nextNode, additionalRelayData: header.additionalInfo }
  }
}

/**
 * Takes a packet as bytestring and returns a decoded output
 * @param _packet bytearray containing the packet
 * @param headerLength length of the header
 * @returns decoded output
 */
function decodePacket(
  _packet: Uint8Array | Buffer,
  headerLength: number
): { alpha: Uint8Array; routingInformation: Uint8Array; mac: Uint8Array; ciphertext: Uint8Array } {
  let packet: Uint8Array

  if (typeof Buffer !== 'undefined' && Buffer.isBuffer(_packet)) {
    packet = Uint8Array.from(_packet)
  } else {
    packet = _packet
  }

  return {
    alpha: packet.subarray(0, COMPRESSED_PUBLIC_KEY_LENGTH),
    routingInformation: packet.subarray(COMPRESSED_PUBLIC_KEY_LENGTH, COMPRESSED_PUBLIC_KEY_LENGTH + headerLength),
    mac: packet.subarray(
      COMPRESSED_PUBLIC_KEY_LENGTH + headerLength,
      COMPRESSED_PUBLIC_KEY_LENGTH + headerLength + MAC_LENGTH
    ),
    ciphertext: packet.subarray(
      COMPRESSED_PUBLIC_KEY_LENGTH + headerLength + MAC_LENGTH,
      COMPRESSED_PUBLIC_KEY_LENGTH + headerLength + MAC_LENGTH + PAYLOAD_SIZE
    )
  }
}
