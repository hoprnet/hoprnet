import { v4 as uuidv4 } from 'uuid'
import { createHash } from 'crypto'

import { HoprDB } from '@hoprnet/hopr-utils'

export type Limit = {
  type: string
  conditions: {
    max?: number
  }
  used?: number
}

export type Capability = {
  endpoint: string
  limits?: Array<Limit>
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

// Authenticate the given token object, verifying its stored in the database.
// @param db Reference to a HoprDB instance.
// @param id Token id which should be authenticated.
// @return the token object which is found in the database, or undefined
export async function authenticateToken(db: HoprDB, id: string): Promise<Token> {
  if (!id) {
    return undefined
  }

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

// Authorize the given token object, verifying its capabilities against the
// chosen endpoint.
// @param db Reference to a HoprDB instance.
// @param token Token object which should be authorized.
// @param endpointRef Logical name of the endpoint the authorization is checked
// for.
// @return true if the token is authorized, false if not
export async function authorizeToken(db: HoprDB, token: Token, endpointRef: string): Promise<boolean> {
  // find relevant endpoint capabilities
  const endpointCaps = token.capabilities.filter((capability: Capability) => capability.endpoint === endpointRef)

  // fail early when no endpoint capabilities were found
  if (endpointCaps.length === 0) {
    return false
  }

  // Go through all specified capabilities. If at least one entry is set to
  // valid, let the request pass through.
  const capsChecks = endpointCaps.map((c) => {
    if (c.limits) {
      // we only verify limits if any are set
      // Go through all specified limits. If all entries are set to valid, set the
      // limit to be passed.
      const limitsChecks = c.limits.map((l) => {
        const limit = supportedCapabilities[endpointRef][l.type] || genericLimits[l.type]

        if (limit) {
          return Object.entries(l.conditions).every(([condition, value]) => {
            // perform runtime check
            const check = limit[condition]?.runtimeCheck
            if (check) {
              const checkResult = check(value, l.used || 0)
              return checkResult
            }
            return false
          })
        }

        // unknown limit type, set to invalid
        return false
      })

      return limitsChecks.every((c) => c === true)
    }
    return true
  })

  const tokenAuthorized = capsChecks.every((c) => c === true)
  if (tokenAuthorized) {
    // update limits before returning
    token.capabilities = endpointCaps.map((c) => {
      if (c.limits) {
        const limits = c.limits.map((l) => {
          if (l.type === 'calls') {
            // Add or increment field 'used'
            const used = l.used ? ++l.used : 1
            l.used = used
          }
          return l
        })
        c.limits = limits
      }
      return c
    })
    await storeToken(db, token)
  }

  return tokenAuthorized
}

// Create a token object from the given parameters, but don't store it in the database yet.
// @param db Reference to a HoprDB instance.
// @param capabilities Capabilities which are attached to the token object.
// @param description Description which is attached to the token object.
// @param lifetime Number of seconds used to calculate the maximum lifetime of the token.
export async function createToken(
  db: HoprDB,
  capabilities: Array<Capability>,
  description?: string,
  lifetime?: number
): Promise<Token> {
  if (!validateTokenCapabilities(capabilities)) {
    throw new Error('invalid token capabilities')
  }

  if (lifetime && lifetime < 1) {
    throw new Error('invalid token lifetime')
  }

  const id = await generateNewId(db)
  const token: Token = {
    id,
    description: description || '',
    capabilities
  }

  if (lifetime) {
    token.valid_until = Date.now() + lifetime
  }

  return token
}

// Store a token in the database.
// @param db Reference to a HoprDB instance.
// @param id Token object.
export async function storeToken(db: HoprDB, token: Token): Promise<void> {
  const serializedToken = serializeToken(token)
  await db.putSerializedObject(ns, token.id, serializedToken)
}

// Delete a token from the database.
// @param db Reference to a HoprDB instance.
// @param id Token id. The operation is a no-op if its an empty string.
export async function deleteToken(db: HoprDB, id: string): Promise<void> {
  if (!id) {
    return
  }
  await db.deleteObject(ns, id)
}

// Serialize the given token object into a byte array.
// @param token Token object which shall be serialized.
// @return Serialized token object.
function serializeToken(token: Token): Uint8Array {
  const stringifiedToken = JSON.stringify(token)
  return Buffer.from(stringifiedToken, 'utf-8')
}

// Deserialize the given array into a token object.
// @param token Array representing a serialized token object.
// @return Deserialized token object.
function deserializeToken(token: Uint8Array): Token {
  const deserializedToken = new TextDecoder('utf-8').decode(token)
  return JSON.parse(deserializedToken)
}

// Generate a token id which is not present yet in the database.
// @param db Reference to a HoprDB instance.
// @return a new unique token id
async function generateNewId(db: HoprDB): Promise<string> {
  let id = undefined

  // iterate until we find a usable id
  while (!id) {
    const uuid = uuidv4()
    const nextId = createHash('sha256').update(uuid).digest('base64url')
    // try to load the token given the new id
    const token = await authenticateToken(db, nextId)
    if (!token) {
      // no token found, id can be used
      id = nextId
    }
  }

  return id
}

// Generic limits which are supported on every supported endpoint.
const genericLimits = {
  calls: {
    max: {
      validityCheck: (v: number): boolean => v > 0,
      runtimeCheck: (v: number, w: number): boolean => v > w
    }
  }
}

// List of endpoints which are supported as capabilitities.
// Each entry also specifies supported endpoint-specific limits.
const supportedCapabilities = {
  tokensCreate: {},
  tokensGetToken: {},
  ticketsGetStatistics: {},
  ticketsRedeemTickets: {},
  ticketsGetTickets: {},
  settingsGetSettings: {},
  nodeGetVersion: {},
  nodeStreamWebsocket: {},
  nodePing: {},
  nodeGetPeers: {},
  nodeGetMetrics: {},
  nodeGetInfo: {},
  nodeGetEntryNodes: {},
  messagesWebsocket: {},
  messagesSign: {},
  messagesSendMessage: {},
  messageSign: {},
  channelsOpenChannel: {},
  channelsGetChannels: {},
  aliasesGetAliases: {},
  aliasesSetAlias: {},
  accountWithdraw: {},
  accountGetBalances: {},
  accountGetAddresses: {},
  accountGetAddress: {},
  tokensDelete: {},
  settingsSetSetting: {},
  peerInfoGetPeerInfo: {},
  channelsRedeemTickets: {},
  channelsGetTickets: {},
  channelsCloseChannel: {},
  channelsGetChannel: {},
  aliasesGetAlias: {},
  aliasesRemoveAlias: {}
}

// Validates the given list of capabilities. Fails if the list is empty or any
// of the capabilities is invalid.
// @param capabilities Non-empty list of capabilities.
// @return true if list is valid, false if any entry is invalid or the list is
// empty.
export function validateTokenCapabilities(capabilities: Array<Capability>): boolean {
  // fail early if list is empty
  if (capabilities.length === 0) {
    return false
  }

  return capabilities.every((c) => {
    if (!(c.endpoint in supportedCapabilities)) {
      // endpoint not supported, validation fails
      return false
    }
    if (!c.limits) {
      // if no limits are set, validation succeeds
      return true
    }

    if (c.limits && c.limits.length === 0) {
      // if limits is set but an empty array, validation fails
      return false
    }

    return c.limits.some((l) => {
      // check endpoint-specific limits
      const limits = supportedCapabilities[c.endpoint]
      if (l.type in limits) {
        return true
      }

      // check generic limits
      if (l.type in genericLimits) {
        const limitConditions = genericLimits[l.type]
        return Object.entries(l.conditions).every(([k, v]) => {
          if (!limitConditions[k]) {
            // limit condition not known, validation fails
            return false
          }
          // run check for condition value, validation fails if check fails
          const check = limitConditions[k].validityCheck
          return check(v)
        })
      }

      // limit is not known, validation fails
      return false
    })
  })
}
