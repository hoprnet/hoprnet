import { BigNumber, utils } from 'ethers'

/**
 * @dev Rewards should be calculated for two blocks: for 1e18 tokens, at a rate of:
 *  - BASE_RATE: 5787
 *  - silver hodler: 158
 *  calculation is done: 1000 * 1e18 * 2 * (5787 + 158) / 1e12;
 * @param baseTokenAmount
 * @param duration
 * @param factors
 * @returns
 */
export const calculateRewards = (baseTokenAmount: number, duration: number, factors: number[]): string => {
  const cumulatedFactors = factors.reduce((acc, cur) => BigNumber.from(cur).add(acc), BigNumber.from(0))
  return utils
    .parseUnits(baseTokenAmount.toFixed(), 'ether')
    .mul(BigNumber.from(duration))
    .mul(cumulatedFactors)
    .div(utils.parseUnits('1.0', 12))
    .toString()
}

export const toSolPercent = (multiplier: number, percent: number): string => {
  return String(Math.floor(percent * multiplier))
}
