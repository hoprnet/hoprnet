import chai, { expect } from 'chai'
import { deployments, ethers } from 'hardhat'
import { smock } from '@defi-wonderland/smock'
import { Contract, Signer } from 'ethers'
import { HoprStakingProxyForNetworkRegistry } from '../../src/types'

chai.should() // if you like should syntax
chai.use(smock.matchers)

const INITIAL_MIN_STAKE = 1500
const NFT_TYPE = [1, 2]
const NFT_RANK = [123, 456]
const HIGH_STAKE = 2000
const LOW_STAKE = 100
const SPECIAL_NFT_TYPE = 3 // 'Dev'
const SPECIAL_NFT_RANK = 0 // 'Rock'

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

    if ([0, 1, 4].findIndex((element) => element === participantIndex) > -1) {
      // participants at index 0, 1, 4 have 2000 staked tokens and others have 100 staked tokens
      stakeV2Fake.stakedHoprTokens.whenCalledWith(participant).returns(HIGH_STAKE)
    } else {
      stakeV2Fake.stakedHoprTokens.whenCalledWith(participant).returns(LOW_STAKE)
    }
  })
  // participant 2 redeemd a special NFT
  stakeV2Fake.isNftTypeAndRankRedeemed3
    .whenCalledWith(SPECIAL_NFT_TYPE, SPECIAL_NFT_RANK, participants[2])
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

  describe('Self register', () => {
    before(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())
      //   add eligible NFT
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])
    })
    it('register staker with stake of high threshold and eligible NFT', async () => {
      expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[0])).to.be.true
    })
    it('fail to register staker with stake of high threshold and non-eligible NFT', async () => {
      expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[1])).to.be.false
    })
    it('fail to register staker with stake of low threshold and eligible NFT', async () => {
      expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[2])).to.be.false
    })
    it('fail to register staker with stake of low threshold and non-eligible NFT', async () => {
      expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[3])).to.be.false
    })
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

  describe(`Owner add an existing NFT`, () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])
    })
    const canSelfRegister = [0]
    const cannotSelfRegister = [1, 2, 3, 4, 5]

    canSelfRegister.forEach((accountIndex) => {
      it(`participant ${accountIndex} is still registered`, async () => {
        expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[accountIndex])).to
          .be.true
      })
    })
    cannotSelfRegister.forEach((accountIndex) => {
      it(`participant ${accountIndex} is not registered`, async () => {
        expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[accountIndex])).to
          .be.false
      })
    })
  })

  describe(`Lower threshold to ${LOW_STAKE}`, () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())

      const threshold = await hoprStakingProxyForNetworkRegistry.stakeThreshold()
      if (threshold.toString() !== LOW_STAKE.toString()) {
        await hoprStakingProxyForNetworkRegistry.connect(owner).ownerUpdateThreshold(LOW_STAKE)
      }
      //   add eligible NFT
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])
    })
    const canSelfRegister = [0, 2, 5]
    const cannotSelfRegister = [1, 3, 4]

    canSelfRegister.forEach((accountIndex) => {
      it(`participant ${accountIndex} is still registered`, async () => {
        expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[accountIndex])).to
          .be.true
      })
    })
    cannotSelfRegister.forEach((accountIndex) => {
      it(`participant ${accountIndex} is not registered`, async () => {
        expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[accountIndex])).to
          .be.false
      })
    })
  })
  describe(`Owner batch-add NFTs`, () => {
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
    const canSelfRegister = [0, 1, 2, 3, 4, 5]
    const cannotSelfRegister = []

    canSelfRegister.forEach((accountIndex) => {
      it(`participant ${accountIndex} is still registered`, async () => {
        expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[accountIndex])).to
          .be.true
      })
    })
    cannotSelfRegister.forEach((accountIndex) => {
      it(`participant ${accountIndex} is not registered`, async () => {
        expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[accountIndex])).to
          .be.false
      })
    })
  })
  describe(`Owner remove NFT`, () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())
      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerUpdateThreshold(LOW_STAKE)
      await hoprStakingProxyForNetworkRegistry
        .connect(owner)
        .ownerBatchAddNftTypeAndRank([NFT_TYPE[0], NFT_TYPE[1]], [NFT_RANK[1], NFT_RANK[0]])

      await hoprStakingProxyForNetworkRegistry.connect(owner).ownerRemoveNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])
    })

    const canSelfRegister = [1, 3, 4]
    const cannotSelfRegister = [0, 2, 5]

    canSelfRegister.forEach((accountIndex) => {
      it(`participant ${accountIndex} is still registered`, async () => {
        expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[accountIndex])).to
          .be.true
      })
    })
    cannotSelfRegister.forEach((accountIndex) => {
      it(`participant ${accountIndex} is not registered`, async () => {
        expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[accountIndex])).to
          .be.false
      })
    })
  })
  describe(`Owner batch-remove NFTs`, () => {
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
    const canSelfRegister = []
    const cannotSelfRegister = [0, 1, 2, 3, 4, 5]

    canSelfRegister.forEach((accountIndex) => {
      it(`participant ${accountIndex} is still registered`, async () => {
        expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[accountIndex])).to
          .be.true
      })
    })
    cannotSelfRegister.forEach((accountIndex) => {
      it(`participant ${accountIndex} is not registered`, async () => {
        expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[accountIndex])).to
          .be.false
      })
    })
  })
  describe(`Special NFTs`, () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprStakingProxyForNetworkRegistry } = await useFixtures())

      await hoprStakingProxyForNetworkRegistry
        .connect(owner)
        .ownerBatchAddSpecialNftTypeAndRank([SPECIAL_NFT_TYPE], [SPECIAL_NFT_RANK])
    })

    const canSelfRegister = [2]
    const cannotSelfRegister = [0, 1, 3, 4, 5]

    canSelfRegister.forEach((accountIndex) => {
      it(`participant ${accountIndex} is still registered`, async () => {
        expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[accountIndex])).to
          .be.true
      })
    })
    cannotSelfRegister.forEach((accountIndex) => {
      it(`participant ${accountIndex} is not registered`, async () => {
        expect(await hoprStakingProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[accountIndex])).to
          .be.false
      })
    })
  })
})
