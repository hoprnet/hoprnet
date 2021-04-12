import { u8aXOR } from '../../u8a'
import { PRG } from '../prg'
import { derivePRGParameters } from './keyDerivation'

/**
 * Writes the filler bitstring into the header such
 * that the integrity tag can be computed
 * @param header header u8a to write the filler
 * @param maxHops amount of relayers to use
 * @param routingInfoLength length of additional data to
 * put next to the routing information
 * @param routingInfoLastHopLength length of the additional
 * data to put next to the routing information of the last
 * hop 
 * @param secrets shared secrets with the creator of the packet 
 */
export function generateFiller(
  header: Uint8Array,
  maxHops: number,
  routingInfoLength: number,
  routingInfoLastHopLength: number,
  secrets: Uint8Array[]
): void {
  if (secrets.length < 2) {
    // nothing to do
    return
  }

  const headerLength = routingInfoLastHopLength + (maxHops - 1) * routingInfoLength
  const paddingLength = (maxHops - secrets.length) * routingInfoLength

  let length = routingInfoLength
  let start = headerLength

  for (let index = 0; index < secrets.length - 1; index++) {
    const prgParams = derivePRGParameters(secrets[index])

    u8aXOR(
      true,
      header.subarray(routingInfoLastHopLength + paddingLength, routingInfoLastHopLength + paddingLength + length),
      PRG.createPRG(prgParams).digest(start, headerLength + routingInfoLength)
    )

    length += routingInfoLength
    start -= routingInfoLength
  }
}
