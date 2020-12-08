import Web3 from 'web3'
import { percentToUint256, createAccount, createTicket } from './utils'

const { soliditySha3 } = Web3.utils

// accountA == partyA
export const ACCOUNT_A = createAccount('0xf54bd518dd7e3e42710e9a96c92b1b244727df5a5afae34611089bee344d6bd4')
// accountB == partyB
export const ACCOUNT_B = createAccount('0xf344315b0389d60ace0c8a5f36da6612d268019c2d88ff77cdb2b37f0ec7ddd5')
/**
 * Channel id of account A and B
 */
export const ACCOUNT_AB_CHANNEL_ID = '0xa5bc13ae60ec79a8babc6d0d4074c1cefd5d5fc19fafe71457214d46c90714d8'

export const SECRET_0 = soliditySha3({ type: 'string', value: 'secret' })
export const SECRET_1 = soliditySha3({ type: 'bytes32', value: SECRET_0 })
export const SECRET_2 = soliditySha3({ type: 'bytes32', value: SECRET_1 })

export const WIN_PROB_100 = percentToUint256(100)
export const WIN_PROB_0 = percentToUint256(0)

/**
 * Winning ticket created by accountA for accountB
 */
export const TICKET_AB_WIN = createTicket(
  {
    recipient: ACCOUNT_B.address,
    proofOfRelaySecret: SECRET_1,
    counter: '1',
    amount: '10',
    winProb: WIN_PROB_100,
    iteration: '1'
  },
  ACCOUNT_A,
  SECRET_1
)

/**
 * Losing ticket created by accountA for accountB
 */
export const TICKET_AB_LOSS = createTicket(
  {
    recipient: ACCOUNT_B.address,
    proofOfRelaySecret: SECRET_1,
    counter: '1',
    amount: '10',
    winProb: WIN_PROB_0,
    iteration: '1'
  },
  ACCOUNT_A,
  SECRET_1
)

/**
 * Winning ticket created by accountB for accountA
 */
export const TICKET_BA_WIN = createTicket(
  {
    recipient: ACCOUNT_A.address,
    proofOfRelaySecret: SECRET_1,
    counter: '1',
    amount: '10',
    winProb: WIN_PROB_100,
    iteration: '1'
  },
  ACCOUNT_B,
  SECRET_1
)
