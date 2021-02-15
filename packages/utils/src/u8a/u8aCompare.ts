export const A_STRICLY_LESS_THAN_B = -1
export const A_EQUALS_B = 0
export const A_STRICTLY_GREATER_THAN_B = 1

export function u8aCompare(a: Uint8Array, b: Uint8Array): number {
  if (a.length != b.length) {
    throw Error(`Cannot compare arrays that have different size.`)
  }

  const length = a.length

  for (let i = 0; i < length; i++) {
    if (a[i] == b[i]) {
      continue
    }

    if (a[i] < b[i]) {
      return A_STRICLY_LESS_THAN_B
    } else {
      return A_STRICTLY_GREATER_THAN_B
    }
  }

  return A_EQUALS_B
}

export function u8aLessThanOrEqual(a: Uint8Array, b: Uint8Array): boolean {
  return [A_STRICLY_LESS_THAN_B, A_EQUALS_B].includes(u8aCompare(a, b))
}

