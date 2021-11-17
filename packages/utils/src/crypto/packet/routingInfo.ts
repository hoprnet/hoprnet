import { u8aEquals, u8aXOR } from '../../u8a'
import { derivePRGParameters } from './keyDerivation'
import {MAC_LENGTH, END_PREFIX, END_PREFIX_LENGTH, PRESECRET_LENGTH} from './constants'
import { SECP256K1_CONSTANTS } from '../constants'
import { randomFillSync } from 'crypto'
import { PRG } from '../prg'
import { generateFiller } from './filler'
import { createMAC } from './mac'
import { publicKeyVerify } from 'secp256k1'
import type PeerId from 'peer-id'

/**
 * Creates the routing information of the mixnet packet
 * @param maxHops maximal number of hops
 * @param path IDs of the nodes along the path
 * @param secrets shared secrets with the nodes along the path
 * @param additionalDataRelayer additional data for each relayer
 * @param additionalDataLastHop additional data for the final recipient
 * @returns bytestring containing the routing information, and the
 * authentication tag
 */
export function createRoutingInfo(
  maxHops: number,
  path: PeerId[],
  secrets: Uint8Array[],
  additionalDataRelayerLength: number,
  additionalDataRelayer: Uint8Array[],
  additionalDataLastHop?: Uint8Array
): { routingInformation: Uint8Array; mac: Uint8Array } {
  if (
    secrets.some((s) => s.length != PRESECRET_LENGTH) ||
    additionalDataRelayer.some((r) => r.length != additionalDataRelayerLength) ||
    secrets.length > maxHops
  ) {
    throw Error(`Invalid arguments`)
  }

  const routingInfoLength = additionalDataRelayerLength + MAC_LENGTH + SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH
  const lastHopLength = (additionalDataLastHop?.length ?? 0) + END_PREFIX_LENGTH

  const headerLength = lastHopLength + (maxHops - 1) * routingInfoLength
  const extendedHeaderLength = lastHopLength + maxHops * routingInfoLength

  const extendedHeader = new Uint8Array(extendedHeaderLength)

  let mac: Uint8Array

  for (let index = 0; index < secrets.length; index++) {
    const invIndex = secrets.length - index - 1
    const secret = secrets[invIndex]
    const params = derivePRGParameters(secret)

    if (index == 0) {
      extendedHeader[0] = END_PREFIX

      if (additionalDataLastHop?.length > 0) {
        extendedHeader.set(additionalDataLastHop, END_PREFIX_LENGTH)
      }

      const paddingLength = (maxHops - secrets.length) * routingInfoLength

      if (paddingLength > 0) {
        randomFillSync(extendedHeader, lastHopLength, paddingLength)
      }

      u8aXOR(
        true,
        extendedHeader.subarray(0, lastHopLength + paddingLength),
        PRG.createPRG(params).digest(0, lastHopLength + paddingLength)
      )

      if (secrets.length > 1) {
        extendedHeader.set(
          generateFiller(maxHops, routingInfoLength, lastHopLength, secrets),
          lastHopLength + paddingLength
        )
      }
    } else {
      extendedHeader.copyWithin(routingInfoLength, 0, headerLength)

      // Add pubkey of next downstream node
      extendedHeader.set(path[invIndex + 1].pubKey.marshal())

      extendedHeader.set(mac, SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH)

      extendedHeader.set(additionalDataRelayer[invIndex], SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH)

      u8aXOR(true, extendedHeader, PRG.createPRG(params).digest(0, extendedHeaderLength))
    }

    mac = createMAC(secret, extendedHeader.subarray(0, headerLength))
  }

  return { routingInformation: extendedHeader.slice(0, headerLength), mac }
}

export type LastNodeOutput = { lastNode: true; additionalData: Uint8Array }
export type RelayNodeOutput = {
  lastNode: false
  header: Uint8Array
  mac: Uint8Array
  nextNode: Uint8Array
  additionalInfo: Uint8Array
}

/**
 * Applies the forward transformation to the header
 * @param secret shared secret with the creator of the packet
 * @param header u8a containing the header
 * @param mac current mac
 * @param maxHops maximal number of hops
 * @param additionalDataRelayerLength length of the additional data for each relayer
 * @param additionalDataLastHopLength length of the additional data for the final
 * destination
 * @returns if the packet is destined for this node, returns the additional data
 * for the final destination, otherwise it returns the transformed header, the
 * next authentication tag, the public key of the next node, and the additional data
 * for the relayer
 */
export function forwardTransform(
  secret: Uint8Array,
  preHeader: Uint8Array | Buffer,
  mac: Uint8Array,
  maxHops: number,
  additionalDataRelayerLength: number,
  additionalDataLastHopLength: number
): LastNodeOutput | RelayNodeOutput {
  const routingInfoLength = additionalDataRelayerLength + SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH
  const lastHopLength = additionalDataLastHopLength + END_PREFIX_LENGTH

  const headerLength = lastHopLength + (maxHops - 1) * routingInfoLength

  if (secret.length != PRESECRET_LENGTH || mac.length != MAC_LENGTH || preHeader.length != headerLength) {
    throw Error(`Invalid arguments`)
  }

  let header: Uint8Array

  if (typeof Buffer !== 'undefined' && Buffer.isBuffer(preHeader)) {
    header = Uint8Array.from(preHeader)
  } else {
    header = preHeader
  }

  if (!u8aEquals(createMAC(secret, header), mac)) {
    throw Error(`Header integrity check failed.`)
  }

  const params = derivePRGParameters(secret)

  const prg = PRG.createPRG(params)

  u8aXOR(true, header, prg.digest(0, headerLength))

  if (header[0] == END_PREFIX) {
    return {
      lastNode: true,
      additionalData: header.slice(END_PREFIX_LENGTH, END_PREFIX_LENGTH + additionalDataLastHopLength)
    }
  }

  let nextHop = header.slice(0, SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH)
  let nextMac = header.slice(
    SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH,
    SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH
  )
  let additionalInfo = header.slice(
    SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH,
    SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH + additionalDataRelayerLength
  )

  if (!publicKeyVerify(nextHop)) {
    throw Error(`Blinding of the group element failed. Result is not a valid curve point.`)
  }

  header.copyWithin(0, routingInfoLength)

  header.set(prg.digest(headerLength, headerLength + routingInfoLength), headerLength - routingInfoLength)

  return { lastNode: false, header: header.subarray(0, headerLength), mac: nextMac, nextNode: nextHop, additionalInfo }
}
