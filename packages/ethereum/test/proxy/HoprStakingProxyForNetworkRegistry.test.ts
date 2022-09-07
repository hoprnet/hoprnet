import chai, { expect } from 'chai'
import { deployments, ethers } from 'hardhat'
import { smock } from '@defi-wonderland/smock'
import { type BigNumber, constants, type Contract, type Signer } from 'ethers'
import { HoprStakingProxyForNetworkRegistry } from '../../src/types'

chai.should() // if you like should syntax
chai.use(smock.matchers)

const INITIAL_MIN_STAKE = 1500
const NFT_TYPE = [1, 2]
const NFT_RANK = [123, 456]
const HIGH_STAKE = 2000
const LOW_STAKE = 100
const SPECIAL_NFT_TYPE = 3 // 'Network_registry'
const SPECIAL_NFT_RANK_TECH = 'developer' // 'Tech'
const SPECIAL_NFT_RANK_COM = 'community' // 'Com'
const MAX_REGISTRATION_TECH = constants.MaxUint256
const MAX_REGISTRATION_COM = 1

const checkMaxAllowance = async (
  registryContract: Contract,
  participantAddresses: string[],
  allowedRegistrationNum: Array<number | BigNumber>
) => {
  participantAddresses.forEach((participantAddress, accountIndex) => {
    it(`participant ${accountIndex} should have allowance of ${allowedRegistrationNum}`, async () => {
      expect(await registryContract.maxAllowedRegistrations(participantAddress)).to.be.equal(
        allowedRegistrationNum[accountIndex]
      )
    })
  })
}

/**
 * Allocation of NFTs and staks
 * | NFT Type      | 0 | 0 | 1 | 1 | Network_registry | Network_registry | Stake |
 * |---------------|---|---|---|---|------------------|------------------|-------|
 * | NFT Rank      | 0 | 1 | 0 | 1 | developer        | community        | --    |
 * |---------------|---|---|---|---|------------------|------------------|-------|
 * | Participant_0 |   | x |   |   |                  |                  | 2000  |
 * | Participant_1 |   |   | x |   |                  |                  | 2000  |
 * | Participant_2 |   | x |   |   | x                |                  | 100   |
 * | Participant_3 |   |   | x |   |                  |                  | 100   |
 * | Participant_4 |   |   | x |   |                  |                  | 2000  |
 * | Participant_5 |   | x |   |   |                  |                  | 100   |
 * | Participant_6 |   |   | x |   |                  | x                | 0     |
 * |---------------|---|---|---|---|------------------|------------------|-------|
 * @param participants
 * @returns
 */

const createFakeStakeV2Contract = async (participants: string[]) => {
  const stakeV2Fake = await smock.fake([
    {
      inputs: [
        {
          internalType: 'address',
          name: '_account',
          type: 'address'
        }
      ],
      name: 'stakedHoprTokens',
      outputs: [
        {
          internalType: 'uint256',
          name: '',
          type: 'uint256'
        }
      ],
      stateMutability: 'view',
      type: 'function'
    },
    {
      inputs: [
        {
          internalType: 'uint256',
          name: 'nftTypeIndex',
          type: 'uint256'
        },
        {
          internalType: 'uint256',
          name: 'boostNumerator',
          type: 'uint256'
        },
        {
          internalType: 'address',
          name: 'hodler',
          type: 'address'
        }
      ],
      name: 'isNftTypeAndRankRedeemed3',
      outputs: [
        {
          internalType: 'bool',
          name: '',
          type: 'bool'
        }
      ],
      stateMutability: 'view',
      type: 'function'
    }
  ])

  participants.forEach((participant, participantIndex) => {
    // no one has NFTs of {NFT_TYPE[0], NFT_RANK[0]} nor NFT_TYPE[1], NFT_RANK[1]
    stakeV2Fake.isNftTypeAndRankRedeemed3.whenCalledWith(NFT_TYPE[1], NFT_RANK[1], participant).returns(false)
    stakeV2Fake.isNftTypeAndRankRedeemed3.whenCalledWith(NFT_TYPE[0], NFT_RANK[0], participant).returns(false)

    if ([0, 2, 5].findIndex((element) => element === participantIndex) > -1) {
      // participants at index 0, 2, 5 hold {NFT_TYPE[0], NFT_RANK[1]} and others have {NFT_TYPE[1], NFT_RANK[0]}
      stakeV2Fake.isNftTypeAndRankRedeemed3.whenCalledWith(NFT_TYPE[0], NFT_RANK[1], participant).returns(true)
      stakeV2Fake.isNftTypeAndRankRedeemed3.whenCalledWith(NFT_TYPE[1], NFT_RANK[0], participant).returns(false)
    } else {
      stakeV2Fake.isNftTypeAndRankRedeemed3.whenCalledWith(NFT_TYPE[0], NFT_RANK[1], participant).returns(false)
      stakeV2Fake.isNftTypeAndRankRedeemed3.whenCalledWith(NFT_TYPE[1], NFT_RANK[0], participant).returns(true)
    }

    if (participantIndex === 6) {
      // participants at index 6 have 0
      stakeV2Fake.stakedHoprTokens.whenCalledWith(participant).returns(0)
    } else if ([0, 1, 4].findIndex((element) => element === participantIndex) > -1) {
      // participants at index 0, 1, 4 have 2000 staked tokens
      stakeV2Fake.stakedHoprTokens.whenCalledWith(participant).returns(HIGH_STAKE)
    } else {
      // others have 100 staked tokens
      stakeV2Fake.stakedHoprTokens.whenCalledWith(participant).returns(LOW_STAKE)
    }
  })
  // participant 2 redeemd a special NFT (TECH)
  stakeV2Fake.isNftTypeAndRankRedeemed3
    .whenCalledWith(SPECIAL_NFT_TYPE, SPECIAL_NFT_RANK_TECH, participants[2])
    .returns(true)
  // participant 6 redeemd a special NFT (COM)
  stakeV2Fake.isNftTypeAndRankRedeemed3
    .whenCalledWith(SPECIAL_NFT_TYPE, SPECIAL_NFT_RANK_COM, participants[6])
    .returns(true)
  return stakeV2Fake
}

const useFixtures = deployments.createFixture(async (_hre) => {
  const [_deployer, owner, ...signers] = await ethers.getSigners()
  const participants = signers.slice(3, 10) // 7 participants

  const ownerAddress = await owner.getAddress()
  const participantAddresses = await Promise.all(participants.map((h) => h.getAddress()))

  // mock staking contract
  const stakeV2Fake = await createFakeStakeV2Contract(participantAddresses)

  // deploy network registry
  const hoprStakingProxyForNetworkRegistry = (await (
    await ethers.getContractFactory('HoprStakingProxyForNetworkRegistry')
  ).deploy(stakeV2Fake.address, ownerAddress, INITIAL_MIN_STAKE)) as HoprStakingProxyForNetworkRegistry

  return {
    owner,
    participants,
    ownerAddress,
    participantAddresses,
    stakeV2Fake,
    hoprStakingProxyForNetworkRegistry
  }
})

describe('Registry proxy for stake v2', () => {
  let owner: Signer
  let participantAddresses: string[]
  let hoprStakingProxyForNetworkRegistry: Contract

  describe('Self register', async () => {
    before(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())
      //   add eligible NFT
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])
    })

    const allowanceWhenTresholdIsLowered = [2, 0, 0, 0, 0, 0, 0]
    await checkMaxAllowance(hoprStakingProxyForNetworkRegistry, participantAddresses, allowanceWhenTresholdIsLowered)
  })

  describe('Update threshold', () => {
    before(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())
    })
    it('fails to update with the same threshold', async () => {
      await expect(
        hoprStakingProxyForNetworkRegistry.connect(owner).ownerUpdateThreshold(INITIAL_MIN_STAKE)
      ).to.be.revertedWith('HoprStakingProxyForNetworkRegistry: try to update with the same staking threshold')
    })
    it('updates with a different threshold', async () => {
      await expect(hoprStakingProxyForNetworkRegistry.connect(owner).ownerUpdateThreshold(LOW_STAKE))
        .to.emit(hoprStakingProxyForNetworkRegistry, 'ThresholdUpdated')
        .withArgs(LOW_STAKE)
    })
  })

  describe(`Owner add an existing NFT`, async () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])
    })
    const allowanceWhenTresholdIsLowered = [2, 0, 0, 0, 0, 0, 0]
    await checkMaxAllowance(hoprStakingProxyForNetworkRegistry, participantAddresses, allowanceWhenTresholdIsLowered)
  })

  describe(`Lower threshold to ${LOW_STAKE}`, async () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())

      const threshold = await hoprStakingProxyForNetworkRegistry.stakeThreshold()
      if (threshold.toString() !== LOW_STAKE.toString()) {
        await hoprStakingProxyForNetworkRegistry.connect(owner).ownerUpdateThreshold(LOW_STAKE)
      }
      //   add eligible NFT
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])
    })

    const allowanceWhenTresholdIsLowered = [2, 0, 0, 0, 0, 0, 0]
    await checkMaxAllowance(hoprStakingProxyForNetworkRegistry, participantAddresses, allowanceWhenTresholdIsLowered)
  })

  describe(`Owner batch-add NFTs`, async () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerUpdateThreshold(LOW_STAKE)
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])

      await hoprStakingProxyForNetworkRegistry
        .connect(owner)
        .ownerBatchAddNftTypeAndRank([NFT_TYPE[0], NFT_TYPE[1]], [NFT_RANK[1], NFT_RANK[0]])
    })

    it('fails to when array length does not match', async () => {
      await expect(
        hoprStakingProxyForNetworkRegistry
          .connect(owner)
          .ownerBatchAddNftTypeAndRank([NFT_TYPE[0]], [NFT_RANK[1], NFT_RANK[0]])
      ).to.be.revertedWith('HoprStakingProxyForNetworkRegistry: ownerBatchAddNftTypeAndRank lengths mismatch')
    })

    const allowanceWhenTresholdIsLowered = [20, 20, 1, 1, 20, 1, 0]
    await checkMaxAllowance(hoprStakingProxyForNetworkRegistry, participantAddresses, allowanceWhenTresholdIsLowered)
  })

  describe(`Owner remove NFT`, async () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerUpdateThreshold(LOW_STAKE)
      await hoprStakingProxyForNetworkRegistry
        .connect(owner)
        .ownerBatchAddNftTypeAndRank([NFT_TYPE[0], NFT_TYPE[1]], [NFT_RANK[1], NFT_RANK[0]])

      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerRemoveNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])
    })

    const allowanceWhenTresholdIsLowered = [0, 20, 0, 1, 20, 0, 0]
    await checkMaxAllowance(hoprStakingProxyForNetworkRegistry, participantAddresses, allowanceWhenTresholdIsLowered)
  })

  describe(`Owner batch-remove NFTs`, async () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerUpdateThreshold(LOW_STAKE)
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[1], NFT_RANK[0])
      await hoprStakingProxyForNetworkRegistry
        .connect(owner)
        .ownerBatchRemoveNftTypeAndRank([NFT_TYPE[0], NFT_TYPE[1]], [NFT_RANK[1], NFT_RANK[0]])
    })

    it('fails to when array length does not match', async () => {
      await expect(
        hoprStakingProxyForNetworkRegistry
          .connect(owner)
          .ownerBatchRemoveNftTypeAndRank([NFT_TYPE[0]], [NFT_RANK[1], NFT_RANK[0]])
      ).to.be.revertedWith('HoprStakingProxyForNetworkRegistry: ownerBatchRemoveNftTypeAndRank lengths mismatch')
    })
    const allowanceWhenTresholdIsLowered = [0, 0, 0, 0, 0, 0, 0]
    await checkMaxAllowance(hoprStakingProxyForNetworkRegistry, participantAddresses, allowanceWhenTresholdIsLowered)
  })

  describe(`Special NFTs only`, async () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())

      await hoprStakingProxyForNetworkRegistry
        .connect(owner)
        .ownerBatchAddSpecialNftTypeAndRank(
          [SPECIAL_NFT_TYPE, SPECIAL_NFT_TYPE],
          [SPECIAL_NFT_RANK_TECH, SPECIAL_NFT_RANK_COM],
          [MAX_REGISTRATION_TECH, MAX_REGISTRATION_COM]
        )
    })

    const allowance = [0, 0, MAX_REGISTRATION_TECH, 0, 0, 0, MAX_REGISTRATION_COM]
    await checkMaxAllowance(hoprStakingProxyForNetworkRegistry, participantAddresses, allowance)
  })

  describe(`Special NFTs on top of normal nfts`, async () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[1], NFT_RANK[0])
      await hoprStakingProxyForNetworkRegistry
        .connect(owner)
        .ownerBatchAddSpecialNftTypeAndRank(
          [SPECIAL_NFT_TYPE, SPECIAL_NFT_TYPE],
          [SPECIAL_NFT_RANK_TECH, SPECIAL_NFT_RANK_COM],
          [MAX_REGISTRATION_TECH, MAX_REGISTRATION_COM]
        )
    })

    const allowance = [2, 0, MAX_REGISTRATION_TECH, 0, 0, 0, MAX_REGISTRATION_COM]
    await checkMaxAllowance(hoprStakingProxyForNetworkRegistry, participantAddresses, allowance)
  })
  describe(`Both special NFTs on top of normal nfts`, async () => {
    beforeEach(async () => {
      let stakeV2Fake
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry, stakeV2Fake } = await useFixtures())
      // participant 6 redeemed two special NFTs (both COM and TECH)
      stakeV2Fake.isNftTypeAndRankRedeemed3
        .whenCalledWith(SPECIAL_NFT_TYPE, SPECIAL_NFT_RANK_TECH, participantAddresses[6])
        .returns(true)
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[1], NFT_RANK[0])
      await hoprStakingProxyForNetworkRegistry
        .connect(owner)
        .ownerBatchAddSpecialNftTypeAndRank(
          [SPECIAL_NFT_TYPE, SPECIAL_NFT_TYPE],
          [SPECIAL_NFT_RANK_TECH, SPECIAL_NFT_RANK_COM],
          [MAX_REGISTRATION_TECH, MAX_REGISTRATION_COM]
        )
    })

    const allowance = [2, 0, MAX_REGISTRATION_TECH, 0, 0, 0, MAX_REGISTRATION_TECH]
    await checkMaxAllowance(hoprStakingProxyForNetworkRegistry, participantAddresses, allowance)
  })
})
