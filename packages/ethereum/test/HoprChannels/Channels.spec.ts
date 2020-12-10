import { deployments } from 'hardhat'
import { expectEvent, expectRevert, constants, singletons, time } from '@openzeppelin/test-helpers'
import { formatChannel } from './utils'
import { ACCOUNT_A, ACCOUNT_B, ACCOUNT_AB_CHANNEL_ID } from './constants'

const ERC777 = artifacts.require('ERC777Mock')
const Channels = artifacts.require('ChannelsMock')

const useFixtures = deployments.createFixture(async (_deployments, { secsClosure }: { secsClosure?: string } = {}) => {
  const [deployer] = await web3.eth.getAccounts()

  // deploy ERC1820Registry required by ERC777 token
  await singletons.ERC1820Registry(deployer)

  // deploy ERC777Mock
  const token = await ERC777.new(deployer, '100', 'Token', 'TKN', [])
  // deploy ChannelsMock
  const channels = await Channels.new(secsClosure ?? '0')

  return {
    token,
    channels,
    deployer
  }
})

describe('Channels', function () {
  it('should fund channel', async function () {
    const { channels } = await useFixtures()

    const response = await channels.fundChannel(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

    expectEvent(response, 'ChannelFunded', {
      accountA: ACCOUNT_A.address,
      accountB: ACCOUNT_B.address,
      funder: ACCOUNT_A.address,
      deposit: '100',
      partyABalance: '70'
    })

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('100')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('0')
    expect(channel.closureByPartyA).to.be.false
  })

  it('should fail to fund channel', async function () {
    const { channels } = await useFixtures()

    await expectRevert(
      channels.fundChannel(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_A.address, '70', '30'),
      'accountA and accountB must not be the same'
    )

    await expectRevert(
      channels.fundChannel(ACCOUNT_A.address, constants.ZERO_ADDRESS, ACCOUNT_B.address, '70', '30'),
      'accountA must not be empty'
    )

    await expectRevert(
      channels.fundChannel(ACCOUNT_A.address, ACCOUNT_A.address, constants.ZERO_ADDRESS, '70', '30'),
      'accountB must not be empty'
    )

    await expectRevert(
      channels.fundChannel(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_B.address, '0', '0'),
      'untA or amountB must be greater than 0'
    )
  })

  it('should open channel', async function () {
    const { channels } = await useFixtures()

    await channels.fundChannel(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_B.address, '100', '0')
    const response = await channels.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    expectEvent(response, 'ChannelOpened', {
      opener: ACCOUNT_A.address,
      counterparty: ACCOUNT_B.address
    })

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('100')
    expect(channel.partyABalance.toString()).to.equal('100')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('1')
    expect(channel.closureByPartyA).to.be.false
  })

  it('should fail to open channel', async function () {
    const { channels } = await useFixtures()

    await expectRevert(
      channels.openChannel(ACCOUNT_A.address, ACCOUNT_A.address),
      'opener and counterparty must not be the same'
    )

    await expectRevert(channels.openChannel(constants.ZERO_ADDRESS, ACCOUNT_B.address), 'opener must not be empty')

    await expectRevert(
      channels.openChannel(ACCOUNT_A.address, constants.ZERO_ADDRESS),
      'counterparty must not be empty'
    )

    await expectRevert(channels.openChannel(ACCOUNT_A.address, ACCOUNT_B.address), 'channel must be funded')
  })

  it('should fail to open channel when channel is already open', async function () {
    const { channels } = await useFixtures()

    await channels.fundChannel(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_B.address, '100', '0')
    await channels.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await expectRevert(
      channels.openChannel(ACCOUNT_A.address, ACCOUNT_B.address),
      'channel must be closed in order to open'
    )
  })

  it('should initialize channel closure', async function () {
    const { channels } = await useFixtures()

    await channels.fundChannel(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_B.address, '100', '0')
    await channels.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    const response = await channels.initiateChannelClosure(ACCOUNT_A.address, ACCOUNT_B.address)
    await expectEvent(response, 'ChannelPendingToClose', {
      initiator: ACCOUNT_A.address,
      counterparty: ACCOUNT_B.address
    })

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('100')
    expect(channel.partyABalance.toString()).to.equal('100')
    expect(channel.closureTime.toString()).to.not.equals('0')
    expect(channel.status.toString()).to.equal('2')
    expect(channel.closureByPartyA).to.be.true
  })

  it('should fail to initialize channel closure when channel is not open', async function () {
    const { channels } = await useFixtures()

    await expectRevert(channels.initiateChannelClosure(ACCOUNT_A.address, ACCOUNT_B.address), 'channel must be open')
  })

  it('should fail to initialize channel closure', async function () {
    const { channels } = await useFixtures()

    await channels.fundChannel(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_B.address, '100', '0')
    await channels.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await expectRevert(
      channels.initiateChannelClosure(ACCOUNT_A.address, ACCOUNT_A.address),
      'initiator and counterparty must not be the same'
    )

    await expectRevert(
      channels.initiateChannelClosure(constants.ZERO_ADDRESS, ACCOUNT_B.address),
      'initiator must not be empty'
    )

    await expectRevert(
      channels.initiateChannelClosure(ACCOUNT_A.address, constants.ZERO_ADDRESS),
      'counterparty must not be empty'
    )
  })

  it('should finalize channel closure', async function () {
    const { token, channels, deployer } = await useFixtures()

    // transfer tokens to contract
    await token.transfer(channels.address, '100')
    await channels.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await channels.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)
    await channels.initiateChannelClosure(ACCOUNT_A.address, ACCOUNT_B.address)

    const response = await channels.finalizeChannelClosure(token.address, ACCOUNT_A.address, ACCOUNT_B.address)
    await expectEvent(response, 'ChannelClosed', {
      initiator: ACCOUNT_A.address,
      counterparty: ACCOUNT_B.address,
      partyAAmount: '70',
      partyBAmount: '30'
    })

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('0')
    expect(channel.partyABalance.toString()).to.equal('0')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('10')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('70')
    const accountBBalance = await token.balanceOf(ACCOUNT_B.address)
    expect(accountBBalance.toString()).to.equal('30')
  })

  it('should finalize channel closure immediately', async function () {
    const { token, channels, deployer } = await useFixtures({ secsClosure: time.duration.minutes(5) })

    // transfer tokens to contract
    await token.transfer(channels.address, '100')
    await channels.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await channels.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)
    await channels.initiateChannelClosure(ACCOUNT_A.address, ACCOUNT_B.address)

    const response = await channels.finalizeChannelClosure(token.address, ACCOUNT_B.address, ACCOUNT_A.address)
    await expectEvent(response, 'ChannelClosed', {
      initiator: ACCOUNT_B.address,
      counterparty: ACCOUNT_A.address,
      partyAAmount: '70',
      partyBAmount: '30'
    })

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('0')
    expect(channel.partyABalance.toString()).to.equal('0')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('10')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('70')
    const accountBBalance = await token.balanceOf(ACCOUNT_B.address)
    expect(accountBBalance.toString()).to.equal('30')
  })

  it('should fail to finalize channel closure when is not pending', async function () {
    const { token, channels, deployer } = await useFixtures()

    // transfer tokens to contract
    await token.transfer(channels.address, '100')
    await channels.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await channels.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)

    await expectRevert(
      channels.finalizeChannelClosure(token.address, ACCOUNT_A.address, ACCOUNT_B.address),
      'channel must be pending to close'
    )
  })

  it('should fail to finalize channel closure', async function () {
    const { token, channels, deployer } = await useFixtures({ secsClosure: time.duration.minutes(5) })

    // transfer tokens to contract
    await token.transfer(channels.address, '100')
    await channels.fundChannel(deployer, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await channels.openChannel(ACCOUNT_A.address, ACCOUNT_B.address)
    await channels.initiateChannelClosure(ACCOUNT_A.address, ACCOUNT_B.address)

    await expectRevert(
      channels.finalizeChannelClosure(constants.ZERO_ADDRESS, ACCOUNT_A.address, ACCOUNT_B.address),
      'token must not be empty'
    )

    await expectRevert(
      channels.finalizeChannelClosure(token.address, ACCOUNT_A.address, ACCOUNT_A.address),
      'initiator and counterparty must not be the same'
    )

    await expectRevert(
      channels.finalizeChannelClosure(token.address, constants.ZERO_ADDRESS, ACCOUNT_B.address),
      'initiator must not be empty'
    )

    await expectRevert(
      channels.finalizeChannelClosure(token.address, ACCOUNT_A.address, constants.ZERO_ADDRESS),
      'counterparty must not be empty'
    )

    await expectRevert(
      channels.finalizeChannelClosure(token.address, ACCOUNT_A.address, ACCOUNT_B.address),
      'closureTime must be before now'
    )
  })

  it('should get channel data', async function () {
    const channels = await Channels.new('0')

    const channelData = await channels.getChannel.call(ACCOUNT_A.address, ACCOUNT_B.address)
    expect(channelData[0]).to.be.equal(ACCOUNT_A.address)
    expect(channelData[1]).to.be.equal(ACCOUNT_B.address)
    expect(channelData[2]).to.be.equal(ACCOUNT_AB_CHANNEL_ID)
  })

  it('should get channel id', async function () {
    const channels = await Channels.new('0')

    const channelId = await channels.getChannelId.call(ACCOUNT_A.address, ACCOUNT_B.address)
    expect(channelId).to.be.equal(ACCOUNT_AB_CHANNEL_ID)
  })

  it('should get channel status', async function () {
    const channels = await Channels.new('0')

    const status = await channels.getChannelStatus.call('11')
    expect(status.toString()).to.be.equal('1')
  })

  it('should get channel iteration', async function () {
    const channels = await Channels.new('0')

    const iteration = await channels.getChannelIteration.call('11')
    expect(iteration.toString()).to.be.equal('2')
  })

  it('should be partyA', async function () {
    const channels = await Channels.new('0')

    const isPartyA = await channels.isPartyA.call(ACCOUNT_A.address, ACCOUNT_B.address)
    expect(isPartyA).to.be.true
  })

  it('should get partyA and partyB', async function () {
    const channels = await Channels.new('0')

    const parties = await channels.getParties.call(ACCOUNT_A.address, ACCOUNT_B.address)
    expect(parties[0]).to.be.equal(ACCOUNT_A.address)
    expect(parties[1]).to.be.equal(ACCOUNT_B.address)
  })
})

// it.skip('should fund channel by hook', async function () {
//   const token = await ERC777Mock(ERC777, ACCOUNT_A.address, '100')
//   const channels = await Channels.new("0")

//   const response = await token.send(
//     channels.address,
//     '100',
//     web3.eth.abi.encodeParameters(['address', 'address'], [ACCOUNT_A.address, ACCOUNT_B.address])
//   )

//   expectEvent(response, 'ChannelFunded', {
//     accountA: ACCOUNT_A.address,
//     accountB: ACCOUNT_B.address,
//     funder: ACCOUNT_A.address,
//     deposit: '100',
//     partyABalance: '100'
//   })

//   const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
//   expect(channel.deposit.toString()).to.equal('100')
//   expect(channel.partyABalance.toString()).to.equal('100')
//   expect(channel.closureTime.toString()).to.equal('0')
//   expect(channel.status.toString()).to.equal('0')
//   expect(channel.closureByPartyA).to.be.false
// })
