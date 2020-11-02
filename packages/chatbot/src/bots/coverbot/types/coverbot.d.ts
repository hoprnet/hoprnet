import Core from '../../../lib/hopr/core'

export type HoprNode = {
  id: string
  address: string
  tweetId: string
  tweetUrl: string
}

export type BalancedHoprNode = {
  node: Core
  balance: string
  hoprBalance: string
}
