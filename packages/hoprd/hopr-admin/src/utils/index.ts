// import { encode, decode } from 'rlp'

// /**
//  * Adds the current timestamp to the message in order to measure the latency.
//  * @param msg the message
//  */
// export function encodeMessage(msg: string): Uint8Array {
//   return encode([msg, Date.now()])
// }

// /**
//  * Tries to decode the message and returns the message as well as
//  * the measured latency.
//  * @param encoded an encoded message
//  */
// export function decodeMessage(encoded: Uint8Array): {
//   latency: number
//   msg: string
// } {
//   let msg: Buffer, time: Buffer
//   try {
//     ;[msg, time] = decode(encoded) as [Buffer, Buffer]

//     return {
//       latency: Date.now() - parseInt(time.toString('hex'), 16),
//       msg: msg.toString()
//     }
//   } catch (err) {
//     console.log(
//       styleValue(`Could not decode received message '${u8aToHex(encoded)}' Error was ${err.message}.`, 'failure')
//     )

//     return {
//       latency: NaN,
//       msg: 'Error: Could not decode message'
//     }
//   }
// }

/**
 * Used for diplay purposes.
 * Trasform a tuple array to a padded output.
 * @returns padded string.
 */
export const toPaddedString = (items: [string, string][]): string => {
  const length = getPaddingLength(items.map((item) => item[0]))

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
  return Math.max(...items.map((str) => str.length)) + 2
}

/**
 * True if instance is running on server
 */
export const isSSR: boolean = typeof window === 'undefined'

/**
 * Inspects the url to find valid settings.
 * @returns settings found in url query
 */
export const getUrlParams = (loc: Location): Partial<Settings> => {
  const params = new URLSearchParams(loc.search)
  return {
    apiEndpoint: params.get('apiEndpoint') || undefined,
    apiToken: params.get('apiToken') || undefined
  }
}

export const API_TOKEN_COOKIE = 'X-Auth-Token'

/**
 * Connectivity settings
 */
export type Settings = {
  apiEndpoint: string
  apiToken?: string
}

/**
 * API paths should start with `/api/v2/`
 */
export type ApiPath = `/api/v2/${string}`

export type Log = { id: string; msg: string; ts: number }
