import * as hre from 'hardhat'
import { BigNumber, constants, Contract, Signer, utils } from 'ethers'
import { expect } from 'chai'
import deployERC1820Registry, { ERC1820_REGISTRY_ABI, ERC1820_REGISTRY_ADDRESS } from '../../deploy/01_ERC1820Registry'
import { advanceTimeForNextBlock, calculateRewards, deployContractFromFactory, latestBlockTime } from '../utils'

describe('HoprStake', function () {
  let deployer: Signer
  let admin: Signer
  let participants: Signer[]

  let deployerAddress: string
  let adminAddress: string
  let participantAddresses: string[]

  let nftContract: Contract
  let stakeContract: Contract
  let stake2Contract: Contract
  let erc1820: Contract
  let erc677: Contract
  let erc777: Contract

  const BASIC_START = 1627387200 // July 27 2021 14:00 CET.
  const PROGRAM_END = 1642424400 // Jan 17 2022 14:00 CET.
  const BASIC_FACTOR_NUMERATOR = 5787
  const BADGES = [
    {
      type: 'HODLr',
      rank: 'silver',
      deadline: BASIC_START,
      nominator: '158' // 0.5% APY
    },
    {
      type: 'HODLr',
      rank: 'platinum',
      deadline: PROGRAM_END,
      nominator: '317' // 1% APY
    },
    {
      type: 'Past',
      rank: 'gold',
      deadline: 123456, // sometime long long ago
      nominator: '100'
    },
    {
      type: 'HODLr',
      rank: 'bronze extra',
      deadline: PROGRAM_END,
      nominator: '79' // 0.25% APY
    },
    {
      type: 'Testnet participant',
      rank: 'gold',
      deadline: PROGRAM_END,
      nominator: '317' // 0.25% APY
    }
  ]

  const reset = async () => {
    let signers: Signer[]
    ;[deployer, admin, ...signers] = await hre.ethers.getSigners()
    participants = signers.slice(3, 6) // 3 participants

    deployerAddress = await deployer.getAddress()
    adminAddress = await admin.getAddress()
    participantAddresses = await Promise.all(participants.map((h) => h.getAddress()))

    // set 1820 registry
    await deployERC1820Registry(hre, deployer)
    // erc1820 = await deployRegistry(deployer);
    erc1820 = await hre.ethers.getContractAt(ERC1820_REGISTRY_ABI, ERC1820_REGISTRY_ADDRESS)
    // set stake and reward tokens
    erc677 = await deployContractFromFactory(deployer, 'ERC677Mock')
    // erc777 is the reward token (wxHOPR). admin account holds 5 million reward tokens
    erc777 = await deployContractFromFactory(deployer, 'ERC777Mock', [
      adminAddress,
      utils.parseUnits('5000000', 'ether'),
      'ERC777Mock',
      'M777',
      []
    ])

    // create NFT and stake contract
    nftContract = await deployContractFromFactory(deployer, 'HoprBoost', [adminAddress, ''])
    stakeContract = await deployContractFromFactory(deployer, 'HoprStake', [
      nftContract.address,
      adminAddress,
      erc677.address,
      erc777.address
    ])
    stake2Contract = await deployContractFromFactory(deployer, 'HoprStake2', [
      nftContract.address,
      adminAddress,
      erc677.address,
      erc777.address
    ])

    // airdrop some NFTs (0,1 and 2) to participants
    await nftContract
      .connect(admin)
      .batchMint(
        participantAddresses.slice(0, 2),
        BADGES[0].type,
        BADGES[0].rank,
        BADGES[0].nominator,
        BADGES[0].deadline
      )
    await nftContract
      .connect(admin)
      .mint(participantAddresses[0], BADGES[1].type, BADGES[1].rank, BADGES[1].nominator, BADGES[1].deadline)
    // airdrop some ERC677 to participants
    await erc677.batchMintInternal(participantAddresses, utils.parseUnits('10000', 'ether')) // each participant holds 10k xHOPR
    // airdrop some NFTs (3,4 and 5) to participants
    await nftContract
      .connect(admin)
      .batchMint(
        participantAddresses.slice(0, 2),
        BADGES[0].type,
        BADGES[0].rank,
        BADGES[0].nominator,
        BADGES[0].deadline
      )
    await nftContract
      .connect(admin)
      .mint(participantAddresses[0], BADGES[1].type, BADGES[1].rank, BADGES[1].nominator, BADGES[1].deadline)

    // -----logs
    console.table([
      ['Deployer', deployerAddress],
      ['Admin', adminAddress],
      ['NFT Contract', nftContract.address],
      ['Stake Contract', stakeContract.address],
      ['participant', JSON.stringify(participantAddresses)]
    ])
  }

  describe('Staking season 1 integration tests', function () {
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
      expect((await nftContract.tokenOfOwnerByIndex(participantAddresses[0], 0)).toString()).to.equal(
        constants.Zero.toString()
      )
    })

    describe('LOCK_TOKEN and other ERC677 token', () => {
      let randomERC677

      it('cannot receive random 677 with `transferAndCall()`', async () => {
        randomERC677 = await deployContractFromFactory(deployer, 'ERC677Mock')
        await randomERC677.batchMintInternal(participantAddresses, utils.parseUnits('10000', 'ether')) // each participant holds 10k randomERC677
        // Revert message was bubbled up, showing only the one from ERC677Mock
        await expect(
          randomERC677.connect(participants[2]).transferAndCall(stakeContract.address, constants.One, '0x')
        ).to.be.revertedWith('ERC677Mock: failed when calling onTokenTransfer')
      })

      it('can receive LOCK_TOKEN with `transferAndCall()`', async () => {
        expect((await erc677.balanceOf(participantAddresses[0])).toString()).to.equal(
          utils.parseUnits('10000', 'ether').toString()
        )
        await expect(
          erc677
            .connect(participants[0])
            .transferAndCall(stakeContract.address, utils.parseUnits('1000', 'ether'), '0x')
        )
          .to.emit(stakeContract, 'Staked')
          .withArgs(participantAddresses[0], utils.parseUnits('1000', 'ether').toString(), constants.Zero.toString())

        expect((await erc677.balanceOf(participantAddresses[0])).toString()).to.equal(
          utils.parseUnits('9000', 'ether').toString()
        )
      })

      it('updates accounts value', async () => {
        const currentAccount = await stakeContract.accounts(participantAddresses[0])
        expect(currentAccount[0].toString()).to.equal(utils.parseUnits('1000', 'ether').toString()) // actualLockedTokenAmount
        expect(currentAccount[1].toString()).to.equal('0') // virtualLockedTokenAmount
        // skip checking lastSyncTimestamp
        expect(currentAccount[3].toString()).to.equal('0') // cumulatedRewards
        expect(currentAccount[4].toString()).to.equal('0') // claimedRewards
      })
    })

    describe('REWARD_TOKEN and other ERC777 token', () => {
      let randomERC777

      it('cannot receive random 777 with `send()`', async () => {
        // participantAddresses[2] account holds 5 million random erc777
        randomERC777 = await deployContractFromFactory(deployer, 'ERC777Mock', [
          participantAddresses[2],
          utils.parseUnits('5000000', 'ether'),
          'ERC777Mock',
          'M777',
          []
        ])

        await expect(
          randomERC777.connect(participants[2]).send(stakeContract.address, constants.One, '0x')
        ).to.be.revertedWith('HoprStake: Sender must be wxHOPR token')
      })

      it('cannot receive REWARD_TOKEN from a random account', async () => {
        await erc777.mintInternal(participantAddresses[2], constants.One, '0x', '0x') // admin account holds 1 random REWARD_TOKEN
        await expect(
          erc777.connect(participants[2]).send(stakeContract.address, constants.One, '0x')
        ).to.be.revertedWith('HoprStake: Only accept owner to provide rewards')
      })

      it('can receive REWARD_TOKEN with `send()`', async () => {
        expect((await erc777.balanceOf(adminAddress)).toString()).to.equal(
          utils.parseUnits('5000000', 'ether').toString()
        )
        await expect(erc777.connect(admin).send(stakeContract.address, constants.One, '0x'))
          .to.emit(stakeContract, 'RewardFueled')
          .withArgs(constants.One.toString())

        expect((await erc777.balanceOf(adminAddress)).toString()).to.equal(
          utils.parseUnits('5000000', 'ether').sub(constants.One).toString()
        )
      })
    })

    describe('nftBoost and other ERC721 tokens', function () {
      let randomERC721
      it('cannot receive an boost-like random ERC721 token', async () => {
        randomERC721 = await deployContractFromFactory(deployer, 'HoprBoost', [adminAddress, ''])
        // create a random NFT
        await randomERC721
          .connect(admin)
          .mint(participantAddresses[0], BADGES[1].type, BADGES[1].rank, BADGES[1].nominator, BADGES[1].deadline)
        await expect(
          randomERC721
            .connect(participants[0])
            .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[0], stakeContract.address, 0)
        ).to.be.revertedWith('HoprStake: Cannot SafeTransferFrom tokens other than HoprBoost.')
      })
      it('cannot redeem a boost when the deadline has passed', async () => {
        // create the 6th NFT.
        await nftContract
          .connect(admin)
          .mint(participantAddresses[2], BADGES[2].type, BADGES[2].rank, BADGES[2].nominator, BADGES[2].deadline)
        await expect(
          nftContract
            .connect(participants[2])
            .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[2], stakeContract.address, 6)
        ).to.be.revertedWith('HoprStake: Cannot redeem an expired boost.')
      })
      it('cannot reclaim a HOPRBoost', async () => {
        await expect(stake2Contract.connect(admin).reclaimErc721Tokens(nftContract.address, 0)).to.be.revertedWith(
          'HoprStake: Cannot claim HoprBoost NFT'
        )
      })
      it('can reclaim an ERC721', async () => {
        await randomERC721.connect(participants[0]).transferFrom(participantAddresses[0], stakeContract.address, 0)
        await stakeContract.connect(admin).reclaimErc721Tokens(randomERC721.address, 0)
        expect((await randomERC721.ownerOf(0)).toString()).to.equal(adminAddress)
      })
    })

    describe('Before program starts', function () {
      it('has not started yet', async function () {
        const blockTime = await latestBlockTime(hre.ethers.provider)
        const startTime = await stakeContract.BASIC_START()
        console.log(`blockTime ${blockTime.toString()} startTime ${startTime.toString()}`)
        expect(blockTime).to.be.lt(startTime)
      })
      it('can redeem HODLr token', async function () {
        await expect(
          nftContract
            .connect(participants[0])
            .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[0], stakeContract.address, 0)
        )
          .to.emit(stakeContract, 'Redeemed')
          .withArgs(participantAddresses[0], constants.Zero.toString(), true)
      })

      it('has nothing to claim', async () => {
        await expect(stakeContract.claimRewards(participantAddresses[0])).to.be.revertedWith(
          'HoprStake: Nothing to claim'
        )
      })
      it('can redeem token #1 and stake some tokens', async () => {
        await nftContract
          .connect(participants[1])
          .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[1], stakeContract.address, 1)
        await erc677.connect(participants[1]).transferAndCall(stakeContract.address, utils.parseUnits('1.'), '0x')
      })
    })
    describe('At BASIC_START', function () {
      it('succeeds in advancing block to BASIC_START', async function () {
        await advanceTimeForNextBlock(hre.ethers.provider, BASIC_START)
        const blockTime = await latestBlockTime(hre.ethers.provider)
        expect(blockTime.toString()).to.equal(BASIC_START.toString())
      })

      it('gets the cumulated rewards right at BASIC_START', async function () {
        const currentAccount = await stakeContract.accounts(participantAddresses[0])
        const reward = await stakeContract.getCumulatedRewardsIncrement(participantAddresses[0])
        const blockTime = await latestBlockTime(hre.ethers.provider)
        expect(currentAccount[3].toString()).to.equal(
          calculateRewards(1000, blockTime - BASIC_START, [BASIC_FACTOR_NUMERATOR, parseInt(BADGES[0].nominator)])
        ) // equals to the expected rewards.
        expect(reward.toString()).to.equal(constants.Zero.toString()) // rewards get synced
      })

      it('has insufficient pool', async () => {
        await expect(stakeContract.claimRewards(participantAddresses[0])).to.be.revertedWith(
          'HoprStake: Insufficient reward pool.'
        )
      })
    })

    describe('During the staking program v1', function () {
      it('advance 2 blocks (10 seconds), there is more rewards to be claimed.', async () => {
        const lastBlockTime = await latestBlockTime(hre.ethers.provider)

        const duration = 10 // in seconds
        await advanceTimeForNextBlock(hre.ethers.provider, lastBlockTime + duration) // advance two blocks - 2 seconds
        const currentAccount = await stakeContract.accounts(participantAddresses[0])

        const reward = await stakeContract.getCumulatedRewardsIncrement(participantAddresses[0])
        const blockTime = await latestBlockTime(hre.ethers.provider)
        expect(blockTime - lastBlockTime).to.equal(duration) // advance duration blocks or second
        expect(currentAccount[3].toString()).to.equal(constants.Zero.toString()) // equals to the expected rewards.
        expect(reward.toString()).to.equal(
          calculateRewards(1000, blockTime - BASIC_START, [BASIC_FACTOR_NUMERATOR, parseInt(BADGES[0].nominator)])
        ) // rewards get synced
      })

      it('can redeem another (less good) HODLr token', async function () {
        // create the 7th NFT.
        await nftContract
          .connect(admin)
          .mint(participantAddresses[0], BADGES[3].type, BADGES[3].rank, BADGES[3].nominator, BADGES[3].deadline)

        // redeem NFT #7
        await expect(
          nftContract
            .connect(participants[0])
            .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[0], stakeContract.address, 7)
        )
          .to.emit(stakeContract, 'Redeemed')
          .withArgs(participantAddresses[0], '7', false)
      })

      it('can redeem another (better) HODLr token', async function () {
        // redeem token number 2
        await expect(
          nftContract
            .connect(participants[0])
            .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[0], stakeContract.address, 2)
        )
          .to.emit(stakeContract, 'Redeemed')
          .withArgs(participantAddresses[0], '2', true)
      })

      it('their token value is synced', async function () {
        const currentAccount = await stakeContract.accounts(participantAddresses[0])
        const reward = await stakeContract.getCumulatedRewardsIncrement(participantAddresses[0])
        const blockTime = await latestBlockTime(hre.ethers.provider)
        expect(currentAccount[3].toString()).to.equal(
          calculateRewards(1000, blockTime - BASIC_START, [BASIC_FACTOR_NUMERATOR, parseInt(BADGES[0].nominator)])
        ) // equals to the expected rewards.
        expect(reward.toString()).to.equal(constants.Zero.toString()) // rewards get synced
      })

      it('receives more claim rewards', async () => {
        await erc777
          .connect(admin)
          .send(stakeContract.address, utils.parseUnits('5000000', 'ether').sub(constants.One), '0x') // propide 5 million REWARD_TOKEN
        expect((await erc777.balanceOf(adminAddress)).toString()).to.equal(constants.Zero.toString())
      })

      it('claims rewards', async () => {
        await expect(stakeContract.claimRewards(participantAddresses[0]))
          .to.emit(stakeContract, 'Claimed')
          .withArgs(participantAddresses[0], (await erc777.balanceOf(participantAddresses[0])).toString())
      })

      it('can redeem another HODLr token of another category', async function () {
        // create the 8th NFT.
        await nftContract
          .connect(admin)
          .mint(participantAddresses[0], BADGES[4].type, BADGES[4].rank, BADGES[4].nominator, BADGES[4].deadline)

        // redeem NFT #8
        await expect(
          nftContract
            .connect(participants[0])
            .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[0], stakeContract.address, 8)
        )
          .to.emit(stakeContract, 'Redeemed')
          .withArgs(participantAddresses[0], '8', true)
      })

      it('cannot lock tokens when length does not match', async () => {
        await expect(stakeContract.connect(admin).lock([participantAddresses[0]], ['1', '2'])).to.be.revertedWith(
          'HoprStake: Length does not match'
        )
      })

      it('can lock tokens', async () => {
        await expect(
          stakeContract.connect(admin).lock([participantAddresses[2]], [utils.parseUnits('1.0', 'ether').toString()])
        )
          .to.emit(stakeContract, 'Staked')
          .withArgs(participantAddresses[2], constants.Zero.toString(), utils.parseUnits('1.0', 'ether').toString())
      })

      it('cannot unlock tokens', async () => {
        await expect(stakeContract.unlock(participantAddresses[0])).to.be.revertedWith(
          'HoprStake: Program is ongoing, cannot unlock stake.'
        )
      })

      it('can reclaim random ERC20', async () => {
        const randomERC20 = await deployContractFromFactory(deployer, 'ERC20Mock', [stakeContract.address, 1])
        // Revert message was bubbled up, showing only the one from ERC677Mock
        await stakeContract.connect(admin).reclaimErc20Tokens(randomERC20.address)
        expect((await randomERC20.balanceOf(adminAddress)).toString()).to.equal('1')
      })

      it('can sync at anytime', async () => {
        // const currentAccount = await stakeContract.accounts(participantAddresses[0]);
        // expect(currentAccount[0].toString()).to.equal(utils.parseUnits('1000', 'ether').toString()); // actualLockedTokenAmount
        // expect(currentAccount[1].toString()).to.equal('0'); // virtualLockedTokenAmount
        // // skip checking lastSyncTimestamp
        // expect(currentAccount[3].toString()).to.equal('0'); // cumulatedRewards
        // expect(currentAccount[4].toString()).to.equal('0'); // claimedRewards
        console.log(JSON.stringify(await stakeContract.accounts(participantAddresses[0]).toString()))
        await stakeContract.sync(participantAddresses[2])
        console.log(JSON.stringify(await stakeContract.accounts(participantAddresses[0]).toString()))
      })
    })

    describe('After v1 PROGRAM_END', function () {
      let balanceBefore
      describe('Before unlock', function () {
        it('succeeds in advancing block to PROGRAM_END+1', async function () {
          balanceBefore = await erc677.balanceOf(participantAddresses[1])
          await advanceTimeForNextBlock(hre.ethers.provider, PROGRAM_END + 1)
          const blockTime = await latestBlockTime(hre.ethers.provider)
          expect(blockTime.toString()).to.equal((PROGRAM_END + 1).toString())
        })

        it('cannot receive random 677 with `transferAndCall()`', async () => {
          // bubbled up
          await expect(
            erc677.connect(participants[1]).transferAndCall(stakeContract.address, constants.One, '0x')
          ).to.be.revertedWith('ERC677Mock: failed when calling onTokenTransfer')
        })
        it('cannot redeem NFT`', async () => {
          // created 9th NFT
          await nftContract
            .connect(admin)
            .mint(participantAddresses[1], BADGES[1].type, BADGES[1].rank, BADGES[1].nominator, BADGES[1].deadline)
          await expect(
            nftContract
              .connect(participants[1])
              .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[1], stakeContract.address, 9)
          ).to.be.revertedWith('HoprStake: Program ended, cannot redeem boosts.')
        })
        it('cannot lock', async () => {
          await expect(stakeContract.connect(admin).lock([participantAddresses[0]], ['1'])).to.be.revertedWith(
            'HoprStake: Program ended, cannot stake anymore.'
          )
        })
      })
      describe('Unlock', function () {
        before('can unlock - reclaim rewards and receives original tokens', async () => {
          const rewards = calculateRewards(1, PROGRAM_END - BASIC_START, [
            BASIC_FACTOR_NUMERATOR,
            parseInt(BADGES[0].nominator)
          ])
          await expect(stakeContract.connect(participants[1]).unlock(participantAddresses[1]))
            .to.emit(stakeContract, 'Claimed') // reclaims rewards
            .withArgs(participantAddresses[1], rewards)
            .to.emit(stakeContract, 'Released') // receives original tokens - emit Released event
            .withArgs(participantAddresses[1], utils.parseUnits('1.0', 'ether').toString(), constants.Zero.toString())
        })
        it('receives original tokens - increased by 1 token', async () => {
          const balanceAfter = await erc677.balanceOf(participantAddresses[1])
          expect(BigNumber.from(balanceAfter).sub(BigNumber.from(balanceBefore)).toString()).to.equal(
            utils.parseUnits('1', 'ether').toString()
          ) // true
        })
        it('receives NFTs', async () => {
          const owner = await nftContract.ownerOf(9)
          expect(owner).to.equal(participantAddresses[1]) // compare bytes32 like address
        })
      })
    })
  })
})
