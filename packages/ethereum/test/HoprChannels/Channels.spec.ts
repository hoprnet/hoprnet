import { deployments, ethers } from 'hardhat'
import { expect } from 'chai'
import { durations } from '@hoprnet/hopr-utils'
import { ACCOUNT_A, ACCOUNT_B, ACCOUNT_AB_CHANNEL_ID, SECRET_2, generateTickets, SECRET_0 } from './constants'
import { ERC777Mock__factory, ChannelsMock__factory } from '../../types'
import deployERC1820Registry from '../../deploy/01_ERC1820Registry'
import { redeemArgs, validateChannel } from './utils'

const abiEncoder = ethers.utils.Interface.getAbiCoder()

const useFixtures = deployments.createFixture(async (hre, { secsClosure }: { secsClosure?: string } = {}) => {
  const [deployer] = await ethers.getSigners()

  // deploy ERC1820Registry required by ERC777 token
  await deployERC1820Registry(hre, deployer)

  // deploy ERC777Mock
  const token = await new ERC777Mock__factory(deployer).deploy(deployer.address, '100', 'Token', 'TKN', [])
  // deploy ChannelsMock
  let channels = await new ChannelsMock__factory(deployer).deploy(token.address, secsClosure ?? '0')
  channels = channels.connect(ACCOUNT_B.wallet)
  const fixtureTickets = await generateTickets()

  return {
    token,
    channels,
    deployer,
    fixtureTickets
  }
})

describe('funding HoprChannel catches failures', async function(){
  let channels
  before(async function () {
    // All of these tests revert, so we can rely on stateless single fixture.
    channels = await useFixtures()
  })

  it('should fail to fund channel A->A', async function () {
    await expect(channels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_A.address, '70', '30')).to.be.revertedWith(
      'accountA and accountB must not be the same'
    )
  })

  it('should fail to fund channel 0->A', async function() {
    await expect(
      channels.fundChannelMulti(ethers.constants.AddressZero, ACCOUNT_B.address, '70', '30')
    ).to.be.revertedWith('accountA must not be empty')
  })

  it('should fail to fund channel A->0', async function() {
    await expect(
      channels.fundChannelMulti(ACCOUNT_A.address, ethers.constants.AddressZero, '70', '30')
    ).to.be.revertedWith('accountB must not be empty')
  })

  it('should fail to fund a channel with 0 amount', async function() {
    await expect(
      channels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '0', '0')
    ).to.be.revertedWith(
      'amountA or amountB must be greater than 0'
    )
  })
})

describe('funding a HoprChannel success', function () {
  it('should fund and open channel A->B', async function () {
    const { channels } = await useFixtures()

    //TODO events
    await channels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, { partyABalance: '70', partyBBalance: '30' })
  })
})


describe('with a funded HoprChannel (A: 70, B: 30)', function() {
  let channels
  beforeEach(async function(){
    const fixtures = await useFixtures()
    channels = fixtures.channels
    await channels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
  })

  it('A can initialize channel closure', async function () {
    await expect(
      channels.connect(ACCOUNT_A.address).initiateChannelClosure(ACCOUNT_B.address)).to.emit(
      channels,
      'ChannelUpdate'
    )
    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, { partyABalance: '70', partyBBalance: '30', status: '2', closureByPartyA: true })
  })

  it('B can initialize channel closure', async function () {
    await expect(
      channels.connect(ACCOUNT_B.address).initiateChannelClosure(ACCOUNT_A.address)).to.emit(
      channels,
      'ChannelUpdate'
    )
    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, { partyABalance: '70', partyBBalance: '30', status: '2', closureByPartyA: false })
  })

  it('should fail to initialize channel closure A->A', async function () {
    await expect(channels.connect(ACCOUNT_A.address).initiateChannelClosure(ACCOUNT_B.address)).to.be.revertedWith(
      'initiator and counterparty must not be the same'
    )
  })

  it('should fail to initialize channel closure A->0', async function () {
    await expect(channels.connect(ACCOUNT_A.address).initiateChannelClosure(ethers.constants.AddressZero)).to.be.revertedWith('counterparty must not be empty')
  })
})


describe('With a pending_to_close HoprChannel (A:70, B:30)', function(){
  let channels
  beforeEach(async function(){
    const fixtures = await useFixtures()
    channels = fixtures.channels
    await channels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await channels.connect(ACCOUNT_A.address).initiateChannelClosure(ACCOUNT_B.address);
  })

  it('should fail to initialize channel closure when channel is not open', async function () {
    await expect(channels.connect(ACCOUNT_A.address).initiateChannelClosure(ACCOUNT_B.address)).to.be.revertedWith(
      'channel must be open'
    )
  })

  it('should finalize channel closure', async function () {
    const { token, channels, deployer } = await useFixtures()

    // transfer tokens to contract
    await token.send(
      channels.address,
      '100',
      abiEncoder.encode(
        ['address', 'address', 'uint256', 'uint256'],
        [ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
      ),
      {
        from: deployer.address
      }
    )
    await channels.connect(ACCOUNT_A.address).initiateChannelClosure(ACCOUNT_B.address)

    await expect(channels.connect(ACCOUNT_A.address).finalizeChannelClosure(ACCOUNT_B.address)).to.emit(
      channels,
      'ChannelUpdate'
    )

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.partyABalance.toString()).to.equal('0')
    expect(channel.partyBBalance.toString()).to.equal('0')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('0')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('70')
    const accountBBalance = await token.balanceOf(ACCOUNT_B.address)
    expect(accountBBalance.toString()).to.equal('30')
  })

  it('should finalize channel closure immediately', async function () {
    const { token, channels, deployer } = await useFixtures({ secsClosure: durations.minutes(5).toString() })

    // transfer tokens to contract
    await token.send(
      channels.address,
      '100',
      abiEncoder.encode(
        ['address', 'address', 'uint256', 'uint256'],
        [ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
      ),
      {
        from: deployer.address
      }
    )
    await channels.connect(ACCOUNT_A.address).initiateChannelClosure(ACCOUNT_B.address)
    await expect(channels.connect(ACCOUNT_B.address).finalizeChannelClosure(ACCOUNT_A.address)).to.emit(
      channels,
      'ChannelUpdate'
    )

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.partyABalance.toString()).to.equal('0')
    expect(channel.partyBBalance.toString()).to.equal('0')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('0')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('70')
    const accountBBalance = await token.balanceOf(ACCOUNT_B.address)
    expect(accountBBalance.toString()).to.equal('30')
  })

  it('should fail to finalize channel closure when is not pending', async function () {
    const { token, channels, deployer } = await useFixtures()

    // transfer tokens to contract
    await token.send(
      channels.address,
      '100',
      abiEncoder.encode(
        ['address', 'address', 'uint256', 'uint256'],
        [ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
      ),
      {
        from: deployer.address
      }
    )
    await expect(channels.connect(ACCOUNT_A.address).finalizeChannelClosure(ACCOUNT_B.address)).to.be.revertedWith(
      'channel must be pending to close'
    )
  })

  it('should fail to finalize channel closure', async function () {
    const { token, channels, deployer } = await useFixtures({ secsClosure: durations.minutes(5).toString() })

    // transfer tokens to contract
    await token.send(
      channels.address,
      '100',
      abiEncoder.encode(
        ['address', 'address', 'uint256', 'uint256'],
        [ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
      ),
      {
        from: deployer.address
      }
    )
    await channels.connect(ACCOUNT_A.address).initiateChannelClosure(ACCOUNT_B.address)

    await expect(channels.connect(ACCOUNT_A.address).finalizeChannelClosure(ACCOUNT_A.address)).to.be.revertedWith(
      'initiator and counterparty must not be the same'
    )

    await expect(
      channels.connect(ACCOUNT_A.address).finalizeChannelClosure(ethers.constants.AddressZero)
    ).to.be.revertedWith('counterparty must not be empty')

    await expect(channels.connect(ACCOUNT_A.address).finalizeChannelClosure(ACCOUNT_B.address)).to.be.revertedWith(
      'closureTime must be before now'
    )
  })
})

describe('test internals with mock', function() {
  let channels
  beforeEach(async function(){
    channels = (await useFixtures()).channels
  })

  it('should get channel data', async function () {
    const channelData = await channels.getChannel(ACCOUNT_A.address, ACCOUNT_B.address)
    expect(channelData[0]).to.be.equal(ACCOUNT_A.address)
    expect(channelData[1]).to.be.equal(ACCOUNT_B.address)
    expect(channelData[2]).to.be.equal(ACCOUNT_AB_CHANNEL_ID)
  })

  it('should get channel id', async function () {
    expect(await channels.getChannelId(ACCOUNT_A.address, ACCOUNT_B.address)).to.be.equal(ACCOUNT_AB_CHANNEL_ID)
  })

  it('should be partyA', async function () {
    expect(await channels.isPartyA(ACCOUNT_A.address, ACCOUNT_B.address)).to.be.true
  })

  it('should not be partyA', async function () {
    expect(await channels.isPartyA(ACCOUNT_B.address, ACCOUNT_A.address)).to.be.false
  })

  it('should get partyA and partyB', async function () {
    const parties = await channels.getParties(ACCOUNT_A.address, ACCOUNT_B.address)
    expect(parties[0]).to.be.equal(ACCOUNT_A.address)
    expect(parties[1]).to.be.equal(ACCOUNT_B.address)
  })

  it('should pack ticket', async function () {
    const { channels, fixtureTickets } = await useFixtures()
    const TICKET_AB_WIN = fixtureTickets.TICKET_AB_WIN

    const encoded = await channels.getEncodedTicketInternal(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.channelEpoch,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb
    )
    expect(encoded).to.equal(TICKET_AB_WIN.encoded)
  })

  it("should get ticket's luck", async function () {
    const { channels, fixtureTickets } = await useFixtures()
    const TICKET_AB_WIN = fixtureTickets.TICKET_AB_WIN

    const luck = await channels.getTicketLuckInternal(
      TICKET_AB_WIN.hash,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.winProb
    )
    expect(luck).to.equal(TICKET_AB_WIN.luck)
  })
})

describe('Tickets', function () {
  it('should redeem ticket', async function () {
    const { channels, fixtureTickets } = await useFixtures()
    await channels.connect(ACCOUNT_B.wallet).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await channels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

    // TODO: add event check
    await channels.redeemTicketInternal(...redeemArgs(fixtureTickets.TICKET_AB_WIN))
    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, {partyABalance: '60', partyBBalance: '40', closureTime: '0', status: '1', closureByPartyA: false})
    expect(channel.partyBCommitment).to.equal(fixtureTickets.TICKET_AB_WIN.nextCommitment)
  })

  it('should fail to redeem ticket when channel in closed', async function () {
    const { channels } = await useFixtures()
    await channels.connect(ACCOUNT_B.wallet).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await channels.initiateClose()
    await channels.finalizeClose()

    await expect(
      // @ts-ignore
      channels.redeemTicket(...redeemArgs(TICKET_AB_WIN))
    ).to.be.revertedWith('channel must be open or pending to close')
  })

  it('should fail to redeem ticket when channel in in different channelEpoch', async function () {
    const { channels, fixtureTickets, deployer } = await useFixtures()
    const TICKET_AB_WIN = fixtureTickets.TICKET_AB_WIN
    await channels.connect(ACCOUNT_B.wallet).bumpChannel(ACCOUNT_A.address, SECRET_2)

    // transfer tokens to contract
    await channels.connect(deployer).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    // open channel and then close it
    await channels.initiateChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address)
    await channels.finalizeChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address)
    // refund and open channel
    await channels.connect(deployer).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

    await expect(
      channels.redeemTicketInternal(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        TICKET_AB_WIN.nextCommitment,
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
    const { channels, fixtureTickets } = await useFixtures()
    const TICKET_AB_WIN = fixtureTickets.TICKET_AB_WIN

    await channels.connect(ACCOUNT_B.wallet).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await channels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

    await channels.redeemTicketInternal(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.counterparty,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    )

    await expect(
      channels.redeemTicketInternal(
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
    ).to.be.revertedWith('ticket epoch must match')

    await expect(
      channels.redeemTicketInternal(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        SECRET_0, // give the next secret so this ticket becomes redeemable
        parseInt(TICKET_AB_WIN.ticketEpoch) + 1 + '',
        TICKET_AB_WIN.ticketIndex,
        TICKET_AB_WIN.proofOfRelaySecret,
        TICKET_AB_WIN.amount,
        TICKET_AB_WIN.winProb,
        TICKET_AB_WIN.signature
      )
    ).to.be.revertedWith('redemptions must be in order')
  })

  it('should fail to redeem ticket when signer is not the issuer', async function () {
    const { channels, fixtureTickets } = await useFixtures()
    const TICKET_AB_WIN = fixtureTickets.TICKET_AB_WIN
    const TICKET_BA_WIN = fixtureTickets.TICKET_BA_WIN

    await channels.connect(ACCOUNT_B.wallet).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await channels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

    await expect(
      channels.redeemTicketInternal(
        TICKET_AB_WIN.recipient,
        TICKET_AB_WIN.counterparty,
        TICKET_AB_WIN.nextCommitment,
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
    const { channels, fixtureTickets } = await useFixtures()
    const TICKET_AB_LOSS = fixtureTickets.TICKET_AB_LOSS

    await channels.connect(ACCOUNT_B.wallet).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await channels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

    await expect(
      channels.redeemTicket(
        TICKET_AB_LOSS.counterparty,
        TICKET_AB_LOSS.nextCommitment,
        TICKET_AB_LOSS.ticketEpoch,
        TICKET_AB_LOSS.ticketIndex,
        TICKET_AB_LOSS.proofOfRelaySecret,
        TICKET_AB_LOSS.amount,
        TICKET_AB_LOSS.winProb,
        TICKET_AB_LOSS.signature
      )
    ).to.be.revertedWith('ticket must be a win')
  })
})
