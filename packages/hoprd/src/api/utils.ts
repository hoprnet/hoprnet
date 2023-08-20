import cookie from 'cookie'
import RLP from 'rlp'
import { debug } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from './v3/utils.js'

const debugLog = debug('hoprd:api:utils')

/**
 * Authenticates a websocket connection.
 * Will first check from `apiToken` parameter in URL,
 * then try to find `X-Auth-Token` in the cookies.
 * @returns true if connection is authenticated
 */
export const authenticateWsConnection = (
  req: { url?: string; headers: Record<any, any> },
  apiToken: string
): boolean => {
  // throw if apiToken is empty
  if (apiToken === '') throw Error('Cannot authenticate empty apiToken')
  let encodedApiToken = encodeURIComponent(apiToken)

  // attempt to authenticate via URL parameter
  if (req.url) {
    try {
      // NB: We use a placeholder domain since req.url only passes query params
      const url = new URL(`https://hoprnet.org${req.url}`)
      const paramApiToken = url.searchParams?.get('apiToken') || ''
      if (encodeURIComponent(paramApiToken) == encodedApiToken) {
        debugLog('ws client connected [ authentication SUCCESS via URL parameter ]')
        return true
      }
    } catch (e) {
      debugLog('invalid URL queried', e)
    }
  }

  // attempt to authenticate via cookie
  if (req.headers.cookie) {
    let cookies: ReturnType<typeof cookie.parse> | undefined
    try {
      cookies = cookie.parse(req.headers.cookie)
    } catch (e) {
      debugLog(`failed parsing cookies`, e)
    }

    // We compare the encoded token against an encoded token from the user, thus avoiding having to decodeURI on the user input
    // and therefore avoiding the need to handle any decoding errors at all.
    // The encodeURIComponent function on an already encoded input acts as an identity function
    if (
      cookies &&
      (encodeURIComponent(cookies['X-Auth-Token'] || '') === encodedApiToken ||
        encodeURIComponent(cookies['x-auth-token'] || '') === encodedApiToken)
    ) {
      debugLog('ws client connected [ authentication SUCCESS via cookie ]')
      return true
    }
  }

  debugLog('ws client failed authentication')
  return false
}

/**
 * Given a URL path, we strip away query parameters and tailing slash.
 * @param path
 * @returns stripped path
 * @example `/api/v2/messages/websocket?apiToken=^^LOCAL-testing-123^^` becomes `/api/v2/messages/websocket`
 * @example `/api/v2/messages/websocket/?apiToken=^^LOCAL-testing-123^^` becomes `/api/v2/messages/websocket`
 */
export const removeQueryParams = (path: string): string => {
  // we use a placeholder domain since req.url only passes query params
  const url = new URL(`https://hoprnet.org${path}`)
  let strippedPath = url.pathname
  if (strippedPath.endsWith('/')) strippedPath = strippedPath.slice(0, -1)
  return strippedPath
}

export const getStatusCodeForInvalidInputInRequest = (inputPath: string) => {
  switch (inputPath.toLocaleLowerCase()) {
    case 'currency':
      return STATUS_CODES.INVALID_CURRENCY
    case 'amount':
    case 'outgoingamount':
    case 'incomingamount':
      return STATUS_CODES.INVALID_AMOUNT
    case 'ethereumaddress':
    case 'peeraddress':
      return STATUS_CODES.INVALID_ADDRESS
    case 'channelid':
      return STATUS_CODES.INVALID_CHANNELID
    case 'peerid':
      return STATUS_CODES.INVALID_PEERID
    case 'setting':
      return STATUS_CODES.INVALID_SETTING
    case 'settingvalue':
      return STATUS_CODES.INVALID_SETTING_VALUE
    case 'quality':
      return STATUS_CODES.INVALID_QUALITY
    default:
      return STATUS_CODES.INVALID_INPUT
  }
}

/**
 * Adds the current timestamp to the message in order to measure the latency.
 * @param msg the message
 */
export function encodeMessage(msg: string): Uint8Array {
  return RLP.encode([msg, Date.now()])
}

/**
 * Tries to decode the message and returns the message as well as
 * the measured latency.
 * @param encoded an encoded message
 */
export function decodeMessage(encoded: Uint8Array): {
  latency: number
  msg: string
} {
  let msg: Uint8Array, time: Uint8Array
  try {
    ;[msg, time] = RLP.decode(encoded) as [Uint8Array, Uint8Array]

    return {
      latency: Date.now() - Buffer.from(time).readUintBE(0, time.length),
      msg: new TextDecoder().decode(msg)
    }
  } catch (err) {
    return {
      latency: NaN,
      msg: 'Error: Could not decode message'
    }
  }
}
