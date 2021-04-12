import { u8aEquals, u8aXOR } from '../../u8a'
import { derivePRGParameters } from './keyDerivation'
import { COMPRESSED_PUBLIC_KEY_LENGTH, MAC_LENGTH, SECRET_LENGTH, END_PREFIX, END_PREFIX_LENGTH } from './constants'
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
  additionalDataRelayer: Uint8Array[],
  additionalDataLastHop: Uint8Array
): { routingInformation: Uint8Array; mac: Uint8Array } {
  const routingInfoLength = additionalDataRelayer[0].length + MAC_LENGTH + COMPRESSED_PUBLIC_KEY_LENGTH
  const lastHopLength = additionalDataLastHop.length + END_PREFIX_LENGTH

  if (
    secrets.some((s) => s.length != SECRET_LENGTH) ||
    additionalDataRelayer.slice(1)?.some((r) => r.length != additionalDataRelayer[0].length) ||
    secrets.length > maxHops
  ) {
    throw Error(`Invalid arguments`)
  }

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

      if (lastHopLength > 0) {
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

      generateFiller(extendedHeader, maxHops, routingInfoLength, lastHopLength, secrets)
    } else {
      extendedHeader.copyWithin(routingInfoLength, 0, headerLength)

      // Add pubkey of next downstream node
      extendedHeader.set(path[invIndex + 1].pubKey.marshal())

      extendedHeader.set(mac, COMPRESSED_PUBLIC_KEY_LENGTH)

      extendedHeader.set(additionalDataRelayer[invIndex], COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH)

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
  _header: Uint8Array,
  mac: Uint8Array,
  maxHops: number,
  additionalDataRelayerLength: number,
  additionalDataLastHopLength: number
): LastNodeOutput | RelayNodeOutput {
  if (secret.length != SECRET_LENGTH || mac.length != MAC_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  const routingInfoLength = additionalDataLastHopLength + COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH
  const lastHopLength = additionalDataLastHopLength + END_PREFIX_LENGTH

  const headerLength = lastHopLength + (maxHops - 1) * routingInfoLength

  let header: Uint8Array

  if (typeof Buffer !== 'undefined' && Buffer.isBuffer(header)) {
    header = Uint8Array.from(_header)
  } else {
    header = _header
  }

  if (!u8aEquals(createMAC(secret, header), mac)) {
    throw Error(`General error.`)
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

  let nextHop = header.slice(0, COMPRESSED_PUBLIC_KEY_LENGTH)
  let nextMac = header.slice(COMPRESSED_PUBLIC_KEY_LENGTH, COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH)
  let additionalInfo = header.slice(
    COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH,
    COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH + additionalDataRelayerLength
  )

  if (!publicKeyVerify(nextHop)) {
    throw Error(`General error.`)
  }

  header.copyWithin(0, routingInfoLength)

  header.set(prg.digest(headerLength, headerLength + routingInfoLength), headerLength - routingInfoLength)

  return { lastNode: false, header: header.subarray(0, headerLength), mac: nextMac, nextNode: nextHop, additionalInfo }
}
