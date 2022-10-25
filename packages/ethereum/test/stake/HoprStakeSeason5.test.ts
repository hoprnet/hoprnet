import * as hre from 'hardhat'
import { type Contract, type Signer, BigNumber, utils } from 'ethers'
import { expect } from 'chai'
import { deployContractFromFactory } from '../utils'

const blockedIndexes = [2, 3, 4, 7, 8, 9, 10, 11, 12, 13]
// known indices that are not blocked and not exceeding 16, with a few more random uint256 indices
const otherIndices = [1, 5, 6, 14, 15, 16].reduce((acc, cur) => {
  const randomUint256 = BigNumber.from(utils.randomBytes(32))
  // random uint256 is not the blocked indice
  if (blockedIndexes.findIndex((blocked) => BigNumber.from(blocked).eq(randomUint256)) > -1) {
    acc.concat(cur.toString()).concat(randomUint256.toString())
  }
  return acc
}, [] as string[])

describe('HoprStakeSeason5', function () {
  let deployer: Signer
  let stakeContract: Contract

  describe('unit tests', function () {
    beforeEach(async function () {
      ;[deployer] = await hre.ethers.getSigners()
      stakeContract = await deployContractFromFactory(deployer, 'HoprStakeSeason5', [
        '0xD9a00176Cf49dFB9cA3Ef61805a2850F45Cb1D05',
        '0x43d13D7B83607F14335cF2cB75E87dA369D056c7',
        '0xD057604A14982FE8D88c5fC25Aac3267eA142a08',
        '0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1'
      ])
    })
    describe('A list of Indexes are blocked', function () {
      for (const index of blockedIndexes) {
        it(`checks that nfts index ${index} is blocked`, async function () {
          const isNftBlocked = await stakeContract.isBlockedNft(index)
          expect(isNftBlocked).to.equal(true)
        })
      }
    })
    describe('other indices are not blocked', function () {
      for (const index of otherIndices) {
        it(`checks that nfts index ${index} is not blocked`, async function () {
          const isNftBlocked = await stakeContract.isBlockedNft(index)
          expect(isNftBlocked).to.equal(false)
        })
      }
    })
  })
})
