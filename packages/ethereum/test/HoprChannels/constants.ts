import { ethers } from 'ethers'
import { percentToUint256, createTicket } from './utils'
import { ACCOUNT_A, ACCOUNT_B } from '../constants'

const { solidityKeccak256 } = ethers.utils

// accountA == partyA
// accountB == partyB
export { ACCOUNT_A, ACCOUNT_B }
/**
 * Channel id of account A and B
 */
export const ACCOUNT_AB_CHANNEL_ID = '0xa5bc13ae60ec79a8babc6d0d4074c1cefd5d5fc19fafe71457214d46c90714d8'

export const PROOF_OF_RELAY_SECRET_0 = solidityKeccak256(['string'], ['PROOF_OF_RELAY_SECRET_0'])
export const PROOF_OF_RELAY_SECRET_1 = solidityKeccak256(['string'], ['PROOF_OF_RELAY_SECRET_1'])

export const SECRET_0 = solidityKeccak256(['string'], ['secret'])
export const SECRET_1 = solidityKeccak256(['bytes32'], [SECRET_0])
export const SECRET_2 = solidityKeccak256(['bytes32'], [SECRET_1])

export const WIN_PROB_100 = percentToUint256(100)
export const WIN_PROB_0 = percentToUint256(0)

export const generateTickets = async () => {
  /**
   * Winning ticket created by accountA for accountB
   */
  const TICKET_AB_WIN = await createTicket(
    {
      recipient: ACCOUNT_B.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      counter: '0',
      ticketEpoch: '0',
      ticketIndex: '1',
      amount: '10',
      winProb: WIN_PROB_100,
      channelEpoch: '1'
    },
    ACCOUNT_A,
    SECRET_1
  )

  /**
   * Winning ticket created by accountA for accountB.
   * Compared to TICKET_AB_WIN it has different proof of secret,
   * this effectively makes it a different ticket that can be
   * redeemed.
   */
  const TICKET_AB_WIN_2 = await createTicket(
    {
      recipient: ACCOUNT_B.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_1,
      counter: '0',
      ticketEpoch: '0',
      ticketIndex: '1',
      amount: '10',
      winProb: WIN_PROB_100,
      channelEpoch: '1'
    },
    ACCOUNT_A,
    SECRET_0
  )

  /**
   * Losing ticket created by accountA for accountB
   */
  const TICKET_AB_LOSS = await createTicket(
    {
      recipient: ACCOUNT_B.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      counter: '0',
      ticketEpoch: '0',
      ticketIndex: '1',
      amount: '10',
      winProb: WIN_PROB_0,
      channelEpoch: '1'
    },
    ACCOUNT_A,
    SECRET_1
  )

  /**
   * Winning ticket created by accountB for accountA
   */
  const TICKET_BA_WIN = await createTicket(
    {
      recipient: ACCOUNT_A.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      counter: '0',
      ticketEpoch: '1',
      ticketIndex: '1',
      amount: '10',
      winProb: WIN_PROB_100,
      channelEpoch: '1'
    },
    ACCOUNT_B,
    SECRET_1
  )

  /**
   * Winning ticket created by accountB for accountA.
   * Compared to TICKET_BA_WIN it has different proof of secret,
   * this effectively makes it a different ticket that can be
   * redeemed.
   */
  const TICKET_BA_WIN_2 = await createTicket(
    {
      recipient: ACCOUNT_A.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_1,
      counter: '0',
      ticketEpoch: '2',
      ticketIndex: '1',
      amount: '10',
      winProb: WIN_PROB_100,
      channelEpoch: '1'
    },
    ACCOUNT_B,
    SECRET_0
  )

  return {
    TICKET_AB_WIN,
    TICKET_AB_WIN_2,
    TICKET_AB_LOSS,
    TICKET_BA_WIN,
    TICKET_BA_WIN_2
  }
}
