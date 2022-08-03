import { utils } from 'ethers'

export const MINTER_ROLE = utils.keccak256(utils.toUtf8Bytes('MINTER_ROLE'))

export const PRIVATE_KEY_ALICE = 'a0a850122d32bb667999d3cf63a0bddc208cee19e2fefa3d0bd495f5fac51ec6'

export const NAME = 'HOPR Boost NFT'
export const SYMBOL = 'HOPR Boost'
export const BASIC_START = 1627387200 // July 27 2021 14:00 CET.
export const PROGRAM_END = 1642424400 // Jan 17 2022 14:00 CET.

export const baseURI = 'hoprboost.eth.limo/'
export const BADGES = [
  {
    type: 'HODLr',
    rank: 'gold',
    deadline: BASIC_START,
    nominator: '317' // 1% APY
  },
  {
    type: 'HODLr',
    rank: 'silver',
    deadline: BASIC_START,
    nominator: '158' // 0.5% APY
  },
  {
    type: 'Testnet participant',
    rank: 'platinum',
    deadline: PROGRAM_END,
    nominator: '317' // 1% APY
  }
]

export const BOOST_CONTRACT_XDAI_PROD = '0x43d13D7B83607F14335cF2cB75E87dA369D056c7'
