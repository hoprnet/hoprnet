import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import { addresses } from '@hoprnet/hopr-ethereum'

export const DEFAULT_URI = 'ws://127.0.0.1:9545/'

export const TOKEN_ADDRESSES = addresses.HOPR_TOKEN

export const CHANNELS_ADDRESSES = addresses.HOPR_CHANNELS

export const FUND_ACCOUNT_PRIVATE_KEY = NODE_SEEDS[0]
export const DEMO_ACCOUNTS = NODE_SEEDS

export const MAX_CONFIRMATIONS = 8
