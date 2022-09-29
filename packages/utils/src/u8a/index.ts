export * from './constants.js'

export * from './concat.js'

export * from './xor.js'
export * from './u8aAdd.js'

export * from './toU8a.js'
export * from './toHex.js'

export * from './u8aToNumber.js'

export * from './equals.js'
export * from './u8aAdd.js'
export * from './u8aCompare.js'

export type U8aAndSize = [Uint8Array, number]

export function serializeToU8a(items: U8aAndSize[]): Uint8Array {
  const totalSize = items.map((x) => x[1]).reduce((x, y) => x + y, 0)
  const arr = new Uint8Array(totalSize)
  let i = 0
  items.forEach((item) => {
    if (item[0].length != item[1]) {
      throw new Error(`Error serializing - expected item of length ${item[1]}, got ${item[0].length}`)
    }
    arr.set(item[0], i)
    i += item[1]
  })
  return arr
}

export function u8aSplit(u8a: Uint8Array, sizes: number[]): Uint8Array[] {
  let totalSize = sizes.reduce((x, y) => x + y, 0)
  if (u8a.length !== totalSize) {
    throw new Error('U8a cannot be split: length != sum(sizes)')
  }
  let i = 0
  return sizes.map((s) => {
    i += s
    return u8a.slice(i - s, i)
  })
}
