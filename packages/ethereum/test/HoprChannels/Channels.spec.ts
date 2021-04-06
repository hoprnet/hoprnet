import { deployments, ethers } from 'hardhat'
import { singletons, time } from '@openzeppelin/test-helpers'
import { expect } from 'chai'
import { ACCOUNT_A, ACCOUNT_B, ACCOUNT_AB_CHANNEL_ID } from './constants'
import { ERC777Mock__factory, ChannelsMock__factory } from '../../types'

const abiEncoder = ethers.utils.Interface.getAbiCoder()

const useFixtures = deployments.createFixture(async ({ ethers }, { secsClosure }: { secsClosure?: string } = {}) => {
  const [deployer] = await ethers.getSigners()

  // deploy ERC1820Registry required by ERC777 token
  await singletons.ERC1820Registry(deployer.address)

  // deploy ERC777Mock
  const token = await new ERC777Mock__factory(deployer).deploy(deployer.address, '100', 'Token', 'TKN', [])
  // deploy ChannelsMock
  const channels = await new ChannelsMock__factory(deployer).deploy(token.address, secsClosure ?? '0')

  return {
    token,
    channels,
    deployer
  }
})

describe('Channels', function () {
  it('should fund channel', async function () {
    const { channels } = await useFixtures()

    expect(channels.fundChannelInternal(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30'))
      .to.emit(channels, 'ChannelFunded')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, ACCOUNT_A.address, '100', '70')

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.deposit.toString()).to.equal('100')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('0')
    expect(channel.closureByPartyA).to.be.false
  })

  it('should fail to fund channel', async function () {
    const { channels } = await useFixtures()

    expect(
      channels.fundChannelInternal(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_A.address, '70', '30')
    ).to.be.revertedWith('accountA and accountB must not be the same')

    expect(
      channels.fundChannelInternal(ACCOUNT_A.address, ethers.constants.AddressZero, ACCOUNT_B.address, '70', '30')
    ).to.be.revertedWith('accountA must not be empty')

    expect(
      channels.fundChannelInternal(ACCOUNT_A.address, ACCOUNT_A.address, ethers.constants.AddressZero, '70', '30')
    ).to.be.revertedWith('accountB must not be empty')

    expect(
      channels.fundChannelInternal(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_B.address, '0', '0')
    ).to.be.revertedWith('untA or amountB must be greater than 0')
  })

  it('should open channel', async function () {
    const { channels } = await useFixtures()

    await channels.fundChannelInternal(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_B.address, '100', '0')
    expect(channels.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address))
      .to.emit(channels, 'ChannelOpened')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address)

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.deposit.toString()).to.equal('100')
    expect(channel.partyABalance.toString()).to.equal('100')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('1')
    expect(channel.closureByPartyA).to.be.false
  })

  it('should fail to open channel', async function () {
    const { channels } = await useFixtures()

    expect(channels.openChannelInternal(ACCOUNT_A.address, ACCOUNT_A.address)).to.be.revertedWith(
      'opener and counterparty must not be the same'
    )

    expect(channels.openChannelInternal(ethers.constants.AddressZero, ACCOUNT_B.address)).to.be.revertedWith(
      'opener must not be empty'
    )

    expect(channels.openChannelInternal(ACCOUNT_A.address, ethers.constants.AddressZero)).to.be.revertedWith(
      'counterparty must not be empty'
    )

    expect(channels.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)).to.be.revertedWith(
      'channel must be funded'
    )
  })

  it('should fail to open channel when channel is already open', async function () {
    const { channels } = await useFixtures()

    await channels.fundChannelInternal(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_B.address, '100', '0')
    await channels.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)

    expect(channels.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)).to.be.revertedWith(
      'channel must be closed in order to open'
    )
  })

  it('should initialize channel closure', async function () {
    const { channels } = await useFixtures()

    await channels.fundChannelInternal(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_B.address, '100', '0')
    await channels.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)

    expect(channels.initiateChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address))
      .to.emit(channels, 'ChannelPendingToClose')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address)

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.deposit.toString()).to.equal('100')
    expect(channel.partyABalance.toString()).to.equal('100')
    expect(channel.closureTime.toString()).to.not.equals('0')
    expect(channel.status.toString()).to.equal('2')
    expect(channel.closureByPartyA).to.be.true
  })

  it('should fail to initialize channel closure when channel is not open', async function () {
    const { channels } = await useFixtures()

    expect(channels.initiateChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address)).to.be.revertedWith(
      'channel must be open'
    )
  })

  it('should fail to initialize channel closure', async function () {
    const { channels } = await useFixtures()

    await channels.fundChannelInternal(ACCOUNT_A.address, ACCOUNT_A.address, ACCOUNT_B.address, '100', '0')
    await channels.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)

    expect(channels.initiateChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_A.address)).to.be.revertedWith(
      'initiator and counterparty must not be the same'
    )

    expect(channels.initiateChannelClosureInternal(ethers.constants.AddressZero, ACCOUNT_B.address)).to.be.revertedWith(
      'initiator must not be empty'
    )

    expect(channels.initiateChannelClosureInternal(ACCOUNT_A.address, ethers.constants.AddressZero)).to.be.revertedWith(
      'counterparty must not be empty'
    )
  })

  it('should finalize channel closure', async function () {
    const { token, channels, deployer } = await useFixtures()

    // transfer tokens to contract
    await token.send(
      channels.address,
      '100',
      abiEncoder.encode(
        ['bool', 'address', 'address', 'uint256', 'uint256'],
        [false, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
      ),
      {
        from: deployer.address
      }
    )
    await channels.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)
    await channels.initiateChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address)

    expect(channels.finalizeChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address))
      .to.emit(channels, 'ChannelClosed')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
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
    await token.send(
      channels.address,
      '100',
      abiEncoder.encode(
        ['bool', 'address', 'address', 'uint256', 'uint256'],
        [false, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
      ),
      {
        from: deployer.address
      }
    )
    await channels.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)
    await channels.initiateChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address)

    expect(channels.finalizeChannelClosureInternal(ACCOUNT_B.address, ACCOUNT_A.address))
      .to.emit(channels, 'ChannelClosed')
      .withArgs(ACCOUNT_B.address, ACCOUNT_A.address, '70', '30')

    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
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
    await token.send(
      channels.address,
      '100',
      abiEncoder.encode(
        ['bool', 'address', 'address', 'uint256', 'uint256'],
        [false, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
      ),
      {
        from: deployer.address
      }
    )
    await channels.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)

    expect(channels.finalizeChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address)).to.be.revertedWith(
      'channel must be pending to close'
    )
  })

  it('should fail to finalize channel closure', async function () {
    const { token, channels, deployer } = await useFixtures({ secsClosure: time.duration.minutes(5) })

    // transfer tokens to contract
    await token.send(
      channels.address,
      '100',
      abiEncoder.encode(
        ['bool', 'address', 'address', 'uint256', 'uint256'],
        [false, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
      ),
      {
        from: deployer.address
      }
    )
    await channels.openChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)
    await channels.initiateChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address)

    expect(channels.finalizeChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_A.address)).to.be.revertedWith(
      'initiator and counterparty must not be the same'
    )

    expect(channels.finalizeChannelClosureInternal(ethers.constants.AddressZero, ACCOUNT_B.address)).to.be.revertedWith(
      'initiator must not be empty'
    )

    expect(channels.finalizeChannelClosureInternal(ACCOUNT_A.address, ethers.constants.AddressZero)).to.be.revertedWith(
      'counterparty must not be empty'
    )

    expect(channels.finalizeChannelClosureInternal(ACCOUNT_A.address, ACCOUNT_B.address)).to.be.revertedWith(
      'closureTime must be before now'
    )
  })

  it('should get channel data', async function () {
    const { channels } = await useFixtures()

    const channelData = await channels.getChannelInternal.call(ACCOUNT_A.address, ACCOUNT_B.address)
    expect(channelData[0]).to.be.equal(ACCOUNT_A.address)
    expect(channelData[1]).to.be.equal(ACCOUNT_B.address)
    expect(channelData[2]).to.be.equal(ACCOUNT_AB_CHANNEL_ID)
  })

  it('should get channel id', async function () {
    const { channels } = await useFixtures()

    const channelId = await channels.getChannelIdInternal.call(ACCOUNT_A.address, ACCOUNT_B.address)
    expect(channelId).to.be.equal(ACCOUNT_AB_CHANNEL_ID)
  })

  it('should get channel status', async function () {
    const { channels } = await useFixtures()

    const status = await channels.getChannelStatusInternal.call('11')
    expect(status.toString()).to.be.equal('1')
  })

  it('should get channel iteration', async function () {
    const { channels } = await useFixtures()

    const iteration = await channels.getChannelIterationInternal.call('11')
    expect(iteration.toString()).to.be.equal('2')
  })

  it('should be partyA', async function () {
    const { channels } = await useFixtures()

    const isPartyA = await channels.isPartyAInternal.call(ACCOUNT_A.address, ACCOUNT_B.address)
    expect(isPartyA).to.be.true
  })

  it('should not be partyA', async function () {
    const { channels } = await useFixtures()

    const isPartyA = await channels.isPartyAInternal.call(ACCOUNT_B.address, ACCOUNT_A.address)
    expect(isPartyA).to.be.false
  })

  it('should get partyA and partyB', async function () {
    const { channels } = await useFixtures()

    const parties = await channels.getPartiesInternal.call(ACCOUNT_A.address, ACCOUNT_B.address)
    expect(parties[0]).to.be.equal(ACCOUNT_A.address)
    expect(parties[1]).to.be.equal(ACCOUNT_B.address)
  })
})
