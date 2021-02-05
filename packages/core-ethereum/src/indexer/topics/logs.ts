import type { Log } from 'web3-core'
import type { Event, EventData } from './types'
import _abiCoder, { AbiCoder } from 'web3-eth-abi'
import BN from 'bn.js'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { AccountId } from '../../types'
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

// transform log into an event
export const toSecretHashSetEvent = (log: Log): Event<'SecretHashSet'> => {
  console.log(log)

  const { account, secretHash, counter } = abiCoder.decodeLog(
    [
      {
        type: 'address',
        name: 'account',
        indexed: true
      },
      {
        type: 'bytes32',
        name: 'secretHash'
      },
      {
        type: 'uint',
        name: 'counter'
      }
    ],
    log.data,
    log.topics
  )

  return logToEvent(log, 'SecretHashSet', {
    account: new AccountId(stringToU8a(account)),
    secretHash: stringToU8a(secretHash),
    counter: new BN(counter)
  })
}

// transform log into an event
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
    funder: new AccountId(stringToU8a(funder)),
    recipient,
    counterparty,
    recipientAmount: new BN(recipientAmount),
    counterpartyAmount: new BN(counterpartyAmount)
  })
}

// transform log into an event
export const toOpenedChannelEvent = (log: Log): Event<'OpenedChannel'> => {
  const [opener, counterparty] = decodePublicKeysFromTopics(log.topics)

  return logToEvent(log, 'OpenedChannel', {
    opener,
    counterparty
  })
}

// transform log into an event
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

// transform log into an event
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

// transform log into an event
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
