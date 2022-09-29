import chai, { expect } from 'chai'
import { deployments, ethers } from 'hardhat'
import { FakeContract, smock } from '@defi-wonderland/smock'
import { constants, Contract, Signer } from 'ethers'
import { HoprNetworkRegistry } from '../src/types'
import { PANIC_CODES } from '@nomicfoundation/hardhat-chai-matchers/panic'

chai.should() // if you like should syntax
chai.use(smock.matchers)

const hoprAddress = (i: number) => `16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x${i}`

const createFakeRegistryProxyContract = async (participants: string[]) => {
  const hoprNetworkRegistryRequirementFake = await smock.fake([
    {
      inputs: [
        {
          internalType: 'address',
          name: 'account',
          type: 'address'
        }
      ],
      name: 'maxAllowedRegistrations',
      outputs: [
        {
          internalType: 'uint256',
          name: 'account',
          type: 'uint256'
        }
      ],
      stateMutability: 'view',
      type: 'function'
    }
  ])

  participants.slice(0, 2).forEach((participant) => {
    // account 0, 1 return max uint256
    // Network_registry NFT like
    hoprNetworkRegistryRequirementFake.maxAllowedRegistrations.whenCalledWith(participant).returns(constants.MaxUint256)
  })
  participants.slice(2, 4).forEach((participant) => {
    // account 2, 3 return 1
    hoprNetworkRegistryRequirementFake.maxAllowedRegistrations.whenCalledWith(participant).returns(1)
  })
  participants.slice(4, 6).forEach((participant) => {
    // account 4, 5 return 0
    hoprNetworkRegistryRequirementFake.maxAllowedRegistrations.whenCalledWith(participant).returns(0)
  })
  return hoprNetworkRegistryRequirementFake
}

const useFixtures = deployments.createFixture(async (_hre) => {
  const [_deployer, owner, ...signers] = await ethers.getSigners()
  const participants = signers.slice(3, 10) // 7 participants

  const ownerAddress = await owner.getAddress()
  const participantAddresses = await Promise.all(participants.map((h) => h.getAddress()))

  // mock staking contract
  const registryFake = await createFakeRegistryProxyContract(participantAddresses)

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
    it('is enabled globally', async () => {
      expect(await hoprNetworkRegistry.enabled()).to.be.true
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
      await expect(hoprNetworkRegistry.enableRegistry()).to.be.revertedWith('Ownable: caller is not the owner')
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
      await expect(hoprNetworkRegistry.connect(owner).disableRegistry())
      await expect(hoprNetworkRegistry.connect(owner).disableRegistry()).to.be.revertedWith(
        'HoprNetworkRegistry: Registry is disabled'
      )
    })
    it('owner enable the registry', async () => {
      await expect(hoprNetworkRegistry.connect(owner).disableRegistry())
      await expect(hoprNetworkRegistry.connect(owner).enableRegistry())
        .to.emit(hoprNetworkRegistry, 'EnabledNetworkRegistry')
        .withArgs(true)
    })
  })
  describe('Register contract for a single time', () => {
    beforeEach(async () => {
      ;({ owner, participants, participantAddresses, registryFake, hoprNetworkRegistry } = await useFixtures())
    })
    it('can self-register when the requirement is fulfilled and emits true', async () => {
      // account 0 registers hoprAddress[0] and hoprAddress[1]
      const participantIndex = 0
      await expect(
        hoprNetworkRegistry
          .connect(participants[participantIndex])
          .selfRegister([hoprAddress(participantIndex), hoprAddress(participantIndex + 1)])
      )
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[participantIndex], true)
        .to.emit(hoprNetworkRegistry, 'Registered')
        .withArgs(participantAddresses[participantIndex], hoprAddress(participantIndex))
        .to.emit(hoprNetworkRegistry, 'Registered')
        .withArgs(participantAddresses[participantIndex], hoprAddress(participantIndex + 1))
    })
    it('cannot self-register when trying to register more than the limit', async () => {
      // account 2 fail to register hoprAddress[2] and hoprAddress[3]
      const participantIndex = 2
      await expect(
        hoprNetworkRegistry
          .connect(participants[participantIndex])
          .selfRegister([hoprAddress(participantIndex), hoprAddress(participantIndex + 1)])
      ).to.be.revertedWith('HoprNetworkRegistry: selfRegister reaches limit, cannot register requested nodes.')
    })
    it('cannot self-register when the requirement is not fulfilled', async () => {
      const participantIndex = 4
      await expect(
        hoprNetworkRegistry.connect(participants[participantIndex]).selfRegister([hoprAddress(participantIndex)])
      ).to.be.revertedWith('HoprNetworkRegistry: selfRegister reaches limit, cannot register requested nodes.')
    })
    it('fail to register when hopr node address is empty', async () => {
      await expect(hoprNetworkRegistry.connect(participants[0]).selfRegister([''])).to.be.revertedWithCustomError(
        hoprNetworkRegistry,
        'InvalidPeerId'
      )
    })
    it('fail to register when hopr node address of wrong length', async () => {
      await expect(
        hoprNetworkRegistry.connect(participants[0]).selfRegister(['16Uiu2HA'])
      ).to.be.revertedWithCustomError(hoprNetworkRegistry, 'InvalidPeerId')
    })
    it('fail to register when hopr node address is of the right length but with wrong prefix', async () => {
      await expect(
        hoprNetworkRegistry.connect(participants[0]).selfRegister([`0x${hoprAddress(5).slice(2)}`])
      ).to.be.revertedWithCustomError(hoprNetworkRegistry, 'InvalidPeerId')
    })
    it('fail to when array length does not match', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerRegister([participantAddresses[5], participantAddresses[6]], [hoprAddress(5)])
      ).to.be.rejectedWith('HoprNetworkRegistry: hoprPeerIdes and accounts lengths mismatch')
    })
    it('can register by the owner and emit RegisteredByOwner', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerRegister([participantAddresses[5], participantAddresses[6]], [hoprAddress(5), hoprAddress(6)])
      )
        .to.emit(hoprNetworkRegistry, 'RegisteredByOwner')
        .withArgs(participantAddresses[5], hoprAddress(5))
        .to.emit(hoprNetworkRegistry, 'RegisteredByOwner')
        .withArgs(participantAddresses[6], hoprAddress(6))
    })
    it('can be deregistered by the owner when an address was not registered. Nothing gets emitted', async () => {
      expect(await hoprNetworkRegistry.countRegisterdNodesPerAccount(participantAddresses[5])).to.equal(0)
      await hoprNetworkRegistry.connect(owner).ownerDeregister([participantAddresses[5]])
      expect(await hoprNetworkRegistry.countRegisterdNodesPerAccount(participantAddresses[5])).to.equal(0)
    })
    it('can be deregistered by the owner when an address was registered', async () => {
      await hoprNetworkRegistry.connect(owner).ownerRegister([participantAddresses[5]], [hoprAddress(5)])
      await expect(hoprNetworkRegistry.connect(owner).ownerDeregister([hoprAddress(5)]))
        .to.emit(hoprNetworkRegistry, 'DeregisteredByOwner')
        .withArgs(participantAddresses[5], hoprAddress(5))
      // .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
      // .withArgs(participantAddresses[5], false)
    })
  })
  describe('Owner force update eligibility', () => {
    beforeEach(async () => {
      ;({ owner, participants, participantAddresses, registryFake, hoprNetworkRegistry } = await useFixtures())
      // owner register participant 1 and 5 with address 1 and 5
      await hoprNetworkRegistry
        .connect(owner)
        .ownerRegister([participantAddresses[1], participantAddresses[5]], [hoprAddress(1), hoprAddress(5)])
    })
    it('can force update eligibility of an account independantly (true), and sync back to its actual eligibility (false)', async () => {
      const ineligibleAccountIndex = 5
      await expect(
        hoprNetworkRegistry.connect(owner).ownerForceEligibility([participantAddresses[ineligibleAccountIndex]], [true])
      )
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[ineligibleAccountIndex], true)
      await expect(hoprNetworkRegistry.connect(owner).sync([hoprAddress(ineligibleAccountIndex)]))
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[ineligibleAccountIndex], false)
    })
    it('can force update eligibility of an account independantly (false), and sync back to its actual eligibility (true)', async () => {
      const eligibleAccountIndex = 1
      await expect(
        hoprNetworkRegistry.connect(owner).ownerForceEligibility([participantAddresses[eligibleAccountIndex]], [false])
      )
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[eligibleAccountIndex], false)
      await expect(hoprNetworkRegistry.connect(owner).sync([hoprAddress(eligibleAccountIndex)]))
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[eligibleAccountIndex], true)
    })
    it('can force update eligibility of an account independantly (true), and sync back to its actual eligibility (true)', async () => {
      const eligibleAccountIndex = 1
      await expect(
        hoprNetworkRegistry.connect(owner).ownerForceEligibility([participantAddresses[eligibleAccountIndex]], [true])
      )
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[eligibleAccountIndex], true)
      await expect(hoprNetworkRegistry.connect(owner).sync([hoprAddress(eligibleAccountIndex)]))
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[eligibleAccountIndex], true)
    })
    it('can force update eligibility of an account independantly (false), and sync back to its actual eligibility (false)', async () => {
      const ineligibleAccountIndex = 5
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerForceEligibility([participantAddresses[ineligibleAccountIndex]], [false])
      )
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[ineligibleAccountIndex], false)
      await expect(hoprNetworkRegistry.connect(owner).sync([hoprAddress(ineligibleAccountIndex)]))
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[ineligibleAccountIndex], false)
    })
  })

  describe('Register contract for multiple times by one', () => {
    const participantIndex = 1
    beforeEach(async () => {
      ;({ owner, participants, participantAddresses, registryFake, hoprNetworkRegistry } = await useFixtures())
      // participant successfully registered itself
      hoprNetworkRegistry.connect(participants[participantIndex]).selfRegister([hoprAddress(participantIndex)])
    })
    it('fails to deregister an non-registered account. Panic when the deregister originial account has zero registered node', async () => {
      await expect(
        hoprNetworkRegistry.connect(participants[participantIndex + 1]).selfDeregister([hoprAddress(participantIndex)])
      ).to.be.revertedWithPanic(PANIC_CODES.ARITHMETIC_UNDER_OR_OVERFLOW)
    })
    it('fails to deregister an non-registered account', async () => {
      hoprNetworkRegistry.connect(participants[participantIndex + 1]).selfRegister([hoprAddress(participantIndex + 1)])
      await expect(
        hoprNetworkRegistry.connect(participants[participantIndex + 1]).selfDeregister([hoprAddress(participantIndex)])
      ).to.be.revertedWith('HoprNetworkRegistry: Cannot delete an entry not associated with the caller.')
    })
    it('can deregister by itself', async () => {
      await expect(
        hoprNetworkRegistry.connect(participants[participantIndex]).selfDeregister([hoprAddress(participantIndex)])
      )
        .to.emit(hoprNetworkRegistry, 'Deregistered')
        .withArgs(participantAddresses[participantIndex], hoprAddress(participantIndex))
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[participantIndex], true)
    })
    it('fails to register the node address by a different account', async () => {
      await expect(
        hoprNetworkRegistry.connect(participants[participantIndex + 1]).selfRegister([hoprAddress(participantIndex)])
      ).to.be.revertedWith('HoprNetworkRegistry: Cannot link a registered node.')
    })
    it('can register an additional peer ID', async () => {
      await expect(
        hoprNetworkRegistry.connect(participants[participantIndex]).selfRegister([hoprAddress(participantIndex + 1)])
      )
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[participantIndex], true)
        .to.emit(hoprNetworkRegistry, 'Registered')
        .withArgs(participantAddresses[participantIndex], hoprAddress(participantIndex + 1))
    })
    it('self-registered account emits true when the requirement is fulfilled, but no longer emits Registered event', async () => {
      await expect(
        hoprNetworkRegistry.connect(participants[participantIndex]).selfRegister([hoprAddress(participantIndex)])
      )
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[participantIndex], true)
    })
    it('fails self-registered when the requirement is not fulfilled', async () => {
      // second time call - allowed registration reaches its limit of 1
      registryFake.maxAllowedRegistrations.whenCalledWith(participantAddresses[participantIndex]).returns(1)
      await expect(
        hoprNetworkRegistry.connect(participants[participantIndex]).selfRegister([hoprAddress(participantIndex)])
      ).to.be.revertedWith('HoprNetworkRegistry: selfRegister reaches limit, cannot register requested nodes.')
    })
  })
  describe('Force emit an eligibility update ', () => {
    it('owner can force emit an eligibility update', async () => {
      await expect(
        hoprNetworkRegistry
          .connect(owner)
          .ownerForceEligibility(
            [participantAddresses[0], participantAddresses[2], participantAddresses[4]],
            [false, true, true]
          )
      )
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[0], false)
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[2], true)
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[4], true)
    })
  })
  describe('Sync with when criteria change', () => {
    const participantIndex = 2
    beforeEach(async () => {
      ;({ owner, participants, participantAddresses, registryFake, hoprNetworkRegistry } = await useFixtures())
      // self-register when it's still eligible
      registryFake.maxAllowedRegistrations.whenCalledWith(participantAddresses[participantIndex]).returns(1)
      await hoprNetworkRegistry.connect(participants[participantIndex]).selfRegister([hoprAddress(participantIndex)])
    })
    it('owner can sync the criteria, before criteria change', async () => {
      await expect(
        hoprNetworkRegistry.connect(owner).sync([hoprAddress(participantIndex), hoprAddress(0), hoprAddress(4)])
      )
        .to.emit(hoprNetworkRegistry, 'EligibilityUpdated')
        .withArgs(participantAddresses[participantIndex], true)
    })
    it('owner can sync the criteria, after criteria change', async () => {
      // second time call - requirement is reduced to zero
      registryFake.maxAllowedRegistrations.whenCalledWith(participantAddresses[participantIndex]).returns(0)

      await expect(
        hoprNetworkRegistry.connect(owner).sync([hoprAddress(participantIndex), hoprAddress(0), hoprAddress(4)])
      )
        // only participant[participantIndex] is registered
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
      registryFake.maxAllowedRegistrations.whenCalledWith(participantAddresses[participantIndex]).returns(0)
      expect(await hoprNetworkRegistry.isAccountRegisteredAndEligible(participantAddresses[participantIndex])).to.be
        .false
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
      registryFake.maxAllowedRegistrations.whenCalledWith(participantAddresses[participantIndex]).returns(0)
      expect(await hoprNetworkRegistry.isNodeRegisteredAndEligible(hoprAddress(participantIndex))).to.be.false
    })
  })
})
