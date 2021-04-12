import type PeerId from 'peer-id'

import { COMPRESSED_PUBLIC_KEY_LENGTH, MAC_LENGTH, END_PREFIX_LENGTH } from './constants'
import { createRoutingInfo, forwardTransform as routingInfoTransform } from './routingInfo'
import { generateKeyShares, forwardTransform as keyShareTransform } from './keyShares'
import { PRP } from '../prp'
import { PAYLOAD_SIZE } from './constants'
import { derivePRPParameters } from './keyDerivation'

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
  msg: Uint8Array,
  path: PeerId[],
  maxHops: number,
  additionalDataRelayer: Uint8Array[],
  additionalDataLastHop: Uint8Array
): Uint8Array {
  if (msg.length > PAYLOAD_SIZE) {
    throw Error(`Invalid arguments. Messages greater than ${PAYLOAD_SIZE} are not yet supported`)
  }

  let paddedMsg: Uint8Array
  if (msg.length < PAYLOAD_SIZE) {
    paddedMsg = Uint8Array.from([...msg, ...new Uint8Array(PAYLOAD_SIZE - msg.length)])
  } else {
    paddedMsg = msg
  }

  const [alpha, secrets] = generateKeyShares(path)

  const [beta, gamma] = createRoutingInfo(maxHops, path, secrets, additionalDataRelayer, additionalDataLastHop)

  const ciphertext = onionEncrypt(paddedMsg, secrets)

  return Uint8Array.from([...alpha, ...beta, ...gamma, ...ciphertext])
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
):
  | [done: false, packet: Uint8Array, nextHop: Uint8Array, additionalRelayData: Uint8Array]
  | [done: true, plaintext: Uint8Array, additionalData: Uint8Array] {
  if (privKey.privKey == null) {
    throw Error(`Invalid arguments`)
  }
  const perHop = COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH + additionalDataRelayerLength
  const lastHop = END_PREFIX_LENGTH + additionalDataLastHopLength

  const headerLength = lastHop + (maxHops - 1) * perHop

  let decoded: [alpha: Uint8Array, beta: Uint8Array, gamma: Uint8Array, ciphertext: Uint8Array]

  if (typeof Buffer !== 'undefined' && Buffer.isBuffer(packet)) {
    decoded = [
      packet.slice(0, COMPRESSED_PUBLIC_KEY_LENGTH),
      packet.slice(COMPRESSED_PUBLIC_KEY_LENGTH, COMPRESSED_PUBLIC_KEY_LENGTH + headerLength),
      packet.slice(
        COMPRESSED_PUBLIC_KEY_LENGTH + headerLength,
        COMPRESSED_PUBLIC_KEY_LENGTH + headerLength + MAC_LENGTH
      ),
      packet.slice(
        COMPRESSED_PUBLIC_KEY_LENGTH + headerLength + MAC_LENGTH,
        COMPRESSED_PUBLIC_KEY_LENGTH + headerLength + MAC_LENGTH + PAYLOAD_SIZE
      )
    ]
  } else {
    decoded = [
      packet.subarray(0, COMPRESSED_PUBLIC_KEY_LENGTH),
      packet.subarray(COMPRESSED_PUBLIC_KEY_LENGTH, COMPRESSED_PUBLIC_KEY_LENGTH + headerLength),
      packet.subarray(
        COMPRESSED_PUBLIC_KEY_LENGTH + headerLength,
        COMPRESSED_PUBLIC_KEY_LENGTH + headerLength + MAC_LENGTH
      ),
      packet.subarray(
        COMPRESSED_PUBLIC_KEY_LENGTH + headerLength + MAC_LENGTH,
        COMPRESSED_PUBLIC_KEY_LENGTH + headerLength + MAC_LENGTH + PAYLOAD_SIZE
      )
    ]
  }

  const [alpha, secret] = keyShareTransform(decoded[0], privKey)

  let header:
    | [beta: Uint8Array, gamma: Uint8Array, nextNode: Uint8Array, additionalInfo: Uint8Array]
    | [additionalData: Uint8Array]

  header = routingInfoTransform(
    secret,
    decoded[1],
    decoded[2],
    maxHops,
    additionalDataRelayerLength,
    additionalDataLastHopLength
  )

  const prp = PRP.createPRP(derivePRPParameters(secret))

  prp.inverse(decoded[3])

  if (header == undefined) {
    return [true, decoded[3], header[0]]
  } else {
    const forwardPacket = Uint8Array.from([...alpha, ...header[0], ...header[1], ...decoded[3]])

    return [false, forwardPacket, header[2], header[3]]
  }
}
