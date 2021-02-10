import type { Log } from 'web3-core'
import type { Event } from './types'
import BN from 'bn.js'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { Public, AccountId } from '../../types'

// taken from: https://ropsten.etherscan.io/tx/0xb340f17ea4e2af4cfd294bb8757c3fab0ffa4d4a28c5787c1706fa7b5e27cedc
export const SECRET_HASHED_SET_LOG: Log = {
  address: '0x25e2e5d8ecc4fe46a9505079ed29266779dc7d6f',
  data:
    '0x03edbda20e27c2fe9ad9d3d674e47ba3e71767fd5e11e0eb22832d00000000000000000000000000000000000000000000000000000000000000000000000001',
  topics: [
    '0xe277423b3c010b1e242fb9be2199ad75ffbbc39eea686e8f29edbda512b19354',
    '0x00000000000000000000000071e37fcf79038c8dcf54e3a09ffca835bd024883'
  ],
  blockNumber: 1,
  blockHash: '0x0blockHash',
  transactionHash: '0x0transactionHash',
  transactionIndex: 2,
  logIndex: 3
}

export const SECRET_HASHED_SET_EVENT: Event<'SecretHashSet'> = {
  name: 'SecretHashSet',
  transactionHash: '0x0transactionHash',
  blockNumber: new BN(1),
  transactionIndex: new BN(2),
  logIndex: new BN(3),
  data: {
    account: new AccountId(stringToU8a('0x71e37fcf79038c8dcf54e3a09ffca835bd024883')),
    secretHash: stringToU8a('0x03edbda20e27c2fe9ad9d3d674e47ba3e71767fd5e11e0eb22832d'),
    counter: new BN(1)
  }
}

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
    funder: new AccountId(stringToU8a('0x74cdeAb5AE6efFDC70699731EC3ab55f35fb41eA')),
    recipient: new Public(stringToU8a('0x024c071e97dea5d8b31fa0a6968b15b98ec905d248f7e4df25d889afedeff0b80e')),
    counterparty: new Public(stringToU8a('0x0235b0535ebdb112bf5e03dc37bd12836b6ccc85d5b293cd7c21d1d36b7a7286a9')),
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

// taken from: https://ropsten.etherscan.io/tx/0x56a8123ddbf90de53376854cfdb2ca9b127934bf608aafdccfdb21cb2831c923
export const CLOSING_LOG: Log = {
  address: '0x25e2e5d8ecc4fe46a9505079ed29266779dc7d6f',
  data: '0x00000000000000000000000000000000000000000000000000000000601d1703',
  topics: [
    '0x53c36f9d9c402fa5e1df8409d0e9dbac333aa641c7aad38ef7efbff41377c948',
    '0x855970bfb7ad20c709835b7d8a41b9168939d5ed434ade6a44da4ee612630d87',
    '0xba562689e2010afe6594ad84e0f2c677312b0b21a1d35e8d36fc905662e30ec2'
  ],
  blockNumber: 1,
  blockHash: '0x0blockHash',
  transactionHash: '0x0transactionHash',
  transactionIndex: 2,
  logIndex: 3
}

export const CLOSING_EVENT: Event<'InitiatedChannelClosure'> = {
  name: 'InitiatedChannelClosure',
  transactionHash: '0x0transactionHash',
  blockNumber: new BN(1),
  transactionIndex: new BN(2),
  logIndex: new BN(3),
  data: {
    initiator: new Public(stringToU8a('0x02855970bfb7ad20c709835b7d8a41b9168939d5ed434ade6a44da4ee612630d87')),
    counterparty: new Public(stringToU8a('0x02ba562689e2010afe6594ad84e0f2c677312b0b21a1d35e8d36fc905662e30ec2')),
    closureTime: new BN('1612519171')
  }
}
