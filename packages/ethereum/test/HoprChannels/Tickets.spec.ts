import { deployments, ethers } from 'hardhat'
import { expect } from 'chai'
import { ACCOUNT_A, ACCOUNT_B, ACCOUNT_AB_CHANNEL_ID, generateTickets, SECRET_0, SECRET_2 } from './constants'
import { ERC777Mock__factory, TicketsMock__factory } from '../../types'
import deployERC1820Registry from '../../deploy/01_ERC1820Registry'

ACCOUNT_A.wallet = ACCOUNT_B.wallet.connect(ethers.provider)
ACCOUNT_B.wallet = ACCOUNT_B.wallet.connect(ethers.provider)

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
    await tickets.connect(ACCOUNT_B.wallet).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await tickets.fundChannelInternal(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

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
      TICKET_AB_WIN.signature
    )

    const ticket = await tickets.tickets(TICKET_AB_WIN.hash)
    expect(ticket).to.be.true

    const channel = await tickets.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.partyABalance.toString()).to.equal('60')
    expect(channel.partyBBalance.toString()).to.equal('40')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('1')
    expect(channel.closureByPartyA).to.be.false
    expect(channel.partyACommitment).to.equal(TICKET_AB_WIN.secret)
  })

  it('should fail to redeem ticket when channel in closed', async function () {
    const { tickets, TICKET_AB_WIN } = await useFixtures()
    await tickets.connect(ACCOUNT_B.wallet).bumpChannel(ACCOUNT_A.address, SECRET_2)

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
        TICKET_AB_WIN.signature
      )
    ).to.be.revertedWith('channel must be open or pending to close')
  })

  it('should fail to redeem ticket when channel in in different iteration', async function () {
    const { tickets, deployer, TICKET_AB_WIN } = await useFixtures()
    await tickets.connect(ACCOUNT_B.wallet).bumpChannel(ACCOUNT_A.address, SECRET_2)

    // transfer tokens to contract
    await tickets.fundChannelInternal(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    // open channel and then close it
    await tickets.initiateChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address)
    await tickets.finalizeChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address)
    // refund and open channel
    await tickets.fundChannelInternal(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

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
        TICKET_AB_WIN.signature
      )
    ).to.be.revertedWith('signer must match the counterparty')
  })

  it('should fail to redeem ticket when ticket has been already redeemed', async function () {
    const { tickets, deployer, TICKET_AB_WIN } = await useFixtures()

    await tickets.connect(ACCOUNT_B.wallet).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await tickets.fundChannelInternal(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

    await tickets.redeemTicketInternal(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.counterparty,
      TICKET_AB_WIN.secret,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
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
        TICKET_AB_WIN.signature
      )
    ).to.be.revertedWith('ticket must not be used twice')
  })

  it('should fail to redeem ticket when signer is not the issuer', async function () {
    const { tickets, deployer, TICKET_AB_WIN, TICKET_BA_WIN } = await useFixtures()

    await tickets.connect(ACCOUNT_B.wallet).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await tickets.fundChannelInternal(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

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
        TICKET_BA_WIN.signature // signature from different ticket
      )
    ).to.be.revertedWith('signer must match the counterparty')
  })

  it("should fail to redeem ticket if it's a loss", async function () {
    const { tickets, deployer, TICKET_AB_LOSS } = await useFixtures()

    await tickets.connect(ACCOUNT_B.wallet).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await tickets.fundChannelInternal(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

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
        TICKET_AB_LOSS.signature
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
