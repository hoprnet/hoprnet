/**
 * Parse JSON while recovering all Buffer elements
 * @param str JSON string
 */
export function parseJSON(str: string): object {
  return JSON.parse(str, (_key, value) => {
    if (value && value.type === 'Buffer') {
      return Buffer.from(value.data)
    }

    return value
  })
}
