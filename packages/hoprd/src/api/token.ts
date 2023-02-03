import { v4 as uuidv4 } from 'uuid'
import { createHash } from 'crypto'

import { HoprDB } from '@hoprnet/hopr-utils'

type Limit = {
  type: string
  conditions: {
    max?: number
  }
  used?: number
}

type Capability = {
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
          // perform runtime check
          const check = limit.runtimeCheck
          return check(l.conditions.max, l.used)
        }

        // unknown limit type, set to invalid
        return false
      })

      return limitsChecks.every((c) => c === true)
    }
    return true
  })

  const tokenAuthorized = capsChecks.some((c) => c === true)
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

export async function createToken(db: HoprDB, capabilities: Array<Capability>, description?: string): Promise<Token> {
  if (!validateCapabilities(capabilities)) {
    throw new Error('invalid token capabilities')
  }

  const id = await generateNewId(db)

  return {
    id,
    description: description || '',
    capabilities
  }
}

export async function storeToken(db: HoprDB, token: Token): Promise<void> {
  const serializedToken = serializeToken(token)
  await db.putSerializedObject(ns, token.id, serializedToken)
}

export async function deleteToken(db: HoprDB, id: string): Promise<void> {
  if (!id) {
    return
  }
  await db.deleteObject(ns, id)
}

function serializeToken(token: Token): Uint8Array {
  const stringifiedToken = JSON.stringify(token)
  return Buffer.from(stringifiedToken, 'utf-8')
}

function deserializeToken(token: Uint8Array): Token {
  const deserializedToken = new TextDecoder('utf-8').decode(token)
  return JSON.parse(deserializedToken)
}

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

function validateCapabilities(capabilities: Array<Capability>): boolean {
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
