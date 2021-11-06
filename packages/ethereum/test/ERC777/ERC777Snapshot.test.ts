import type { ERC777SnapshotMock__factory, ERC777SnapshotMock } from '../../src/types'
import { deployments, ethers } from 'hardhat'
import { expect } from 'chai'
import { advanceBlock } from '../utils'
import deployERC1820Registry from '../../deploy/01_ERC1820Registry'

const useFixtures = deployments.createFixture(async (hre) => {
  const [initialHolder, recipient, other] = await ethers.getSigners()
  const HoprTokenFactory = (await hre.ethers.getContractFactory('ERC777SnapshotMock')) as ERC777SnapshotMock__factory

  await deployERC1820Registry(hre, initialHolder)

  const name = 'My Token'
  const symbol = 'MTKN'
  const initialSupply = '100'
  const token = await HoprTokenFactory.deploy(name, symbol, initialHolder.address, initialSupply)

  const initialMintBlock = await ethers.provider.getBlockNumber()

  return {
    initialHolder: initialHolder.address,
    recipient: recipient.address,
    other: other.address,
    token,
    initialSupply,
    initialMintBlock
  }
})

describe('ERC777Snapshot', function () {
  let initialHolder: string
  let recipient: string
  let other: string
  let token: ERC777SnapshotMock
  let initialSupply: string
  let initialMintBlock: number

  beforeEach(async function () {
    const fixtures = await useFixtures()

    initialHolder = fixtures.initialHolder
    recipient = fixtures.recipient
    other = fixtures.other
    token = fixtures.token
    initialSupply = fixtures.initialSupply
    initialMintBlock = fixtures.initialMintBlock
  })

  it('should revert when trying to snapshot unsupported amount', async function () {
    await expect(token.updateValueAtNowAccount(initialHolder, ethers.constants.MaxUint256)).to.be.revertedWith(
      'casting overflow'
    )
  })

  describe('valueAt', function () {
    it('should return account balance 0 at block 0', async function () {
      const balance = await token.getAccountValueAt(initialHolder, 0)
      expect(balance.toString()).to.equal('0')
    })

    it('should return unknown account balance 0 at block 0', async function () {
      const balance = await token.getAccountValueAt(other, 0)
      expect(balance.toString()).to.equal('0')
    })

    it('should return total supply balance 0 at block 0', async function () {
      const totalSupply = await token.getTotalSupplyValueAt(0)
      expect(totalSupply.toString()).to.equal('0')
    })

    it('should return account balance at block', async function () {
      const blockNumber = await ethers.provider.getBlockNumber()
      const blocks = 10

      for (let i = 0; i < blocks; i++) {
        await token.transfer(recipient, 1)
      }

      for (let i = 0; i < blocks; i++) {
        expect((await token.balanceOfAt(recipient, blockNumber + i + 1)).toString()).to.equal(String(i + 1))
      }
    })
  })

  describe('totalSupplyAt', function () {
    it('should return 0 at block 0', async function () {
      const totalSupply = await token.totalSupplyAt(0)
      expect(totalSupply).to.equal('0')
    })

    it('should return latest totalSupply at block number after creation', async function () {
      const totalSupply = await token.totalSupplyAt(initialMintBlock)
      expect(totalSupply).to.equal(initialSupply)
    })

    it('should return latest totalSupply at a not-yet-created block number', async function () {
      const blockNumber = (await ethers.provider.getBlockNumber()) + 1
      const totalSupply = await token.totalSupplyAt(blockNumber)
      expect(totalSupply).to.equal(initialSupply)
    })

    context('with initial snapshot', function () {
      beforeEach(async function () {
        await advanceBlock(ethers.provider)
      })

      context('with no supply changes after the snapshot', function () {
        it('returns the current total supply', async function () {
          const totalSupply = await token.totalSupplyAt(await ethers.provider.getBlockNumber())
          expect(totalSupply).equal(initialSupply)
        })
      })

      context('with supply changes after the snapshot', function () {
        let firstBlockNumber: number

        beforeEach(async function () {
          firstBlockNumber = await ethers.provider.getBlockNumber()
          await token.mint(other, ethers.BigNumber.from('50'), ethers.constants.HashZero, ethers.constants.HashZero)
          await token['burn(address,uint256,bytes,bytes)'](
            initialHolder,
            ethers.BigNumber.from('20'),
            ethers.constants.HashZero,
            ethers.constants.HashZero
          )
        })

        it('returns the total supply before the changes', async function () {
          const totalSupply = await token.totalSupplyAt(firstBlockNumber)
          expect(totalSupply).equal(initialSupply)
        })

        context('with a second snapshot after supply changes', function () {
          let secondBlockNumber: number

          beforeEach(async function () {
            await advanceBlock(ethers.provider)
            secondBlockNumber = await ethers.provider.getBlockNumber()
          })

          it('snapshots return the supply before and after the changes', async function () {
            const totalSupplyFirst = await token.totalSupplyAt(initialMintBlock)
            const totalSupplySecond = await token.totalSupplyAt(secondBlockNumber)

            expect(totalSupplyFirst).to.equal(initialSupply)
            expect(totalSupplySecond).to.equal('130')
            expect(totalSupplySecond).to.equal(await token.totalSupply())
          })
        })

        context('with multiple snapshots after supply changes', function () {
          const blockNumbers: number[] = []

          beforeEach(async function () {
            for (let i = 0; i < 5; i++) {
              await advanceBlock(ethers.provider)
              blockNumbers.push(await ethers.provider.getBlockNumber())
            }
          })

          it('all posterior snapshots return the supply after the changes', async function () {
            expect(await token.totalSupplyAt(initialMintBlock)).to.equal(initialSupply)

            const currentSupply = await token.totalSupply()

            for (const blockNumber of blockNumbers) {
              expect(await token.totalSupplyAt(blockNumber)).to.equal(currentSupply)
            }
          })
        })
      })
    })
  })

  describe('balanceOfAt', function () {
    it('should return 0 at block 0', async function () {
      const balance = await token.balanceOfAt(initialHolder, 0)
      expect(balance).to.equal('0')
    })

    it('should return latest balance at block number after creation', async function () {
      const balance = await token.balanceOfAt(initialHolder, initialMintBlock)
      expect(balance).to.equal(initialSupply)
    })

    it('should return latest balance at a not-yet-created block number', async function () {
      const blockNumber = (await ethers.provider.getBlockNumber()) + 1
      const balance = await token.balanceOfAt(initialHolder, blockNumber)
      expect(balance).to.equal(initialSupply)
    })

    context('with initial snapshot', function () {
      beforeEach(async function () {
        await advanceBlock(ethers.provider)
      })

      context('with no balance changes after the snapshot', function () {
        it('returns the current balance for all accounts', async function () {
          expect(await token.balanceOfAt(initialHolder, initialMintBlock)).equal(initialSupply)
          expect(await token.balanceOfAt(recipient, initialMintBlock)).equal('0')
          expect(await token.balanceOfAt(other, initialMintBlock)).equal('0')
        })
      })

      context('with balance changes after the snapshot', function () {
        beforeEach(async function () {
          await token.transfer(recipient, ethers.BigNumber.from('10'), { from: initialHolder })
          await token.mint(recipient, ethers.BigNumber.from('50'), ethers.constants.HashZero, ethers.constants.HashZero)
          await token['burn(address,uint256,bytes,bytes)'](
            initialHolder,
            ethers.BigNumber.from('20'),
            ethers.constants.HashZero,
            ethers.constants.HashZero
          )
        })

        it('returns the balances before the changes', async function () {
          expect(await token.balanceOfAt(initialHolder, initialMintBlock)).equal(initialSupply)
          expect(await token.balanceOfAt(recipient, initialMintBlock)).equal('0')
          expect(await token.balanceOfAt(other, initialMintBlock)).equal('0')
        })

        context('with a second snapshot after supply changes', function () {
          let firstBlockNumber: number

          beforeEach(async function () {
            firstBlockNumber = await ethers.provider.getBlockNumber()
          })

          it('snapshots return the balances before and after the changes', async function () {
            expect(await token.balanceOfAt(initialHolder, initialMintBlock)).to.equal(initialSupply)
            expect(await token.balanceOfAt(recipient, initialMintBlock)).to.equal('0')
            expect(await token.balanceOfAt(other, initialMintBlock)).to.equal('0')

            expect(await token.balanceOfAt(initialHolder, firstBlockNumber)).to.equal(
              await token.balanceOf(initialHolder)
            )
            expect(await token.balanceOfAt(recipient, firstBlockNumber)).to.equal(await token.balanceOf(recipient))
            expect(await token.balanceOfAt(other, firstBlockNumber)).to.equal(await token.balanceOf(other))
          })
        })

        context('with multiple snapshots after supply changes', function () {
          const blockNumbers: number[] = []

          beforeEach(async function () {
            for (let i = 0; i < 5; i++) {
              await advanceBlock(ethers.provider)
              blockNumbers.push(await ethers.provider.getBlockNumber())
            }
          })

          it('all posterior snapshots return the supply after the changes', async function () {
            expect(await token.balanceOfAt(initialHolder, initialMintBlock)).to.equal(initialSupply)
            expect(await token.balanceOfAt(recipient, initialMintBlock)).to.equal('0')
            expect(await token.balanceOfAt(other, initialMintBlock)).to.equal('0')

            for (const id of blockNumbers) {
              expect(await token.balanceOfAt(initialHolder, id)).to.equal(await token.balanceOf(initialHolder))
              expect(await token.balanceOfAt(recipient, id)).to.equal(await token.balanceOf(recipient))
              expect(await token.balanceOfAt(other, id)).to.equal(await token.balanceOf(other))
            }
          })
        })
      })
    })
  })
})
