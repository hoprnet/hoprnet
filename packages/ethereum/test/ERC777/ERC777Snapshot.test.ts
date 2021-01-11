import type { ERC777SnapshotMockContract, ERC777SnapshotMockInstance } from '../../types'
import { singletons, BN, constants, expectRevert } from '@openzeppelin/test-helpers'
import { vmErrorMessage } from '../utils'

const ERC777SnapshotMock: ERC777SnapshotMockContract = artifacts.require('ERC777SnapshotMock')

describe('ERC777Snapshot', function () {
  const name = 'My Token'
  const symbol = 'MTKN'
  const initialSupply = '100'
  let initialHolder: string
  let recipient: string
  let other: string
  let token: ERC777SnapshotMockInstance
  let initialMintBlock: number

  const triggerSnapshot = () => token.transfer(initialHolder, 1)
  const latestBlockNumber = () => web3.eth.getBlockNumber()

  beforeEach(async function () {
    ;[initialHolder, recipient, other] = await web3.eth.getAccounts()

    await singletons.ERC1820Registry(initialHolder)
    token = await ERC777SnapshotMock.new(name, symbol, initialHolder, initialSupply)
    initialMintBlock = await latestBlockNumber()
  })

  it('should revert when trying to snapshot unsupported amount', async function () {
    await expectRevert(
      token.updateValueAtNowAccount(initialHolder, constants.MAX_UINT256),
      vmErrorMessage('casting overflow')
    )
  })

  describe('totalSupplyAt', function () {
    it('should return 0 at block 0', async function () {
      const totalSupply = await token.totalSupplyAt(0)
      expect(totalSupply.toString()).to.equal('0')
    })

    it('should return latest totalSupply at block number after creation', async function () {
      const totalSupply = await token.totalSupplyAt(initialMintBlock)
      expect(totalSupply.toString()).to.equal(initialSupply)
    })

    it('should return latest totalSupply at a not-yet-created block number', async function () {
      const blockNumber = (await latestBlockNumber()) + 1
      const totalSupply = await token.totalSupplyAt(blockNumber.toString())
      expect(totalSupply.toString()).to.equal(initialSupply)
    })

    context('with initial snapshot', function () {
      beforeEach(async function () {
        await triggerSnapshot()
      })

      context('with no supply changes after the snapshot', function () {
        it('returns the current total supply', async function () {
          const totalSupply = await token.totalSupplyAt(await latestBlockNumber())
          expect(totalSupply.toString()).equal(initialSupply)
        })
      })

      context('with supply changes after the snapshot', function () {
        let firstBlockNumber: number

        beforeEach(async function () {
          firstBlockNumber = await latestBlockNumber()
          await token.mint(other, new BN('50'), '0x00', '0x00')
          await token.methods['burn(address,uint256,bytes,bytes)'](initialHolder, new BN('20'), '0x00', '0x00')
        })

        it('returns the total supply before the changes', async function () {
          const totalSupply = await token.totalSupplyAt(firstBlockNumber)
          expect(totalSupply.toString()).equal(initialSupply)
        })

        context('with a second snapshot after supply changes', function () {
          let secondBlockNumber: number

          beforeEach(async function () {
            await triggerSnapshot()
            secondBlockNumber = await latestBlockNumber()
          })

          it('snapshots return the supply before and after the changes', async function () {
            const totalSupplyFirst = await token.totalSupplyAt(initialMintBlock)
            const totalSupplySecond = await token.totalSupplyAt(secondBlockNumber)

            expect(totalSupplyFirst.toString()).to.equal(initialSupply)
            expect(totalSupplySecond.toString()).to.equal('130')
            expect(totalSupplySecond.toString()).to.equal((await token.totalSupply()).toString())
          })
        })

        context('with multiple snapshots after supply changes', function () {
          const blockNumbers: number[] = []

          beforeEach(async function () {
            for (let i = 0; i < 5; i++) {
              await triggerSnapshot()
              blockNumbers.push(await latestBlockNumber())
            }
          })

          it('all posterior snapshots return the supply after the changes', async function () {
            expect((await token.totalSupplyAt(initialMintBlock)).toString()).to.equal(initialSupply)

            const currentSupply = await token.totalSupply()

            for (const blockNumber of blockNumbers) {
              expect((await token.totalSupplyAt(blockNumber)).toString()).to.equal(currentSupply.toString())
            }
          })
        })
      })
    })
  })

  describe('balanceOfAt', function () {
    it('should return 0 at block 0', async function () {
      const balance = await token.balanceOfAt(initialHolder, 0)
      expect(balance.toString()).to.equal('0')
    })

    it('should return latest balance at block number after creation', async function () {
      const balance = await token.balanceOfAt(initialHolder, initialMintBlock)
      expect(balance.toString()).to.equal(initialSupply)
    })

    it('should return latest balance at a not-yet-created block number', async function () {
      const blockNumber = (await latestBlockNumber()) + 1
      const balance = await token.balanceOfAt(initialHolder, blockNumber.toString())
      expect(balance.toString()).to.equal(initialSupply)
    })

    context('with initial snapshot', function () {
      beforeEach(async function () {
        await triggerSnapshot()
      })

      context('with no balance changes after the snapshot', function () {
        it('returns the current balance for all accounts', async function () {
          expect((await token.balanceOfAt(initialHolder, initialMintBlock)).toString()).equal(initialSupply)
          expect((await token.balanceOfAt(recipient, initialMintBlock)).toString()).equal('0')
          expect((await token.balanceOfAt(other, initialMintBlock)).toString()).equal('0')
        })
      })

      context('with balance changes after the snapshot', function () {
        beforeEach(async function () {
          await token.transfer(recipient, new BN('10'), { from: initialHolder })
          await token.mint(recipient, new BN('50'), '0x00', '0x00')
          await token.methods['burn(address,uint256,bytes,bytes)'](initialHolder, new BN('20'), '0x00', '0x00')
        })

        it('returns the balances before the changes', async function () {
          expect((await token.balanceOfAt(initialHolder, initialMintBlock)).toString()).equal(initialSupply)
          expect((await token.balanceOfAt(recipient, initialMintBlock)).toString()).equal('0')
          expect((await token.balanceOfAt(other, initialMintBlock)).toString()).equal('0')
        })

        context('with a second snapshot after supply changes', function () {
          let firstBlockNumber: number

          beforeEach(async function () {
            firstBlockNumber = await latestBlockNumber()
          })

          it('snapshots return the balances before and after the changes', async function () {
            expect((await token.balanceOfAt(initialHolder, initialMintBlock)).toString()).to.equal(initialSupply)
            expect((await token.balanceOfAt(recipient, initialMintBlock)).toString()).to.equal('0')
            expect((await token.balanceOfAt(other, initialMintBlock)).toString()).to.equal('0')

            expect((await token.balanceOfAt(initialHolder, firstBlockNumber)).toString()).to.equal(
              (await token.balanceOf(initialHolder)).toString()
            )
            expect((await token.balanceOfAt(recipient, firstBlockNumber)).toString()).to.equal(
              (await token.balanceOf(recipient)).toString()
            )
            expect((await token.balanceOfAt(other, firstBlockNumber)).toString()).to.equal(
              (await token.balanceOf(other)).toString()
            )
          })
        })

        context('with multiple snapshots after supply changes', function () {
          const blockNumbers: number[] = []

          beforeEach(async function () {
            for (let i = 0; i < 5; i++) {
              await triggerSnapshot()
              blockNumbers.push(await latestBlockNumber())
            }
          })

          it('all posterior snapshots return the supply after the changes', async function () {
            expect((await token.balanceOfAt(initialHolder, initialMintBlock)).toString()).to.equal(initialSupply)
            expect((await token.balanceOfAt(recipient, initialMintBlock)).toString()).to.equal('0')
            expect((await token.balanceOfAt(other, initialMintBlock)).toString()).to.equal('0')

            for (const id of blockNumbers) {
              expect((await token.balanceOfAt(initialHolder, id)).toString()).to.equal(
                (await token.balanceOf(initialHolder)).toString()
              )
              expect((await token.balanceOfAt(recipient, id)).toString()).to.equal(
                (await token.balanceOf(recipient)).toString()
              )
              expect((await token.balanceOfAt(other, id)).toString()).to.equal(
                (await token.balanceOf(other)).toString()
              )
            }
          })
        })
      })
    })
  })
})
