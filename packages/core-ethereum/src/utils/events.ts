import { Public } from '../types'
import { u8aToHex, u8aConcat, stringToU8a } from '@hoprnet/hopr-utils'
import createKeccakHash from 'keccak'
import type { Log } from 'web3-core'
import BN from 'bn.js'

const rawOpenedChannelTopic = createKeccakHash('keccak256').update('OpenedChannel(uint,uint)').digest()
const rawClosedChannelTopic = createKeccakHash('keccak256').update('ClosedChannel(uint,uint,uint,uint)').digest()

export function OpenedChannelTopics(
  opener?: Public,
  counterparty?: Public,
  bidirectional?: boolean
): (undefined | string | string[])[] {
  return getTopics(rawOpenedChannelTopic, opener, counterparty, bidirectional)
}

export function ClosedChannelTopics(
  closer?: Public,
  counterparty?: Public,
  bidirectional?: boolean
): (undefined | string | string[])[] {
  return getTopics(rawClosedChannelTopic, closer, counterparty, bidirectional)
}

function getTopics(
  rawTopic: Uint8Array,
  first?: Public,
  second?: Public,
  bidirectional?: boolean
): (undefined | string | string[])[] {
  if (bidirectional && (first == null || second == null)) {
    throw Error(`Bidirectional property can only be used if 'first' and 'second' are set`)
  }

  const topic0 = []

  if ((first == null && second == null) || (bidirectional && first[0] % 4 != second[0] % 4)) {
    for (let i = 0; i < 4; i++) {
      topic0.push(getTopic0(rawTopic, (i >> 1) % 2, i % 2))
    }
  } else if (first != null) {
    for (let i = 0; i < 2; i++) {
      topic0.push(getTopic0(rawTopic, first[0], i % 2))
    }
  } else if (second != null) {
    for (let i = 0; i < 2; i++) {
      topic0.push(getTopic0(rawTopic, (i >> 1) % 2, second[0]))
    }
  } else {
    topic0.push(getTopic0(rawTopic, first[0], second[0]))
  }

  if (bidirectional) {
    return [
      topic0,
      [u8aToHex(first.slice(1, 33)), u8aToHex(second.slice(1, 33))],
      [u8aToHex(first.slice(1, 33)), u8aToHex(second.slice(1, 33))]
    ]
  } else {
    return [
      topic0,
      first != null ? u8aToHex(first.slice(1, 33)) : undefined,
      second != null ? u8aToHex(second.slice(1, 33)) : undefined
    ]
  }
}

function getTopic0(rawTopic: Uint8Array, first: number, second: number) {
  return u8aToHex(
    u8aConcat(rawTopic.slice(0, 31), new Uint8Array([((rawTopic[31] >> 2) << 2) | (first % 2 << 1) | second % 2]))
  )
}

export function decodeOpenedChannelEvent(_event: Log) {
  return {
    event: 'OpenedChannel',
    blockNumber: _event.blockNumber,
    transactionHash: _event.transactionHash,
    transactionIndex: _event.transactionIndex,
    logIndex: _event.logIndex,
    returnValues: {
      opener: new Public(
        u8aConcat(
          new Uint8Array([2 + ((parseInt(_event.topics[0].slice(64, 66), 16) >> 1) % 2)]),
          stringToU8a(_event.topics[1])
        )
      ),
      counterparty: new Public(
        u8aConcat(
          new Uint8Array([2 + (parseInt(_event.topics[0].slice(64, 66), 16) % 2)]),
          stringToU8a(_event.topics[2])
        )
      )
    }
  }
}

export function decodeClosedChannelEvent(_event: Log) {
  return {
    event: 'ClosedChannel',
    blockNumber: _event.blockNumber,
    transactionHash: _event.transactionHash,
    transactionIndex: _event.transactionIndex,
    logIndex: _event.logIndex,
    returnValues: {
      closer: new Public(
        u8aConcat(
          new Uint8Array([2 + ((parseInt(_event.topics[0].slice(64, 66), 16) >> 1) % 2)]),
          stringToU8a(_event.topics[1])
        )
      ),
      counterparty: new Public(
        u8aConcat(
          new Uint8Array([2 + (parseInt(_event.topics[0].slice(64, 66), 16) % 2)]),
          stringToU8a(_event.topics[2])
        )
      ),
      partyAAmount: new BN(_event.data.slice(-128, -64), 16),
      partyBAmount: new BN(_event.data.slice(-64), 16)
    }
  }
}
