import type { LogStream } from './../logs'
import cookie from 'cookie'

/**
 * Authenticates a websocket connection.
 * Will first check from `apiToken` parameter in URL,
 * then try to find `X-Auth-Token` in the cookies.
 * @returns true if connection is authenticated
 */
export const authenticateWsConnection = (
  logs: LogStream,
  req: { url?: string; headers: Record<any, any> },
  apiToken?: string
): boolean => {
  // authentication is disabled
  if (!apiToken) {
    logs.log('ws client connected [ authentication DISABLED ]')
    return true
  }

  // Authenticate by URL parameter
  // Other clients different to `hopr-admin` might pass the `apiToken` via a
  // query param since they won't be on the same domain the node is hosted,
  // and thus, unable to set the `apiToken` via cookies. Using `req.url` we
  // can detect these cases and provide the ability for any client that
  // knows the `apiToken` to reach your HOPR node.
  if (req.url) {
    try {
      // NB: We use a placeholder domain since req.url only passes query params
      const url = new URL(`https://hoprnet.org${req.url}`)
      const paramApiToken = url.searchParams?.get('apiToken') || ''
      if (decodeURI(paramApiToken) == apiToken) {
        logs.log('ws client connected [ authentication ENABLED ]')
        return true
      }
    } catch (e) {
      logs.error('invalid URL queried', e)
    }
  }

  // Authenticate by cookie
  if (req.headers.cookie) {
    let cookies: ReturnType<typeof cookie.parse> | undefined
    try {
      cookies = cookie.parse(req.headers.cookie)
    } catch (e) {
      logs.error(`failed parsing cookies`, e)
    }

    if (
      cookies &&
      (decodeURI(cookies['X-Auth-Token'] || '') === apiToken || decodeURI(cookies['x-auth-token'] || '') === apiToken)
    ) {
      logs.log('ws client connected [ authentication ENABLED ]')
      return true
    }
  }

  logs.log('ws client failed authentication')
  return false
}
