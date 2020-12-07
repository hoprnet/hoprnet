import { expectEvent, expectRevert, constants, singletons } from '@openzeppelin/test-helpers'
import { vmErrorMessage } from '../utils'
import { formatAccount, ERC777Mock } from './utils'
import { ACCOUNT_A, ACCOUNT_B, SECRET, SECRET_PRE_IMAGE, TICKET_AB_WIN } from './constants'

const ERC777 = artifacts.require('ERC777Mock')
const Tickets = artifacts.require('TicketsMock')

describe.only('Tickets', function () {
  let deployer: string

  before(async function () {
    const accounts = await web3.eth.getAccounts()
    deployer = accounts[0]

    // deploy ERC1820Registry required by ERC777 token
    await singletons.ERC1820Registry(deployer)
  })

  it('should redeem ticket', async function () {
    const token = await ERC777Mock(ERC777, deployer, '100')
    const tickets = await Tickets.new('0')

    await tickets.initializeAccount(ACCOUNT_A.address, ACCOUNT_A.pubKeyFirstHalf, ACCOUNT_A.pubKeySecondHalf, SECRET)

    // transfer tokens to contract
    await token.transfer(tickets.address, '100')
    await tickets.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await tickets.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await tickets.redeemTicket(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.counterparty,
      SECRET_PRE_IMAGE,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.r,
      TICKET_AB_WIN.s,
      TICKET_AB_WIN.v + 27
    )

    const ticket = await tickets.tickets(TICKET_AB_WIN.hash)
    expect(ticket).to.be.true
  })
})
