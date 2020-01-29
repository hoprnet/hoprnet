export const CRAWLING_RESPONSE_NODES = 10

// export const RELAY_FEE = toWei('100', 'wei')

export const PACKET_SIZE = 500

export const MAX_HOPS = 3

export const MARSHALLED_PUBLIC_KEY_SIZE = 37

export const NAME = 'ipfs' // 'hopr'

const VERSION = '0.0.1'
const BASESTRING = `/${NAME}/${VERSION}`

export const PROTOCOL_STRING = `${BASESTRING}/msg`

export const PROTOCOL_ACKNOWLEDGEMENT = `${BASESTRING}/ack`

export const PROTOCOL_CRAWLING = `${BASESTRING}/crawl`

export const PROTOCOL_PAYMENT_CHANNEL = `${BASESTRING}/payment/open`

export const PROTOCOL_DELIVER_PUBKEY = `${BASESTRING}/pubKey`

export const PROTOCOL_ONCHAIN_KEY = `${BASESTRING}/onChainKey`

export const PROTOCOL_SETTLE_CHANNEL = `${BASESTRING}/payment/settle`

export const PROTOCOL_STUN = `${BASESTRING}/stun`

export const PROTOCOL_HEARTBEAT = `${BASESTRING}/heartbeat`

export const PROTOCOL_WEBRTC_TURN_REQUEST = `${BASESTRING}/webrtc_turn_request`

export const PROTOCOL_WEBRTC_TURN = `${BASESTRING}/webrtc_turn`

export const STAKE_GAS_AMOUNT = '70000'

export const UNSTAKE_GAS_AMOUNT = '40000'

export const CREATE_GAS_AMOUNT = '150000'

export const FUND_GAS_AMOUNT = '50000'

export const DEFAULT_GAS_AMOUNT = '600000'

export const ETH_SEND_GAS_AMOUNT = '21000'
