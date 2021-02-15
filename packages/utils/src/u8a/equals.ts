/**
 * Checks if the contents of the given Uint8Arrays are equal. Returns once at least
 * one different entry is found.
 * @param a first array
 * @param b second array
 * @param arrays additional arrays
 */
function u8aEquals(a: Uint8Array, b: Uint8Array, ...arrays: Uint8Array[]) {
  const aLength = a.length

  if (aLength != b.length) {
    return false
  }

  let canUse32 = a.byteOffset % 4 == 0 && b.byteOffset % 4 == 0
  let canUse16 = a.byteOffset % 2 == 0 && b.byteOffset % 2 == 0

  if (arrays?.length) {
    for (const arr of arrays) {
      if (arr == undefined) {
        return false
      }
      canUse32 &&= arr.byteOffset % 4 == 0
      canUse16 &&= arr.byteOffset % 2 == 0

      if (aLength != arr.length) {
        return false
      }
    }
  }

  let index = 0

  if (canUse32) {
    for (; index + 4 <= aLength; index += 4) {
      let aArr = new Uint32Array(a.buffer, a.byteOffset + index, 1)[0]

      if (aArr != new Uint32Array(b.buffer, b.byteOffset + index, 1)[0]) {
        return false
      }
      if (arrays?.length) {
        for (const arr of arrays) {
          if (aArr != new Uint32Array(arr.buffer, arr.byteOffset + index, 1)[0]) {
            return false
          }
        }
      }
    }
  }

  if (canUse16) {
    for (; index + 2 <= aLength; index += 2) {
      let aArr = new Uint16Array(a.buffer, a.byteOffset + index, 1)[0]

      if (aArr != new Uint16Array(b.buffer, b.byteOffset + index, 1)[0]) {
        return false
      }
      if (arrays?.length) {
        for (const arr of arrays) {
          if (aArr != new Uint16Array(arr.buffer, arr.byteOffset + index, 1)[0]) {
            return false
          }
        }
      }
    }
  }

  for (; index < aLength; index++) {
    if (a[index] != b[index]) {
      return false
    }

    if (arrays?.length) {
      for (const arr of arrays) {
        if (a[index] != arr[index]) {
          return false
        }
      }
    }
  }

  return true
}

export function u8aIsEmpty(a: Uint8Array, size): boolean {
  return u8aEquals(a, new Uint8Array(size).fill(0x00))
}

export { u8aEquals }
