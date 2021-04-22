import { deployments, ethers } from 'hardhat'
import { expect } from 'chai'
import { PromiseValue, durations } from '@hoprnet/hopr-utils'
import { createTicket } from './utils'
import { increaseTime } from '../utils'
import {
  ACCOUNT_A,
  ACCOUNT_B,
  ACCOUNT_AB_CHANNEL_ID,
  SECRET_2,
  generateTickets,
  PROOF_OF_RELAY_SECRET_0,
  PROOF_OF_RELAY_SECRET_1,
  WIN_PROB_100,
  SECRET_1,
  SECRET_0
} from './constants'
import { HoprToken__factory, HoprChannels__factory } from '../../types'

const abiEncoder = ethers.utils.Interface.getAbiCoder()

const useFixtures = deployments.createFixture(async () => {
  const [deployer] = await ethers.getSigners()
  const accountA = await ethers.getSigner(ACCOUNT_A.address)
  const accountB = await ethers.getSigner(ACCOUNT_B.address)

  // run migrations
  const contracts = await deployments.fixture()
  const hoprToken = HoprToken__factory.connect(contracts['HoprToken'].address, ethers.provider)
  const hoprChannels = HoprChannels__factory.connect(contracts['HoprChannels'].address, ethers.provider)

  // create deployer the minter
  const minterRole = await hoprToken.MINTER_ROLE()
  await hoprToken.connect(deployer).grantRole(minterRole, deployer.address)

  // mint tokens for accountA and accountB
  await hoprToken.connect(deployer).mint(ACCOUNT_A.address, '100', ethers.constants.HashZero, ethers.constants.HashZero)
  await hoprToken.connect(deployer).mint(ACCOUNT_B.address, '100', ethers.constants.HashZero, ethers.constants.HashZero)

  // mocked tickets
  const mockedTickets = await generateTickets()

  return {
    hoprToken,
    hoprChannels,
    deployer,
    accountA,
    accountB,
    ...mockedTickets
  }
})

describe('HoprChannels', function () {
  it('should fund one direction', async function () {
    const { hoprToken, hoprChannels, accountA } = await useFixtures()

    await hoprToken.connect(accountA).approve(hoprChannels.address, '70')

    await expect(hoprChannels.connect(accountA).fundChannel(ACCOUNT_A.address, ACCOUNT_B.address, '70'))
      .to.emit(hoprChannels, 'ChannelFunded')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, ACCOUNT_A.address, '70', '70')

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.deposit.toString()).to.equal('70')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('0')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('30')
  })

  it('should fund both directions', async function () {
    const { hoprToken, hoprChannels, accountA } = await useFixtures()

    await hoprToken.connect(accountA).approve(hoprChannels.address, '100')

    await expect(hoprChannels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30'))
      .to.emit(hoprChannels, 'ChannelFunded')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, ACCOUNT_A.address, '100', '70')

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.deposit.toString()).to.equal('100')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('0')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('0')
  })

  it('should fund and open channel', async function () {
    const { hoprToken, hoprChannels, accountA } = await useFixtures()

    await hoprToken.connect(accountA).approve(hoprChannels.address, '70')

    await expect(hoprChannels.connect(accountA).fundAndOpenChannel(ACCOUNT_A.address, ACCOUNT_B.address, '70', '0'))
      .to.emit(hoprChannels, 'ChannelFunded')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, ACCOUNT_A.address, '70', '70')
      .and.to.emit(hoprChannels, 'ChannelOpened')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address)

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.deposit.toString()).to.equal('70')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('1')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('30')
  })

  it('should fund and open channel by accountB', async function () {
    const { hoprToken, hoprChannels, accountB } = await useFixtures()

    await hoprToken.connect(accountB).approve(hoprChannels.address, '70')

    await expect(hoprChannels.connect(accountB).fundAndOpenChannel(ACCOUNT_A.address, ACCOUNT_B.address, '70', '0'))
      .to.emit(hoprChannels, 'ChannelFunded')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, ACCOUNT_B.address, '70', '70')
      .and.to.emit(hoprChannels, 'ChannelOpened')
      .withArgs(ACCOUNT_B.address, ACCOUNT_A.address)

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.deposit.toString()).to.equal('70')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('1')
    expect(channel.closureByPartyA).to.be.false

    const accountBBalance = await hoprToken.balanceOf(ACCOUNT_B.address)
    expect(accountBBalance.toString()).to.equal('30')
  })

  it('should fund using send', async function () {
    const { hoprToken, hoprChannels, accountA } = await useFixtures()

    await expect(
      hoprToken
        .connect(accountA)
        .send(
          hoprChannels.address,
          '70',
          abiEncoder.encode(['bool', 'address', 'address'], [false, ACCOUNT_A.address, ACCOUNT_B.address])
        )
    )
      .to.emit(hoprChannels, 'ChannelFunded')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, ACCOUNT_A.address, '70', '70')

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.deposit.toString()).to.equal('70')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('0')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('30')
  })

  it('should fund and open using send', async function () {
    const { hoprToken, hoprChannels, accountA } = await useFixtures()

    await expect(
      hoprToken
        .connect(accountA)
        .send(
          hoprChannels.address,
          '70',
          abiEncoder.encode(['bool', 'address', 'address'], [true, ACCOUNT_A.address, ACCOUNT_B.address])
        )
    )
      .to.emit(hoprChannels, 'ChannelFunded')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, ACCOUNT_A.address, '70', '70')

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.deposit.toString()).to.equal('70')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('1')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('30')
  })

  it('should fund both parties using send', async function () {
    const { hoprToken, hoprChannels, accountA } = await useFixtures()

    await expect(
      hoprToken
        .connect(accountA)
        .send(
          hoprChannels.address,
          '100',
          abiEncoder.encode(
            ['bool', 'address', 'address', 'uint256', 'uint256'],
            [false, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
          )
        )
    )
      .to.emit(hoprChannels, 'ChannelFunded')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, ACCOUNT_A.address, '100', '70')

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
    expect(channel.deposit.toString()).to.equal('100')
    expect(channel.partyABalance.toString()).to.equal('70')
    expect(channel.closureTime.toString()).to.equal('0')
    expect(channel.status.toString()).to.equal('0')
    expect(channel.closureByPartyA).to.be.false

    const accountABalance = await hoprToken.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('0')
  })

  it('should fund both parties and open using send', async function () {
    const { hoprToken, hoprChannels, accountA } = await useFixtures()

    await expect(
      hoprToken
        .connect(accountA)
        .send(
          hoprChannels.address,
          '100',
          abiEncoder.encode(
            ['bool', 'address', 'address', 'uint256', 'uint256'],
            [true, ACCOUNT_A.address, ACCOUNT_B.address, '70', '30']
          )
        )
    )
      .to.emit(hoprChannels, 'ChannelFunded')
      .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, ACCOUNT_A.address, '100', '70')

    const channel = await hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
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
  let f: PromiseValue<ReturnType<typeof useFixtures>>

  before(async function () {
    f = await useFixtures()
  })

  context('on a fresh channel', function () {
    it('should initialize accountA', async function () {
      await expect(f.hoprChannels.connect(f.accountA).initializeAccount(ACCOUNT_A.uncompressedPublicKey, SECRET_2))
        .to.emit(f.hoprChannels, 'AccountInitialized')
        .withArgs(ACCOUNT_A.address, ACCOUNT_A.uncompressedPublicKey, SECRET_2)

      const account = await f.hoprChannels.accounts(ACCOUNT_A.address)
      expect(account.secret).to.equal(SECRET_2)
      expect(account.counter.toString()).to.equal('1')
    })

    it('should initialize accountB', async function () {
      await expect(f.hoprChannels.connect(f.accountB).initializeAccount(ACCOUNT_B.uncompressedPublicKey, SECRET_2))
        .to.emit(f.hoprChannels, 'AccountInitialized')
        .withArgs(ACCOUNT_B.address, ACCOUNT_B.uncompressedPublicKey, SECRET_2)

      const account = await f.hoprChannels.accounts(ACCOUNT_B.address)
      expect(account.secret).to.equal(SECRET_2)
      expect(account.counter.toString()).to.equal('1')
    })

    it('should fund accountA', async function () {
      await f.hoprToken.connect(f.accountA).approve(f.hoprChannels.address, '70')

      await expect(f.hoprChannels.connect(f.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '0'))
        .to.emit(f.hoprChannels, 'ChannelFunded')
        .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, ACCOUNT_A.address, '70', '70')

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('70')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('0')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should fund accountB using send', async function () {
      await expect(
        f.hoprToken
          .connect(f.accountB)
          .send(
            f.hoprChannels.address,
            '30',
            abiEncoder.encode(['bool', 'address', 'address'], [false, ACCOUNT_B.address, ACCOUNT_A.address])
          )
      )
        .to.emit(f.hoprChannels, 'ChannelFunded')
        .withArgs(ACCOUNT_B.address, ACCOUNT_A.address, ACCOUNT_B.address, '100', '70')

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('100')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('0')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should open channel', async function () {
      await expect(f.hoprChannels.connect(f.accountA).openChannel(ACCOUNT_B.address))
        .to.emit(f.hoprChannels, 'ChannelOpened')
        .withArgs(ACCOUNT_A.address, ACCOUNT_B.address)

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('100')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('1')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should reedem ticket for accountA', async function () {
      await f.hoprChannels
        .connect(f.accountA)
        .redeemTicket(
          f.TICKET_BA_WIN.counterparty,
          f.TICKET_BA_WIN.secret,
          f.TICKET_BA_WIN.proofOfRelaySecret,
          f.TICKET_BA_WIN.amount,
          f.TICKET_BA_WIN.winProb,
          f.TICKET_BA_WIN.r,
          f.TICKET_BA_WIN.s,
          f.TICKET_BA_WIN.v
        )

      const ticket = await f.hoprChannels.tickets(f.TICKET_BA_WIN.hash)
      expect(ticket).to.be.true

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('100')
      expect(channel.partyABalance.toString()).to.equal('80')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('1')
      expect(channel.closureByPartyA).to.be.false

      const account = await f.hoprChannels.accounts(ACCOUNT_A.address)
      expect(account.secret).to.equal(SECRET_1)
    })

    it('should reedem ticket for accountB', async function () {
      await f.hoprChannels
        .connect(f.accountB)
        .redeemTicket(
          f.TICKET_AB_WIN.counterparty,
          f.TICKET_AB_WIN.secret,
          f.TICKET_AB_WIN.proofOfRelaySecret,
          f.TICKET_AB_WIN.amount,
          f.TICKET_AB_WIN.winProb,
          f.TICKET_AB_WIN.r,
          f.TICKET_AB_WIN.s,
          f.TICKET_AB_WIN.v
        )

      const ticket = await f.hoprChannels.tickets(f.TICKET_AB_WIN.hash)
      expect(ticket).to.be.true

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('100')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('1')
      expect(channel.closureByPartyA).to.be.false

      const account = await f.hoprChannels.accounts(ACCOUNT_B.address)
      expect(account.secret).to.equal(SECRET_1)
    })

    it('should initialize channel closure', async function () {
      await expect(f.hoprChannels.connect(f.accountB).initiateChannelClosure(ACCOUNT_A.address)).to.emit(
        f.hoprChannels,
        'ChannelPendingToClose'
      )
      // TODO: implement
      // .withArgs(ACCOUNT_B.address, ACCOUNT_A.address)

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('100')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.not.be.equal('0')
      expect(channel.status.toString()).to.equal('2')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should reedem ticket for accountA', async function () {
      await f.hoprChannels
        .connect(f.accountA)
        .redeemTicket(
          f.TICKET_BA_WIN_2.counterparty,
          f.TICKET_BA_WIN_2.secret,
          f.TICKET_BA_WIN_2.proofOfRelaySecret,
          f.TICKET_BA_WIN_2.amount,
          f.TICKET_BA_WIN_2.winProb,
          f.TICKET_BA_WIN_2.r,
          f.TICKET_BA_WIN_2.s,
          f.TICKET_BA_WIN_2.v
        )

      const ticket = await f.hoprChannels.tickets(f.TICKET_BA_WIN_2.hash)
      expect(ticket).to.be.true

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('100')
      expect(channel.partyABalance.toString()).to.equal('80')
      expect(channel.closureTime.toString()).to.not.be.equal('0')
      expect(channel.status.toString()).to.equal('2')
      expect(channel.closureByPartyA).to.be.false

      const account = await f.hoprChannels.accounts(ACCOUNT_A.address)
      expect(account.secret).to.equal(SECRET_0)
    })

    it('should close channel', async function () {
      await increaseTime(ethers.provider, durations.days(3))

      await expect(f.hoprChannels.connect(f.accountA).finalizeChannelClosure(ACCOUNT_B.address))
        .to.emit(f.hoprChannels, 'ChannelClosed')
        .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, '80', '20')

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('0')
      expect(channel.partyABalance.toString()).to.equal('0')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('10')
      expect(channel.closureByPartyA).to.be.false

      const accountABalance = await f.hoprToken.balanceOf(ACCOUNT_A.address)
      expect(accountABalance.toString()).to.equal('110')
      const accountBBalance = await f.hoprToken.balanceOf(ACCOUNT_B.address)
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
      // this tickets are for channel iteration 2
      // and account counter 2
      TICKET_AB_WIN_RECYCLED = await createTicket(
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

      TICKET_BA_WIN_RECYCLED = await createTicket(
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

      TICKET_BA_WIN_RECYCLED_2 = await createTicket(
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
    })

    it('should update accountA', async function () {
      await expect(f.hoprChannels.connect(f.accountA).updateAccountSecret(SECRET_2))
        .to.emit(f.hoprChannels, 'AccountSecretUpdated')
        .withArgs(ACCOUNT_A.address, SECRET_2, '2')

      const account = await f.hoprChannels.accounts(ACCOUNT_A.address)
      expect(account.secret).to.equal(SECRET_2)
      expect(account.counter.toString()).to.equal('2')
    })

    it('should update accountB', async function () {
      await expect(f.hoprChannels.connect(f.accountB).updateAccountSecret(SECRET_2))
        .to.emit(f.hoprChannels, 'AccountSecretUpdated')
        .withArgs(ACCOUNT_B.address, SECRET_2, '2')

      const account = await f.hoprChannels.accounts(ACCOUNT_B.address)
      expect(account.secret).to.equal(SECRET_2)
      expect(account.counter.toString()).to.equal('2')
    })

    it('should fund both parties and open channel', async function () {
      await f.hoprToken.connect(f.accountA).approve(f.hoprChannels.address, '110')

      await expect(
        f.hoprChannels.connect(f.accountA).fundAndOpenChannel(ACCOUNT_A.address, ACCOUNT_B.address, '70', '40')
      )
        .to.emit(f.hoprChannels, 'ChannelFunded')
        .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, ACCOUNT_A.address, '110', '70')

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('110')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('11')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should reedem ticket for accountA', async function () {
      await f.hoprChannels
        .connect(f.accountA)
        .redeemTicket(
          TICKET_BA_WIN_RECYCLED.counterparty,
          TICKET_BA_WIN_RECYCLED.secret,
          TICKET_BA_WIN_RECYCLED.proofOfRelaySecret,
          TICKET_BA_WIN_RECYCLED.amount,
          TICKET_BA_WIN_RECYCLED.winProb,
          TICKET_BA_WIN_RECYCLED.r,
          TICKET_BA_WIN_RECYCLED.s,
          TICKET_BA_WIN_RECYCLED.v
        )

      const ticket = await f.hoprChannels.tickets(TICKET_BA_WIN_RECYCLED.hash)
      expect(ticket).to.be.true

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('110')
      expect(channel.partyABalance.toString()).to.equal('80')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('11')
      expect(channel.closureByPartyA).to.be.false

      const account = await f.hoprChannels.accounts(ACCOUNT_A.address)
      expect(account.secret).to.equal(SECRET_1)
    })

    it('should reedem ticket for accountB', async function () {
      await f.hoprChannels
        .connect(f.accountB)
        .redeemTicket(
          TICKET_AB_WIN_RECYCLED.counterparty,
          TICKET_AB_WIN_RECYCLED.secret,
          TICKET_AB_WIN_RECYCLED.proofOfRelaySecret,
          TICKET_AB_WIN_RECYCLED.amount,
          TICKET_AB_WIN_RECYCLED.winProb,
          TICKET_AB_WIN_RECYCLED.r,
          TICKET_AB_WIN_RECYCLED.s,
          TICKET_AB_WIN_RECYCLED.v
        )

      const ticket = await f.hoprChannels.tickets(TICKET_AB_WIN_RECYCLED.hash)
      expect(ticket).to.be.true

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('110')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('11')
      expect(channel.closureByPartyA).to.be.false

      const account = await f.hoprChannels.accounts(ACCOUNT_A.address)
      expect(account.secret).to.equal(SECRET_1)
    })

    it('should initialize channel closure', async function () {
      await expect(f.hoprChannels.connect(f.accountB).initiateChannelClosure(ACCOUNT_A.address)).to.emit(
        f.hoprChannels,
        'ChannelPendingToClose'
      )
      // TODO: implement
      // .withArgs(ACCOUNT_B.address, ACCOUNT_A.address)

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('110')
      expect(channel.partyABalance.toString()).to.equal('70')
      expect(channel.closureTime.toString()).to.not.be.equal('0')
      expect(channel.status.toString()).to.equal('12')
      expect(channel.closureByPartyA).to.be.false
    })

    it('should reedem ticket for accountA', async function () {
      await f.hoprChannels
        .connect(f.accountA)
        .redeemTicket(
          TICKET_BA_WIN_RECYCLED_2.counterparty,
          TICKET_BA_WIN_RECYCLED_2.secret,
          TICKET_BA_WIN_RECYCLED_2.proofOfRelaySecret,
          TICKET_BA_WIN_RECYCLED_2.amount,
          TICKET_BA_WIN_RECYCLED_2.winProb,
          TICKET_BA_WIN_RECYCLED_2.r,
          TICKET_BA_WIN_RECYCLED_2.s,
          TICKET_BA_WIN_RECYCLED_2.v
        )

      const ticket = await f.hoprChannels.tickets(TICKET_BA_WIN_RECYCLED_2.hash)
      expect(ticket).to.be.true

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('110')
      expect(channel.partyABalance.toString()).to.equal('80')
      expect(channel.closureTime.toString()).to.not.be.equal('0')
      expect(channel.status.toString()).to.equal('12')
      expect(channel.closureByPartyA).to.be.false

      const account = await f.hoprChannels.accounts(ACCOUNT_A.address)
      expect(account.secret).to.equal(SECRET_0)
    })

    it('should close channel', async function () {
      await increaseTime(ethers.provider, durations.days(3))

      await expect(f.hoprChannels.connect(f.accountA).finalizeChannelClosure(ACCOUNT_B.address))
        .to.emit(f.hoprChannels, 'ChannelClosed')
        .withArgs(ACCOUNT_A.address, ACCOUNT_B.address, '80', '30')

      const channel = await f.hoprChannels.channels(ACCOUNT_AB_CHANNEL_ID)
      expect(channel.deposit.toString()).to.equal('0')
      expect(channel.partyABalance.toString()).to.equal('0')
      expect(channel.closureTime.toString()).to.equal('0')
      expect(channel.status.toString()).to.equal('20')
      expect(channel.closureByPartyA).to.be.false

      const accountABalance = await f.hoprToken.balanceOf(ACCOUNT_A.address)
      expect(accountABalance.toString()).to.equal('80')
      const accountBBalance = await f.hoprToken.balanceOf(ACCOUNT_B.address)
      expect(accountBBalance.toString()).to.equal('120')
    })
  })
})
