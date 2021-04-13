import { u8aXOR } from '../../u8a'
import { PRG } from '../prg'
import { derivePRGParameters } from './keyDerivation'

/**
 * Writes the filler bitstring into the header such
 * that the integrity tag can be computed
 * @param maxHops amount of relayers to use
 * @param routingInfoLength length of additional data to
 * put next to the routing information
 * @param routingInfoLastHopLength length of the additional
 * data to put next to the routing information of the last
 * hop
 * @param secrets shared secrets with the creator of the packet
 */
export function generateFiller(
  maxHops: number,
  routingInfoLength: number,
  routingInfoLastHopLength: number,
  secrets: Uint8Array[]
): Uint8Array {
  if (secrets.length < 2) {
    // nothing to do
    return
  }

  const headerLength = routingInfoLastHopLength + (maxHops - 1) * routingInfoLength
  const paddingLength = (maxHops - secrets.length) * routingInfoLength

  const arr = new Uint8Array(headerLength - paddingLength - routingInfoLastHopLength)

  let length = routingInfoLength
  let start = headerLength

  for (let index = 0; index < secrets.length - 1; index++) {
    const prgParams = derivePRGParameters(secrets[index])

    u8aXOR(true, arr.subarray(0, length), PRG.createPRG(prgParams).digest(start, headerLength + routingInfoLength))

    length += routingInfoLength
    start -= routingInfoLength
  }

  return arr
}
