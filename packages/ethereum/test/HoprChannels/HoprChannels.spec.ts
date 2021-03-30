import type { HoprTokenInstance, HoprChannelsInstance } from '../../types'
import { deployments } from 'hardhat'
import { expectEvent, time } from '@openzeppelin/test-helpers'
import { formatAccount, formatChannel, createTicket } from './utils'
import {
  ACCOUNT_A,
  ACCOUNT_B,
  ACCOUNT_AB_CHANNEL_ID,
  SECRET_2,
  TICKET_AB_WIN,
  TICKET_BA_WIN,
  TICKET_BA_WIN_2,
  PROOF_OF_RELAY_SECRET_0,
  PROOF_OF_RELAY_SECRET_1,
  WIN_PROB_100,
  SECRET_1,
  SECRET_0
} from './constants'

const HoprToken = artifacts.require('HoprToken')
const HoprChannels = artifacts.require('HoprChannels')

const useFixtures = deployments.createFixture(async () => {
  const [deployer] = await web3.eth.getAccounts()

  // run migrations
  await deployments.fixture()

  const hoprTokenDeployment = await deployments.get('HoprToken')
  const hoprToken = await HoprToken.at(hoprTokenDeployment.address)

  const hoprChannelsDeployment = await deployments.get('HoprChannels')
  const hoprChannels = await HoprChannels.at(hoprChannelsDeployment.address)

  // create deployer the minter
  const minterRole = await hoprToken.MINTER_ROLE()
  await hoprToken.grantRole(minterRole, deployer)

  // mint tokens for accountA and accountB
  await hoprToken.mint(ACCOUNT_A.address, '100', '0x0', '0x0')
  await hoprToken.mint(ACCOUNT_B.address, '100', '0x0', '0x0')

  return {
    hoprToken,
    hoprChannels,
    deployer
  }
})

describe('HoprChannels', function () {
  it('should fund one direction', async function () {
    const { hoprToken, hoprChannels } = await useFixtures()

    await hoprToken.approve(hoprChannels.address, '70', {
      from: ACCOUNT_A.address
    })

    const response = await hoprChannels.fundChannel(ACCOUNT_A.address, ACCOUNT_B.address, '70', {
      from: ACCOUNT_A.address
    })

    expectEvent(response, 'ChannelFunded', {
      accountA: ACCOUNT_A.address,
      accountB: ACCOUNT_B.address,
      funder: ACCOUNT_A.address,
      deposit: '70',
      partyABalance: '70'
    })

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('70')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('0')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('30')
  })

  it('should fund both directions', async function () {
    const { hoprToken, hoprChannels } = await useFixtures()

    await hoprToken.approve(hoprChannels.address, '100', {
      from: ACCOUNT_A.address
    })

    const response = await hoprChannels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30', {
      from: ACCOUNT_A.address
    })

    expectEvent(response, 'ChannelFunded', {
      accountA: ACCOUNT_A.address,
      accountB: ACCOUNT_B.address,
      funder: ACCOUNT_A.address,
      deposit: '100',
      partyABalance: '70'
    })

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('100')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('0')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('0')
  })

  it('should fund and open channel', async function () {
    const { hoprToken, hoprChannels } = await useFixtures()

    await hoprToken.approve(hoprChannels.address, '70', {
      from: ACCOUNT_A.address
    })

    const response = await hoprChannels.fundAndOpenChannel(ACCOUNT_A.address, ACCOUNT_B.address, '70', '0', {
      from: ACCOUNT_A.address
    })

    expectEvent(response, 'ChannelFunded', {
      accountA: ACCOUNT_A.address,
      accountB: ACCOUNT_B.address,
      funder: ACCOUNT_A.address,
      deposit: '70',
      partyABalance: '70'
    })

    expectEvent(response, 'ChannelOpened', {
      opener: ACCOUNT_A.address,
      counterparty: ACCOUNT_B.address
    })

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('70')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('1')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('30')
  })

  it('should fund and open channel by accountB', async function () {
    const { hoprToken, hoprChannels } = await useFixtures()

    await hoprToken.approve(hoprChannels.address, '70', {
      from: ACCOUNT_B.address
    })

    const response = await hoprChannels.fundAndOpenChannel(ACCOUNT_A.address, ACCOUNT_B.address, '70', '0', {
      from: ACCOUNT_B.address
    })

    expectEvent(response, 'ChannelFunded', {
      accountA: ACCOUNT_A.address,
      accountB: ACCOUNT_B.address,
      funder: ACCOUNT_B.address,
      deposit: '70',
      partyABalance: '70'
    })

    expectEvent(response, 'ChannelOpened', {
      opener: ACCOUNT_B.address,
      counterparty: ACCOUNT_A.address
    })

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('70')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('1')
    expect(channel.closureByPartyA).to.be.false

    const accountBBalance = await hoprToken.balanceOf(ACCOUNT_B.address)
    expect(accountBBalance.toString()).to.equal('30')
  })

  it('should fund using send', async function () {
    const { hoprToken, hoprChannels } = await useFixtures()

    const response = await hoprToken.send(
      hoprChannels.address,
      '70',
      web3.eth.abi.encodeParameters(['bool', 'address', 'address'], [false, ACCOUNT_A.address, ACCOUNT_B.address]),
      {
        from: ACCOUNT_A.address
      }
    )

    expectEvent.inTransaction(response.tx, HoprChannels, 'ChannelFunded', {
      accountA: ACCOUNT_A.address,
      accountB: ACCOUNT_B.address,
      funder: ACCOUNT_A.address,
      deposit: '70',
      partyABalance: '70'
    })

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('70')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('0')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('30')
  })

  it('should fund and open using send', async function () {
    const { hoprToken, hoprChannels } = await useFixtures()

    const response = await hoprToken.send(
      hoprChannels.address,
      '70',
      web3.eth.abi.encodeParameters(['bool', 'address', 'address'], [true, ACCOUNT_A.address, ACCOUNT_B.address]),
      {
        from: ACCOUNT_A.address
      }
    )

    expectEvent.inTransaction(response.tx, HoprChannels, 'ChannelFunded', {
      accountA: ACCOUNT_A.address,
      accountB: ACCOUNT_B.address,
      funder: ACCOUNT_A.address,
      deposit: '70',
      partyABalance: '70'
    })

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('70')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('1')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('30')
  })

  it('should fund both parties using send', async function () {
    const { hoprToken, hoprChannels } = await useFixtures()

    const response = await hoprToken.send(
      hoprChannels.address,
      '100',
      web3.eth.abi.encodeParameters(
        ['bool', 'address', 'address', 'uint256', 'uint256'],
        [false, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
      ),
      {
        from: ACCOUNT_A.address
      }
    )

    expectEvent.inTransaction(response.tx, HoprChannels, 'ChannelFunded', {
      accountA: ACCOUNT_A.address,
      accountB: ACCOUNT_B.address,
      funder: ACCOUNT_A.address,
      deposit: '100',
      partyABalance: '70'
    })

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('100')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('0')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('0')
  })

  it('should fund both parties and open using send', async function () {
    const { hoprToken, hoprChannels } = await useFixtures()

    const response = await hoprToken.send(
      hoprChannels.address,
      '100',
      web3.eth.abi.encodeParameters(
        ['bool', 'address', 'address', 'uint256', 'uint256'],
        [true, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
      ),
      {
        from: ACCOUNT_A.address
      }
    )

    expectEvent.inTransaction(response.tx, HoprChannels, 'ChannelFunded', {
      accountA: ACCOUNT_A.address,
      accountB: ACCOUNT_B.address,
      funder: ACCOUNT_A.address,
      deposit: '100',
      partyABalance: '70'
    })

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
    expect(channel.deposit.toString()).to.equal('100')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('1')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('0')
  })
})

describe('HoprChannels intergration tests', function () {
  let hoprToken: HoprTokenInstance
  let hoprChannels: HoprChannelsInstance

  before(async () => {
    const fixtures = await useFixtures()
    hoprToken = fixtures.hoprToken
    hoprChannels = fixtures.hoprChannels
  })

  context('on a fresh channel', function () {
    it('should initialize accountA', async function () {
      const response = await hoprChannels.initializeAccount(ACCOUNT_A.uncompressedPubKey, SECRET_2, {
        from: ACCOUNT_A.address
      })

      expectEvent(response, 'AccountInitialized', {
        account: ACCOUNT_A.address,
        uncompressedPubKey: ACCOUNT_A.uncompressedPubKey,
        secret: SECRET_2
      })

      const account = await hoprChannels.accounts(ACCOUNT_A.address).then(formatAccount)
      expect(account.secret).to.equal(SECRET_2)
      expect(account.counter.toString()).to.equal('1')
    })

    it('should initialize accountB', async function () {
      const response = await hoprChannels.initializeAccount(ACCOUNT_B.uncompressedPubKey, SECRET_2, {
        from: ACCOUNT_B.address
      })

      expectEvent(response, 'AccountInitialized', {
        account: ACCOUNT_B.address,
        uncompressedPubKey: ACCOUNT_B.uncompressedPubKey,
        secret: SECRET_2
      })

      const account = await hoprChannels.accounts(ACCOUNT_B.address).then(formatAccount)
      expect(account.secret).to.equal(SECRET_2)
      expect(account.counter.toString()).to.equal('1')
    })

    it('should fund accountA', async function () {
      await hoprToken.approve(hoprChannels.address, '70', {
        from: ACCOUNT_A.address
      })

      const response = await hoprChannels.fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '0', {
        from: ACCOUNT_A.address
      })

      expectEvent(response, 'ChannelFunded', {
        accountA: ACCOUNT_A.address,
        accountB: ACCOUNT_B.address,
        funder: ACCOUNT_A.address,
        deposit: '70',
        partyABalance: '70'
      })

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('70')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('0')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should fund accountB using send', async function () {
      const response = await hoprToken.send(
        hoprChannels.address,
        '30',
        web3.eth.abi.encodeParameters(['bool', 'address', 'address'], [false, ACCOUNT_B.address, ACCOUNT_A.address]),
        {
          from: ACCOUNT_B.address
        }
      )

      expectEvent.inTransaction(response.tx, HoprChannels, 'ChannelFunded', {
        accountA: ACCOUNT_B.address,
        accountB: ACCOUNT_A.address,
        funder: ACCOUNT_B.address,
        deposit: '100',
        partyABalance: '70'
      })

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('100')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('0')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should open channel', async function () {
      const response = await hoprChannels.openChannel(ACCOUNT_B.address, {
        from: ACCOUNT_A.address
      })

      expectEvent(response, 'ChannelOpened', {
        opener: ACCOUNT_A.address,
        counterparty: ACCOUNT_B.address
      })

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('100')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('1')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should reedem ticket for accountA', async function () {
      await hoprChannels.redeemTicket(
        TICKET_BA_WIN.counterparty,
        TICKET_BA_WIN.secret,
        TICKET_BA_WIN.proofOfRelaySecret,
        TICKET_BA_WIN.amount,
        TICKET_BA_WIN.winProb,
        TICKET_BA_WIN.r,
        TICKET_BA_WIN.s,
        TICKET_BA_WIN.v,
        {
          from: ACCOUNT_A.address
        }
      )

      const ticket = await hoprChannels.tickets(TICKET_BA_WIN.hash)
      expect(ticket).to.be.true

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('100')
      expect(channel.partyABalance.toString()).to.equal('80')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('1')
      expect(channel.closureByPartyA).to.be.false

      const account = await hoprChannels.accounts(ACCOUNT_A.address).then(formatAccount)
      expect(account.secret).to.equal(SECRET_1)
    })

    it('should reedem ticket for accountB', async function () {
      await hoprChannels.redeemTicket(
        TICKET_AB_WIN.counterparty,
        TICKET_AB_WIN.secret,
        TICKET_AB_WIN.proofOfRelaySecret,
        TICKET_AB_WIN.amount,
        TICKET_AB_WIN.winProb,
        TICKET_AB_WIN.r,
        TICKET_AB_WIN.s,
        TICKET_AB_WIN.v,
        {
          from: ACCOUNT_B.address
        }
      )

      const ticket = await hoprChannels.tickets(TICKET_AB_WIN.hash)
      expect(ticket).to.be.true

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('100')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('1')
      expect(channel.closureByPartyA).to.be.false

      const account = await hoprChannels.accounts(ACCOUNT_B.address).then(formatAccount)
      expect(account.secret).to.equal(SECRET_1)
    })

    it('should initialize channel closure', async function () {
      const response = await hoprChannels.initiateChannelClosure(ACCOUNT_A.address, {
        from: ACCOUNT_B.address
      })

      await expectEvent(response, 'ChannelPendingToClose', {
        initiator: ACCOUNT_B.address,
        counterparty: ACCOUNT_A.address
      })

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('100')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.not.be.equal('0')
      expect(channel.status.toString()).to.equal('2')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should reedem ticket for accountA', async function () {
      await hoprChannels.redeemTicket(
        TICKET_BA_WIN_2.counterparty,
        TICKET_BA_WIN_2.secret,
        TICKET_BA_WIN_2.proofOfRelaySecret,
        TICKET_BA_WIN_2.amount,
        TICKET_BA_WIN_2.winProb,
        TICKET_BA_WIN_2.r,
        TICKET_BA_WIN_2.s,
        TICKET_BA_WIN_2.v,
        {
          from: ACCOUNT_A.address
        }
      )

      const ticket = await hoprChannels.tickets(TICKET_BA_WIN_2.hash)
      expect(ticket).to.be.true

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('100')
      expect(channel.partyABalance.toString()).to.equal('80')
      expect(channel.closureTime.toString()).to.not.be.equal('0')
      expect(channel.status.toString()).to.equal('2')
      expect(channel.closureByPartyA).to.be.false

      const account = await hoprChannels.accounts(ACCOUNT_A.address).then(formatAccount)
      expect(account.secret).to.equal(SECRET_0)
    })

    it('should close channel', async function () {
      await time.increase(time.duration.days(3))

      const response = await hoprChannels.finalizeChannelClosure(ACCOUNT_B.address, {
        from: ACCOUNT_A.address
      })

      await expectEvent(response, 'ChannelClosed', {
        initiator: ACCOUNT_A.address,
        counterparty: ACCOUNT_B.address,
        partyAAmount: '80',
        partyBAmount: '20'
      })

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('0')
      expect(channel.partyABalance.toString()).to.equal('0')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('10')
      expect(channel.closureByPartyA).to.be.false

      const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
      expect(accountABalance.toString()).to.equal('110')
      const accountBBalance = await hoprToken.balanceOf(ACCOUNT_B.address)
      expect(accountBBalance.toString()).to.equal('90')
    })
  })

  context('on a recycled channel', function () {
    // the key difference between these tickets
    // and tickets from constants.ts is that
    // this tickets are for channel iteration 2
    // and account counter 2
    const TICKET_AB_WIN_RECYCLED = createTicket(
      {
        recipient: ACCOUNT_B.address,
        proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
        counter: '2',
        amount: '10',
        winProb: WIN_PROB_100,
        iteration: '2'
      },
      ACCOUNT_A,
      SECRET_1
    )

    const TICKET_BA_WIN_RECYCLED = createTicket(
      {
        recipient: ACCOUNT_A.address,
        proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
        counter: '2',
        amount: '10',
        winProb: WIN_PROB_100,
        iteration: '2'
      },
      ACCOUNT_B,
      SECRET_1
    )

    const TICKET_BA_WIN_RECYCLED_2 = createTicket(
      {
        recipient: ACCOUNT_A.address,
        proofOfRelaySecret: PROOF_OF_RELAY_SECRET_1,
        counter: '2',
        amount: '10',
        winProb: WIN_PROB_100,
        iteration: '2'
      },
      ACCOUNT_B,
      SECRET_0
    )

    it('should update accountA', async function () {
      const response = await hoprChannels.updateAccountSecret(SECRET_2, {
        from: ACCOUNT_A.address
      })

      expectEvent(response, 'AccountSecretUpdated', {
        account: ACCOUNT_A.address,
        secret: SECRET_2
      })

      const account = await hoprChannels.accounts(ACCOUNT_A.address).then(formatAccount)
      expect(account.secret).to.equal(SECRET_2)
      expect(account.counter.toString()).to.equal('2')
    })

    it('should update accountB', async function () {
      const response = await hoprChannels.updateAccountSecret(SECRET_2, {
        from: ACCOUNT_B.address
      })

      expectEvent(response, 'AccountSecretUpdated', {
        account: ACCOUNT_B.address,
        secret: SECRET_2
      })

      const account = await hoprChannels.accounts(ACCOUNT_B.address).then(formatAccount)
      expect(account.secret).to.equal(SECRET_2)
      expect(account.counter.toString()).to.equal('2')
    })

    it('should fund both parties and open channel', async function () {
      await hoprToken.approve(hoprChannels.address, '110', {
        from: ACCOUNT_A.address
      })

      const response = await hoprChannels.fundAndOpenChannel(ACCOUNT_A.address, ACCOUNT_B.address, '70', '40', {
        from: ACCOUNT_A.address
      })

      expectEvent(response, 'ChannelFunded', {
        accountA: ACCOUNT_A.address,
        accountB: ACCOUNT_B.address,
        funder: ACCOUNT_A.address,
        deposit: '110',
        partyABalance: '70'
      })

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('110')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('11')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should reedem ticket for accountA', async function () {
      await hoprChannels.redeemTicket(
        TICKET_BA_WIN_RECYCLED.counterparty,
        TICKET_BA_WIN_RECYCLED.secret,
        TICKET_BA_WIN_RECYCLED.proofOfRelaySecret,
        TICKET_BA_WIN_RECYCLED.amount,
        TICKET_BA_WIN_RECYCLED.winProb,
        TICKET_BA_WIN_RECYCLED.r,
        TICKET_BA_WIN_RECYCLED.s,
        TICKET_BA_WIN_RECYCLED.v,
        {
          from: ACCOUNT_A.address
        }
      )

      const ticket = await hoprChannels.tickets(TICKET_BA_WIN_RECYCLED.hash)
      expect(ticket).to.be.true

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('110')
      expect(channel.partyABalance.toString()).to.equal('80')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('11')
      expect(channel.closureByPartyA).to.be.false

      const account = await hoprChannels.accounts(ACCOUNT_A.address).then(formatAccount)
      expect(account.secret).to.equal(SECRET_1)
    })

    it('should reedem ticket for accountB', async function () {
      await hoprChannels.redeemTicket(
        TICKET_AB_WIN_RECYCLED.counterparty,
        TICKET_AB_WIN_RECYCLED.secret,
        TICKET_AB_WIN_RECYCLED.proofOfRelaySecret,
        TICKET_AB_WIN_RECYCLED.amount,
        TICKET_AB_WIN_RECYCLED.winProb,
        TICKET_AB_WIN_RECYCLED.r,
        TICKET_AB_WIN_RECYCLED.s,
        TICKET_AB_WIN_RECYCLED.v,
        {
          from: ACCOUNT_B.address
        }
      )

      const ticket = await hoprChannels.tickets(TICKET_AB_WIN_RECYCLED.hash)
      expect(ticket).to.be.true

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('110')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('11')
      expect(channel.closureByPartyA).to.be.false

      const account = await hoprChannels.accounts(ACCOUNT_A.address).then(formatAccount)
      expect(account.secret).to.equal(SECRET_1)
    })

    it('should initialize channel closure', async function () {
      const response = await hoprChannels.initiateChannelClosure(ACCOUNT_A.address, {
        from: ACCOUNT_B.address
      })

      await expectEvent(response, 'ChannelPendingToClose', {
        initiator: ACCOUNT_B.address,
        counterparty: ACCOUNT_A.address
      })

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('110')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.not.be.equal('0')
      expect(channel.status.toString()).to.equal('12')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should reedem ticket for accountA', async function () {
      await hoprChannels.redeemTicket(
        TICKET_BA_WIN_RECYCLED_2.counterparty,
        TICKET_BA_WIN_RECYCLED_2.secret,
        TICKET_BA_WIN_RECYCLED_2.proofOfRelaySecret,
        TICKET_BA_WIN_RECYCLED_2.amount,
        TICKET_BA_WIN_RECYCLED_2.winProb,
        TICKET_BA_WIN_RECYCLED_2.r,
        TICKET_BA_WIN_RECYCLED_2.s,
        TICKET_BA_WIN_RECYCLED_2.v,
        {
          from: ACCOUNT_A.address
        }
      )

      const ticket = await hoprChannels.tickets(TICKET_BA_WIN_RECYCLED_2.hash)
      expect(ticket).to.be.true

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('110')
      expect(channel.partyABalance.toString()).to.equal('80')
      expect(channel.closureTime.toString()).to.not.be.equal('0')
      expect(channel.status.toString()).to.equal('12')
      expect(channel.closureByPartyA).to.be.false

      const account = await hoprChannels.accounts(ACCOUNT_A.address).then(formatAccount)
      expect(account.secret).to.equal(SECRET_0)
    })

    it('should close channel', async function () {
      await time.increase(time.duration.days(3))

      const response = await hoprChannels.finalizeChannelClosure(ACCOUNT_B.address, {
        from: ACCOUNT_A.address
      })

      await expectEvent(response, 'ChannelClosed', {
        initiator: ACCOUNT_A.address,
        counterparty: ACCOUNT_B.address,
        partyAAmount: '80',
        partyBAmount: '30'
      })

      const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID).then(formatChannel)
      expect(channel.deposit.toString()).to.equal('0')
      expect(channel.partyABalance.toString()).to.equal('0')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('20')
      expect(channel.closureByPartyA).to.be.false

      const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
      expect(accountABalance.toString()).to.equal('80')
      const accountBBalance = await hoprToken.balanceOf(ACCOUNT_B.address)
      expect(accountBBalance.toString()).to.equal('120')
    })
  })
})
