/**
 * Computes the greatest common divisor of two integers
 * @param a first number
 * @param b second number
 */
export function gcd(a: number, b: number): number {
  if ([a, b].includes(0)) {
    return 1
  }

  a = Math.abs(a)
  b = Math.abs(b)

  if (b > a) {
    let temp = a
    a = b
    b = temp
  }

  while (true) {
    if (b == 0) return a
    a %= b
    if (a == 0) return b
    b %= a
  }
}
