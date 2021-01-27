import type { Log } from 'web3-core'
import type { Event, EventData } from './types'
import _abiCoder, { AbiCoder } from 'web3-eth-abi'
import BN from 'bn.js'
import { decodePublicKeysFromTopics } from './utils'

// HACK: wrong types provided by library ¯\_(ツ)_/¯
const abiCoder = (_abiCoder as unknown) as AbiCoder

/**
 * Convert a log data and decoded data to an event.
 * @param log
 * @param name
 * @param data
 * @returns event
 */
export const logToEvent = <N extends keyof EventData>(log: Log, name: N, data: EventData[N]): Event<N> => {
  return {
    name,
    blockNumber: new BN(log.blockNumber),
    transactionHash: log.transactionHash,
    transactionIndex: new BN(log.transactionIndex),
    logIndex: new BN(log.logIndex),
    data
  }
}

export const toFundedChannelEvent = (log: Log): Event<'FundedChannel'> => {
  const { funder, recipientAmount, counterpartyAmount } = abiCoder.decodeLog(
    [
      {
        type: 'uint256',
        name: 'recipient',
        indexed: true
      },
      {
        type: 'uint256',
        name: 'counterparty',
        indexed: true
      },
      {
        type: 'uint256',
        name: 'recipientAmount'
      },
      {
        type: 'uint256',
        name: 'counterpartyAmount'
      },
      {
        type: 'address',
        name: 'funder'
      }
    ],
    log.data,
    log.topics
  )

  const [recipient, counterparty] = decodePublicKeysFromTopics(log.topics)

  return logToEvent(log, 'FundedChannel', {
    funder,
    recipient,
    counterparty,
    recipientAmount: new BN(recipientAmount),
    counterpartyAmount: new BN(counterpartyAmount)
  })
}

export const toOpenedChannelEvent = (log: Log): Event<'OpenedChannel'> => {
  const [opener, counterparty] = decodePublicKeysFromTopics(log.topics)

  return logToEvent(log, 'OpenedChannel', {
    opener,
    counterparty
  })
}

export const toRedeemedTicketEvent = (log: Log): Event<'RedeemedTicket'> => {
  const { amount } = abiCoder.decodeLog(
    [
      {
        type: 'uint256',
        name: 'redeemer',
        indexed: true
      },
      {
        type: 'uint256',
        name: 'counterparty',
        indexed: true
      },
      {
        type: 'uint256',
        name: 'amount'
      }
    ],
    log.data,
    log.topics
  )

  const [redeemer, counterparty] = decodePublicKeysFromTopics(log.topics)

  return logToEvent(log, 'RedeemedTicket', {
    redeemer,
    counterparty,
    amount: new BN(amount)
  })
}

export const toInitiatedChannelClosureEvent = (log: Log): Event<'InitiatedChannelClosure'> => {
  const { closureTime } = abiCoder.decodeLog(
    [
      {
        type: 'uint256',
        name: 'initiator',
        indexed: true
      },
      {
        type: 'uint256',
        name: 'counterparty',
        indexed: true
      },
      {
        type: 'uint256',
        name: 'closureTime'
      }
    ],
    log.data,
    log.topics
  )

  const [initiator, counterparty] = decodePublicKeysFromTopics(log.topics)

  return logToEvent(log, 'InitiatedChannelClosure', {
    initiator,
    counterparty,
    closureTime: new BN(closureTime)
  })
}

export const toClosedChannelEvent = (log: Log): Event<'ClosedChannel'> => {
  const { partyAAmount, partyBAmount } = abiCoder.decodeLog(
    [
      {
        type: 'uint256',
        name: 'closer',
        indexed: true
      },
      {
        type: 'uint256',
        name: 'counterparty',
        indexed: true
      },
      {
        type: 'uint256',
        name: 'partyAAmount'
      },
      {
        type: 'uint256',
        name: 'partyBAmount'
      }
    ],
    log.data,
    log.topics
  )

  const [closer, counterparty] = decodePublicKeysFromTopics(log.topics)

  return logToEvent(log, 'ClosedChannel', {
    closer,
    counterparty,
    partyAAmount: new BN(partyAAmount),
    partyBAmount: new BN(partyBAmount)
  })
}
