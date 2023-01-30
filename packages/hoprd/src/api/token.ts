import { HoprDB } from '@hoprnet/hopr-utils'

export type Token = {
  id: string,
  description: string,
  capabilities: any
}

const ns = 'authenticationTokens'

export async function authenticateToken(db: HoprDB, token: string): Promise<Token> {
  // token is used as key, the returned object includes the associated data
  const fullToken = await db.getSerializedObject(ns, token)

  if (!fullToken) {
    return undefined
  }

  return deserializeToken(fullToken)
}

export function authorizeToken(db: HoprDB, token: Token, endpointRef: string): boolean {
  // check capabilties
  // update limits
  return false
}

export async function createToken(db: HoprDB, token: Token): Promise<string> {
  const serializedToken = serializeToken(token)
  const key = "todo"
  await db.putSerializedObject(ns, key, serializedToken)
  return key
}

function serializeToken(token: Token): Uint8Array {
  const string = JSON.stringify(token)
  return Buffer.from(string, "base64")
}

function deserializeToken(token: Uint8Array): Token {
  const string = new TextDecoder("base64").decode(token)
  return JSON.parse(string)
}
