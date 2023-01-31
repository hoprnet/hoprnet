import { HoprDB } from '@hoprnet/hopr-utils'

export type Token = {
  // the id is used as the key, as well as the secret used during authentication
  id: string
  description: string
  capabilities: any
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

  return deserializeToken(token)
}

export function authorizeToken(db: HoprDB, token: Token, endpointRef: string): boolean {
  // check capabilties
  // update limits
  return false
}

export async function createToken(db: HoprDB, token: Token): Promise<void> {
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
