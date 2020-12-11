import { deployments } from 'hardhat'
import { expectRevert, singletons } from '@openzeppelin/test-helpers'
import { formatAccount, formatChannel } from './utils'
import { vmErrorMessage } from '../utils'
import {
  ACCOUNT_A,
  ACCOUNT_B,
  ACCOUNT_AB_CHANNEL_ID,
  SECRET_2,
  TICKET_AB_WIN,
  TICKET_BA_WIN,
  TICKET_AB_LOSS,
  SECRET_0
} from './constants'

const ERC777 = artifacts.require('ERC777Mock')
const Tickets = artifacts.require('TicketsMock')

const useFixtures = deployments.createFixture(async (_deployments, { secsClosure }: { secsClosure?: string } = {}) => {
  const [deployer] = await web3.eth.getAccounts()

  // deploy ERC1820Registry required by ERC777 token
  await singletons.ERC1820Registry(deployer)

  // deploy ERC777Mock
  const token = await ERC777.new(deployer, '100', 'Token', 'TKN', [])
  // deploy TicketsMock
  const tickets = await Tickets.new(secsClosure ?? '0')

  return {
    token,
    tickets,
    deployer
  }
})

describe('Tickets', function () {
  it('should redeem ticket', async function () {
    const { tickets, deployer } = await useFixtures()

    await tickets.initializeAccount(ACCOUNT_B.address, ACCOUNT_B.pubKeyFirstHalf, ACCOUNT_B.pubKeySecondHalf, SECRET_2)
    await tickets.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await tickets.redeemTicket(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.counterparty,
      TICKET_AB_WIN.secret,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.r,
      TICKET_AB_WIN.s,
      TICKET_AB_WIN.v
    )

    const ticket = await tickets.tickets(TICKET_AB_WIN.hash)
    expect(ticket).to.be.true

    const channel = await tickets.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('100')
    expect(channel.partyABalance.toString()).to.equal('60')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('1')
    expect(channel.closureByPartyA).to.be.false

    const account = await tickets.accounts(ACCOUNT_B.address).then(formatAccount)
    expect(account.secret).to.equal(TICKET_AB_WIN.secret)
  })

  it('should fail to redeem ticket when channel in closed', async function () {
    const { tickets } = await useFixtures()

    await tickets.initializeAccount(ACCOUNT_B.address, ACCOUNT_B.pubKeyFirstHalf, ACCOUNT_B.pubKeySecondHalf, SECRET_2)

    await expectRevert(
      tickets.redeemTicket(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        TICKET_AB_WIN.secret,
        TICKET_AB_WIN.proofOfRelaySecret,
        TICKET_AB_WIN.amount,
        TICKET_AB_WIN.winProb,
        TICKET_AB_WIN.r,
        TICKET_AB_WIN.s,
        TICKET_AB_WIN.v
      ),
      vmErrorMessage('channel must be open or pending to close')
    )
  })

  it('should fail to redeem ticket when channel in in different iteration', async function () {
    const { token, tickets, deployer } = await useFixtures()

    await tickets.initializeAccount(ACCOUNT_B.address, ACCOUNT_B.pubKeyFirstHalf, ACCOUNT_B.pubKeySecondHalf, SECRET_2)

    // open channel and then close it
    await token.transfer(tickets.address, '100')
    await tickets.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)
    await tickets.initiateChannelClosure(ACCOUNT_A.address, ACCOUNT_B.address)
    await tickets.finalizeChannelClosure(token.address, ACCOUNT_A.address, ACCOUNT_B.address)
    // refund and open channel
    await tickets.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await expectRevert(
      tickets.redeemTicket(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        TICKET_AB_WIN.secret,
        TICKET_AB_WIN.proofOfRelaySecret,
        TICKET_AB_WIN.amount,
        TICKET_AB_WIN.winProb,
        TICKET_AB_WIN.r,
        TICKET_AB_WIN.s,
        TICKET_AB_WIN.v
      ),
      vmErrorMessage('signer must match the counterparty')
    )
  })

  it('should fail to redeem ticket when ticket has been already redeemed', async function () {
    const { tickets, deployer } = await useFixtures()

    await tickets.initializeAccount(ACCOUNT_B.address, ACCOUNT_B.pubKeyFirstHalf, ACCOUNT_B.pubKeySecondHalf, SECRET_2)
    await tickets.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await tickets.redeemTicket(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.counterparty,
      TICKET_AB_WIN.secret,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.r,
      TICKET_AB_WIN.s,
      TICKET_AB_WIN.v
    )

    await expectRevert(
      tickets.redeemTicket(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        SECRET_0, // give the next secret so this ticket becomes redeemable
        TICKET_AB_WIN.proofOfRelaySecret,
        TICKET_AB_WIN.amount,
        TICKET_AB_WIN.winProb,
        TICKET_AB_WIN.r,
        TICKET_AB_WIN.s,
        TICKET_AB_WIN.v
      ),
      vmErrorMessage('ticket must not be used twice')
    )
  })

  it('should fail to redeem ticket when signer is not the issuer', async function () {
    const { tickets, deployer } = await useFixtures()

    await tickets.initializeAccount(ACCOUNT_B.address, ACCOUNT_B.pubKeyFirstHalf, ACCOUNT_B.pubKeySecondHalf, SECRET_2)
    await tickets.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await expectRevert(
      tickets.redeemTicket(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        TICKET_AB_WIN.secret,
        TICKET_AB_WIN.proofOfRelaySecret,
        TICKET_AB_WIN.amount,
        TICKET_AB_WIN.winProb,
        TICKET_BA_WIN.r, // signature from different ticket
        TICKET_BA_WIN.s, // signature from different ticket
        TICKET_AB_WIN.v
      ),
      vmErrorMessage('signer must match the counterparty')
    )
  })

  it("should fail to redeem ticket if it's a loss", async function () {
    const { tickets, deployer } = await useFixtures()

    await tickets.initializeAccount(ACCOUNT_B.address, ACCOUNT_B.pubKeyFirstHalf, ACCOUNT_B.pubKeySecondHalf, SECRET_2)
    await tickets.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await expectRevert(
      tickets.redeemTicket(
        TICKET_AB_LOSS.recipient,
        TICKET_AB_LOSS.counterparty,
        TICKET_AB_LOSS.secret,
        TICKET_AB_LOSS.proofOfRelaySecret,
        TICKET_AB_LOSS.amount,
        TICKET_AB_LOSS.winProb,
        TICKET_AB_LOSS.r,
        TICKET_AB_LOSS.s,
        TICKET_AB_LOSS.v
      ),
      vmErrorMessage('ticket must be a win')
    )
  })

  it('should pack ticket', async function () {
    const { tickets } = await useFixtures()

    const encoded = await tickets.getEncodedTicket(
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
    const { tickets } = await useFixtures()

    const hash = await tickets.getTicketHash(TICKET_AB_WIN.encoded)
    expect(hash).to.equal(TICKET_AB_WIN.hash)
  })

  it("should get ticket's luck", async function () {
    const { tickets } = await useFixtures()

    const luck = await tickets.getTicketLuck(
      TICKET_AB_WIN.hash,
      TICKET_AB_WIN.secret,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.winProb
    )
    expect(luck).to.equal(TICKET_AB_WIN.luck)
  })
})
