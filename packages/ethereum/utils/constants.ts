import { constants, utils } from 'ethers'

export const MIN_STAKE = utils.parseEther('1000')

const S3_PROGRAM_END = 1658836800

export const getHoprStakeContractName = (latestBlockTimestamp: number): string => {
  if (latestBlockTimestamp <= S3_PROGRAM_END) {
    // deploy season 3
    return 'HoprStakeSeason3'
  } else {
    // deploy season 4
    return 'HoprStakeSeason4'
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

export const CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES = [
  '0x6c150A63941c6d58a2f2687a23d5a8E0DbdE181C', // nat_with_stake
  '0x0Fd4C32CC8C6237132284c1600ed94D06AC478C6', // nat_with_nft
  '0xBA28EE6743d008ed6794D023B10D212bc4Eb7e75', // public_with_stake
  '0xf84Ba32dd2f2EC2F355fB63F3fC3e048900aE3b2' // public_with_nft
]
