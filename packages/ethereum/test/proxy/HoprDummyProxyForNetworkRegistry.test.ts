import chai, { expect } from 'chai'
import { deployments, ethers } from 'hardhat'
import { smock } from '@defi-wonderland/smock'
import { Contract, Signer } from 'ethers'
import { HoprDummyProxyForNetworkRegistry } from '../../src/types'

chai.should() // if you like should syntax
chai.use(smock.matchers)

const useFixtures = deployments.createFixture(async (_hre) => {
  const [_deployer, owner, ...signers] = await ethers.getSigners()
  const participants = signers.slice(3, 10) // 7 participants

  const ownerAddress = await owner.getAddress()
  const participantAddresses = await Promise.all(participants.map((h) => h.getAddress()))

  // deploy network registry
  const hoprDummyProxyForNetworkRegistry = (await (
    await ethers.getContractFactory('HoprDummyProxyForNetworkRegistry')
  ).deploy(ownerAddress)) as HoprDummyProxyForNetworkRegistry

  return {
    owner,
    participants,
    ownerAddress,
    participantAddresses,
    hoprDummyProxyForNetworkRegistry
  }
})

describe('Registry proxy for stake v2', () => {
  let owner: Signer
  let participantAddresses: string[]
  let hoprDummyProxyForNetworkRegistry: Contract

  describe('Add account(s)', () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprDummyProxyForNetworkRegistry } = await useFixtures())
    })
    it('fails to add account by non-owner', async () => {
      await expect(hoprDummyProxyForNetworkRegistry.ownerAddAccount(participantAddresses[3])).to.be.revertedWith(
        'Ownable: caller is not the owner'
      )
    })
    it('add an account by owner', async () => {
      await expect(hoprDummyProxyForNetworkRegistry.connect(owner).ownerAddAccount(participantAddresses[3]))
        .to.emit(hoprDummyProxyForNetworkRegistry.connect(owner), 'AccountRegistered')
        .withArgs(participantAddresses[3])
    })
    it('add accounts by owner', async () => {
      await hoprDummyProxyForNetworkRegistry.connect(owner).ownerAddAccount(participantAddresses[3])
      await expect(
        hoprDummyProxyForNetworkRegistry.connect(owner).ownerBatchAddAccounts(participantAddresses.slice(3, 6))
      )
        .to.emit(hoprDummyProxyForNetworkRegistry.connect(owner), 'AccountRegistered')
        .withArgs(participantAddresses[4])
        .to.emit(hoprDummyProxyForNetworkRegistry.connect(owner), 'AccountRegistered')
        .withArgs(participantAddresses[5])
    })
  })
  describe('Remove account', () => {
    beforeEach(async () => {
      ;({ owner, participantAddresses, hoprDummyProxyForNetworkRegistry } = await useFixtures())
      await hoprDummyProxyForNetworkRegistry.connect(owner).ownerAddAccount(participantAddresses[3])
    })
    it(`participant is still eligible`, async () => {
      expect(await hoprDummyProxyForNetworkRegistry.isRequirementFulfilled(participantAddresses[3])).to.be.true
    })
    it('fails to remove account by non-owner', async () => {
      await expect(hoprDummyProxyForNetworkRegistry.ownerRemoveAccount(participantAddresses[3])).to.be.revertedWith(
        'Ownable: caller is not the owner'
      )
    })
    it('remove an account by owner', async () => {
      await expect(hoprDummyProxyForNetworkRegistry.connect(owner).ownerRemoveAccount(participantAddresses[3]))
        .to.emit(hoprDummyProxyForNetworkRegistry.connect(owner), 'AccountDeregistered')
        .withArgs(participantAddresses[3])
    })
    it('remove accounts by owner', async () => {
      await expect(
        hoprDummyProxyForNetworkRegistry.connect(owner).ownerBatchRemoveAccounts(participantAddresses.slice(3, 6))
      )
        .to.emit(hoprDummyProxyForNetworkRegistry.connect(owner), 'AccountDeregistered')
        .withArgs(participantAddresses[3])
    })
  })
})
