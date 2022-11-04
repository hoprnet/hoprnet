import { constants, utils } from 'ethers'

export const MIN_STAKE = utils.parseEther('1000')

const S3_PROGRAM_END = 1658836800
const S4_PROGRAM_END = 1666785600

export const getHoprStakeContractName = (latestBlockTimestamp: number): string => {
  if (latestBlockTimestamp <= S3_PROGRAM_END) {
    // deploy season 3
    return 'HoprStakeSeason3'
  } else if (latestBlockTimestamp <= S4_PROGRAM_END) {
    // deploy season 4
    return 'HoprStakeSeason4'
  } else {
    // deploy season 5
    return 'HoprStakeSeason5'
  }
}

export const NR_NFT_BOOST = 0
export const NR_NFT_TYPE = 'Network_registry'
export const NR_NFT_TYPE_INDEX = 26 // as seen in https://dune.com/queries/837196/1463806
export const NR_NFT_RANK_TECH = 'developer'
export const NR_NFT_RANK_COM = 'community'
export const NR_NFT_MAX_REGISTRATION_TECH = constants.MaxUint256
export const NR_NFT_MAX_REGISTRATION_COM = 1
export type NetworkRegistryNftRank = typeof NR_NFT_RANK_TECH | typeof NR_NFT_RANK_COM
