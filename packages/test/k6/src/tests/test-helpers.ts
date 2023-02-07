/**
 * Generate a random string
 * @param length Size of the string
 * @returns
 */
export function randomString(length: number): string {
  const charset = 'abcdefghijklmnopqrstuvwxyz'
  let res = ''
  while (length--) res += charset[(Math.random() * charset.length) | 0]
  return res
}
