/**
 * Checks if the contents of the given Uint8Arrays are equal. Returns once at least
 * one different entry is found.
 * @param a first array
 * @param b second array
 * @param arrays additional arrays
 */
function u8aEquals(a: Uint8Array, b: Uint8Array, ...arrays: Uint8Array[]) {
  if (arrays == null) {
    const aLength = a.length
    const bLength = b.length

    if (aLength != bLength) {
      return false
    }

    for (let i = 0; i < aLength; i++) {
      if (a[i] != b[i]) {
        return false
      }
    }
  } else {
    arrays.push(a, b)

    const firstLength = arrays[0].length
    for (let i = 1; i < arrays.length; i++) {
      if (firstLength != arrays[i].length) {
        return false
      }
    }

    for (let i = 0; i < arrays.length; i++) {
      for (let j = i + 1; j < arrays.length; j++) {
        for (let k = 0; k < firstLength; k++) {
          if (arrays[i][k] != arrays[j][k]) {
            return false
          }
        }
      }
    }
  }

  return true
}

export { u8aEquals }
