import { u8aEquals, u8aXOR } from '../../u8a'
import { derivePRGParameters } from './blinding'
import { COMPRESSED_PUBLIC_KEY_LENGTH, MAC_LENGTH, SECRET_LENGTH } from './constants'
import { randomFillSync } from 'crypto'
import { PRG } from '../prg'
import { generateFiller } from './filler'
import { createMAC } from './mac'
import type PeerId from 'peer-id'

const END_PREFIX_LENGTH = 1
const END_PREFIX = 0xff

export function createRoutingInfo(
  maxHops: number,
  peerIds: PeerId[],
  secrets: Uint8Array[],
  additionalDataRelayer: Uint8Array[],
  additionalDataLastHop: Uint8Array
) {
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
      extendedHeader.set(peerIds[invIndex + 1].pubKey.marshal())

      extendedHeader.set(mac, COMPRESSED_PUBLIC_KEY_LENGTH)

      extendedHeader.set(additionalDataRelayer[invIndex], COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH)

      u8aXOR(true, extendedHeader, PRG.createPRG(params).digest(0, extendedHeaderLength))
    }

    mac = createMAC(secret, extendedHeader.subarray(0, headerLength))
  }

  return [extendedHeader.slice(0, headerLength), mac]
}

export function forwardTransform(
  secret: Uint8Array,
  header: Uint8Array,
  mac: Uint8Array,
  maxHops: number,
  additionalDataRelayerLength: number,
  additionalDataLastHopLength: number
): undefined | [header: Uint8Array, mac: Uint8Array, nextNode: Uint8Array, additionalInfo: Uint8Array] {
  if (secret.length != SECRET_LENGTH || mac.length != MAC_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  const routingInfoLength = additionalDataLastHopLength + COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH
  const lastHopLength = additionalDataLastHopLength + END_PREFIX_LENGTH

  const headerLength = lastHopLength + (maxHops - 1) * routingInfoLength

  if (!u8aEquals(createMAC(secret, header), mac)) {
    throw Error(`General error.`)
  }

  let nextMac: Uint8Array
  let nextHop: Uint8Array
  let additionalInfo: Uint8Array

  const params = derivePRGParameters(secret)

  const prg = PRG.createPRG(params)

  u8aXOR(true, header, prg.digest(0, headerLength))

  if (header[0] == END_PREFIX) {
    return undefined
  }

  if (typeof Buffer !== 'undefined' && Buffer.isBuffer(header)) {
    nextHop = header.subarray(0, COMPRESSED_PUBLIC_KEY_LENGTH)
    nextMac = header.subarray(COMPRESSED_PUBLIC_KEY_LENGTH, COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH)
    additionalInfo = header.subarray(
      COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH,
      COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH + additionalDataRelayerLength
    )
  } else {
    nextHop = header.slice(0, COMPRESSED_PUBLIC_KEY_LENGTH)
    nextMac = header.slice(COMPRESSED_PUBLIC_KEY_LENGTH, COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH)
    additionalInfo = header.slice(
      COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH,
      COMPRESSED_PUBLIC_KEY_LENGTH + MAC_LENGTH + additionalDataRelayerLength
    )
  }

  header.copyWithin(0, routingInfoLength)

  header.set(prg.digest(headerLength, headerLength + routingInfoLength), headerLength - routingInfoLength)

  return [header.subarray(0, headerLength), nextMac, nextHop, additionalInfo]
}
