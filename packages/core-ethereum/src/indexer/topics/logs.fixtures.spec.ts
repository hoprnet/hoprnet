import type { Log } from 'web3-core'
import type { Event } from './types'
import BN from 'bn.js'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { Public } from '../../types'

// taken from: https://ropsten.etherscan.io/tx/0x94f42fd50e17ea18fb936c3ef9182e8794eaeab00b9d8316373ed942eaf66bf3
export const FUNDED_LOG: Log = {
  address: '0x25e2e5d8ecc4fe46a9505079ed29266779dc7d6f',
  data:
    '0x000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000074cdeab5ae6effdc70699731ec3ab55f35fb41ea',
  topics: [
    '0x47f26a9cfbf6979a650c63549d7f750c0815fc1e0d8d214dadfa8b43bf95a8e0',
    '0x4c071e97dea5d8b31fa0a6968b15b98ec905d248f7e4df25d889afedeff0b80e',
    '0x35b0535ebdb112bf5e03dc37bd12836b6ccc85d5b293cd7c21d1d36b7a7286a9'
  ],
  blockNumber: 1,
  blockHash: '0x0blockHash',
  transactionHash: '0x0transactionHash',
  transactionIndex: 2,
  logIndex: 3
}

export const FUNDED_EVENT: Event<'FundedChannel'> = {
  name: 'FundedChannel',
  transactionHash: '0x0transactionHash',
  blockNumber: new BN(1),
  transactionIndex: new BN(2),
  logIndex: new BN(3),
  data: {
    recipient: new Public(stringToU8a('0x024c071e97dea5d8b31fa0a6968b15b98ec905d248f7e4df25d889afedeff0b80e')),
    counterparty: new Public(stringToU8a('0x0235b0535ebdb112bf5e03dc37bd12836b6ccc85d5b293cd7c21d1d36b7a7286a9')),
    funder: '0x74cdeAb5AE6efFDC70699731EC3ab55f35fb41eA',
    recipientAmount: new BN('100000000000000000'),
    counterpartyAmount: new BN(0)
  }
}

// taken from: https://ropsten.etherscan.io/tx/0xa0ec471bd940a8f80021fd2b196e10bf3690e4f77cd751044e9c3c725712f0b5
export const OPENED_LOG: Log = {
  address: '0x25e2e5d8ecc4fe46a9505079ed29266779dc7d6f',
  data: '0x00000000000000000000000020516d47c46bcd67d19898fda6ae8a68b3022429',
  topics: [
    '0xb987d9e5089b99bfb321b68c309fccc154ae69aaa5d71efd7e12ff8a69f94027',
    '0xc8dea13a24a429dceef64dbb527ec177c3d337eb93b49429c9f0c3ba3b80c475',
    '0xcb00a10ccd6492abf7604a55ea6c76c9b53726733653d80c9236a6e7bc754244'
  ],
  blockNumber: 1,
  blockHash: '0x0blockHash',
  transactionHash: '0x0transactionHash',
  transactionIndex: 2,
  logIndex: 3
}

export const OPENED_EVENT: Event<'OpenedChannel'> = {
  name: 'OpenedChannel',
  transactionHash: '0x0transactionHash',
  blockNumber: new BN(1),
  transactionIndex: new BN(2),
  logIndex: new BN(3),
  data: {
    opener: new Public(stringToU8a('0x03c8dea13a24a429dceef64dbb527ec177c3d337eb93b49429c9f0c3ba3b80c475')),
    counterparty: new Public(stringToU8a('0x03cb00a10ccd6492abf7604a55ea6c76c9b53726733653d80c9236a6e7bc754244'))
  }
}
