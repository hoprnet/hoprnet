import { deployments, ethers } from 'hardhat'
import { expect } from 'chai'
import { durations } from '@hoprnet/hopr-utils'
import { ACCOUNT_A, ACCOUNT_B, ACCOUNT_AB_CHANNEL_ID, SECRET_2, generateTickets, SECRET_0 } from './constants'
import { HoprToken__factory, ChannelsMock__factory } from '../../types'
import { redeemArgs, validateChannel } from './utils'
import { PromiseValue } from '@hoprnet/hopr-utils'
import { createTicket } from './utils'
import { increaseTime } from '../utils'
import {
  PROOF_OF_RELAY_SECRET_0,
  PROOF_OF_RELAY_SECRET_1,
  WIN_PROB_100,
  SECRET_1,
} from './constants'

const abiEncoder = ethers.utils.Interface.getAbiCoder()

const useFixtures = deployments.createFixture(async () => {
  const [deployer] = await ethers.getSigners()
  const accountA = await ethers.getSigner(ACCOUNT_A.address)
  const accountB = await ethers.getSigner(ACCOUNT_B.address)

  // run migrations
  const contracts = await deployments.fixture()
  const token = HoprToken__factory.connect(contracts['HoprToken'].address, ethers.provider)
  const channels = ChannelsMock__factory.connect(contracts['HoprChannels'].address, ethers.provider)

  // create deployer the minter
  const minterRole = await token.MINTER_ROLE()
  await token.connect(deployer).grantRole(minterRole, deployer.address)

  const fixtureTickets = await generateTickets()

  const fund = async (addr, amount) => await token.connect(deployer).mint(addr, amount + '', ethers.constants.HashZero, ethers.constants.HashZero)

  return {
    token,
    channels,
    deployer,
    fixtureTickets,
    accountA,
    accountB,
    fund
  }
})

describe('funding HoprChannel catches failures', async function(){
  // TODO fund single, fund by send
  let fixtures, channels, accountA
  before(async function () {
    // All of these tests revert, so we can rely on stateless single fixture.
    fixtures = await useFixtures()
    channels = fixtures.channels
    accountA = fixtures.accountA 
    await fixtures.fund(accountA.address, 100)
  })

  it('should fail to fund channel A->A', async function () {
    await expect(channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_A.address, '70', '30'))
      .to.be.revertedWith('accountA and accountB must not be the same')
  })

  it('should fail to fund channel 0->A', async function() {
    await expect(
      channels.connect(accountA).fundChannelMulti(ethers.constants.AddressZero, ACCOUNT_B.address, '70', '30')
    ).to.be.revertedWith('accountA must not be empty')
  })

  it('should fail to fund channel A->0', async function() {
    await expect(
      channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ethers.constants.AddressZero, '70', '30')
    ).to.be.revertedWith('accountB must not be empty')
  })

  it('should fail to fund a channel with 0 amount', async function() {
    await expect(
      channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '0', '0')
    ).to.be.revertedWith('amountA or amountB must be greater than 0')
  })
})

describe('funding a HoprChannel success', function () {
  // TODO test single fund, events
  it('should multi fund and open channel A->B', async function () {
    const { channels } = await useFixtures()
    await expect(
      channels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    ).to.emit(channels, 'ChannelUpdate')
    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, { partyABalance: '70', partyBBalance: '30' })
  })

  it('should fund A->B using send', async function () {
    const { token, accountB, channels } = await useFixtures()
    await expect(
      token
        .connect(accountB)
        .send(
          channels.address,
          '30',
          abiEncoder.encode(['address', 'address'], [ACCOUNT_B.address, ACCOUNT_A.address])
        )
    ).to.emit(channels, 'ChannelUpdate')
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), {
      partyABalance: '70',
      partyBBalance: '30',
      closureTime: '0',
      status: '1',
      closureByPartyA: false
    })
  })
})


describe('with a funded HoprChannel (A: 70, B: 30), secrets initialized', function() {
  let channels
  let fixtures
  beforeEach(async function(){
    fixtures = await useFixtures()
    channels = fixtures.channels
    await channels.connect(fixtures.accountA).bumpChannel(ACCOUNT_B.address, SECRET_2)
    await channels.connect(fixtures.accountB).bumpChannel(ACCOUNT_A.address, SECRET_2) // TODO secret per account
    await channels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
  })

  it('should redeem ticket for account A', async function () {
    await channels
      .connect(fixtures.accountA)
      //@ts-ignore
      .redeemTicket(...redeemArgs(f.TICKET_BA_WIN))

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, {
      partyABalance: '80',
      partyBBalance: '20',
      closureTime: '0',
      status: '1',
      closureByPartyA: false
    })
    expect(channel.partyACommitment).to.equal(SECRET_1)
  })

  it('should reedem ticket for account B', async function () {
    await channels
      .connect(fixtures.accountB)
      //@ts-ignore
      .redeemTicket(...redeemArgs(f.TICKET_AB_WIN))

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, {
      partyABalance: '60',
      partyBBalance: '40',
      closureTime: '0',
      status: '1',
      closureByPartyA: false
    })
    expect(channel.partyBCommitment).to.equal(SECRET_1)
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

describe('with a closed channel', function(){
  // TODO Close channel
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
})

describe('with a reopened channel', function(){
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



// -------------------------------------------------------

describe('HoprChannels', function () {
  it('should fund one direction', async function () {
    const { token, channels, accountA } = await useFixtures()

    await token.connect(accountA).approve(channels.address, '70')
    await channels.connect(accountA).fundChannel(ACCOUNT_A.address, ACCOUNT_B.address, '70')

    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), {
      partyABalance: '70',
      partyBBalance: '0',
      closureTime: '0',
      status: '1',
      closureByPartyA: false
    })
    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('30')
  })

  it('should fund both directions', async function () {
    const { token, channels, accountA } = await useFixtures()

    await token.connect(accountA).approve(channels.address, '100')
    await channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, {
      partyABalance: '70',
      partyBBalance: '30',
      closureTime: '0',
      status: '1',
      closureByPartyA: false
    })
    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('0')
  })

  it('should fund and open channel', async function () {
    const { token, channels, accountA } = await useFixtures()

    await token.connect(accountA).approve(channels.address, '70')
    await channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '0')

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, {
      partyABalance: '70',
      partyBBalance: '0',
      closureTime: '0',
      status: '1',
      closureByPartyA: false
    })
    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('30')
  })

  it('should fund and open channel by accountB', async function () {
    const { token, channels, accountB } = await useFixtures()

    await token.connect(accountB).approve(channels.address, '70')
    await channels.connect(accountB).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '0')

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, {
      partyABalance: '70',
      partyBBalance: '0',
      closureTime: '0',
      status: '1',
      closureByPartyA: false
    })
    const accountBBalance = await token.balanceOf(ACCOUNT_B.address)
    expect(accountBBalance.toString()).to.equal('30')
  })

  it('should fund using send', async function () {
    const { token, channels, accountA } = await useFixtures()

    await token
      .connect(accountA)
      .send(channels.address, '70', abiEncoder.encode(['address', 'address'], [ACCOUNT_A.address, ACCOUNT_B.address]))

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, {
      partyABalance: '70',
      partyBBalance: '0',
      closureTime: '0',
      status: '1',
      closureByPartyA: false
    })
    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('30')
  })

  it('should fund and open using send', async function () {
    const { token, channels, accountA } = await useFixtures()

    await token
      .connect(accountA)
      .send(channels.address, '70', abiEncoder.encode(['address', 'address'], [ACCOUNT_A.address, ACCOUNT_B.address]))

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, {
      partyABalance: '70',
      partyBBalance: '0',
      closureTime: '0',
      status: '1',
      closureByPartyA: false
    })
    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('30')
  })

  it('should fund both parties using send', async function () {
    const { token, channels, accountA } = await useFixtures()

    await token
      .connect(accountA)
      .send(
        channels.address,
        '100',
        abiEncoder.encode(
          ['address', 'address', 'uint256', 'uint256'],
          [ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
        )
      )

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, {
      partyABalance: '70',
      partyBBalance: '30',
      closureTime: '0',
      status: '1',
      closureByPartyA: false
    })
    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('0')
  })
})

describe('HoprChannels lifecycle', function () {
  let f: PromiseValue<ReturnType<typeof useFixtures>>

  before(async function () {
    f = await useFixtures()
  })

  context('on a fresh channel', function () {

    it('should initialize channel closure', async function () {
      await expect(f.channels.connect(f.accountB).initiateChannelClosure(ACCOUNT_A.address)).to.emit(
        f.channels,
        'ChannelUpdate'
      )
      // TODO: implement
      // .withArgs(ACCOUNT_B.address, ACCOUNT_A.address)

      const channel = await f.channels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.partyBBalance.toString()).to.equal('30')
      expect(channel.closureTime.toString()).to.not.be.equal('0')
      expect(channel.status.toString()).to.equal('2')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should reedeem ticket for accountA', async function () {
      await f.channels
        .connect(f.accountA)
        //@ts-ignore
        .redeemTicket(...redeemArgs(f.TICKET_BA_WIN))

      const channel = await f.channels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.partyABalance.toString()).to.equal('80')
      expect(channel.partyBBalance.toString()).to.equal('20')
      expect(channel.closureTime.toString()).to.not.be.equal('0')
      expect(channel.status.toString()).to.equal('2')
      expect(channel.closureByPartyA).to.be.false
      expect(channel.partyACommitment).to.equal(SECRET_0)
    })

    it('should close channel', async function () {
      await increaseTime(ethers.provider, durations.days(3))

      await expect(f.channels.connect(f.accountA).finalizeChannelClosure(ACCOUNT_B.address)).to.emit(
        f.channels,
        'ChannelUpdate'
      )

      const channel = await f.channels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.partyABalance.toString()).to.equal('0')
      expect(channel.partyBBalance.toString()).to.equal('0')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('0')
      expect(channel.closureByPartyA).to.be.false

      const accountABalance = await f.token.balanceOf(ACCOUNT_A.address)
      expect(accountABalance.toString()).to.equal('110')
      const accountBBalance = await f.token.balanceOf(ACCOUNT_B.address)
      expect(accountBBalance.toString()).to.equal('90')
    })
  })

  context('on a recycled channel', function () {
    let TICKET_AB_WIN_RECYCLED: PromiseValue<ReturnType<typeof createTicket>>
    let TICKET_BA_WIN_RECYCLED: PromiseValue<ReturnType<typeof createTicket>>
    let TICKET_BA_WIN_RECYCLED_2: PromiseValue<ReturnType<typeof createTicket>>

    before(async function () {
      // the key difference between these tickets
      // and tickets from constants.ts is that
      // this tickets are for channel channelEpoch 2
      // and account counter 2
      TICKET_AB_WIN_RECYCLED = await createTicket(
        {
          recipient: ACCOUNT_B.address,
          proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
          ticketIndex: '0',
          ticketEpoch: '0',
          amount: '10',
          winProb: WIN_PROB_100,
          channelEpoch: '2'
        },
        ACCOUNT_A,
        SECRET_1
      )

      TICKET_BA_WIN_RECYCLED = await createTicket(
        {
          recipient: ACCOUNT_A.address,
          proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
          ticketIndex: '0',
          ticketEpoch: '0',
          amount: '10',
          winProb: WIN_PROB_100,
          channelEpoch: '2'
        },
        ACCOUNT_B,
        SECRET_1
      )

      TICKET_BA_WIN_RECYCLED_2 = await createTicket(
        {
          recipient: ACCOUNT_A.address,
          proofOfRelaySecret: PROOF_OF_RELAY_SECRET_1,
          ticketIndex: '0',
          ticketEpoch: '0',
          amount: '10',
          winProb: WIN_PROB_100,
          channelEpoch: '2'
        },
        ACCOUNT_B,
        SECRET_0
      )
    })

    it('should fund both parties and open channel', async function () {
      await f.token.connect(f.accountA).approve(f.channels.address, '110')

      f.channels.connect(f.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '40')

      const channel = await f.channels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.partyBBalance.toString()).to.equal('40')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('1')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should reedem ticket for accountA', async function () {
      await f.channels
        .connect(f.accountA)
        .redeemTicket(
          TICKET_BA_WIN_RECYCLED.counterparty,
          TICKET_BA_WIN_RECYCLED.nextCommitment,
          f.fixtureTickets.TICKET_BA_WIN.ticketEpoch,
          f.fixtureTickets.TICKET_BA_WIN.ticketIndex,
          TICKET_BA_WIN_RECYCLED.proofOfRelaySecret,
          TICKET_BA_WIN_RECYCLED.amount,
          TICKET_BA_WIN_RECYCLED.winProb,
          TICKET_BA_WIN_RECYCLED.signature
        )

      const ticket = await f.channels.tickets(TICKET_BA_WIN_RECYCLED.hash)
      expect(ticket).to.be.true

      const channel = await f.channels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.partyABalance.toString()).to.equal('80')
      expect(channel.partyBBalance.toString()).to.equal('30')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('1')
      expect(channel.closureByPartyA).to.be.false
      expect(channel.partyBCommitment).to.equal(SECRET_1)
    })

    it('should reedem ticket for accountB', async function () {
      await f.channels
        .connect(f.accountB)
        .redeemTicket(
          TICKET_AB_WIN_RECYCLED.counterparty,
          TICKET_AB_WIN_RECYCLED.nextCommitment,
          f.fixtureTickets.TICKET_BA_WIN.ticketEpoch,
          f.fixtureTickets.TICKET_BA_WIN.ticketIndex,
          TICKET_AB_WIN_RECYCLED.proofOfRelaySecret,
          TICKET_AB_WIN_RECYCLED.amount,
          TICKET_AB_WIN_RECYCLED.winProb,
          TICKET_AB_WIN_RECYCLED.signature
        )

      const ticket = await f.channels.tickets(TICKET_AB_WIN_RECYCLED.hash)
      expect(ticket).to.be.true

      const channel = await f.channels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.partyBBalance.toString()).to.equal('40')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('1')
      expect(channel.closureByPartyA).to.be.false
      expect(channel.partyACommitment).to.equal(SECRET_1)
    })

    it('should initialize channel closure', async function () {
      f.channels.connect(f.accountB).initiateChannelClosure(ACCOUNT_A.address)
      const channel = await f.channels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.partyBBalance.toString()).to.equal('40')
      expect(channel.closureTime.toString()).to.not.be.equal('0')
      expect(channel.status.toString()).to.equal('2')
      expect(channel.channelEpoch.toString()).to.equal('1')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should reedem ticket for accountA', async function () {
      await f.channels
        .connect(f.accountA)
        .redeemTicket(
          TICKET_BA_WIN_RECYCLED_2.counterparty,
          TICKET_BA_WIN_RECYCLED_2.nextCommitment,
          f.fixtureTickets.TICKET_BA_WIN.ticketEpoch,
          f.fixtureTickets.TICKET_BA_WIN.ticketIndex,
          TICKET_BA_WIN_RECYCLED_2.proofOfRelaySecret,
          TICKET_BA_WIN_RECYCLED_2.amount,
          TICKET_BA_WIN_RECYCLED_2.winProb,
          TICKET_BA_WIN_RECYCLED_2.signature
        )

      const channel = await f.channels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.partyABalance.toString()).to.equal('80')
      expect(channel.partyBBalance.toString()).to.equal('30')
      expect(channel.closureTime.toString()).to.not.be.equal('0')
      expect(channel.status.toString()).to.equal('2')
      expect(channel.channelEpoch.toString()).to.equal('1')
      expect(channel.closureByPartyA).to.be.false
      expect(channel.partyACommitment).to.equal(SECRET_0)
    })

    it('should close channel', async function () {
      await increaseTime(ethers.provider, durations.days(3))

      f.channels.connect(f.accountA).finalizeChannelClosure(ACCOUNT_B.address)
      const channel = await f.channels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.partyABalance.toString()).to.equal('0')
      expect(channel.partyBBalance.toString()).to.equal('0')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('0')
      expect(channel.closureByPartyA).to.be.false

      const accountABalance = await f.token.balanceOf(ACCOUNT_A.address)
      expect(accountABalance.toString()).to.equal('80')
      const accountBBalance = await f.token.balanceOf(ACCOUNT_B.address)
      expect(accountBBalance.toString()).to.equal('120')
    })
  })
})
