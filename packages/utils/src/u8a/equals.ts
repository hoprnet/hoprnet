/**
 * Checks if the contents of the given Uint8Arrays are equal. Returns once at least
 * one different entry is found.
 * @param a first array
 * @param b second array
 * @param arrays additional arrays
 */
export function u8aEquals(...arrays: (Uint8Array | undefined)[]) {
  const aLength = arrays.length

  if (aLength <= 1) {
    return true
  }

  if (arrays[0] == undefined) {
    for (let i = 1; i < aLength; i++) {
      if (arrays[i] != undefined) {
        return false
      }
    }
    return true
  }

  const firstLength = arrays[0].length

  if (firstLength == 0) {
    for (let i = 1; i < aLength; i++) {
      if (arrays[i] == undefined) {
        return false
      }

      if (arrays[i].length != 0) {
        return false
      }
    }
    return true
  }

  let canUse64 = arrays[0].byteOffset % 8 == 0
  let canUse32 = arrays[0].byteOffset % 4 == 0
  let canUse16 = arrays[0].byteOffset % 2 == 0

  for (let i = 1; i < aLength; i++) {
    if (arrays[i] == undefined) {
      return false
    }
    if (firstLength != arrays[i].length) {
      return false
    }

    canUse64 = canUse64 && arrays[i].byteOffset % 8 == 0
    canUse32 = canUse32 && arrays[i].byteOffset % 4 == 0
    canUse16 = canUse16 && arrays[i].byteOffset % 2 == 0
  }

  const views: DataView[] = Array.from(
    { length: aLength },
    (_, i: number) => new DataView(arrays[i].buffer, arrays[i].byteOffset, firstLength)
  )

  let index = 0

  if (canUse64) {
    for (; index + 8 <= firstLength; index += 8) {
      const first = views[0].getBigUint64(index)

      for (let i = 1; i < aLength; i++) {
        if (first != views[i].getBigUint64(index)) {
          return false
        }
      }
    }
  }

  if (canUse32) {
    for (; index + 4 <= firstLength; index += 4) {
      const first = views[0].getUint32(index)

      for (let i = 1; i < aLength; i++) {
        if (first != views[i].getUint32(index)) {
          return false
        }
      }
    }
  }

  if (canUse16) {
    for (; index + 2 <= firstLength; index += 2) {
      const first = views[0].getUint16(index)

      for (let i = 1; i < aLength; i++) {
        if (first != views[i].getUint16(index)) {
          return false
        }
      }
    }
  }

  if (firstLength & 1) {
    const first = views[0].getUint8(index)

    for (let i = 1; i < aLength; i++) {
      if (first != views[i].getUint8(index)) {
        return false
      }
    }
  }

  return true
}
