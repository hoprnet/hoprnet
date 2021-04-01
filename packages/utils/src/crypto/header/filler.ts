import { u8aXOR } from '../../u8a'
import { PRG } from '../prg'
import { derivePRGParameters } from './blinding'

export function generateFiller(
  header: Uint8Array,
  routingInfoLength: number,
  routingInfoLastHopLength: number,
  secrets: Uint8Array[]
): void {
  if (secrets.length < 2) {
    // nothing to do
    return
  }

  const packetSize = routingInfoLastHopLength + (secrets.length - 1) * routingInfoLength

  let length = 0
  let start = packetSize
  let end = packetSize

  for (let index = 0; index < secrets.length - 1; index++) {
    const prgParams = derivePRGParameters(secrets[index])

    start -= routingInfoLength
    length += routingInfoLength

    u8aXOR(
      true,
      header.subarray(routingInfoLastHopLength, routingInfoLastHopLength + length),
      PRG.createPRG(prgParams).digest(start, end)
    )
  }
}
