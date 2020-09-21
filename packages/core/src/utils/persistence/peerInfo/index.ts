type SerializedPeerId = Buffer

type SerializedMultiaddr = Buffer

type SerializedPublicKey = Buffer

export type SerializedPeerInfo =
  | [SerializedPeerId, SerializedMultiaddr[]]
  | [SerializedPeerId, SerializedMultiaddr[], SerializedPublicKey]

export * from './serialize'
export * from './deserialize'
