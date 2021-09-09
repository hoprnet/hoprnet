import BN from 'bn.js'

export const CHANNELS_PER_COVER_TRAFFIC_NODE = 10
export const CHANNEL_STAKE = new BN('1000')
export const MINIMUM_STAKE_BEFORE_CLOSURE = new BN('0')
export const CT_INTERMEDIATE_HOPS = 2 // 3  // NB. min is 2
export const MESSAGE_FAIL_THRESHOLD = 1000 // Failed sends to channel before we autoclose
export const CT_PATH_RANDOMNESS = 0.2
export const CT_NETWORK_QUALITY_THRESHOLD = 0.15
