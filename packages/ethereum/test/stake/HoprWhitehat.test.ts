import * as hre from 'hardhat'
import { constants, Contract, Signer, utils } from 'ethers'
import { expect } from 'chai'
import deployERC1820Registry, { ERC1820_REGISTRY_ABI, ERC1820_REGISTRY_ADDRESS } from '../../deploy/01_ERC1820Registry'
import { advanceTimeForNextBlock, deployContractFromFactory, latestBlockTime } from '../utils'

describe('HoprWhitehat', function () {
  let deployer: Signer
  let admin: Signer
  let stakers: Signer[]

  let deployerAddress: string
  let adminAddress: string
  let stakerAddresses: string[]

  let nftContract: Contract
  let stakeContract: Contract
  let whitehatContract: Contract
  let erc1820: Contract
  let xHopr: Contract // ERC677
  let wxHopr: Contract // ERC777

  const PROGRAM_S1_END = 1642424400 // Jan 17 2022 14:00 CET.
  // all valid NFTs types and ranks
  const BADGES = [
    {
      type: 'HODLr',
      rank: 'silver',
      deadline: PROGRAM_S1_END,
      nominator: '158' // 0.5% APY
    },
    {
      type: 'Wildhorn',
      rank: 'bronze',
      deadline: PROGRAM_S1_END,
      nominator: '317' // 1% APY
    },
    {
      type: 'DAOv2',
      rank: 'gold',
      deadline: PROGRAM_S1_END,
      nominator: '100'
    }
  ]

  const reset = async () => {
    let signers: Signer[]
    ;[deployer, admin, ...signers] = await hre.ethers.getSigners()
    stakers = signers.slice(3, 7) // 4 stakers

    deployerAddress = await deployer.getAddress()
    adminAddress = await admin.getAddress()
    stakerAddresses = await Promise.all(stakers.map((s) => s.getAddress()))

    // set 1820 registry
    await deployERC1820Registry(hre, deployer)
    // erc1820 = await deployRegistry(deployer);
    erc1820 = await hre.ethers.getContractAt(ERC1820_REGISTRY_ABI, ERC1820_REGISTRY_ADDRESS)
    // set stake and reward tokens
    xHopr = await deployContractFromFactory(deployer, 'ERC677Mock')
    // erc777 is the reward token (wxHOPR). admin account holds 5 million reward tokens
    wxHopr = await deployContractFromFactory(deployer, 'ERC777Mock', [
      adminAddress,
      utils.parseUnits('5000000', 'ether'),
      'ERC777Mock',
      'M777',
      [adminAddress]
    ])

    // create NFT and stake contract
    nftContract = await deployContractFromFactory(deployer, 'HoprBoost', [adminAddress, ''])
    stakeContract = await deployContractFromFactory(deployer, 'HoprStake', [
      nftContract.address,
      adminAddress,
      xHopr.address,
      wxHopr.address
    ])

    // create whitehat contract
    whitehatContract = await deployContractFromFactory(deployer, 'HoprWhitehat', [
      adminAddress,
      nftContract.address,
      stakeContract.address,
      xHopr.address,
      wxHopr.address
    ])

    // airdrop some NFTs of type (0,1 and 2) to staker[0]
    await Promise.all(
      BADGES.map((badge) =>
        nftContract.connect(admin).mint(stakerAddresses[0], badge.type, badge.rank, badge.nominator, badge.deadline)
      )
    )
    // airdrop some NFTs of type (0,1 and 2) to staker[2]
    await Promise.all(
      BADGES.map((badge) =>
        nftContract.connect(admin).mint(stakerAddresses[2], badge.type, badge.rank, badge.nominator, badge.deadline)
      )
    )
    // staker[1] and staker[3] do not have any NFTs

    // airdrop some ERC677 to participants
    await xHopr.batchMintInternal(stakerAddresses, utils.parseUnits('10000', 'ether')) // each participant holds 10k xHOPR
    await wxHopr.mintInternal(adminAddress, utils.parseUnits('5000000', 'ether'), '0x', '0x') // admin account holds 5 million wxHOPR

    // activate
    await whitehatContract.connect(admin).activate()
    // -----logs
    console.table([
      ['Deployer', deployerAddress],
      ['Admin', adminAddress],
      ['NFT Contract', nftContract.address],
      ['Stake Contract', stakeContract.address],
      ['Stakers', JSON.stringify(stakerAddresses)]
    ])
  }

  const checkAccount = async (accountAddr: string, stakeInEth: string) => {
    const currentAccount = await stakeContract.accounts(accountAddr)
    expect(currentAccount[0].toString()).to.equal(utils.parseUnits(stakeInEth, 'ether').toString()) // actualLockedTokenAmount
    expect(currentAccount[1].toString()).to.equal('0') // virtualLockedTokenAmount
    // skip checking lastSyncTimestamp
    console.log(currentAccount[3].toString(), currentAccount[4].toString())
    // expect(currentAccount[4].toString()).to.equal('0'); // claimedRewards
  }
  describe('unit tests', function () {
    before(async function () {
      await reset()
    })
    it('implements ERC777 tokensReceived hook', async function () {
      const interfaceHash = utils.keccak256(utils.toUtf8Bytes('ERC777TokensRecipient'))
      const implementer = await erc1820.getInterfaceImplementer(stakeContract.address, interfaceHash)
      expect(interfaceHash).to.equal('0xb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b')
      expect(implementer).to.equal(stakeContract.address)
    })
    it('participants have received ERC721', async function () {
      expect((await nftContract.tokenOfOwnerByIndex(stakerAddresses[0], 0)).toString()).to.equal(
        constants.Zero.toString()
      )
    })
    it('has the correct address setup for the whitehat contract', async function () {
      expect((await whitehatContract.myHoprBoost()).toString()).to.equal(nftContract.address)
      expect((await whitehatContract.myHoprStake()).toString()).to.equal(stakeContract.address)
      expect((await whitehatContract.wxHopr()).toString()).to.equal(wxHopr.address)
      expect((await whitehatContract.xHopr()).toString()).to.equal(xHopr.address)
    })

    describe('LOCK_TOKEN and other ERC677 token', () => {
      let randomERC677
      before(async function () {
        randomERC677 = await deployContractFromFactory(deployer, 'ERC677Mock')

        await randomERC677.batchMintInternal(stakerAddresses, utils.parseUnits('10000', 'ether')) // each participant holds 10k randomERC677
      })
      it('can receive random 677 with `transferAndCall()`', async () => {
        await expect(
          randomERC677
            .connect(stakers[0])
            .transferAndCall(whitehatContract.address, utils.parseUnits('1000', 'ether'), '0x')
        )
          .to.emit(whitehatContract, 'Received677')
          .withArgs(randomERC677.address, stakerAddresses[0], utils.parseUnits('1000', 'ether').toString())
        expect((await randomERC677.balanceOf(stakerAddresses[0])).toString()).to.equal(
          utils.parseUnits('9000', 'ether').toString()
        )
      })
      it('does not update rescuedXHoprAmount for random677', async () => {
        expect((await whitehatContract.rescuedXHoprAmount()).toString()).to.equal('0') // not updated
      })
      it('can receive xHopr with `transferAndCall()`', async () => {
        await expect(
          xHopr.connect(stakers[0]).transferAndCall(whitehatContract.address, utils.parseUnits('1000', 'ether'), '0x')
        )
          .to.emit(whitehatContract, 'Received677')
          .withArgs(xHopr.address, stakerAddresses[0], utils.parseUnits('1000', 'ether').toString())
        expect((await xHopr.balanceOf(stakerAddresses[0])).toString()).to.equal(
          utils.parseUnits('9000', 'ether').toString()
        )
      })
      it('does update rescuedXHoprAmount for random677', async () => {
        expect((await whitehatContract.rescuedXHoprAmount()).toString()).to.equal(
          utils.parseUnits('1000', 'ether').toString()
        ) // rescuedXHoprAmount should be updated
      })
    })

    describe('REWARD_TOKEN and other ERC777 token', () => {
      let randomERC777
      before(async function () {
        randomERC777 = await deployContractFromFactory(deployer, 'ERC777Mock', [
          adminAddress,
          utils.parseUnits('5000000', 'ether'),
          'ERC777Mock',
          'M777',
          [adminAddress]
        ])
        await randomERC777.mintInternal(stakerAddresses[0], utils.parseUnits('5000000', 'ether'), '0x', '0x') // admin account holds 5 million random erc777
        await wxHopr.mintInternal(stakerAddresses[0], utils.parseUnits('1', 'ether'), '0x', '0x') // admin account holds 5 million wxHOPR
      })
      describe('global switch is on', () => {
        it('check switch is on', async () => {
          expect(await whitehatContract.isActive()).to.equal(true) // default value: true
        })
        it('can not receive random 777 with `send()` when the globalSwitch is on', async () => {
          await expect(
            randomERC777.connect(stakers[0]).send(whitehatContract.address, constants.One, '0x')
          ).to.be.revertedWith('can only be called from wxHOPR')
        })
        it('can receive wxHopr with `send()` when the globalSwitch is on', async () => {
          await expect(wxHopr.connect(stakers[0]).send(whitehatContract.address, constants.One, '0x'))
            .to.emit(whitehatContract, 'Called777HookForFunding')
            .withArgs(wxHopr.address, stakerAddresses[0], '1')
          expect((await wxHopr.balanceOf(whitehatContract.address)).toString()).to.equal('1')
        })
      })
    })

    describe('nftBoost and other ERC721 tokens', function () {
      before(async function () {
        await reset()
        // stakers stake all the NFTs and 1000 xHOPR token
        const nftIds = Array.from({ length: 6 }, (v, i) => i)
        await Promise.all(
          nftIds.map((nftId) => {
            const ownerIndex = nftId < 3 ? 0 : 2 // [0, 1, 2]: staker0; [3, 4, 5]: staker2
            return nftContract
              .connect(stakers[ownerIndex])
              .functions['safeTransferFrom(address,address,uint256)'](
                stakerAddresses[ownerIndex],
                stakeContract.address,
                nftId
              )
          })
        )
        // transfer ownership
        await stakeContract.connect(admin).transferOwnership(whitehatContract.address)
      })
      it('nfts are staked in the staking contract', async () => {
        expect((await nftContract.ownerOf(0)).toString()).to.equal(stakeContract.address)
        expect((await nftContract.ownerOf(1)).toString()).to.equal(stakeContract.address)
        expect((await nftContract.ownerOf(2)).toString()).to.equal(stakeContract.address)
        expect((await nftContract.ownerOf(3)).toString()).to.equal(stakeContract.address)
        expect((await nftContract.ownerOf(4)).toString()).to.equal(stakeContract.address)
        expect((await nftContract.ownerOf(5)).toString()).to.equal(stakeContract.address)
      })
      it('allows owner to reclaim boost nfts in batch', async () => {
        await expect(whitehatContract.connect(admin).ownerRescueBoosterNftInBatch(stakerAddresses[0]))
          .to.emit(whitehatContract, 'ReclaimedBoost')
          .withArgs(stakerAddresses[0], 0)
          .to.emit(whitehatContract, 'ReclaimedBoost')
          .withArgs(stakerAddresses[0], 1)
          .to.emit(whitehatContract, 'ReclaimedBoost')
          .withArgs(stakerAddresses[0], 2)
      })
      it('hopr boost nfts are returned to original stakers', async () => {
        expect((await nftContract.ownerOf(0)).toString()).to.equal(stakerAddresses[0])
        expect((await nftContract.ownerOf(1)).toString()).to.equal(stakerAddresses[0])
        expect((await nftContract.ownerOf(2)).toString()).to.equal(stakerAddresses[0])
      })
    })
  })

  describe('Integration: When s1 PROGRAM_S1_END', function () {
    const INITIAL_STAKE = '1000'
    before(async function () {
      await reset()
      // Before staking program ends
      // stakers stake all the NFTs and 1000 xHOPR token
      const nftIds = Array.from({ length: 6 }, (v, i) => i)
      await Promise.all(
        nftIds.map((nftId) => {
          const ownerIndex = nftId < 3 ? 0 : 2 // [0, 1, 2]: staker0; [3, 4, 5]: staker2
          return nftContract
            .connect(stakers[ownerIndex])
            .functions['safeTransferFrom(address,address,uint256)'](
              stakerAddresses[ownerIndex],
              stakeContract.address,
              nftId
            )
        })
      )
      // stakers stake 1000 xHOPR token
      await Promise.all(
        stakers.map((staker) =>
          xHopr.connect(staker).transferAndCall(stakeContract.address, utils.parseUnits(INITIAL_STAKE, 'ether'), '0x')
        )
      )
      // provide 100k reward
      await wxHopr.connect(admin).send(stakeContract.address, utils.parseUnits('100000', 'ether'), '0x')
    })
    it('succeeds in advancing block to PROGRAM_S1_END', async function () {
      await advanceTimeForNextBlock(hre.ethers.provider, PROGRAM_S1_END)
      const blockTime = await latestBlockTime(hre.ethers.provider)
      expect(blockTime.toString()).to.equal(PROGRAM_S1_END.toString())
    })
    describe('After v1 PROGRAM_S1_END', function () {
      const deadLocked = [0, 1]
      const canLock = [2, 3]
      before(async function () {
        // staker 0, 1 are deadlocked
        await Promise.all(
          deadLocked.map((stakerIndex) =>
            stakeContract.connect(stakers[stakerIndex]).claimRewards(stakerAddresses[stakerIndex])
          )
        )
        await checkAccount(stakerAddresses[0], INITIAL_STAKE)
      })
      deadLocked.map((lockedIndex) => {
        it(`cannnot unlock for deadlocked staker ${lockedIndex}`, async function () {
          await expect(
            stakeContract.connect(stakers[lockedIndex]).unlock(stakerAddresses[lockedIndex])
          ).to.be.revertedWith('HoprStake: Nothing to claim')
        })
      })
      describe('Use WhiteHat contract', function () {
        let xHoprBalanceInStakeBefore
        const interfaceHash = utils.keccak256(utils.toUtf8Bytes('ERC777TokensRecipient'))
        before(async function () {
          // transfer ownership
          await stakeContract.connect(admin).transferOwnership(whitehatContract.address)
          // canLock stakers set implementer
          await Promise.all(
            canLock.map((stakerIndex) =>
              erc1820
                .connect(stakers[stakerIndex])
                .setInterfaceImplementer(stakerAddresses[stakerIndex], interfaceHash, whitehatContract.address)
            )
          )
          // whitehat contract has enough wxHOPR, by providing 500k reward
          await wxHopr.connect(admin).send(whitehatContract.address, utils.parseUnits('500000', 'ether'), '0x')
        })
        canLock.map((stakerIndex) => {
          it(`should have the right implementer for staker ${stakerIndex}`, async function () {
            const implementer = await erc1820.getInterfaceImplementer(stakerAddresses[stakerIndex], interfaceHash)
            expect(implementer).to.equal(whitehatContract.address)
          })
        })
        it(`should have balance of 4000 xhopr, before calling gimme`, async function () {
          xHoprBalanceInStakeBefore = await xHopr.balanceOf(stakeContract.address)
          expect(xHoprBalanceInStakeBefore).to.equal(utils.parseUnits('4000', 'ether').toString())
          // each account has actual stake of 1000
          await Promise.all(stakerAddresses.map((account) => checkAccount(account, '1000')))
        })
        // canLock.map((stakerIndex) => {
        //     it(`can lock for others ${stakerIndex}`, async function () {
        //         await whitehatContract.connect(stakers[stakerIndex]).gimmeToken()
        //     });
        // })
        // it(`should have balance of 0 xhopr, after calling gimme`, async function () {
        //     const xHoprBalanceInStakeAfter = await xHopr.balanceOf(stakeContract.address);
        //     expect(xHoprBalanceInStakeAfter).to.equal('0');
        // });
        it(`staker 2 can lock for others`, async function () {
          await whitehatContract.connect(stakers[2]).gimmeToken()
        })
        it(`should have balance of 2000 xhopr, after calling gimme`, async function () {
          const xHoprBalanceInStakeAfter = await xHopr.balanceOf(stakeContract.address)
          expect(xHoprBalanceInStakeAfter).to.equal(utils.parseUnits('2000', 'ether').toString())
        })
        it(`staker 3 can lock for others`, async function () {
          await whitehatContract.connect(stakers[3]).gimmeToken()
        })
        it(`should have balance of 1000 xhopr, after calling gimme`, async function () {
          const xHoprBalanceInStakeAfter = await xHopr.balanceOf(stakeContract.address)
          expect(xHoprBalanceInStakeAfter).to.equal(utils.parseUnits('1000', 'ether').toString())
        })
      })
    })
  })
})
