import { utils } from 'ethers'

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

export const DEV_NFT_BOOST = 0
