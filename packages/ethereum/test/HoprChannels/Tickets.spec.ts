import { deployments, ethers } from 'hardhat'
import { expect } from 'chai'
import { ACCOUNT_A, ACCOUNT_B, ACCOUNT_AB_CHANNEL_ID, SECRET_2, generateTickets, SECRET_0 } from './constants'
import { ERC777Mock__factory, TicketsMock__factory } from '../../types'
import deployERC1820Registry from '../../deploy/01_ERC1820Registry'

const abiEncoder = ethers.utils.Interface.getAbiCoder()

const useFixtures = deployments.createFixture(async (hre, { secsClosure }: { secsClosure?: string } = {}) => {
  const [deployer] = await ethers.getSigners()

  // deploy ERC1820Registry required by ERC777 token
  await deployERC1820Registry(hre, deployer)

  // deploy ERC777Mock
  const token = await new ERC777Mock__factory(deployer).deploy(deployer.address, '100', 'Token', 'TKN', [])
  // deploy TicketsMock
  const tickets = await new TicketsMock__factory(deployer).deploy(token.address, secsClosure ?? '0')

  // mocked tickets
  const mockedTickets = await generateTickets()

  return {
    token,
    tickets,
    deployer: deployer.address,
    ...mockedTickets
  }
})

describe('Tickets', function () {
  it('should redeem ticket', async function () {
    const { tickets, deployer, TICKET_AB_WIN } = await useFixtures()

    await tickets.initializeAccountInternal(ACCOUNT_B.address, ACCOUNT_B.uncompressedPublicKey, SECRET_2)
    await tickets.fundChannelInternal(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)

    // TODO: add event check
    await tickets.redeemTicketInternal(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.counterparty,
      TICKET_AB_WIN.secret,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.r,
      TICKET_AB_WIN.s,
      TICKET_AB_WIN.v
    )

    const ticket = await tickets.tickets(TICKET_AB_WIN.hash)
    expect(ticket).to.be.true

    const channel = await tickets.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.partyABalance.toString()).to.equal('60')
    expect(channel.partyBBalance.toString()).to.equal('40')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('1')
    expect(channel.closureByPartyA).to.be.false

    const account = await tickets.accounts(ACCOUNT_B.address)
    expect(account.secret).to.equal(TICKET_AB_WIN.secret)
  })

  it('should fail to redeem ticket when channel in closed', async function () {
    const { tickets, TICKET_AB_WIN } = await useFixtures()

    await tickets.initializeAccountInternal(ACCOUNT_B.address, ACCOUNT_B.uncompressedPublicKey, SECRET_2)

    await expect(
      tickets.redeemTicketInternal(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        TICKET_AB_WIN.secret,
        TICKET_AB_WIN.ticketEpoch,
        TICKET_AB_WIN.ticketIndex,
        TICKET_AB_WIN.proofOfRelaySecret,
        TICKET_AB_WIN.amount,
        TICKET_AB_WIN.winProb,
        TICKET_AB_WIN.r,
        TICKET_AB_WIN.s,
        TICKET_AB_WIN.v
      )
    ).to.be.revertedWith('channel must be open or pending to close')
  })

  it('should fail to redeem ticket when channel in in different iteration', async function () {
    const { token, tickets, deployer, TICKET_AB_WIN } = await useFixtures()

    await tickets.initializeAccountInternal(ACCOUNT_B.address, ACCOUNT_B.uncompressedPublicKey, SECRET_2)

    // transfer tokens to contract
    await token.send(
      tickets.address,
      '100',
      abiEncoder.encode(
        ['bool', 'address', 'address', 'uint256', 'uint256'],
        [false, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
      ),
      {
        from: deployer
      }
    )
    // open channel and then close it
    await tickets.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)
    await tickets.initiateChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address)
    await tickets.finalizeChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address)
    // refund and open channel
    await tickets.fundChannelInternal(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)

    await expect(
      tickets.redeemTicketInternal(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        TICKET_AB_WIN.secret,
        TICKET_AB_WIN.ticketEpoch,
        TICKET_AB_WIN.ticketIndex,
        TICKET_AB_WIN.proofOfRelaySecret,
        TICKET_AB_WIN.amount,
        TICKET_AB_WIN.winProb,
        TICKET_AB_WIN.r,
        TICKET_AB_WIN.s,
        TICKET_AB_WIN.v
      )
    ).to.be.revertedWith('signer must match the counterparty')
  })

  it('should fail to redeem ticket when ticket has been already redeemed', async function () {
    const { tickets, deployer, TICKET_AB_WIN } = await useFixtures()

    await tickets.initializeAccountInternal(ACCOUNT_B.address, ACCOUNT_B.uncompressedPublicKey, SECRET_2)
    await tickets.fundChannelInternal(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)

    await tickets.redeemTicketInternal(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.counterparty,
      TICKET_AB_WIN.secret,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.r,
      TICKET_AB_WIN.s,
      TICKET_AB_WIN.v
    )

    await expect(
      tickets.redeemTicketInternal(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        SECRET_0, // give the next secret so this ticket becomes redeemable
        TICKET_AB_WIN.ticketEpoch,
        TICKET_AB_WIN.ticketIndex,
        TICKET_AB_WIN.proofOfRelaySecret,
        TICKET_AB_WIN.amount,
        TICKET_AB_WIN.winProb,
        TICKET_AB_WIN.r,
        TICKET_AB_WIN.s,
        TICKET_AB_WIN.v
      )
    ).to.be.revertedWith('ticket must not be used twice')
  })

  it('should fail to redeem ticket when signer is not the issuer', async function () {
    const { tickets, deployer, TICKET_AB_WIN, TICKET_BA_WIN } = await useFixtures()

    await tickets.initializeAccountInternal(ACCOUNT_B.address, ACCOUNT_B.uncompressedPublicKey, SECRET_2)
    await tickets.fundChannelInternal(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)

    await expect(
      tickets.redeemTicketInternal(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        TICKET_AB_WIN.secret,
        TICKET_AB_WIN.ticketEpoch,
        TICKET_AB_WIN.ticketIndex,
        TICKET_AB_WIN.proofOfRelaySecret,
        TICKET_AB_WIN.amount,
        TICKET_AB_WIN.winProb,
        TICKET_BA_WIN.r, // signature from different ticket
        TICKET_BA_WIN.s, // signature from different ticket
        TICKET_AB_WIN.v
      )
    ).to.be.revertedWith('signer must match the counterparty')
  })

  it("should fail to redeem ticket if it's a loss", async function () {
    const { tickets, deployer, TICKET_AB_LOSS } = await useFixtures()

    await tickets.initializeAccountInternal(ACCOUNT_B.address, ACCOUNT_B.uncompressedPublicKey, SECRET_2)
    await tickets.fundChannelInternal(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)

    await expect(
      tickets.redeemTicketInternal(
        TICKET_AB_LOSS.recipient,
        TICKET_AB_LOSS.counterparty,
        TICKET_AB_LOSS.secret,
        TICKET_AB_LOSS.ticketEpoch,
        TICKET_AB_LOSS.ticketIndex,
        TICKET_AB_LOSS.proofOfRelaySecret,
        TICKET_AB_LOSS.amount,
        TICKET_AB_LOSS.winProb,
        TICKET_AB_LOSS.r,
        TICKET_AB_LOSS.s,
        TICKET_AB_LOSS.v
      )
    ).to.be.revertedWith('ticket must be a win')
  })

  it('should pack ticket', async function () {
    const { tickets, TICKET_AB_WIN } = await useFixtures()

    const encoded = await tickets.getEncodedTicketInternal(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.counter,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.iteration,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb
    )
    expect(encoded).to.equal(TICKET_AB_WIN.encoded)
  })

  it('should hash ticket', async function () {
    const { tickets, TICKET_AB_WIN } = await useFixtures()

    const hash = await tickets.getTicketHashInternal(TICKET_AB_WIN.encoded)
    expect(hash).to.equal(TICKET_AB_WIN.hash)
  })

  it("should get ticket's luck", async function () {
    const { tickets, TICKET_AB_WIN } = await useFixtures()

    const luck = await tickets.getTicketLuckInternal(
      TICKET_AB_WIN.hash,
      TICKET_AB_WIN.secret,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.winProb
    )
    expect(luck).to.equal(TICKET_AB_WIN.luck)
  })
})
