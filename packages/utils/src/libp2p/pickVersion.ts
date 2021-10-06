/**
 * Used by our network stack and deployment scripts to determine.
 * @param full_version
 * @returns major and minor versions, ex: `1.8.5` -> `1.8.0`
 */
export const pickVersion = (full_version: string): string => {
  const split = full_version.split('.')
  return split[0] + '.' + split[1] + '.0'
}
