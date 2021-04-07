import type PeerId from 'peer-id'

import { COMPRESSED_PUBLIC_KEY_LENGTH, MAC_LENGTH, END_PREFIX_LENGTH } from './constants'
import { createRoutingInfo, forwardTransform as routingInfoTransform } from './routingInfo'
import { generateKeyShares, forwardTransform as keyShareTransform } from './keyShares'
import { PRP } from '../prp'
import { PAYLOAD_SIZE } from './constants'
import { derivePRPParameters } from './blinding'

function encrypt(text: Uint8Array, secrets: Uint8Array[]): Uint8Array {
  for (let i = 0; i < secrets.length; i++) {
    const prp = PRP.createPRP(derivePRPParameters(secrets[secrets.length - i - 1]))

    prp.permutate(text)
  }

  return text
}

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

  const ciphertext = encrypt(paddedMsg, secrets)

  return Uint8Array.from([...alpha, ...beta, ...gamma, ...ciphertext])
}

export function forwardTransform(
  privKey: PeerId,
  packet: Uint8Array,
  additionalDataRelayerLength: number,
  additionalDataLastHopLength: number,
  maxHops: number
):
  | [done: false, packet: Uint8Array, nextHop: Uint8Array, additionalRelayData: Uint8Array]
  | [done: true, plaintext: Uint8Array] {
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

  let header: [beta: Uint8Array, gamma: Uint8Array, nextNode: Uint8Array, additionalInfo: Uint8Array] | undefined

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
    return [true, decoded[3]]
  } else {
    const forwardPacket = Uint8Array.from([...alpha, ...header[0], ...header[1], ...decoded[3]])

    return [false, forwardPacket, header[2], header[3]]
  }
}
