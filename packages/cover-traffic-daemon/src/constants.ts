import { PRICE_PER_PACKET } from '@hoprnet/hopr-utils'

export const CHANNELS_PER_COVER_TRAFFIC_NODE = 10
export const CT_INTERMEDIATE_HOPS = 2 // 3  // NB. min is 2
export const MINIMUM_STAKE_BEFORE_CLOSURE = PRICE_PER_PACKET.muln(CT_INTERMEDIATE_HOPS) // at least one more traffic
export const CHANNEL_STAKE = PRICE_PER_PACKET.muln(CT_INTERMEDIATE_HOPS).muln(50) // Fund for new CT channels for 50 packets
export const MESSAGE_FAIL_THRESHOLD = 10 // Failed sends to channel before we autoclose
export const CT_PATH_RANDOMNESS = 0.2
export const CT_NETWORK_QUALITY_THRESHOLD = 0.15 // Minimum channel quality to keep a channel open.
export const CT_OPEN_CHANNEL_QUALITY_THRESHOLD = 0.6 // Minimum channel quality to open a new channel.
export const CT_CHANNEL_STALL_TIMEOUT = 600000 // 10 min. Timeout to close WAIT_FOR_COMMITMENT cover traffic channels.
