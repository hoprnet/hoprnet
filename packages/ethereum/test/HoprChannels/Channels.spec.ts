import { deployments, ethers } from 'hardhat'
import { expect } from 'chai'
import { HoprToken__factory, ChannelsMock__factory, HoprChannels__factory } from '../../types'
import { redeemArgs, validateChannel } from './utils'
import { percentToUint256, createTicket } from './utils'
import { increaseTime } from '../utils'
import { ACCOUNT_A, ACCOUNT_B } from '../constants'

const { solidityKeccak256 } = ethers.utils

// accountA == partyA
// accountB == partyB
/**
 * Channel id of account A and B
 */
export const ACCOUNT_AB_CHANNEL_ID = '0xa5bc13ae60ec79a8babc6d0d4074c1cefd5d5fc19fafe71457214d46c90714d8'

export const PROOF_OF_RELAY_SECRET_0 = solidityKeccak256(['string'], ['PROOF_OF_RELAY_SECRET_0'])
export const PROOF_OF_RELAY_SECRET_1 = solidityKeccak256(['string'], ['PROOF_OF_RELAY_SECRET_1'])

export const SECRET_0 = solidityKeccak256(['string'], ['secret'])
export const SECRET_1 = solidityKeccak256(['bytes32'], [SECRET_0])
export const SECRET_2 = solidityKeccak256(['bytes32'], [SECRET_1])

export const WIN_PROB_100 = percentToUint256(100)
export const WIN_PROB_0 = percentToUint256(0)
const ENOUGH_TIME_FOR_CLOSURE = 100

export const generateTickets = async () => {
  /**
   * Winning ticket created by accountA for accountB
   */
  const TICKET_AB_WIN = await createTicket(
    {
      recipient: ACCOUNT_B.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      ticketEpoch: '0',
      ticketIndex: '1',
      amount: '10',
      winProb: WIN_PROB_100,
      channelEpoch: '1'
    },
    ACCOUNT_A,
    SECRET_1
  )

  /**
   * Winning ticket created by accountA for accountB.
   * Compared to TICKET_AB_WIN it has different proof of secret,
   * this effectively makes it a different ticket that can be
   * redeemed.
   */
  const TICKET_AB_WIN_2 = await createTicket(
    {
      recipient: ACCOUNT_B.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_1,
      ticketEpoch: '0',
      ticketIndex: '1',
      amount: '10',
      winProb: WIN_PROB_100,
      channelEpoch: '1'
    },
    ACCOUNT_A,
    SECRET_0
  )

  /**
   * Losing ticket created by accountA for accountB
   */
  const TICKET_AB_LOSS = await createTicket(
    {
      recipient: ACCOUNT_B.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      ticketEpoch: '0',
      ticketIndex: '1',
      amount: '10',
      winProb: WIN_PROB_0,
      channelEpoch: '1'
    },
    ACCOUNT_A,
    SECRET_1
  )

  /**
   * Winning ticket created by accountB for accountA
   */
  const TICKET_BA_WIN = await createTicket(
    {
      recipient: ACCOUNT_A.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      ticketEpoch: '1',
      ticketIndex: '1',
      amount: '10',
      winProb: WIN_PROB_100,
      channelEpoch: '1'
    },
    ACCOUNT_B,
    SECRET_1
  )

  /**
   * Winning ticket created by accountB for accountA.
   * Compared to TICKET_BA_WIN it has different proof of secret,
   * this effectively makes it a different ticket that can be
   * redeemed.
   */
  const TICKET_BA_WIN_2 = await createTicket(
    {
      recipient: ACCOUNT_A.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_1,
      ticketEpoch: '2',
      ticketIndex: '1',
      amount: '10',
      winProb: WIN_PROB_100,
      channelEpoch: '1'
    },
    ACCOUNT_B,
    SECRET_0
  )

  // Recycled

  const TICKET_AB_WIN_RECYCLED = await createTicket(
    {
      recipient: ACCOUNT_B.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      ticketIndex: '1',
      ticketEpoch: '0',
      amount: '10',
      winProb: WIN_PROB_100,
      channelEpoch: '2'
    },
    ACCOUNT_A,
    SECRET_1
  )

  const TICKET_BA_WIN_RECYCLED = await createTicket(
    {
      recipient: ACCOUNT_A.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      ticketIndex: '1',
      ticketEpoch: '2',
      amount: '10',
      winProb: WIN_PROB_100,
      channelEpoch: '2'
    },
    ACCOUNT_B,
    SECRET_1
  )

  return {
    TICKET_AB_WIN,
    TICKET_AB_WIN_2,
    TICKET_AB_LOSS,
    TICKET_BA_WIN,
    TICKET_BA_WIN_2,
    TICKET_AB_WIN_RECYCLED,
    TICKET_BA_WIN_RECYCLED
  }
}

const abiEncoder = ethers.utils.Interface.getAbiCoder()

const useFixtures = deployments.createFixture(async () => {
  const [deployer] = await ethers.getSigners()
  const accountA = await ethers.getSigner(ACCOUNT_A.address)
  const accountB = await ethers.getSigner(ACCOUNT_B.address)

  // run migrations
  const contracts = await deployments.fixture()
  const token = HoprToken__factory.connect(contracts['HoprToken'].address, ethers.provider)
  const channels = HoprChannels__factory.connect(contracts['HoprChannels'].address, ethers.provider)
  const mockChannels = await new ChannelsMock__factory(deployer).deploy(token.address, 0)

  // create deployer the minter
  const minterRole = await token.MINTER_ROLE()
  await token.connect(deployer).grantRole(minterRole, deployer.address)

  const fixtureTickets = await generateTickets()

  const fund = async (addr, amount) =>
    await token.connect(deployer).mint(addr, amount + '', ethers.constants.HashZero, ethers.constants.HashZero)

  const approve = async (account, amount) => await token.connect(account).approve(channels.address, amount)

  const fundAndApprove = async (account, amount) => {
    await fund(account.address, amount)
    await approve(account, amount)
  }

  return {
    token,
    channels,
    deployer,
    fixtureTickets,
    accountA,
    accountB,
    fund,
    approve,
    mockChannels,
    fundAndApprove
  }
})

describe('funding HoprChannel catches failures', async function () {
  let fixtures, channels, accountA
  before(async function () {
    // All of these tests revert, so we can rely on stateless single fixture.
    fixtures = await useFixtures()
    channels = fixtures.channels
    accountA = fixtures.accountA
    await fixtures.fundAndApprove(accountA, 100)
  })

  it('should fail to fund channel A->A', async function () {
    await expect(
      channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_A.address, '70', '30')
    ).to.be.revertedWith('accountA and accountB must not be the same')
  })

  it('should fail to fund channel 0->A', async function () {
    await expect(
      channels.connect(accountA).fundChannelMulti(ethers.constants.AddressZero, ACCOUNT_B.address, '70', '30')
    ).to.be.revertedWith('accountA must not be empty')
  })

  it('should fail to fund channel A->0', async function () {
    await expect(
      channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ethers.constants.AddressZero, '70', '30')
    ).to.be.revertedWith('accountB must not be empty')
  })

  it('should fail to fund a channel with 0 amount', async function () {
    await expect(
      channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '0', '0')
    ).to.be.revertedWith('amountA or amountB must be greater than 0')
  })
})

describe('funding a HoprChannel success', function () {
  // TODO events
  it('should multi fund and open channel A->B', async function () {
    const { channels, accountA, fundAndApprove, token } = await useFixtures()
    await fundAndApprove(accountA, 100)
    await expect(channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')).to.emit(
      channels,
      'ChannelUpdate'
    )
    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, { partyABalance: '70', partyBBalance: '30', status: '1' })
    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal('0')
  })

  it('should multi fund and open channel B->A', async function () {
    const { channels, accountB, fundAndApprove } = await useFixtures()
    await fundAndApprove(accountB, 100)
    await expect(channels.connect(accountB).fundChannelMulti(ACCOUNT_B.address, ACCOUNT_A.address, '30', '70')).to.emit(
      channels,
      'ChannelUpdate'
    )
    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, { partyABalance: '70', partyBBalance: '30', status: '1' })
  })

  it('should fund A->B using send', async function () {
    const { token, accountB, channels, fund } = await useFixtures()
    await fund(accountB.address, '30')
    await expect(
      token
        .connect(accountB)
        .send(
          channels.address,
          '30',
          abiEncoder.encode(
            ['address', 'address', 'uint256', 'uint256'],
            [ACCOUNT_B.address, ACCOUNT_A.address, '30', '0']
          )
        )
    ).to.emit(channels, 'ChannelUpdate')
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), {
      partyABalance: '0',
      partyBBalance: '30',
      status: '1'
    })
  })
})

describe('with a funded HoprChannel (A: 70, B: 30), secrets initialized', function () {
  let channels
  let fixtures
  beforeEach(async function () {
    fixtures = await useFixtures()
    channels = fixtures.channels
    fixtures.fundAndApprove(fixtures.accountA, 100)
    await channels.connect(fixtures.accountA).bumpChannel(ACCOUNT_B.address, SECRET_2)
    await channels.connect(fixtures.accountB).bumpChannel(ACCOUNT_A.address, SECRET_2) // TODO secret per account
    await channels.connect(fixtures.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
  })

  it('should redeem ticket for account A', async function () {
    await channels
      .connect(fixtures.accountA)
      //@ts-ignore
      .redeemTicket(...redeemArgs(fixtures.fixtureTickets.TICKET_BA_WIN))

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
      .redeemTicket(...redeemArgs(fixtures.fixtureTickets.TICKET_AB_WIN))

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
    const TICKET_AB_WIN = fixtures.fixtureTickets.TICKET_AB_WIN

    await channels
      .connect(fixtures.accountB)
      //@ts-ignore
      .redeemTicket(...redeemArgs(TICKET_AB_WIN))

    await expect(
      channels.connect(fixtures.accountB).redeemTicket(
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
      channels.connect(fixtures.accountB).redeemTicket(
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
    const TICKET_AB_WIN = fixtures.fixtureTickets.TICKET_AB_WIN
    const TICKET_BA_WIN = fixtures.fixtureTickets.TICKET_BA_WIN
    await expect(
      channels.connect(fixtures.accountB).redeemTicket(
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
    const TICKET_AB_LOSS = fixtures.fixtureTickets.TICKET_AB_LOSS
    await expect(
      channels
        .connect(fixtures.accountB)
        .redeemTicket(
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
    await expect(channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_B.address)).to.emit(
      channels,
      'ChannelUpdate'
    )
    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, { partyABalance: '70', partyBBalance: '30', status: '2', closureByPartyA: true })
  })

  it('B can initialize channel closure', async function () {
    await expect(channels.connect(fixtures.accountB).initiateChannelClosure(ACCOUNT_A.address)).to.emit(
      channels,
      'ChannelUpdate'
    )
    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(channel, { partyABalance: '70', partyBBalance: '30', status: '2', closureByPartyA: false })
  })

  it('should fail to initialize channel closure A->A', async function () {
    await expect(channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_A.address)).to.be.revertedWith(
      'initiator and counterparty must not be the same'
    )
  })

  it('should fail to initialize channel closure A->0', async function () {
    await expect(
      channels.connect(ACCOUNT_A.address).initiateChannelClosure(ethers.constants.AddressZero)
    ).to.be.revertedWith('counterparty must not be empty')
  })

  it('should fail to finalize channel closure when is not pending', async function () {
    await expect(channels.connect(ACCOUNT_A.address).finalizeChannelClosure(ACCOUNT_B.address)).to.be.revertedWith(
      'channel must be pending to close'
    )
  })
})

describe('With a pending_to_close HoprChannel (A:70, B:30)', function () {
  let channels, token, fixtures
  beforeEach(async function () {
    fixtures = await useFixtures()
    channels = fixtures.channels
    token = fixtures.token
    await fixtures.fundAndApprove(fixtures.accountA, 100)
    await channels.connect(fixtures.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_B.address)
  })

  it('should fail to initialize channel closure when channel is not open', async function () {
    await expect(channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_B.address)).to.be.revertedWith(
      'channel must be open'
    )
  })

  it('should finalize channel closure', async function () {
    await increaseTime(ethers.provider, ENOUGH_TIME_FOR_CLOSURE)
    await expect(channels.connect(fixtures.accountA).finalizeChannelClosure(ACCOUNT_B.address)).to.emit(
      channels,
      'ChannelUpdate'
    )
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), {
      partyABalance: '0',
      partyBBalance: '0',
      status: '0',
      closureByPartyA: false
    })
    expect((await token.balanceOf(ACCOUNT_A.address)).toString()).to.equal('70')
    expect((await token.balanceOf(ACCOUNT_B.address)).toString()).to.equal('30')
  })

  it('should fail to finalize channel closure', async function () {
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

describe('with a closed channel', function () {
  let channels, fixtures
  beforeEach(async function () {
    fixtures = await useFixtures()
    channels = fixtures.channels
    await fixtures.fundAndApprove(fixtures.accountA, 100)
    await channels.connect(fixtures.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_B.address)
    await channels.connect(fixtures.accountB).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await increaseTime(ethers.provider, ENOUGH_TIME_FOR_CLOSURE)
    await channels.connect(fixtures.accountA).finalizeChannelClosure(ACCOUNT_B.address)
  })

  it('should fail to redeem ticket when channel in closed', async function () {
    await expect(
      // @ts-ignore
      channels.connect(fixtures.accountB).redeemTicket(...redeemArgs(fixtures.fixtureTickets.TICKET_AB_WIN))
    ).to.be.revertedWith('channel must be open or pending to close')
  })

  it('should allow a fund to reopen channel', async function () {
    await fixtures.fundAndApprove(fixtures.accountA, 100)
    await channels.connect(fixtures.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), {
      partyABalance: '70',
      partyBBalance: '30',
      status: '1',
      closureByPartyA: false
    })
  })
})

describe('with a reopened channel', function () {
  let channels, fixtures
  beforeEach(async function () {
    fixtures = await useFixtures()
    channels = fixtures.channels
    await fixtures.fundAndApprove(fixtures.accountA, 100)
    await channels.connect(fixtures.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_B.address)
    await channels.connect(fixtures.accountB).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await channels.connect(fixtures.accountA).bumpChannel(ACCOUNT_B.address, SECRET_2)
    await increaseTime(ethers.provider, ENOUGH_TIME_FOR_CLOSURE)
    await channels.connect(fixtures.accountA).finalizeChannelClosure(ACCOUNT_B.address)
    await fixtures.fundAndApprove(fixtures.accountA, 100)
    await channels.connect(fixtures.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
  })

  it('should fail to redeem ticket when channel in in different channelEpoch', async function () {
    await expect(
      channels
        .connect(fixtures.accountB)
        //@ts-ignore
        .redeemTicket(...redeemArgs(fixtures.fixtureTickets.TICKET_AB_WIN))
    ).to.be.revertedWith('signer must match the counterparty')
  })

  it('should redeem ticket for account A', async function () {
    await channels
      .connect(fixtures.accountA)
      //@ts-ignore
      .redeemTicket(...redeemArgs(fixtures.fixtureTickets.TICKET_BA_WIN_RECYCLED))

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
      .redeemTicket(...redeemArgs(fixtures.fixtureTickets.TICKET_AB_WIN_RECYCLED))

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

  it('should allow closure', async function () {
    await channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_B.address)
    await increaseTime(ethers.provider, ENOUGH_TIME_FOR_CLOSURE)
    await channels.connect(fixtures.accountA).finalizeChannelClosure(ACCOUNT_B.address)
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), {
      partyABalance: '0',
      partyBBalance: '0',
      status: '0',
      closureByPartyA: false
    })
  })
})

describe('test internals with mock', function () {
  let channels
  beforeEach(async function () {
    channels = (await useFixtures()).mockChannels
  })

  it('should get channel data', async function () {
    const channelData = await channels.getChannelInternal(ACCOUNT_A.address, ACCOUNT_B.address)
    expect(channelData[0]).to.be.equal(ACCOUNT_A.address)
    expect(channelData[1]).to.be.equal(ACCOUNT_B.address)
    expect(channelData[2]).to.be.equal(ACCOUNT_AB_CHANNEL_ID)
  })

  it('should get channel id', async function () {
    expect(await channels.getChannelIdInternal(ACCOUNT_A.address, ACCOUNT_B.address)).to.be.equal(ACCOUNT_AB_CHANNEL_ID)
  })

  it('should be partyA', async function () {
    expect(await channels.isPartyAInternal(ACCOUNT_A.address, ACCOUNT_B.address)).to.be.true
  })

  it('should not be partyA', async function () {
    expect(await channels.isPartyAInternal(ACCOUNT_B.address, ACCOUNT_A.address)).to.be.false
  })

  it('should get partyA and partyB', async function () {
    const parties = await channels.getPartiesInternal(ACCOUNT_A.address, ACCOUNT_B.address)
    expect(parties[0]).to.be.equal(ACCOUNT_A.address)
    expect(parties[1]).to.be.equal(ACCOUNT_B.address)
  })

  it('should pack ticket', async function () {
    const { fixtureTickets } = await useFixtures()
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
    const { fixtureTickets } = await useFixtures()
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
