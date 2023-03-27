export const TOKEN_DECIMALS = 18
export const TOKEN_NATIVE_MIN = 0.01
export const TOKEN_HOPR_MIN = 10

export enum ConnectivityStatus {
  Green = 'Green',
  Yellow = 'Yellow',
  Orange = 'Orange'
}

export enum ChannelType {
  incomming = 'incomming',
  outgoing = 'outgoing'
}

export interface Addresses {
  native: string
  hopr: string
}

export class NodeBalance {
  native: number
  hopr: number

  public constructor(balance: { native: string; hopr: string }) {
    this.native = Number(balance.native) / Math.pow(10, TOKEN_DECIMALS)
    this.hopr = Number(balance.hopr) / Math.pow(10, TOKEN_DECIMALS)
  }
}

export interface SendMessageRequest {
  body: string
  recipient: string
  path?: string[]
}

export interface Channel {
  type: ChannelType
  channelId: string
  peerId: string
  status: string
  balance: string
}

export interface ChannelResponse {
  incoming: Channel[]
  outgoing: Channel[]
}

export interface HoprNode {
  alias: string
  url: string
  apiToken: string
  thresholds: {
    incommingOpenChannels: number
    outgoingOpenChannels: number
    connectivityStatus: ConnectivityStatus
    peersQuality: number
  }

  scenarios: string[]
  sleepTime: {
    defaultMin: number
    defaultMax: number
  }
}

export interface Peer {
  peerId: string
  multiAddr: string[]
  heartbeats: {
    sent: number
    success: number
  }
  lastSeen: number
  quality: number
  backoff: number
  isNew: boolean
}

export interface PeerResponse {
  connected: Peer[]
  announced: Peer[]
}
