/**
 * Checks if the contents of the given Uint8Arrays are equal. Returns once at least
 * one different entry is found.
 * @param a first array
 * @param b second array
 * @param arrays additional arrays
 */
function u8aEquals(a: Uint8Array, b: Uint8Array, ...arrays: Uint8Array[]) {
  if (arrays == null) {
    return a.length == b.length && a.every((value: number, index: number) => value == b[index])
  } else {
    arrays.push(a, b)

    if (arrays.slice(1).some((arr: Uint8Array) => arr.length != arrays[0].length)) {
      return false
    }

    for (let i = 0; i < arrays.length; i++) {
      for (let j = i + 1; j < arrays.length; j++) {
        if (arrays[i].some((value: number, index: number) => value != arrays[j][index])) {
          return false
        }
      }
    }

    return true
  }
}

export { u8aEquals }
