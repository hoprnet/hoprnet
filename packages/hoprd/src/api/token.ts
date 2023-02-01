import { HoprDB } from '@hoprnet/hopr-utils'

type Limit = {
  type: string
  used?: number
  max?: number
}

type Capability = {
  endpoint: string
  limits: Array<Limit>
}

export type Token = {
  // the id is used as the key, as well as the secret used during authentication
  id: string
  capabilities: Array<Capability>
  description?: string
  valid_until?: number
}

// this namespace is used as a prefix for all stored keys
const ns = 'authenticationTokens'

export async function authenticateToken(db: HoprDB, id: string): Promise<Token> {
  // id is used as key, the returned object includes the associated data
  const token = await db.getSerializedObject(ns, id)

  // if no token was found, we return directly, otherwise the result is
  // deserialized first
  if (!token) {
    return undefined
  }

  const deserializedToken = deserializeToken(token)

  // delete token if lifetime has passed, otherwise return
  const now = Date.now()
  if (deserializedToken.valid_until && deserializedToken.valid_until < now) {
    await deleteToken(db, deserializedToken.id)
    return undefined
  }

  return deserializedToken
}

export async function authorizeToken(_db: HoprDB, token: Token, endpointRef: string): Promise<boolean> {
  // find relevant endpoint capabilities
  const endpointCaps = token.capabilities.filter((capability: Capability) => capability.endpoint === endpointRef)

  // Go through all specified capabilities. If at least one entry is set to
  // valid, let the request pass through.
  const capsChecks = endpointCaps.map((c) => {
    // Go through all specified limits. If all entries are set to valid, set the
    // limit to be passed.
    const limitsChecks = c.limits.map((l) => {
      if (l.type === 'calls') {
        // check that max calls is more than the consumed number
        return l.max > l.used
      }
      // unknown limit type, set to invalid
      return false
    })

    return limitsChecks.every((c) => c === true)
  })

  const tokenAuthorized = capsChecks.some((c) => c === true)
  if (tokenAuthorized) {
    // update limits before returning
    token.capabilities = endpointCaps.map((c) => {
      return c.limits.map((l) => {
        if (l.type === 'calls') {
          // Add or increment field 'used'
          const used = l.used ? ++l.used : 1
          l.used = used
        }
        return l
      })
    })
    await storeToken(db, token)
  }

  return tokenAuthorized
}

export async function storeToken(db: HoprDB, token: Token): Promise<void> {
  const serializedToken = serializeToken(token)
  await db.putSerializedObject(ns, token.id, serializedToken)
}

export async function deleteToken(db: HoprDB, id: string): Promise<void> {
  await db.deleteObject(ns, id)
}

function serializeToken(token: Token): Uint8Array {
  const stringifiedToken = JSON.stringify(token)
  return Buffer.from(stringifiedToken, 'base64')
}

function deserializeToken(token: Uint8Array): Token {
  const deserializedToken = new TextDecoder('base64').decode(token)
  return JSON.parse(deserializedToken)
}
