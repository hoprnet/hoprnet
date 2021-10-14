import type { Wallet } from '@ethersproject/wallet'
import type { SignerWithAddress } from '@nomiclabs/hardhat-ethers/signers'
import { deployments, ethers } from 'hardhat'
import { expect } from 'chai'
import BN from 'bn.js'
import {
  HoprToken__factory,
  ChannelsMock__factory,
  HoprChannels__factory,
  HoprChannels,
  HoprToken,
  ChannelsMock
} from '../types'
import { increaseTime } from './utils'
import { ACCOUNT_A, ACCOUNT_B } from './constants'
import {
  Address,
  Challenge,
  UINT256,
  Balance,
  stringToU8a,
  u8aAdd,
  u8aToHex,
  toU8a,
  Ticket,
  Hash,
  Response,
  AcknowledgedTicket,
  PublicKey,
  generateChannelId,
  ChannelStatus,
  PromiseValue
} from '@hoprnet/hopr-utils'
import { BigNumber } from 'ethers'

type TicketValues = {
  recipient: string
  proofOfRelaySecret: string
  amount: string
  winProb: string
  channelEpoch: string
  ticketIndex: string
  ticketEpoch: string
}

const percentToUint256 = (percent: any) => ethers.constants.MaxUint256.mul(percent).div(100)

export const redeemArgs = (ticket: AcknowledgedTicket): Parameters<HoprChannels['redeemTicket']> => [
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

type ChannelArrProps = [BigNumber, string, BigNumber, BigNumber, number, BigNumber, number]
type ChannelObjProps = {
  balance?: BigNumber
  commitment?: string
  ticketEpoch?: BigNumber
  ticketIndex?: BigNumber
  status?: number
  channelEpoch?: BigNumber
  closureTime?: number
}
type ChannelProps = ChannelArrProps & ChannelObjProps

const createMockChannelFromProps = (props: ChannelObjProps): ChannelProps => {
  const _emptyChannelProps = {
    balance: undefined,
    commitment: undefined,
    ticketEpoch: undefined,
    ticketIndex: undefined,
    status: undefined,
    channelEpoch: undefined,
    closureTime: undefined
  }
  const channel = []
  Object.keys(_emptyChannelProps).map((key, index) => {
    if (props[key] != undefined) {
      channel[index] = props[key]
      channel[key] = props[key]
    } else {
      channel[index] = _emptyChannelProps[key]
      channel[key] = _emptyChannelProps[key]
    }
  })
  return channel as ChannelProps
}

export const createMockChannelFromMerge = (channel: ChannelProps, props: ChannelProps) => {
  let newChannel = []
  Object.keys(channel).map((key) => (newChannel[key] = props[key] != undefined ? props[key] : channel[key]))
  return newChannel
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
const MULTI_ADDR = []

// recover the public key from the signer passed by ethers
// could not find a way to get the public key through the API
const recoverPublicKeyFromSigner = async (deployer: SignerWithAddress): Promise<PublicKey> => {
  const msg = 'hello'
  const sig = await deployer.signMessage(msg)
  const msgHash = ethers.utils.hashMessage(msg)
  const msgHashBytes = ethers.utils.arrayify(msgHash)
  return PublicKey.fromUncompressedPubKey(ethers.utils.arrayify(ethers.utils.recoverPublicKey(msgHashBytes, sig)))
}

const abiEncoder = ethers.utils.Interface.getAbiCoder()

const useFixtures = deployments.createFixture(
  async (_, ops: { skipAnnounceForAccountA: boolean; skipAnnounceForAccountB: boolean }) => {
    const [deployer] = await ethers.getSigners()
    const deployerPubKey = await recoverPublicKeyFromSigner(deployer)
    const accountA = new ethers.Wallet(ACCOUNT_A.privateKey).connect(ethers.provider)
    const accountAPubKey = PublicKey.fromPrivKey(ethers.utils.arrayify(accountA.privateKey))
    const accountB = new ethers.Wallet(ACCOUNT_B.privateKey).connect(ethers.provider)
    const accountBPubKey = PublicKey.fromPrivKey(ethers.utils.arrayify(accountB.privateKey))

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
        ticketEpoch: '0',
        ticketIndex: '1',
        amount: '10',
        winProb: WIN_PROB_100.toString(),
        channelEpoch: '1'
      },
      ACCOUNT_A,
      SECRET_1
    )

    await fundEther(accountA.address, ethers.utils.parseEther(ChannelStatus.Open + ''))
    await fundEther(accountB.address, ethers.utils.parseEther(ChannelStatus.Open + ''))

    // announce
    if (!ops?.skipAnnounceForAccountA) {
      await channels.connect(accountA).announce(ethers.utils.arrayify(accountAPubKey.toUncompressedPubKeyHex()), [])
    }
    if (!ops?.skipAnnounceForAccountB) {
      await channels.connect(accountB).announce(ethers.utils.arrayify(accountBPubKey.toUncompressedPubKeyHex()), [])
    }

    return {
      token,
      channels,
      deployer,
      deployerPubKey,
      accountA,
      accountAPubKey,
      accountB,
      accountBPubKey,
      fund,
      approve,
      mockChannels,
      fundAndApprove,
      TICKET_AB_WIN
    }
  }
)

describe('announce user', function () {
  it('should announce user', async function () {
    const { channels, deployer, deployerPubKey } = await useFixtures()

    await expect(channels.connect(deployer).announce(deployerPubKey.toUncompressedPubKeyHex(), MULTI_ADDR))
      .to.emit(channels, 'Announcement')
      .withArgs(deployer.address, deployerPubKey.toUncompressedPubKeyHex(), MULTI_ADDR)
  })

  it('should fail to announce user', async function () {
    const { channels, deployer, accountA } = await useFixtures()

    await expect(
      channels
        .connect(deployer)
        .announce(
          PublicKey.fromPrivKey(ethers.utils.arrayify(accountA.privateKey)).toUncompressedPubKeyHex(),
          MULTI_ADDR
        )
    ).to.be.revertedWith("publicKey's address does not match senders")
  })
})

describe('funding HoprChannel without announcements', function () {
  it('should fail to fund without accountA announcement', async function () {
    const { accountA, channels, fundAndApprove } = await useFixtures({
      skipAnnounceForAccountA: true,
      skipAnnounceForAccountB: false
    })

    await fundAndApprove(accountA, 100)
    await expect(
      channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    ).to.be.revertedWith('source has not announced')
  })

  it('should fail to fund without accountB announcement', async function () {
    const { accountA, channels, fundAndApprove } = await useFixtures({
      skipAnnounceForAccountA: false,
      skipAnnounceForAccountB: true
    })

    await fundAndApprove(accountA, 100)
    await expect(
      channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    ).to.be.revertedWith('destination has not announced')
  })
})

describe('funding HoprChannel catches failures', function () {
  let fixtures: PromiseValue<ReturnType<typeof useFixtures>>, channels: HoprChannels, accountA: Wallet
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
      channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '0', '0')
    ).to.be.revertedWith('amount must be greater than 0')
  })
})

describe('funding a HoprChannel success', function () {
  // TODO events
  // NB: Adding withArgs will break test, see https://github.com/EthWorks/Waffle/issues/537
  it('should multi fund and open channel A->B, no commitment', async function () {
    const { channels, accountA, fundAndApprove, token } = await useFixtures()
    await fundAndApprove(accountA, 100)
    await expect(channels.connect(accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30'))
      .to.emit(channels, 'ChannelUpdated')
      .and.to.emit(channels, 'ChannelFunded')
      .and.not.to.emit(channels, 'ChannelOpened')
    const ab = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    const ba = await channels.channels(ACCOUNT_BA_CHANNEL_ID)
    validateChannel(ab, { balance: '70', status: ChannelStatus.WaitingForCommitment + '' })
    validateChannel(ba, { balance: '30', status: ChannelStatus.WaitingForCommitment + '' })
    const accountABalance = await token.balanceOf(ACCOUNT_A.address)
    expect(accountABalance.toString()).to.equal(ChannelStatus.Closed + '')
  })

  it('should multi fund and open channel B->A, no commitment', async function () {
    const { channels, accountB, fundAndApprove } = await useFixtures()
    await fundAndApprove(accountB, 100)
    await expect(channels.connect(accountB).fundChannelMulti(ACCOUNT_B.address, ACCOUNT_A.address, '30', '70'))
      .to.emit(channels, 'ChannelUpdated')
      .and.to.emit(channels, 'ChannelFunded')
      .and.not.to.emit(channels, 'ChannelOpened')
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), {
      balance: '70',
      status: ChannelStatus.WaitingForCommitment + ''
    })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), {
      balance: '30',
      status: ChannelStatus.WaitingForCommitment + ''
    })
  })

  it('should multi fund and open channel B->A, commit afterwards', async function () {
    const { channels, accountA, accountB, fundAndApprove } = await useFixtures()
    await fundAndApprove(accountB, 100)
    await expect(channels.connect(accountB).fundChannelMulti(ACCOUNT_B.address, ACCOUNT_A.address, '30', '70'))
      .to.emit(channels, 'ChannelUpdated')
      .and.to.emit(channels, 'ChannelFunded')
      .and.not.to.emit(channels, 'ChannelOpened')
    await channels.connect(accountA).bumpChannel(ACCOUNT_B.address, SECRET_2)
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), {
      balance: '70',
      status: ChannelStatus.WaitingForCommitment + ''
    })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: ChannelStatus.Open + '' })
  })

  it('should multi fund and open channel B->A, pre-commitment', async function () {
    const { channels, accountA, accountB, fundAndApprove } = await useFixtures()
    await fundAndApprove(accountB, 100)
    await channels.connect(accountA).bumpChannel(ACCOUNT_B.address, SECRET_2)
    await expect(channels.connect(accountB).fundChannelMulti(ACCOUNT_B.address, ACCOUNT_A.address, '30', '70'))
      .to.emit(channels, 'ChannelUpdated')
      .and.to.emit(channels, 'ChannelFunded')
      .and.to.emit(channels, 'ChannelOpened')
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), {
      balance: '70',
      status: ChannelStatus.WaitingForCommitment + ''
    })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: ChannelStatus.Open + '' })
  })

  it('should multi fund and open channel B->A, commit both', async function () {
    const { channels, accountA, accountB, fundAndApprove } = await useFixtures()
    await fundAndApprove(accountB, 100)
    await channels.connect(accountA).bumpChannel(ACCOUNT_B.address, SECRET_2)
    await channels.connect(accountB).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await expect(channels.connect(accountB).fundChannelMulti(ACCOUNT_B.address, ACCOUNT_A.address, '30', '70'))
      .to.emit(channels, 'ChannelUpdated')
      .and.to.emit(channels, 'ChannelFunded')
      .and.to.emit(channels, 'ChannelOpened')
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: ChannelStatus.Open + '' })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: ChannelStatus.Open + '' })
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
            [ACCOUNT_B.address, ACCOUNT_A.address, '30', ChannelStatus.Closed + '']
          )
        )
    )
      .to.emit(channels, 'ChannelUpdated')
      .withArgs(
        ACCOUNT_B.address,
        ACCOUNT_A.address,
        createMockChannelFromMerge(
          await channels.channels(ACCOUNT_BA_CHANNEL_ID),
          createMockChannelFromProps({
            balance: BigNumber.from(30),
            status: ChannelStatus.WaitingForCommitment,
            channelEpoch: BigNumber.from(1)
          })
        )
      )
      .and.to.emit(channels, 'ChannelFunded')
      .withArgs(accountB.address, ACCOUNT_B.address, ACCOUNT_A.address, '30')
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), {
      balance: ChannelStatus.Closed + '',
      status: ChannelStatus.Closed + ''
    })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), {
      balance: '30',
      status: ChannelStatus.WaitingForCommitment + ''
    })
  })
})

describe('with single funded HoprChannels: AB: 70', function () {
  let channels: HoprChannels
  let fixtures: PromiseValue<ReturnType<typeof useFixtures>>

  beforeEach(async function () {
    fixtures = await useFixtures()
    channels = fixtures.channels
    await fixtures.fundAndApprove(fixtures.accountA, 100)
    await channels.connect(fixtures.accountB).bumpChannel(ACCOUNT_A.address, SECRET_2)
    await channels.connect(fixtures.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '100', '0')
  })

  it('should reedem ticket for account B -> directly to wallet', async function () {
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '100', status: ChannelStatus.Open + '' })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '0', status: ChannelStatus.Closed + '' })
    await channels.connect(fixtures.accountB).redeemTicket(...redeemArgs(fixtures.TICKET_AB_WIN.ticket))
    const ab = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    const ba = await channels.channels(ACCOUNT_BA_CHANNEL_ID)
    validateChannel(ab, { balance: '90', status: ChannelStatus.Open + '' })
    validateChannel(ba, { balance: '0', status: ChannelStatus.Closed + '' })
    expect(ab.commitment).to.equal(SECRET_1)
    expect((await fixtures.token.balanceOf(ACCOUNT_A.address)).toString()).to.equal('0')
    expect((await fixtures.token.balanceOf(ACCOUNT_B.address)).toString()).to.equal('10')
  })
})

describe('with funded HoprChannels: AB: 70, BA: 30, secrets initialized', function () {
  let channels: HoprChannels
  let fixtures: PromiseValue<ReturnType<typeof useFixtures>>
  let blockTimestamp: number

  beforeEach(async function () {
    fixtures = await useFixtures()
    channels = fixtures.channels
    await fixtures.fundAndApprove(fixtures.accountA, 100)
    await channels.connect(fixtures.accountA).bumpChannel(ACCOUNT_B.address, SECRET_2)
    await channels.connect(fixtures.accountB).bumpChannel(ACCOUNT_A.address, SECRET_2) // TODO secret per account
    await channels.connect(fixtures.accountA).fundChannelMulti(ACCOUNT_A.address, ACCOUNT_B.address, '70', '30')
    blockTimestamp = (await ethers.provider.getBlock('latest')).timestamp
  })

  it('should redeem ticket for account A', async function () {
    const TICKET_BA_WIN = await createTicket(
      {
        recipient: ACCOUNT_A.address,
        proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
        ticketEpoch: '0',
        ticketIndex: '1',
        amount: '10',
        winProb: WIN_PROB_100.toString(),
        channelEpoch: '1'
      },
      ACCOUNT_B,
      SECRET_1
    )

    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: ChannelStatus.Open + '' })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: ChannelStatus.Open + '' })

    await expect(await channels.connect(fixtures.accountA).redeemTicket(...redeemArgs(TICKET_BA_WIN.ticket)))
      .to.emit(channels, 'ChannelUpdated')
      .and.to.emit(channels, 'TicketRedeemed')

    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '80', status: ChannelStatus.Open + '' })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '20', status: ChannelStatus.Open + '' })
    const channel = await channels.channels(ACCOUNT_BA_CHANNEL_ID)
    expect(channel.commitment).to.equal(SECRET_1)
  })

  it('should reedem ticket for account B', async function () {
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: ChannelStatus.Open + '' })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: ChannelStatus.Open + '' })
    await channels.connect(fixtures.accountB).redeemTicket(...redeemArgs(fixtures.TICKET_AB_WIN.ticket))

    const ab = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    const ba = await channels.channels(ACCOUNT_BA_CHANNEL_ID)
    validateChannel(ab, { balance: '60', status: ChannelStatus.Open + '' })
    validateChannel(ba, { balance: '40', status: ChannelStatus.Open + '' })
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
    ).to.be.revertedWith('ticket epoch must match')
  })

  it('should fail to redeem ticket when signer is not the issuer', async function () {
    const TICKET_AB_WIN = fixtures.TICKET_AB_WIN
    const FAKE_SIGNATURE = await fixtures.accountA.signMessage(ethers.utils.id(ChannelStatus.Closed + ''))
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
        ticketEpoch: '0',
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
    await expect(channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_B.address))
      .to.emit(channels, 'ChannelUpdated')
      .withArgs(
        fixtures.accountA.address,
        ACCOUNT_B.address,
        createMockChannelFromMerge(
          await channels.channels(ACCOUNT_AB_CHANNEL_ID),
          createMockChannelFromProps({
            status: ChannelStatus.PendingToClose,
            closureTime:
              blockTimestamp + // Block timestamp
              (await channels.secsClosure()) + // Contract secs
              1 // Tick
          })
        )
      )
      .and.to.emit(channels, 'ChannelClosureInitiated')
      .withArgs(fixtures.accountA.address, ACCOUNT_B.address, blockTimestamp + 1)
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), {
      balance: '70',
      status: ChannelStatus.PendingToClose + ''
    })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '30', status: ChannelStatus.Open + '' })
  })

  it('B can initialize channel closure', async function () {
    await expect(channels.connect(fixtures.accountB).initiateChannelClosure(ACCOUNT_A.address))
      .to.emit(channels, 'ChannelUpdated')
      .withArgs(
        fixtures.accountB.address,
        ACCOUNT_A.address,
        createMockChannelFromMerge(
          await channels.channels(ACCOUNT_BA_CHANNEL_ID),
          createMockChannelFromProps({
            status: ChannelStatus.PendingToClose,
            closureTime:
              blockTimestamp + // Block timestamp
              (await channels.secsClosure()) + // Contract secs
              1
          })
        )
      )
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: ChannelStatus.Open + '' })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), {
      balance: '30',
      status: ChannelStatus.PendingToClose + ''
    })
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
  let channels: HoprChannels
  let fixtures: PromiseValue<ReturnType<typeof useFixtures>>
  let token: HoprToken

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
    const channel = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    await increaseTime(ethers.provider, ENOUGH_TIME_FOR_CLOSURE)
    await expect(channels.connect(fixtures.accountA).finalizeChannelClosure(ACCOUNT_B.address))
      .to.emit(channels, 'ChannelUpdated')
      .withArgs(
        fixtures.accountA.address,
        fixtures.accountB.address,
        createMockChannelFromMerge(
          channel,
          createMockChannelFromProps({
            balance: BigNumber.from(0),
            closureTime: 0,
            status: 0
          })
        )
      )
      .and.to.emit(channels, 'ChannelClosureFinalized')
      .withArgs(fixtures.accountA.address, fixtures.accountB.address, channel.closureTime, channel.balance)
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '0', status: ChannelStatus.Closed + '' })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), {
      balance: '30',
      status: ChannelStatus.WaitingForCommitment + ''
    })
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
  let channels: HoprChannels
  let fixtures: PromiseValue<ReturnType<typeof useFixtures>>

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
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: ChannelStatus.Open + '' })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), {
      balance: '60',
      status: ChannelStatus.WaitingForCommitment + ''
    }) // 30 + 30
  })
})

describe('with a reopened channel', function () {
  let channels: HoprChannels
  let fixtures: PromiseValue<ReturnType<typeof useFixtures>>
  let TICKET_AB_WIN_RECYCLED: PromiseValue<ReturnType<typeof createTicket>>

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
        ticketEpoch: '0',
        amount: '10',
        winProb: WIN_PROB_100.toString(),
        channelEpoch: '2'
      },
      ACCOUNT_A,
      SECRET_1
    )
  })

  it('sanity check', async function () {
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '70', status: ChannelStatus.Open + '' })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '60', status: ChannelStatus.Open + '' }) // 30 + 30
  })

  it('should fail to redeem ticket when channel in in different channelEpoch', async function () {
    await expect(
      channels.connect(fixtures.accountB).redeemTicket(...redeemArgs(fixtures.TICKET_AB_WIN.ticket))
    ).to.be.revertedWith('signer must match the counterparty')
  })

  it('should reedem ticket for account B', async function () {
    await channels.connect(fixtures.accountB).redeemTicket(...redeemArgs(TICKET_AB_WIN_RECYCLED.ticket))
    const ab = await channels.channels(ACCOUNT_AB_CHANNEL_ID)
    validateChannel(ab, { balance: '60', status: ChannelStatus.Open + '' })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '70', status: ChannelStatus.Open + '' }) // 30 + 30 + 10
    expect(ab.commitment).to.equal(SECRET_1)
  })

  it('should allow closure', async function () {
    await channels.connect(fixtures.accountA).initiateChannelClosure(ACCOUNT_B.address)
    await increaseTime(ethers.provider, ENOUGH_TIME_FOR_CLOSURE)
    await channels.connect(fixtures.accountA).finalizeChannelClosure(ACCOUNT_B.address)
    validateChannel(await channels.channels(ACCOUNT_AB_CHANNEL_ID), { balance: '0', status: ChannelStatus.Closed + '' })
    validateChannel(await channels.channels(ACCOUNT_BA_CHANNEL_ID), { balance: '60', status: ChannelStatus.Open + '' })
  })
})

describe('test internals with mock', function () {
  let channels: ChannelsMock

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
    const response = Response.createMock()

    const challenge = await channels.computeChallengeInternal(response.toHex())

    expect(challenge.toLowerCase()).to.equal(response.toChallenge().toEthereumChallenge().toHex())
  })

  it('should get the right challenge - edge cases', async function () {
    const FIELD_ORDER = stringToU8a('0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141')
    const INVALID_FIELD_ELEMENT_GREATER_THAN_FIELD_ORDER = u8aAdd(false, FIELD_ORDER, toU8a(1, 32))

    await expect(channels.computeChallengeInternal(toU8a(0, 32))).to.be.revertedWith(
      'Invalid response. Value must be within the field'
    )

    await expect(channels.computeChallengeInternal(u8aToHex(FIELD_ORDER))).to.be.revertedWith(
      'Invalid response. Value must be within the field'
    )

    await expect(
      channels.computeChallengeInternal(u8aToHex(INVALID_FIELD_ELEMENT_GREATER_THAN_FIELD_ORDER))
    ).to.be.revertedWith('Invalid response. Value must be within the field')
  })
})
