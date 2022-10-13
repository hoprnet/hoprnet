import * as hre from 'hardhat'
import { type Contract, type Signer, BigNumber, utils } from 'ethers'
import { expect } from 'chai'
import { deployContractFromFactory } from '../utils'

const blockedIndexes = [2, 3, 4, 7, 8, 9, 10, 11, 12, 13]
// known indices that are not blocked and not exceeding 16, with 4 more random uint256 indices
const otherIndices = [1, 5, 6, 14, 15, 16].reduce(
  (acc, cur) => acc.concat(cur.toString()).concat(BigNumber.from(utils.randomBytes(32)).toString()),
  [] as string[]
)

describe('HoprStakeSeason5', function () {
  let deployer: Signer
  let stakeContract: Contract

  describe('unit tests', function () {
    beforeEach(async function () {
      ;[deployer] = await hre.ethers.getSigners()
      stakeContract = await deployContractFromFactory(deployer, 'HoprStakeSeason5')
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
