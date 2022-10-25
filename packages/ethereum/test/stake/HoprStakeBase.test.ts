import * as hre from 'hardhat'
import { BigNumber, constants, Contract, Signer, utils } from 'ethers'
import { expect } from 'chai'
import deployERC1820Registry from '../../deploy/01_ERC1820Registry'
import { advanceTimeForNextBlock, deployContractFromFactory, latestBlockTime } from '../utils'

describe('HoprStakeBase', function () {
  let deployer: Signer
  let admin: Signer
  let participants: Signer[]

  let deployerAddress: string
  let adminAddress: string
  let participantAddresses: string[]

  let nftContract: Contract
  let stakeContract: Contract
  let erc677: Contract
  let erc777: Contract

  const BASE_URI = 'https://stake.hoprnet.org/'
  // const FACTOR_DENOMINATOR = 1e12; //
  // const LOCK_TOKEN_ADDRESS = "0xD057604A14982FE8D88c5fC25Aac3267eA142a08";
  // const REWARD_TOKEN_ADDRESS = "0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1";
  // const NFT_CONTRACT_ADDRESS = "0x43d13D7B83607F14335cF2cB75E87dA369D056c7";
  // const DEFAULT_OWNER_ADDRESS = "0xD9a00176Cf49dFB9cA3Ef61805a2850F45Cb1D05"

  const BADGES = [
    {
      type: 'demo',
      rank: 'demo',
      deadline: 0,
      nominator: '158' // 0.5% APY
    },
    {
      type: 'HODLr',
      rank: 'silver',
      deadline: 0,
      nominator: '158' // 0.5% APY
    },
    {
      type: 'HODLr',
      rank: 'platinum',
      deadline: 0,
      nominator: '317' // 1% APY
    },
    {
      type: 'Past',
      rank: 'gold',
      deadline: 0, // sometime long long ago
      nominator: '100'
    },
    {
      type: 'HODLr',
      rank: 'bronze extra',
      deadline: 0,
      nominator: '79' // 0.25% APY
    },
    {
      type: 'Testnet participant',
      rank: 'gold',
      deadline: 0,
      nominator: '317' // 0.25% APY
    }
  ]

  const programStart = 1666785600 // 2pm CEST 26th October 2022
  const programEnd = 1674738000 // 2pm CET 26th January 2023
  const basicFactorNumerator = 793
  const boostCap = utils.parseUnits('200000', 'ether') // 200k

  const reset = async () => {
    let signers: Signer[]
    ;[deployer, admin, ...signers] = await hre.ethers.getSigners()
    participants = signers.slice(3, 6) // 3 participants

    deployerAddress = await deployer.getAddress()
    adminAddress = await admin.getAddress()
    participantAddresses = await Promise.all(participants.map((h) => h.getAddress()))

    // set 1820 registry
    await deployERC1820Registry(hre, deployer)
    // set stake and reward tokens
    erc677 = await deployContractFromFactory(deployer, 'ERC677Mock')
    // erc777 is the reward token (wxHOPR). admin account holds 5 million reward tokens
    erc777 = await deployContractFromFactory(deployer, 'ERC777Mock', [
      adminAddress,
      utils.parseUnits('5000000', 'ether'),
      'ERC777Mock',
      'M777',
      [adminAddress]
    ])

    // create NFT and stake contract
    nftContract = await deployContractFromFactory(deployer, 'HoprBoost', [adminAddress, BASE_URI])
    stakeContract = await deployContractFromFactory(deployer, 'HoprStakeBase', [
      adminAddress,
      programStart,
      programEnd,
      basicFactorNumerator,
      boostCap,
      nftContract.address,
      erc677.address,
      erc777.address
    ])

    // airdrop some NFTs (0,1,2,3,4) to participants
    await nftContract
      .connect(admin)
      .batchMint(
        participantAddresses.slice(0, 2),
        BADGES[0].type,
        BADGES[0].rank,
        BADGES[0].nominator,
        BADGES[0].deadline
      ) // nft nr 0,1: demo (typeIndex 1)
    await nftContract
      .connect(admin)
      .mint(participantAddresses[0], BADGES[1].type, BADGES[1].rank, BADGES[1].nominator, BADGES[1].deadline) // nft nr 2: HODLr (typeIndex 2)
    await nftContract
      .connect(admin)
      .mint(participantAddresses[0], BADGES[4].type, BADGES[4].rank, BADGES[4].nominator, BADGES[4].deadline) // nft nr 3: HODLr (typeIndex 2)
    await nftContract
      .connect(admin)
      .mint(participantAddresses[2], BADGES[0].type, BADGES[0].rank, BADGES[0].nominator, BADGES[0].deadline) // nft nr 4: demo (typeIndex 1)
    // airdrop some ERC677 to participants
    await erc677.batchMintInternal(participantAddresses, utils.parseUnits('10000', 'ether')) // each participant holds 10k xHOPR

    // stake some tokens
    await erc677
      .connect(participants[0])
      .transferAndCall(stakeContract.address, utils.parseUnits('1000', 'ether'), '0x') // stake 1000 LOCK_TOKEN
    // redeem a demo token - silver
    await nftContract
      .connect(participants[0])
      .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[0], stakeContract.address, 0)
    // redeem a demo token - platinum
    await nftContract
      .connect(participants[2])
      .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[2], stakeContract.address, 4)
    // provide 5 million REWARD_TOKEN
    await erc777.connect(admin).send(stakeContract.address, utils.parseUnits('5000000', 'ether'), '0x')
  }

  describe('unit tests', function () {
    const hodlrNftTokenId = 2
    beforeEach(async function () {
      await reset()
    })
    describe('Can redeem allowed NFT', function () {
      it(`succeed to redeem nfts nr ${hodlrNftTokenId}`, async () => {
        await expect(
          nftContract
            .connect(participants[0])
            .functions['safeTransferFrom(address,address,uint256)'](
              participantAddresses[0],
              stakeContract.address,
              hodlrNftTokenId
            )
        )
          .to.emit(stakeContract, 'Redeemed')
          .withArgs(participantAddresses[0], hodlrNftTokenId, true)
      })
    })
    describe('For whitelisting', function () {
      describe('redeemed token', function () {
        it('can get redeemed token with isNftTypeAndRankRedeemed1', async function () {
          const isNftTypeAndRankRedeemed1 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed1(BADGES[0].type, BADGES[0].rank, participantAddresses[0])
          expect(isNftTypeAndRankRedeemed1).to.equal(true)
        })
        it('can get redeemed token with isNftTypeAndRankRedeemed2', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed2 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed2(1, BADGES[0].rank, participantAddresses[0])
          expect(isNftTypeAndRankRedeemed2).to.equal(true)
        })
        it('can get redeemed token with isNftTypeAndRankRedeemed3', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed3 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed3(1, BADGES[0].nominator, participantAddresses[0])
          expect(isNftTypeAndRankRedeemed3).to.equal(true)
        })
        it('can get redeemed token with isNftTypeAndRankRedeemed4', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed4 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed4(BADGES[0].type, BADGES[0].nominator, participantAddresses[0])
          expect(isNftTypeAndRankRedeemed4).to.equal(true)
        })
        it('can get redeemed token with isNftTypeAndRankRedeemed4', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed4 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed4(BADGES[0].type, BADGES[0].nominator, participantAddresses[0])
          expect(isNftTypeAndRankRedeemed4).to.equal(true)
        })
      })
      describe('redeemed token but wrong info', function () {
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed1, different rank', async function () {
          const isNftTypeAndRankRedeemed1 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed1(BADGES[0].type, 'diamond', participantAddresses[0])
          expect(isNftTypeAndRankRedeemed1).to.equal(false)
        })
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed1, different type', async function () {
          const isNftTypeAndRankRedeemed1 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed1('Rando type', BADGES[0].rank, participantAddresses[0])
          expect(isNftTypeAndRankRedeemed1).to.equal(false)
        })
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed2, different rank', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed2 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed2(1, 'diamond', participantAddresses[0])
          expect(isNftTypeAndRankRedeemed2).to.equal(false)
        })
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed2, different type', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed2 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed2(2, BADGES[0].rank, participantAddresses[0])
          expect(isNftTypeAndRankRedeemed2).to.equal(false)
        })
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed3, different factor', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed3 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed3(1, 888, participantAddresses[0])
          expect(isNftTypeAndRankRedeemed3).to.equal(false)
        })
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed4, different type', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed3 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed3(2, BADGES[0].nominator, participantAddresses[0])
          expect(isNftTypeAndRankRedeemed3).to.equal(false)
        })
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed4, different factor', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed4 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed4(BADGES[0].type, 888, participantAddresses[0])
          expect(isNftTypeAndRankRedeemed4).to.equal(false)
        })
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed4, different type', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed4 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed4('Rando type', BADGES[0].nominator, participantAddresses[0])
          expect(isNftTypeAndRankRedeemed4).to.equal(false)
        })
      })
      describe('owned but not redeemed token', function () {
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed1', async function () {
          const isNftTypeAndRankRedeemed1 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed1(BADGES[0].type, BADGES[0].rank, participantAddresses[1])
          expect(isNftTypeAndRankRedeemed1).to.equal(false)
        })
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed2', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed2 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed2(1, BADGES[0].rank, participantAddresses[1])
          expect(isNftTypeAndRankRedeemed2).to.equal(false)
        })
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed3', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed3 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed3(1, BADGES[0].nominator, participantAddresses[1])
          expect(isNftTypeAndRankRedeemed3).to.equal(false)
        })
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed4', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed4 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed4(BADGES[0].type, BADGES[0].nominator, participantAddresses[1])
          expect(isNftTypeAndRankRedeemed4).to.equal(false)
        })
        it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed4', async function () {
          // type index starts from 1
          const isNftTypeAndRankRedeemed4 = await stakeContract
            .connect(deployer)
            .isNftTypeAndRankRedeemed4(BADGES[0].type, BADGES[0].nominator, participantAddresses[1])
          expect(isNftTypeAndRankRedeemed4).to.equal(false)
        })
      })
    })
  })

  describe('After programEnd', function () {
    before(async function () {
      await reset()

      // -----logs
      console.table([
        ['Deployer', deployerAddress],
        ['Admin', adminAddress],
        ['NFT Contract', nftContract.address],
        ['Stake Contract', stakeContract.address],
        ['participant', JSON.stringify(participantAddresses)]
      ])
    })
    it('succeeds in advancing block to PROGRAM_S3_END + 1', async function () {
      await advanceTimeForNextBlock(hre.ethers.provider, programEnd + 1)
      const blockTime = await latestBlockTime(hre.ethers.provider)
      expect(blockTime.toString()).to.equal((programEnd + 1).toString())
    })

    it('cannot receive random 677 with `transferAndCall()`', async () => {
      // bubbled up
      await expect(
        erc677.connect(participants[1]).transferAndCall(stakeContract.address, constants.One, '0x')
      ).to.be.revertedWith('ERC677Mock: failed when calling onTokenTransfer')
    })
    it('cannot redeem NFT', async () => {
      // created #4 NFT
      await nftContract
        .connect(admin)
        .mint(participantAddresses[1], BADGES[1].type, BADGES[1].rank, BADGES[1].nominator, BADGES[1].deadline)
      await expect(
        nftContract
          .connect(participants[1])
          .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[1], stakeContract.address, 1)
      ).to.be.revertedWith('HoprStake: Program ended, cannot redeem boosts.')
    })
    it('can unlock', async () => {
      await stakeContract.connect(participants[0]).unlock()
    })
    it('receives original tokens - total balance matches old one ', async () => {
      const balance = await erc677.balanceOf(participantAddresses[0])
      expect(BigNumber.from(balance).toString()).to.equal(utils.parseUnits('10000', 'ether').toString()) // true
    })
    it('receives original tokens - total balance matches old one ', async () => {
      const balance = await erc677.balanceOf(participantAddresses[0])
      expect(BigNumber.from(balance).toString()).to.equal(utils.parseUnits('10000', 'ether').toString()) // true
    })
    it('receives NFTs', async () => {
      const owner = await nftContract.ownerOf(0)
      expect(owner).to.equal(participantAddresses[0]) // compare bytes32 like address
    })
    it('can claim back NFT without anything at stake', async () => {
      await expect(stakeContract.connect(participants[2]).unlock())
        .to.emit(stakeContract, 'Released')
        .withArgs(participantAddresses[2], 0)
        .to.emit(nftContract, 'Transfer')
        .withArgs(stakeContract.address, participantAddresses[2], 4)
    })
  })
})
