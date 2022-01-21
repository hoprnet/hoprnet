import chai, { expect } from 'chai'
import { deployments, ethers } from 'hardhat'
import { FakeContract, smock } from '@defi-wonderland/smock'
import { BaseContract, Contract, Signer } from 'ethers'
import { HoprNetworkRegistry } from '../src/types'
import { INITIAL_MIN_STAKE } from '../deploy/06_HoprNetworkRegistry'

chai.should() // if you like should syntax
chai.use(smock.matchers)

const NFT_TYPE = [1, 2]
const NFT_RANK = [123, 456]


const hoprAddress = (i: number) => `16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x${i}`

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
      stakeV2Fake.stakedHoprTokens.whenCalledWith(participant).returns(2000)
    } else {
      stakeV2Fake.stakedHoprTokens.whenCalledWith(participant).returns(100)
    }
  })

  return stakeV2Fake
}

const useFixtures = deployments.createFixture(async (hre) => {
  const [_deployer, owner, ...signers] = await ethers.getSigners()
  const participants = signers.slice(3, 10) // 7 participants

  const ownerAddress = await owner.getAddress()
  const participantAddresses = await Promise.all(participants.map((h) => h.getAddress()))

  // mock staking contract
  const stakeV2Fake = await createFakeStakeV2Contract(participantAddresses)

  // deploy network registry
  const hoprNetworkRegistry = (await (
    await ethers.getContractFactory('HoprNetworkRegistry')
  ).deploy(stakeV2Fake.address, ownerAddress, INITIAL_MIN_STAKE)) as HoprNetworkRegistry

  return {
    owner,
    participants,
    ownerAddress,
    participantAddresses,
    stakeV2Fake,
    hoprNetworkRegistry
  }
})

describe('HoprNetworkRegistry', () => {
  let owner: Signer
  let participants: Signer[]
  let ownerAddress: string
  let participantAddresses: string[]

  let stakeV2Fake: FakeContract<BaseContract>
  let hoprNetworkRegistry: Contract

  describe('Owner can update important parameters of the contract', () => {
    before(async () => {
      ;({
        owner,
        participants,
        ownerAddress,
        participantAddresses,
        stakeV2Fake,
        hoprNetworkRegistry,
        ownerAddress,
        participantAddresses
      } = await useFixtures())
    })
    it('owner to add eligible NFTs {type: 0, rank: 1}', async () => {
      // const {deployer, owner, participants, ownerAddress, participantAddresses, stakeV2Fake, hoprNetworkRegistry } = await useFixtures()
      await expect(hoprNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1]))
        .to.emit(hoprNetworkRegistry, 'AddedNftTypeAndRank')
        .withArgs(NFT_TYPE[0], NFT_RANK[1])
    })
    it('fail to batch-add NFTs existing {type: 0, rank: 1} and {type: 1, rank: 1} when length does not match', async () => {
      await expect(
        hoprNetworkRegistry.connect(owner).ownerBatchAddNftTypeAndRank([NFT_TYPE[0], NFT_RANK[1]], [1])
      ).to.be.revertedWith('HoprNetworkRegistry: ownerBatchAddNftTypeAndRank lengths mismatch')
    })
    it('owner to batch-add NFTs existing {type: 0, rank: 1} and {type: 1, rank: 1}', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerBatchAddNftTypeAndRank([NFT_TYPE[0], NFT_TYPE[1]], [NFT_RANK[1], NFT_RANK[1]])
      )
        .to.emit(hoprNetworkRegistry, 'AddedNftTypeAndRank')
        .withArgs(NFT_TYPE[1], NFT_RANK[1])
    })
    it('owner to remove eligible NFTs {type: 0, rank: 1}', async () => {
      await expect(hoprNetworkRegistry.connect(owner).ownerRemoveNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1]))
        .to.emit(hoprNetworkRegistry, 'RemovedNftTypeAndRank')
        .withArgs(NFT_TYPE[0], NFT_RANK[1])
    })
    it('owner to batch-remove NFTs existing {type: 0, rank: 1} and {type: 1, rank: 1}', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerBatchRemoveNftTypeAndRank([NFT_TYPE[0], NFT_TYPE[1]], [NFT_RANK[1], NFT_RANK[1]])
      )
        .to.emit(hoprNetworkRegistry, 'RemovedNftTypeAndRank')
        .withArgs(NFT_TYPE[1], NFT_RANK[1])
    })
    it('fail to let owner to batch-remove when nftTypes and nftRanks have different length', async () => {
      await expect(
        hoprNetworkRegistry.connect(owner).ownerBatchRemoveNftTypeAndRank([NFT_TYPE[0], NFT_TYPE[1]], [NFT_RANK[1]])
      ).to.be.revertedWith('HoprNetworkRegistry: ownerRemoveNftTypeAndRank lengths mismatch')
    })
    it('owner to whitelist addresses with amount < threshold and >= threshold', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerAddToWhitelist([hoprAddress(0), hoprAddress(1)], participantAddresses.slice(0, 2), [300, 1800])
      )
        .to.emit(hoprNetworkRegistry, 'OwnerAddedToWhitelist')
        .withArgs(participantAddresses[0], 300, hoprAddress(0))
        .to.emit(hoprNetworkRegistry, 'OwnerAddedToWhitelist')
        .withArgs(participantAddresses[1], 1800, hoprAddress(1))
    })
    it('fail to let owner to whitelist when hoprAddresses and stakers have different length', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerAddToWhitelist([hoprAddress(0), hoprAddress(1)], participantAddresses.slice(0, 1), [300, 1800])
      ).to.be.revertedWith('HoprNetworkRegistry: hoprAddresses and stakers lengths mismatch')
    })
    it('fail to let owner to whitelist when amounts and stakers have different length', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerAddToWhitelist([hoprAddress(0), hoprAddress(1)], participantAddresses.slice(0, 2), [300])
      ).to.be.revertedWith('HoprNetworkRegistry: amounts and stakers lengths mismatch')
    })
    it('fail to let owner to whitelist when an amount is zero', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerAddToWhitelist([hoprAddress(0), hoprAddress(1)], participantAddresses.slice(0, 2), [300, 0])
      ).to.be.revertedWith('HoprNetworkRegistry: staked amount should be above zero')
    })
    it('should let owner to update threshold', async () => {
      await expect(hoprNetworkRegistry.connect(owner).ownerUpdateThreshold(2000))
        .to.emit(hoprNetworkRegistry, 'UpdatedThreshold')
        .withArgs(2000)
    })
    it('owner to batch-remove from whitelist, provided a long list but only two get removed', async () => {
      await expect(hoprNetworkRegistry.connect(owner).ownerRemoveFromWhitelist(participantAddresses.slice(0, 3)))
        .to.emit(hoprNetworkRegistry, 'OwnerRemovedFromWhitelist')
        .withArgs(participantAddresses[0])
        .to.emit(hoprNetworkRegistry, 'OwnerRemovedFromWhitelist')
        .withArgs(participantAddresses[1])
    })
  })

  describe('Self whitelist', () => {
    before(async () => {
      // const fixture = await useFixtures()
      // owner = fixture.owner
      // participants = fixture.participants
      // ownerAddress = fixture.ownerAddress
      // participantAddresses = fixture.participantAddresses
      // stakeV2Fake = fixture.stakeV2Fake
      // hoprNetworkRegistry = fixture.hoprNetworkRegistry
      // ownerAddress = fixture.ownerAddress
      // participantAddresses = fixture.participantAddresses
      ;({
        owner,
        participants,
        ownerAddress,
        participantAddresses,
        stakeV2Fake,
        hoprNetworkRegistry,
        ownerAddress,
        participantAddresses
      } = await useFixtures())
      // add eligible NFT
      await hoprNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])
    })
    it('whitelist staker with stake of high threshold and eligible NFT', async () => {
      await expect(hoprNetworkRegistry.connect(participants[0]).addToWhitelist(hoprAddress(0)))
        .to.emit(hoprNetworkRegistry, 'AddedToWhitelist')
        .withArgs(participantAddresses[0], hoprAddress(0))

      expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(0))).to.equal(true)
    })
    it('fail to whitelist staker with stake of high threshold and non-eligible NFT', async () => {
      await hoprNetworkRegistry.connect(participants[1]).addToWhitelist(hoprAddress(1))
      expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(1))).to.equal(false)
    })
    it('fail to whitelist staker with stake of low threshold and eligible NFT', async () => {
      await expect(hoprNetworkRegistry.connect(participants[2]).addToWhitelist(hoprAddress(2))).to.be.revertedWith(
        'HoprNetworkRegistry: staked amount does not meet the threshold'
      )
    })
    it('fail to whitelist staker with stake of low threshold and non-eligible NFT', async () => {
      await expect(hoprNetworkRegistry.connect(participants[3]).addToWhitelist(hoprAddress(3))).to.be.revertedWith(
        'HoprNetworkRegistry: staked amount does not meet the threshold'
      )
    })
    it('can be whitelisted by the owner', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerAddToWhitelist(
            [hoprAddress(1), hoprAddress(2), hoprAddress(3)],
            participantAddresses.slice(1, 4),
            [2000, 100, 100]
          )
      )
        .to.emit(hoprNetworkRegistry, 'OwnerAddedToWhitelist')
        .withArgs(participantAddresses[1], 2000, hoprAddress(1))
        .to.emit(hoprNetworkRegistry, 'OwnerAddedToWhitelist')
        .withArgs(participantAddresses[2], 100, hoprAddress(2))
        .to.emit(hoprNetworkRegistry, 'OwnerAddedToWhitelist')
        .withArgs(participantAddresses[3], 100, hoprAddress(3))
    })
  })
  describe('Integration test', () => {
    beforeEach(async () => {
      ;({
        owner,
        participants,
        ownerAddress,
        participantAddresses,
        stakeV2Fake,
        hoprNetworkRegistry,
        ownerAddress,
        participantAddresses
      } = await useFixtures())
      // add eligible NFT
      await hoprNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])
      // whitelist participant 0, 1, 2, 3
      await hoprNetworkRegistry.connect(participants[0]).addToWhitelist(hoprAddress(0))
      await hoprNetworkRegistry
        .connect(owner)
        .ownerAddToWhitelist(
          [hoprAddress(1), hoprAddress(2), hoprAddress(3)],
          participantAddresses.slice(1, 4),
          [2000, 100, 100]
        )
    })

    describe('Lower threshold to 100', () => {
      beforeEach(async () => {
        await hoprNetworkRegistry.connect(owner).ownerUpdateThreshold(100)
      })
      const whitelisted = [0, 1, 2, 3]
      const nonWitelisted = [4, 5, 6]
      const cannotSelfRegister = [4, 6]
      const canSelfRegister = [5]

      whitelisted.forEach((accountIndex) => {
        it(`participant ${accountIndex} is still whitelisted`, async () => {
          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(true)
        })
      })
      nonWitelisted.forEach((accountIndex) => {
        it(`participant ${accountIndex} is not whitelisted`, async () => {
          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(false)
        })
      })
      cannotSelfRegister.forEach((accountIndex) => {
        it(`fail to whitelist staker ${accountIndex} with stake of high threshold and non-eligible NFT`, async () => {
          await hoprNetworkRegistry.connect(participants[accountIndex]).addToWhitelist(hoprAddress(accountIndex))
          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(false)
        })
      })
      canSelfRegister.forEach((accountIndex) => {
        it(`whitelist staker ${accountIndex} with stake of high threshold and eligible NFT`, async () => {
          await expect(
            hoprNetworkRegistry.connect(participants[accountIndex]).addToWhitelist(hoprAddress(accountIndex))
          )
            .to.emit(hoprNetworkRegistry, 'AddedToWhitelist')
            .withArgs(participantAddresses[accountIndex], hoprAddress(accountIndex))

          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(true)
        })
      })
    })
    describe('Lift threshold to 2000', () => {
      beforeEach(async () => {
        await hoprNetworkRegistry.connect(owner).ownerUpdateThreshold(2000)
      })
      const whitelisted = [0, 1]
      const nonWitelisted = [2, 3, 4, 5, 6]
      const cannotSelfRegister = [4]
      const cannotSelfRegisterDueToThreshold = [5, 6]
      const canSelfRegister = []

      whitelisted.forEach((accountIndex) => {
        it(`participant ${accountIndex} is still whitelisted`, async () => {
          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(true)
        })
      })
      nonWitelisted.forEach((accountIndex) => {
        it(`participant ${accountIndex} is not whitelisted`, async () => {
          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(false)
        })
      })
      cannotSelfRegister.forEach((accountIndex) => {
        it(`fail to whitelist staker ${accountIndex} with stake of high threshold and non-eligible NFT`, async () => {
          await hoprNetworkRegistry.connect(participants[accountIndex]).addToWhitelist(hoprAddress(accountIndex))
          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(false)
        })
      })
      cannotSelfRegisterDueToThreshold.forEach((accountIndex) => {
        it(`fail to whitelist staker ${accountIndex} with stake of low threshold`, async () => {
          await expect(
            hoprNetworkRegistry.connect(participants[accountIndex]).addToWhitelist(hoprAddress(accountIndex))
          ).to.be.revertedWith('HoprNetworkRegistry: staked amount does not meet the threshold')
        })
      })
      canSelfRegister.forEach((accountIndex) => {
        it(`whitelist staker ${accountIndex} with stake of high threshold and eligible NFT`, async () => {
          await expect(
            hoprNetworkRegistry.connect(participants[accountIndex]).addToWhitelist(hoprAddress(accountIndex))
          )
            .to.emit(hoprNetworkRegistry, 'AddedToWhitelist')
            .withArgs(participantAddresses[accountIndex], hoprAddress(accountIndex))

          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(true)
        })
      })
    })
    describe('Remove eligible NFT', () => {
      beforeEach(async () => {
        await hoprNetworkRegistry.connect(owner).ownerRemoveNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])
      })
      const whitelisted = [1] // still whitelisted because it's manually added by the owner
      const nonWitelisted = [0, 2, 3, 4, 5, 6]
      const cannotSelfRegister = [4]
      const cannotSelfRegisterDueToThreshold = [5, 6]
      const canSelfRegister = []

      whitelisted.forEach((accountIndex) => {
        it(`participant ${accountIndex} is still whitelisted`, async () => {
          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(true)
        })
      })
      nonWitelisted.forEach((accountIndex) => {
        it(`participant ${accountIndex} is not whitelisted`, async () => {
          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(false)
        })
      })
      cannotSelfRegister.forEach((accountIndex) => {
        it(`fail to whitelist staker ${accountIndex} with stake of high threshold and non-eligible NFT`, async () => {
          await hoprNetworkRegistry.connect(participants[accountIndex]).addToWhitelist(hoprAddress(accountIndex))
          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(false)
        })
      })
      cannotSelfRegisterDueToThreshold.forEach((accountIndex) => {
        it(`fail to whitelist staker ${accountIndex} with stake of low threshold`, async () => {
          await expect(
            hoprNetworkRegistry.connect(participants[accountIndex]).addToWhitelist(hoprAddress(accountIndex))
          ).to.be.revertedWith('HoprNetworkRegistry: staked amount does not meet the threshold')
        })
      })
      canSelfRegister.forEach((accountIndex) => {
        it(`whitelist staker ${accountIndex} with stake of high threshold and eligible NFT`, async () => {
          await expect(
            hoprNetworkRegistry.connect(participants[accountIndex]).addToWhitelist(hoprAddress(accountIndex))
          )
            .to.emit(hoprNetworkRegistry, 'AddedToWhitelist')
            .withArgs(participantAddresses[accountIndex], hoprAddress(accountIndex))

          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(true)
        })
      })
    })
    describe('Change eligible NFT', () => {
      beforeEach(async () => {
        await hoprNetworkRegistry.connect(owner).ownerRemoveNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1])
        await hoprNetworkRegistry.connect(owner).ownerAddNftTypeAndRank(NFT_TYPE[1], NFT_RANK[0])
      })
      const whitelisted = [1]
      const nonWitelisted = [0, 2, 3, 4, 5, 6]
      const cannotSelfRegister = []
      const cannotSelfRegisterDueToThreshold = [5, 6]
      const canSelfRegister = [4]

      whitelisted.forEach((accountIndex) => {
        it(`participant ${accountIndex} is still whitelisted`, async () => {
          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(true)
        })
      })
      nonWitelisted.forEach((accountIndex) => {
        it(`participant ${accountIndex} is not whitelisted`, async () => {
          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(false)
        })
      })
      cannotSelfRegister.forEach((accountIndex) => {
        it(`fail to whitelist staker ${accountIndex} with stake of high threshold and non-eligible NFT`, async () => {
          await hoprNetworkRegistry.connect(participants[accountIndex]).addToWhitelist(hoprAddress(accountIndex))
          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(false)
        })
      })
      cannotSelfRegisterDueToThreshold.forEach((accountIndex) => {
        it(`fail to whitelist staker ${accountIndex} with stake of low threshold`, async () => {
          await expect(
            hoprNetworkRegistry.connect(participants[accountIndex]).addToWhitelist(hoprAddress(accountIndex))
          ).to.be.revertedWith('HoprNetworkRegistry: staked amount does not meet the threshold')
        })
      })
      canSelfRegister.forEach((accountIndex) => {
        it(`whitelist staker ${accountIndex} with stake of high threshold and eligible NFT`, async () => {
          await expect(
            hoprNetworkRegistry.connect(participants[accountIndex]).addToWhitelist(hoprAddress(accountIndex))
          )
            .to.emit(hoprNetworkRegistry, 'AddedToWhitelist')
            .withArgs(participantAddresses[accountIndex], hoprAddress(accountIndex))

          expect(await hoprNetworkRegistry.isWhitelisted(hoprAddress(accountIndex))).to.equal(true)
        })
      })
    })
  })
})
