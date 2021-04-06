import { u8aXOR } from '../../u8a'
import { PRG } from '../prg'
import { derivePRGParameters } from './blinding'

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
      header.subarray(routingInfoLength + routingInfoLastHopLength + paddingLength, routingInfoLength + routingInfoLastHopLength + paddingLength + length),
      PRG.createPRG(prgParams).digest(start, headerLength + routingInfoLength)
    )

    length += routingInfoLength
    start -= routingInfoLength
  }
}
