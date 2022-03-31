import chai, { expect } from 'chai'
import { deployments, ethers } from 'hardhat'
import { FakeContract, smock } from '@defi-wonderland/smock'
import { constants, Contract, Signer } from 'ethers'
import { HoprNetworkRegistry } from '../src/types'

chai.should() // if you like should syntax
chai.use(smock.matchers)

const hoprAddress = (i: number) => `16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x${i}`

const createFakeRegistryContract = async (participants: string[]) => {
  const hoprNetworkRegistryRequirementFake = await smock.fake([
    {
      inputs: [
        {
          internalType: 'address',
          name: 'account',
          type: 'address'
        }
      ],
      name: 'isRequirementFulfilled',
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

  participants.slice(0, 3).forEach((participant) => {
    // account 0, 1, 2 return true
    hoprNetworkRegistryRequirementFake.isRequirementFulfilled.whenCalledWith(participant).returns(true)
  })
  participants.slice(3, 6).forEach((participant) => {
    // account 3, 4, 5 return false
    hoprNetworkRegistryRequirementFake.isRequirementFulfilled.whenCalledWith(participant).returns(false)
  })
  return hoprNetworkRegistryRequirementFake
}

const useFixtures = deployments.createFixture(async (_hre) => {
  const [_deployer, owner, ...signers] = await ethers.getSigners()
  const participants = signers.slice(3, 10) // 7 participants

  const ownerAddress = await owner.getAddress()
  const participantAddresses = await Promise.all(participants.map((h) => h.getAddress()))

  // mock staking contract
  const registryFake = await createFakeRegistryContract(participantAddresses)

  // deploy network registry
  const hoprNetworkRegistry = (await (
    await ethers.getContractFactory('HoprNetworkRegistry')
  ).deploy(registryFake.address, ownerAddress)) as HoprNetworkRegistry

  return {
    owner,
    participants,
    ownerAddress,
    participantAddresses,
    registryFake,
    hoprNetworkRegistry
  }
})

describe('HoprNetworkRegistry', () => {
  let owner: Signer
  let participants: Signer[]
  let participantAddresses: string[]
  let registryFake: FakeContract
  let hoprNetworkRegistry: Contract

  describe('Owner can update important parameters of the contract', () => {
    beforeEach(async () => {
      ;({ owner, participants, participantAddresses, hoprNetworkRegistry } = await useFixtures())
    })
    it('owner to update the registry', async () => {
      // const {deployer, owner, participants, ownerAddress, participantAddresses, stakeV2Fake, hoprNetworkRegistry } = await useFixtures()
      await expect(hoprNetworkRegistry.connect(owner).updateRequirementImplementation(constants.AddressZero))
        .to.emit(hoprNetworkRegistry, 'RequirementUpdated')
        .withArgs(constants.AddressZero)
    })
    it('fail to update the registry', async () => {
      await expect(hoprNetworkRegistry.updateRequirementImplementation(constants.AddressZero)).to.be.revertedWith(
        'Ownable: caller is not the owner'
      )
    })
    it('fail to enable the registry by non-owner account', async () => {
      await expect(hoprNetworkRegistry.enableRegistry()).to.be.revertedWith(
        'Ownable: caller is not the owner'
      )
    })
    it('failed to enable the registry by owner', async () => {
      await expect(hoprNetworkRegistry.connect(owner).enableRegistry()).to.be.revertedWith(
        'HoprNetworkRegistry: Registry is enabled'
      )
    })
    it('owner disable the registry', async () => {
      await expect(hoprNetworkRegistry.connect(owner).disableRegistry())
        .to.emit(hoprNetworkRegistry, 'EnabledNetworkRegistry')
        .withArgs(false)
    })
    it('failed to disable the registry by owner', async () => {
      await expect(hoprNetworkRegistry.connect(owner).disableRegistry());
      await expect(hoprNetworkRegistry.connect(owner).disableRegistry()).to.be.revertedWith(
        'HoprNetworkRegistry: Registry is disabled'
      )
    })
    it('owner enable the registry', async () => {
      await expect(hoprNetworkRegistry.connect(owner).disableRegistry());
      await expect(hoprNetworkRegistry.connect(owner).enableRegistry())
        .to.emit(hoprNetworkRegistry, 'EnabledNetworkRegistry')
        .withArgs(true)
    })
  })
  describe('Register contract', () => {
    beforeEach(async () => {
      ;({ owner, participants, participantAddresses, registryFake, hoprNetworkRegistry } = await useFixtures())
    })
    it('can self-register when the requirement is fulfilled and emits false', async () => {
      const participantIndex = 1
      await expect(
        hoprNetworkRegistry.connect(participants[participantIndex]).selfRegister(hoprAddress(participantIndex))
      )
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[participantIndex], true)
        .to.emit(hoprNetworkRegistry, 'Registered')
        .withArgs(participantAddresses[participantIndex], hoprAddress(participantIndex))
    })
    it('can self-register when the requirement is not fulfilled, but emits nothing', async () => {
      const participantIndex = 3
      const tx = await hoprNetworkRegistry
        .connect(participants[participantIndex])
        .selfRegister(hoprAddress(participantIndex))
      expect(tx.value.toString()).to.be.equal('0')
    })
    it('fail to when array length does not match', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerRegister([participantAddresses[5], participantAddresses[6]], [hoprAddress(5)])
      ).to.be.revertedWith('HoprNetworkRegistry: hoprAddresses and accounts lengths mismatch')
    })
    it('can register by the owner', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerRegister([participantAddresses[5], participantAddresses[6]], [hoprAddress(5), hoprAddress(6)])
      )
        .to.emit(hoprNetworkRegistry, 'RegisteredByOwner')
        .withArgs(participantAddresses[5], hoprAddress(5))
        .to.emit(hoprNetworkRegistry, 'RegisteredByOwner')
        .withArgs(participantAddresses[6], hoprAddress(6))
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[5], true)
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[6], true)
    })
    it('can be deregistered by the owner', async () => {
      await expect(hoprNetworkRegistry.connect(owner).ownerDeregister([participantAddresses[5]]))
        .to.emit(hoprNetworkRegistry, 'DeregisteredByOwner')
        .withArgs(participantAddresses[5])
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[5], false)
    })
  })
  describe('Sync with when criteria change', () => {
    const participantIndex = 1
    beforeEach(async () => {
      ;({ owner, participants, participantAddresses, registryFake, hoprNetworkRegistry } = await useFixtures())
      // // first time call - requirement is reverted
      registryFake.isRequirementFulfilled.whenCalledWith(participantAddresses[participantIndex]).returns(true)
      // self-register when it's still eligible
      await hoprNetworkRegistry.connect(participants[participantIndex]).selfRegister(hoprAddress(participantIndex))
    })
    it('owner can sync the criteria, before criteria change', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .sync([participantAddresses[participantIndex], participantAddresses[0], participantAddresses[4]])
      )
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[participantIndex], true)
    })
    it('owner can sync the criteria, after criteria change', async () => {
      // second time call - requirement is reverted
      registryFake.isRequirementFulfilled.whenCalledWith(participantAddresses[participantIndex]).returns(false)

      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .sync([participantAddresses[participantIndex], participantAddresses[0], participantAddresses[4]])
      )
        // only participant[participantIndex] is registered
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[participantIndex], false)
    })
    it('cannot self-register with a new address when the requirement is fulfilled and emits false', async () => {
      // second time call - requirement is reverted
      registryFake.isRequirementFulfilled.whenCalledWith(participantAddresses[participantIndex]).returns(false)

      await expect(hoprNetworkRegistry.connect(participants[participantIndex]).selfRegister(hoprAddress(9)))
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[participantIndex], false)
    })
    it('anyone can check the eligibility', async () => {
      expect(await hoprNetworkRegistry.isAccountRegisteredAndEligible(participantAddresses[participantIndex])).to.be
        .true
    })
    it('anyone can check the eligibility, it returns false although it meets criteria but not registered', async () => {
      expect(await hoprNetworkRegistry.isAccountRegisteredAndEligible(participantAddresses[0])).to.be.false
    })
    it('anyone can check the eligibility, it returns false when the criteria is not met', async () => {
      expect(await hoprNetworkRegistry.isAccountRegisteredAndEligible(participantAddresses[4])).to.be.false
    })
    it('anyone can check the eligibility, it returns false the criteria changes', async () => {
      // second time call - requirement is reverted
      registryFake.isRequirementFulfilled.whenCalledWith(participantAddresses[participantIndex]).returns(false)
      expect(await hoprNetworkRegistry.isAccountRegisteredAndEligible(participantAddresses[participantIndex])).to.be
        .false
    })
    it('self-registered account emits false when the requirement is not fulfilled', async () => {
      // second time call - requirement is reverted
      registryFake.isRequirementFulfilled.whenCalledWith(participantAddresses[participantIndex]).returns(false)

      await expect(
        hoprNetworkRegistry.connect(participants[participantIndex]).selfRegister(hoprAddress(participantIndex))
      )
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[participantIndex], false)
    })
    it('anyone can check the node eligibility', async () => {
      expect(await hoprNetworkRegistry.isNodeRegisteredAndEligible(hoprAddress(participantIndex))).to.be.true
    })
    it('anyone can check the node eligibility, it returns false although it meets criteria but not registered', async () => {
      expect(await hoprNetworkRegistry.isNodeRegisteredAndEligible(hoprAddress(0))).to.be.false
    })
    it('anyone can check the node eligibility, it returns false when the criteria is not met', async () => {
      expect(await hoprNetworkRegistry.isNodeRegisteredAndEligible(hoprAddress(4))).to.be.false
    })
    it('anyone can check the node eligibility, it returns false the criteria changes', async () => {
      // second time call - requirement is reverted
      registryFake.isRequirementFulfilled.whenCalledWith(participantAddresses[participantIndex]).returns(false)
      expect(await hoprNetworkRegistry.isNodeRegisteredAndEligible(hoprAddress(participantIndex))).to.be.false
    })
  })
})
