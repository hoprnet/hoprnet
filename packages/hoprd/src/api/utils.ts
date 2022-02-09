import cookie from 'cookie'
import { debug } from '@hoprnet/hopr-utils'

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

  // attempt to authenticate via URL parameter
  if (req.url) {
    try {
      // NB: We use a placeholder domain since req.url only passes query params
      const url = new URL(`https://hoprnet.org${req.url}`)
      const paramApiToken = url.searchParams?.get('apiToken') || ''
      if (decodeURI(paramApiToken) == apiToken) {
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

    if (
      cookies &&
      (decodeURI(cookies['X-Auth-Token'] || '') === apiToken || decodeURI(cookies['x-auth-token'] || '') === apiToken)
    ) {
      debugLog('ws client connected [ authentication SUCCESS via cookie ]')
      return true
    }
  }

  debugLog('ws client failed authentication')
  return false
}
