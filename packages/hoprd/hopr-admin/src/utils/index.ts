/**
 * Used for diplay purposes.
 * Trasform a tuple array to a padded output.
 * @returns padded string.
 */
export const toPaddedString = (items: [string, string][]): string => {
  // add 2 spaces after max pad length
  const length = getPaddingLength(items.map((item) => item[0])) + 2

  return items
    .map(([valA, valB]) => {
      return valA.padEnd(length) + valB
    })
    .join('\n')
}

/**
 * Used for diplay purposes.
 * @returns the max length a string can be after adding padding
 */
export const getPaddingLength = (items: string[]): number => {
  return Math.max(...items.map((str) => str.length))
}

/**
 * True if instance is running on server
 */
export const isSSR: boolean = typeof window === 'undefined'

/**
 * Inspects the url to find valid settings.
 * @returns config found in url query
 */
export const getUrlParams = (loc: Location): Partial<Configuration> => {
  const params = new URLSearchParams(loc.search)
  return {
    apiEndpoint: params.get('apiEndpoint') || undefined,
    apiToken: params.get('apiToken') || undefined
  }
}

export const API_TOKEN_COOKIE = 'X-Auth-Token'

/**
 * Connectivity configuration
 */
export type Configuration = {
  apiEndpoint: string
  apiToken?: string
}

/**
 * API paths should start with `/api/v2/`
 */
export type ApiPath = `/api/v2/${string}`

export type Log = { id: string; msg: string; ts: number }

export const createLog = (msg: string, ts?: number): Log => {
  return {
    msg,
    id: String(Math.random()),
    ts: ts || +new Date()
  }
}

export enum HealthStatus {
  Unknown = '❔',
  Red = '🔴',
  Orange = '🟠',
  Yellow = '🟡',
  Green = '🟢'
}
