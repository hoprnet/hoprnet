import { randomBytes } from 'crypto'

export function randomFloat() {
  let exponent = 1023

  const buf = new Uint8Array(8).fill(0x00)

  buf[0] |= exponent >> 4
  buf[1] |= (exponent & 15) << 4

  const bytes = randomBytes(7)

  buf[1] |= bytes[0] & 15

  for (let i = 2; i < 8; i++) {
    buf[i] = bytes[i - 1]
  }

  return new DataView(buf.buffer).getFloat64(0) - 1
}
