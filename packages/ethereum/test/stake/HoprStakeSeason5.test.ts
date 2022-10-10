import * as hre from 'hardhat'
import type { Contract, Signer } from 'ethers'
import { expect } from 'chai'
import { deployContractFromFactory } from '../utils'

describe('HoprStakeSeason5', function () {
  let deployer: Signer
  let stakeContract: Contract

  describe('unit tests', function () {
    beforeEach(async function () {
      ;[deployer] = await hre.ethers.getSigners()
      stakeContract = await deployContractFromFactory(deployer, 'HoprStakeSeason5')
    })
    describe('A list of Indexes are blocked', function () {
      const blockedIndexes = [2, 3, 4, 7, 8, 9, 10, 11, 12, 13]
      for (const index of blockedIndexes) {
        it(`checks that nfts index ${index} is blocked`, async function () {
          const isNftBlocked = await stakeContract.isBlockedNft(index)
          expect(isNftBlocked).to.equal(true)
        })
      }
    })
  })
})
