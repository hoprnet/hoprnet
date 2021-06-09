import { deployments, ethers } from 'hardhat'
import Multiaddr from 'multiaddr'
import { expect } from 'chai'
import BN from 'bn.js'
import { HoprToken__factory, ChannelsMock__factory, HoprChannels__factory } from '../types'
import { increaseTime } from './utils'
import { ACCOUNT_A, ACCOUNT_B } from './constants'
import {
  Address,
  Challenge,
  UINT256,
  Balance,
  stringToU8a,
  Ticket,
  Hash,
  Response,
  AcknowledgedTicket,
  PublicKey,
  generateChannelId
} from '@hoprnet/hopr-utils'
import { randomBytes } from 'crypto'

type TicketValues = {
  recipient: string
  proofOfRelaySecret: string
  amount: string
  winProb: string
  channelEpoch: string
  ticketIndex: string
  ticketEpoch: string
}

const WAITING_FOR_COMMITMENT = '1'
const OPEN = '2'
const CLOSED = '0'
const PENDING_CLOSURE = '3'

const percentToUint256 = (percent: any) => ethers.constants.MaxUint256.mul(percent).div(100)

export const redeemArgs = (ticket: AcknowledgedTicket) => [
  ticket.signer.toAddress().toHex(),
  ticket.preImage.toHex(),
  ticket.ticket.epoch.toHex(),
  ticket.ticket.index.toHex(),
  ticket.response.toHex(),
  ticket.ticket.amount.toHex(),
  ticket.ticket.winProb.toHex(),
  ticket.ticket.signature.serializeEthereum()
]

export const validateChannel = (actual, expected) => {
  expect(actual.balance.toString()).to.equal(expected.balance)
  expect(actual.status.toString()).to.equal(expected.status)
}

export const createTicket = async (
  ticketValues: TicketValues,
  counterparty: {
    privateKey: string
    address: string
  },
  nextCommitment: string
): Promise<
  TicketValues & {
    nextCommitment: string
    counterparty: string
    ticket: AcknowledgedTicket
  }
> => {
  const challenge = Challenge.fromExponent(stringToU8a(ticketValues.proofOfRelaySecret))

  const ticket = Ticket.create(
    Address.fromString(ticketValues.recipient),
    challenge,
    UINT256.fromString(ticketValues.ticketEpoch),
    UINT256.fromString(ticketValues.ticketIndex),
    new Balance(new BN(ticketValues.amount)),
    UINT256.fromString(ticketValues.winProb),
    UINT256.fromString(ticketValues.channelEpoch),
    stringToU8a(counterparty.privateKey)
  )

  const ackedTicket = new AcknowledgedTicket(
    ticket,
    new Response(stringToU8a(ticketValues.proofOfRelaySecret, Response.SIZE)),
    new Hash(stringToU8a(nextCommitment, Hash.SIZE)),
    PublicKey.fromPrivKey(stringToU8a(counterparty.privateKey))
  )

  return {
    ...ticketValues,
    ticket: ackedTicket,
    nextCommitment,
    counterparty: counterparty.address
  }
}

/**
 * Channel id of account A and B
 */
export const ACCOUNT_AB_CHANNEL_ID = generateChannelId(
  Address.fromString(ACCOUNT_A.address),
  Address.fromString(ACCOUNT_B.address)
).toHex()
export const ACCOUNT_BA_CHANNEL_ID = generateChannelId(
  Address.fromString(ACCOUNT_B.address),
  Address.fromString(ACCOUNT_A.address)
).toHex()

export const PROOF_OF_RELAY_SECRET_0 = ethers.utils.solidityKeccak256(['string'], ['PROOF_OF_RELAY_SECRET_0'])
export const PROOF_OF_RELAY_SECRET_1 = ethers.utils.solidityKeccak256(['string'], ['PROOF_OF_RELAY_SECRET_1'])

export const SECRET_0 = ethers.utils.solidityKeccak256(['string'], ['secret'])
export const SECRET_1 = ethers.utils.solidityKeccak256(['bytes32'], [SECRET_0])
export const SECRET_2 = ethers.utils.solidityKeccak256(['bytes32'], [SECRET_1])

export const WIN_PROB_100 = percentToUint256(100)
export const WIN_PROB_0 = percentToUint256(0)
const ENOUGH_TIME_FOR_CLOSURE = 100
const MULTI_ADDR = ethers.utils.hexlify(
  Multiaddr('/ip4/127.0.0.1/tcp/0/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg').bytes
)

const abiEncoder = ethers.utils.Interface.getAbiCoder()

const useFixtures = deployments.createFixture(async () => {
  const [deployer] = await ethers.getSigners()
  const accountA = new ethers.Wallet(ACCOUNT_A.privateKey).connect(ethers.provider)
  const accountB = new ethers.Wallet(ACCOUNT_B.privateKey).connect(ethers.provider)

  // run migrations
  const contracts = await deployments.fixture()
  const token = HoprToken__factory.connect(contracts['HoprToken'].address, ethers.provider)
  const channels = HoprChannels__factory.connect(contracts['HoprChannels'].address, ethers.provider)
  const mockChannels = await new ChannelsMock__factory(deployer).deploy(token.address, 0)

  // create deployer the minter
  const minterRole = await token.MINTER_ROLE()
  await token.connect(deployer).grantRole(minterRole, deployer.address)

  const fundEther = async (addr, amount) => await deployer.sendTransaction({ to: addr, value: amount })

  const fund = async (addr, amount) =>
    await token.connect(deployer).mint(addr, amount + '', ethers.constants.HashZero, ethers.constants.HashZero)

  const approve = async (account, amount) => await token.connect(account).approve(channels.address, amount)

  const fundAndApprove = async (account, amount) => {
    await fund(account.address, amount)
    await approve(account, amount)
  }

  const TICKET_AB_WIN = await createTicket(
    {
      recipient: ACCOUNT_B.address,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      ticketEpoch: '1',
      ticketIndex: '1',
      amount: '10',
      winProb: WIN_PROB_100.toString(),
      channelEpoch: '1'
    },
    ACCOUNT_A,
    SECRET_1
  )

  await fundEther(accountA.address, ethers.utils.parseEther(OPEN))
  await fundEther(accountB.address, ethers.utils.parseEther(OPEN))

  return {
    token,
    channels,
    deployer,
    accountA,
    accountB,
    fund,
    approve,
    mockChannels,
    fundAndApprove,
    TICKET_AB_WIN
  }
})

describe('announce user', function () {
  it('should announce user', async function () {
    const { channels, deployer } = await useFixtures()

    await expect(channels.connect(deployer).announce(MULTI_ADDR))
      .to.emit(channels, 'Announcement')
      .withArgs(deployer.address, MULTI_ADDR)
  })
})

describe('funding HoprChannel catches failures', function () {
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
    ).to.be.revertedWith('source and destination must not be the same')
  })

  it('should fail to fund channel 0->A', async function () {
    await expect(
      channels.connect(accountA).fundChannelMulti(ethers.constants.AddressZero, ACCOUNT_B.address, '70', '30')
    ).to.be.revertedWith('source must not be empty')
  })

  it('should fail to fund channel A->0', async function () {
    await expect(
      channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ethers.constants.AddressZero, '70', '30')
    ).to.be.revertedWith('destination must not be empty')
  })

  it('should fail to fund a channel with 0 amount', async function () {
    await expect(
      channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, CLOSED, CLOSED)
    ).to.be.revertedWith('amount must be greater than 0')
  })
})

describe('funding a HoprChannel success', function () {
  // TODO events
  it('should multi fund and open channel A->B, no commitment', async function () {
    const { channels, accountA, fundAndApprove, token } = await useFixtures()
    await fundAndApprove(accountA, 100)
    await expect(channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')).to.emit(
      channels,
      'ChannelUpdate'
    )
    const ab = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    const ba = await channels.channels(ACCOUNT_BA_CHANNEL_ID)
    validateChannel(ab, { balance: '70', status: WAITING_FOR_COMMITMENT })
    validateChannel(ba, { balance: '30', status: WAITING_FOR_COMMITMENT })
    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal(CLOSED)
  })

  it('should multi fund and open channel B->A, no commitment', async function () {
    const { channels, accountB, fundAndApprove } = await useFixtures()
    await fundAndApprove(accountB, 100)
    await expect(channels.connect(accountB).fundChannelMulti(ACCOUNT_B.address, ACCOUNT_A.address, '30', '70')).to.emit(
      channels,
      'ChannelUpdate'
    )
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: WAITING_FOR_COMMITMENT })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: WAITING_FOR_COMMITMENT })
  })

  it('should multi fund and open channel B->A, commit afterwards', async function () {
    const { channels, accountA, accountB, fundAndApprove } = await useFixtures()
    await fundAndApprove(accountB, 100)
    await expect(channels.connect(accountB).fundChannelMulti(ACCOUNT_B.address, ACCOUNT_A.address, '30', '70')).to.emit(
      channels,
      'ChannelUpdate'
    )
    await channels.connect(accountA).bumpChannel(ACCOUNT_B.address, SECRET_2)
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: WAITING_FOR_COMMITMENT })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: OPEN })
  })

  it('should multi fund and open channel B->A, pre-commitment', async function () {
    const { channels, accountA, accountB, fundAndApprove } = await useFixtures()
    await fundAndApprove(accountB, 100)
    await channels.connect(accountA).bumpChannel(ACCOUNT_B.address, SECRET_2)
    await expect(channels.connect(accountB).fundChannelMulti(ACCOUNT_B.address, ACCOUNT_A.address, '30', '70')).to.emit(
      channels,
      'ChannelUpdate'
    )
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: WAITING_FOR_COMMITMENT })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: OPEN })
  })

  it('should multi fund and open channel B->A, commit both', async function () {
    const { channels, accountA, accountB, fundAndApprove } = await useFixtures()
    await fundAndApprove(accountB, 100)
    await channels.connect(accountA).bumpChannel(ACCOUNT_B.address, SECRET_2)
    await channels.connect(accountB).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await expect(channels.connect(accountB).fundChannelMulti(ACCOUNT_B.address, ACCOUNT_A.address, '30', '70')).to.emit(
      channels,
      'ChannelUpdate'
    )
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: OPEN })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: OPEN })
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
            [ACCOUNT_B.address, ACCOUNT_A.address, '30', CLOSED]
          )
        )
    ).to.emit(channels, 'ChannelUpdate')
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: CLOSED, status: CLOSED })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: WAITING_FOR_COMMITMENT })
  })
})

describe('with funded HoprChannels: AB: 70, BA: 30, secrets initialized', function () {
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
    const TICKET_BA_WIN = await createTicket(
      {
        recipient: ACCOUNT_A.address,
        proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
        ticketEpoch: '1',
        ticketIndex: '1',
        amount: '10',
        winProb: WIN_PROB_100.toString(),
        channelEpoch: '1'
      },
      ACCOUNT_B,
      SECRET_1
    )

    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: OPEN })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: OPEN })

    await channels.connect(fixtures.accountA).redeemTicket(...redeemArgs(TICKET_BA_WIN.ticket))

    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '80', status: OPEN })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '20', status: OPEN })
    const channel = await channels.channels(ACCOUNT_BA_CHANNEL_ID)
    expect(channel.commitment).to.equal(SECRET_1)
  })

  it('should reedem ticket for account B', async function () {
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: OPEN })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: OPEN })
    await channels.connect(fixtures.accountB).redeemTicket(...redeemArgs(fixtures.TICKET_AB_WIN.ticket))

    const ab = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    const ba = await channels.channels(ACCOUNT_BA_CHANNEL_ID)
    validateChannel(ab, { balance: '60', status: OPEN })
    validateChannel(ba, { balance: '40', status: OPEN })
    expect(ab.commitment).to.equal(SECRET_1)
  })

  it('should fail to redeem ticket when ticket has been already redeemed', async function () {
    const TICKET_AB_WIN = fixtures.TICKET_AB_WIN

    await channels.connect(fixtures.accountB).redeemTicket(...redeemArgs(TICKET_AB_WIN.ticket))

    await expect(
      channels.connect(fixtures.accountB).redeemTicket(
        TICKET_AB_WIN.ticket.signer.toAddress().toHex(),
        SECRET_0, // give the next secret so this ticket becomes redeemable
        TICKET_AB_WIN.ticket.ticket.epoch.toHex(),
        TICKET_AB_WIN.ticket.ticket.index.toHex(),
        TICKET_AB_WIN.ticket.response.toHex(),
        TICKET_AB_WIN.ticket.ticket.amount.toHex(),
        TICKET_AB_WIN.ticket.ticket.winProb.toHex(),
        TICKET_AB_WIN.ticket.ticket.signature.serializeEthereum()
      )
    ).to.be.revertedWith('redemptions must be in order')

    await expect(
      channels.connect(fixtures.accountB).redeemTicket(
        TICKET_AB_WIN.ticket.signer.toAddress().toHex(),
        SECRET_0, // give the next secret so this ticket becomes redeemable
        UINT256.fromString(parseInt(TICKET_AB_WIN.ticketEpoch) + 1 + '').toHex(),
        TICKET_AB_WIN.ticket.ticket.index.toHex(),
        TICKET_AB_WIN.ticket.response.toHex(),
        TICKET_AB_WIN.ticket.ticket.amount.toHex(),
        TICKET_AB_WIN.ticket.ticket.winProb.toHex(),
        TICKET_AB_WIN.ticket.ticket.signature.serializeEthereum()
      )
    ).to.be.revertedWith('revert ticket epoch must match')
  })

  it('should fail to redeem ticket when signer is not the issuer', async function () {
    const TICKET_AB_WIN = fixtures.TICKET_AB_WIN
    const FAKE_SIGNATURE = await fixtures.accountA.signMessage(ethers.utils.id(CLOSED))
    await expect(
      channels
        .connect(fixtures.accountB)
        .redeemTicket(
          TICKET_AB_WIN.ticket.signer.toAddress().toHex(),
          TICKET_AB_WIN.ticket.preImage.toHex(),
          TICKET_AB_WIN.ticket.ticket.epoch.toHex(),
          TICKET_AB_WIN.ticket.ticket.index.toHex(),
          TICKET_AB_WIN.ticket.response.toHex(),
          TICKET_AB_WIN.ticket.ticket.amount.toHex(),
          TICKET_AB_WIN.ticket.ticket.winProb.toHex(),
          FAKE_SIGNATURE
        )
    ).to.be.revertedWith('signer must match the counterparty')
  })

  it("should fail to redeem ticket if it's a loss", async function () {
    const TICKET_AB_LOSS = await createTicket(
      {
        recipient: ACCOUNT_B.address,
        proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
        ticketEpoch: '1',
        ticketIndex: '1',
        amount: '10',
        winProb: WIN_PROB_0.toString(),
        channelEpoch: '1'
      },
      ACCOUNT_A,
      SECRET_1
    )
    await expect(
      channels.connect(fixtures.accountB).redeemTicket(...redeemArgs(TICKET_AB_LOSS.ticket))
    ).to.be.revertedWith('ticket must be a win')
  })

  it('A can initialize channel closure', async function () {
    await expect(channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_B.address)).to.emit(
      channels,
      'ChannelUpdate'
    )
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: '3' })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: OPEN })
  })

  it('B can initialize channel closure', async function () {
    await expect(channels.connect(fixtures.accountB).initiateChannelClosure(ACCOUNT_A.address)).to.emit(
      channels,
      'ChannelUpdate'
    )
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: OPEN })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: PENDING_CLOSURE })
  })

  it('should fail to initialize channel closure A->A', async function () {
    await expect(channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_A.address)).to.be.revertedWith(
      'source and destination must not be the same'
    )
  })

  it('should fail to initialize channel closure A->0', async function () {
    await expect(
      channels.connect(ACCOUNT_A.address).initiateChannelClosure(ethers.constants.AddressZero)
    ).to.be.revertedWith('destination must not be empty')
  })

  it('should fail to finalize channel closure when is not pending', async function () {
    await expect(channels.connect(ACCOUNT_A.address).finalizeChannelClosure(ACCOUNT_B.address)).to.be.revertedWith(
      'channel must be pending to close'
    )
  })
})

describe('with a pending_to_close HoprChannel (A:70, B:30)', function () {
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

    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '0', status: CLOSED })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: WAITING_FOR_COMMITMENT })
    expect((await token.balanceOf(ACCOUNT_A.address)).toString()).to.equal('70')
    expect((await token.balanceOf(ACCOUNT_B.address)).toString()).to.equal('0')
  })

  it('should fail to finalize channel closure', async function () {
    await expect(channels.connect(ACCOUNT_A.address).finalizeChannelClosure(ACCOUNT_A.address)).to.be.revertedWith(
      'source and destination must not be the same'
    )

    await expect(
      channels.connect(ACCOUNT_A.address).finalizeChannelClosure(ethers.constants.AddressZero)
    ).to.be.revertedWith('destination must not be empty')

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
      channels.connect(fixtures.accountB).redeemTicket(...redeemArgs(fixtures.TICKET_AB_WIN.ticket))
    ).to.be.revertedWith('channel must be open or pending to close')
  })

  it('should allow a fund to reopen channel', async function () {
    await fixtures.fundAndApprove(fixtures.accountA, 100)
    await channels.connect(fixtures.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: OPEN })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '60', status: WAITING_FOR_COMMITMENT }) // 30 + 30
  })
})

describe('with a reopened channel', function () {
  let channels, fixtures, TICKET_AB_WIN_RECYCLED, TICKET_BA_WIN_RECYCLED
  beforeEach(async function () {
    fixtures = await useFixtures()
    channels = fixtures.channels
    await fixtures.fundAndApprove(fixtures.accountA, 100)
    await channels.connect(fixtures.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    await channels.connect(fixtures.accountB).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await channels.connect(fixtures.accountA).bumpChannel(ACCOUNT_B.address, SECRET_2)
    await channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_B.address)
    await increaseTime(ethers.provider, ENOUGH_TIME_FOR_CLOSURE)
    await channels.connect(fixtures.accountA).finalizeChannelClosure(ACCOUNT_B.address)
    await fixtures.fundAndApprove(fixtures.accountA, 100)
    await channels.connect(fixtures.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    TICKET_AB_WIN_RECYCLED = await createTicket(
      {
        recipient: ACCOUNT_B.address,
        proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
        ticketIndex: '1',
        ticketEpoch: '1',
        amount: '10',
        winProb: WIN_PROB_100.toString(),
        channelEpoch: '2'
      },
      ACCOUNT_A,
      SECRET_1
    )
    TICKET_BA_WIN_RECYCLED = await createTicket(
      {
        recipient: ACCOUNT_A.address,
        proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
        ticketIndex: '1',
        ticketEpoch: '1',
        amount: '10',
        winProb: WIN_PROB_100.toString(),
        channelEpoch: '1'
      },
      ACCOUNT_B,
      SECRET_1
    )
  })

  it('sanity check', async function(){
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: OPEN })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '60', status: OPEN }) // 30 + 30
  })

  it('should fail to redeem ticket when channel in in different channelEpoch', async function () {
    await expect(
      channels.connect(fixtures.accountB).redeemTicket(...redeemArgs(fixtures.TICKET_AB_WIN.ticket))
    ).to.be.revertedWith('signer must match the counterparty')
  })

  it('should redeem ticket for account A', async function () {
    await channels.connect(fixtures.accountA).redeemTicket(...redeemArgs(TICKET_BA_WIN_RECYCLED.ticket))
    const ab = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    const ba = await channels.channels(ACCOUNT_BA_CHANNEL_ID)
    validateChannel(ab, { balance: '80', status: OPEN })
    validateChannel(ba, { balance: '50', status: OPEN })
    expect(ba.commitment).to.equal(SECRET_1)
  })

  it('should reedem ticket for account B', async function () {
    await channels.connect(fixtures.accountB).redeemTicket(...redeemArgs(TICKET_AB_WIN_RECYCLED.ticket))
    const ab = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(ab, { balance: '60', status: OPEN })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '70', status: OPEN }) // 30 + 30 + 10
    expect(ab.commitment).to.equal(SECRET_1)
  })

  it('should allow closure', async function () {
    await channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_B.address)
    await increaseTime(ethers.provider, ENOUGH_TIME_FOR_CLOSURE)
    await channels.connect(fixtures.accountA).finalizeChannelClosure(ACCOUNT_B.address)
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '0', status: CLOSED })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '60', status: OPEN })
  })
})

describe('test internals with mock', function () {
  let channels
  beforeEach(async function () {
    channels = (await useFixtures()).mockChannels
  })

  it('should get channel id', async function () {
    expect(await channels.getChannelIdInternal(ACCOUNT_A.address, ACCOUNT_B.address)).to.be.equal(ACCOUNT_AB_CHANNEL_ID)
  })

  it('should pack ticket', async function () {
    const { TICKET_AB_WIN } = await useFixtures()
    const encoded = await channels.getEncodedTicketInternal(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticket.response.toHex(),
      TICKET_AB_WIN.channelEpoch,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.winProb
    )

    expect(Hash.create(stringToU8a(encoded)).toHex()).to.equal(
      Hash.create(TICKET_AB_WIN.ticket.ticket.serializeUnsigned()).toHex()
    )
  })

  it('should correctly hash ticket', async function () {
    const { TICKET_AB_WIN } = await useFixtures()
    const ticketHash = await channels.getTicketHashInternal(
      TICKET_AB_WIN.recipient,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticket.response.toHex(),
      TICKET_AB_WIN.channelEpoch,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.winProb
    )

    expect(ticketHash).to.equal(TICKET_AB_WIN.ticket.ticket.getHash().toHex())
  })

  it("should get ticket's luck", async function () {
    const { TICKET_AB_WIN } = await useFixtures()

    const luck = await channels.getTicketLuckInternal(
      TICKET_AB_WIN.ticket.ticket.getHash().toHex(),
      TICKET_AB_WIN.ticket.preImage.toHex(),
      TICKET_AB_WIN.ticket.response.toHex()
    )
    expect(luck).to.equal(
      TICKET_AB_WIN.ticket.ticket.getLuck(TICKET_AB_WIN.ticket.preImage, TICKET_AB_WIN.ticket.response).toHex()
    )
  })

  it('should get the right challenge', async function () {
    const response = new Response(Uint8Array.from(randomBytes(Response.SIZE)))

    const challenge = await channels.computeChallengeInternal(response.toHex())

    expect(challenge.toLowerCase()).to.equal(response.toChallenge().toEthereumChallenge().toHex())
  })
})
