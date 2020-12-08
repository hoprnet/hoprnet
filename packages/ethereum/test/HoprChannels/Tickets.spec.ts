import { expectRevert, singletons } from '@openzeppelin/test-helpers'
import { vmErrorMessage } from '../utils'
import {
  ACCOUNT_A,
  ACCOUNT_B,
  SECRET_2,
  SECRET_1,
  SECRET_0,
  TICKET_AB_WIN,
  TICKET_BA_WIN,
  TICKET_AB_LOSS
} from './constants'

const Tickets = artifacts.require('TicketsMock')

describe('Tickets', function () {
  let deployer: string

  before(async function () {
    const accounts = await web3.eth.getAccounts()
    deployer = accounts[0]

    // deploy ERC1820Registry required by ERC777 token
    await singletons.ERC1820Registry(deployer)
  })

  it('should redeem ticket', async function () {
    const tickets = await Tickets.new('0')

    await tickets.initializeAccount(ACCOUNT_B.address, ACCOUNT_B.pubKeyFirstHalf, ACCOUNT_B.pubKeySecondHalf, SECRET_2)
    await tickets.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await tickets.redeemTicket(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.counterparty,
      SECRET_1,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.r,
      TICKET_AB_WIN.s,
      TICKET_AB_WIN.v
    )

    const ticket = await tickets.tickets(TICKET_AB_WIN.hash)
    expect(ticket).to.be.true
  })

  it('should fail to redeem ticket when channel in closed', async function () {
    const tickets = await Tickets.new('0')

    await tickets.initializeAccount(ACCOUNT_B.address, ACCOUNT_B.pubKeyFirstHalf, ACCOUNT_B.pubKeySecondHalf, SECRET_2)

    await expectRevert(
      tickets.redeemTicket(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        SECRET_1,
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

  it('should fail to redeem ticket when ticket has been already redeemed', async function () {
    const tickets = await Tickets.new('0')

    await tickets.initializeAccount(ACCOUNT_B.address, ACCOUNT_B.pubKeyFirstHalf, ACCOUNT_B.pubKeySecondHalf, SECRET_2)
    await tickets.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await tickets.redeemTicket(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.counterparty,
      SECRET_1,
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
        SECRET_0,
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
    const tickets = await Tickets.new('0')

    await tickets.initializeAccount(ACCOUNT_B.address, ACCOUNT_B.pubKeyFirstHalf, ACCOUNT_B.pubKeySecondHalf, SECRET_2)
    await tickets.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await expectRevert(
      tickets.redeemTicket(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        SECRET_1,
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
    const tickets = await Tickets.new('0')

    await tickets.initializeAccount(ACCOUNT_B.address, ACCOUNT_B.pubKeyFirstHalf, ACCOUNT_B.pubKeySecondHalf, SECRET_2)
    await tickets.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await expectRevert(
      tickets.redeemTicket(
        TICKET_AB_LOSS.recipient,
        TICKET_AB_LOSS.counterparty,
        SECRET_1,
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
    const tickets = await Tickets.new('0')

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
    const tickets = await Tickets.new('0')

    const hash = await tickets.getTicketHash(TICKET_AB_WIN.encoded)
    expect(hash).to.equal(TICKET_AB_WIN.hash)
  })

  it("should get ticket's luck", async function () {
    const tickets = await Tickets.new('0')

    const luck = await tickets.getTicketLuck(
      TICKET_AB_WIN.hash,
      SECRET_1,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.winProb
    )
    expect(luck).to.equal(TICKET_AB_WIN.luck)
  })
})
