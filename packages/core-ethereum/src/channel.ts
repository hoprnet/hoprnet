import type { Channel as IChannel, Indexer } from '@hoprnet/hopr-core-connector-interface'
import type Connector from '.'
import BN from 'bn.js'
import { Public, Balance, Hash, UINT256, Ticket, SignedTicket, AcknowledgedTicket } from './types'
import { getId, waitForConfirmation, computeWinningProbability, Log, checkChallenge, isWinningTicket } from './utils'

const log = Log(['channel'])
const EMPTY_PRE_IMAGE = new Hash(new Uint8Array(Hash.SIZE).fill(0x00))

class Channel implements IChannel {
  constructor(
    private readonly indexer: Indexer,
    private readonly connector: Connector, // TODO: remove this
    private readonly self: Public,
    public readonly counterparty: Public
  ) {}

  async getState() {
    const channelId = await getId(await this.self.toAddress(), await this.counterparty.toAddress())
    return this.indexer.getChannel(channelId)
  }

  async getBalances() {
    const state = await this.getState()
    const { partyA, partyB } = state.getBalances()
    const [self, counterparty] = state.partyA.eq(await this.self.toAddress()) ? [partyA, partyB] : [partyB, partyA]

    return {
      self,
      counterparty
    }
  }

  async open(fundAmount: Balance) {
    const state = await this.getState()

    if (state.getStatus() !== 'CLOSED') {
      throw Error('Channel is already opened')
    }

    const myAddress = await this.self.toAddress()
    const myBalance = await this.connector.hoprToken.methods.balanceOf(myAddress.toHex()).call()
    if (new BN(myBalance).lt(fundAmount.toBN())) {
      throw Error('We do not have enough balance to open a channel')
    }

    try {
      await waitForConfirmation(
        (
          await this.connector.account.signTransaction(
            {
              from: myAddress.toHex(),
              to: this.connector.hoprToken.options.address
            },
            this.connector.hoprToken.methods.send(
              this.connector.hoprChannels.options.address,
              fundAmount.toBN().toString(),
              this.connector.web3.eth.abi.encodeParameters(
                ['bool', 'address', 'address'],
                [true, myAddress, (await this.counterparty.toAddress()).toHex()]
              )
            )
          )
        ).send()
      )
    } catch (err) {
      // TODO: catch race-condition
      console.log(err)
      throw Error(`Failed to open channel`)
    }
  }

  async initializeClosure() {
    const state = await this.getState()
    const myAddress = await this.self.toAddress()
    const counterpartyAddress = await this.counterparty.toAddress()

    if (state.getStatus() !== 'OPEN') {
      throw Error('Channel status is not OPEN')
    }

    try {
      await waitForConfirmation(
        (
          await this.connector.account.signTransaction(
            {
              from: myAddress.toHex(),
              to: this.connector.hoprChannels.options.address
            },
            this.connector.hoprChannels.methods.initiateChannelClosure(counterpartyAddress.toHex())
          )
        ).send()
      )
    } catch (err) {
      // TODO: catch race-condition
      console.log(err)
      throw Error(`Failed to initialize channel closure`)
    }
  }

  async finalizeClosure() {
    const state = await this.getState()
    const myAddress = await this.self.toAddress()
    const counterpartyAddress = await this.counterparty.toAddress()

    if (state.getStatus() !== 'PENDING_TO_CLOSE') {
      throw Error('Channel status is not PENDING_TO_CLOSE')
    }

    try {
      await waitForConfirmation(
        (
          await this.connector.account.signTransaction(
            {
              from: myAddress.toHex(),
              to: this.connector.hoprChannels.options.address
            },
            this.connector.hoprChannels.methods.finalizeChannelClosure(counterpartyAddress.toHex())
          )
        ).send()
      )
    } catch (err) {
      // TODO: catch race-condition
      console.log(err)
      throw Error(`Failed to finilize channel closure`)
    }
  }

  async createTicket(amount: Balance, challenge: Hash, winProb: number) {
    const ticketWinProb = computeWinningProbability(winProb)
    const channelState = await this.getState()
    const counterpartyAddress = await this.counterparty.toAddress()
    const counterpartyState = await this.indexer.getAccount(counterpartyAddress)
    const channelIteration = new UINT256(channelState.getIteration())

    const ticket = new Ticket(undefined, {
      counterparty: counterpartyAddress,
      challenge,
      epoch: new UINT256(counterpartyState!.counter),
      amount,
      winProb: ticketWinProb,
      channelIteration
    })

    // TODO: simplify
    const signature = await ticket.sign(this.connector.account.keys.onChain.privKey)

    return new SignedTicket(undefined, {
      signature,
      ticket
    })
  }

  async createDummyTicket(challenge: Hash): Promise<SignedTicket> {
    const ticketWinProb = computeWinningProbability(1) // Value is unimportant here.
    const counterpartyAddress = await this.counterparty.toAddress()

    const ticket = new Ticket(undefined, {
      counterparty: counterpartyAddress,
      challenge,
      epoch: UINT256.fromString('0'),
      amount: new Balance(new BN(0)),
      winProb: ticketWinProb,
      channelIteration: UINT256.fromString('0')
    })

    // TODO: simplify
    const signature = await ticket.sign(this.connector.account.keys.onChain.privKey)

    return new SignedTicket(undefined, {
      signature,
      ticket
    })
  }

  // async verifyTicket(signedTicket: SignedTicket) {
  //   try {
  //     await this.connector.channel.testAndSetNonce(signedTicket)
  //   } catch {
  //     return false
  //   }

  //   return await signedTicket.verify(await this.channel.offChainCounterparty)
  // }

  async submitTicket(ackTicket: AcknowledgedTicket): ReturnType<IChannel['submitTicket']> {
    try {
      const signedTicket = await ackTicket.signedTicket
      const ticket = signedTicket.ticket

      log('Submitting ticket', ackTicket.response.toHex())
      const { hoprChannels, account, utils } = this.connector
      const { r, s, v } = utils.getSignatureParameters(signedTicket.signature)

      const hasPreImage = !ackTicket.preImage.eq(EMPTY_PRE_IMAGE)
      if (!hasPreImage) {
        log(`Failed to submit ticket ${ackTicket.response.toHex()}: 'PreImage is empty.'`)
        return {
          status: 'FAILURE',
          message: 'PreImage is empty.'
        }
      }

      const validChallenge = await checkChallenge(ticket.challenge, ackTicket.response)
      if (!validChallenge) {
        log(`Failed to submit ticket ${ackTicket.response.toHex()}: 'Invalid challenge.'`)
        return {
          status: 'FAILURE',
          message: 'Invalid challenge.'
        }
      }

      const isWinning = await isWinningTicket(await ticket.hash, ackTicket.response, ackTicket.preImage, ticket.winProb)
      if (!isWinning) {
        log(`Failed to submit ticket ${ackTicket.response.toHex()}:  'Not a winning ticket.'`)
        return {
          status: 'FAILURE',
          message: 'Not a winning ticket.'
        }
      }

      const counterparty = await this.connector.utils.pubKeyToAddress(await signedTicket.signer)
      console.log('>>>>', ackTicket.preImage.toHex(), ackTicket.response.toHex())

      const transaction = await account.signTransaction(
        {
          from: (await account.address).toHex(),
          to: hoprChannels.options.address
        },
        hoprChannels.methods.redeemTicket(
          counterparty.toHex(),
          ackTicket.preImage.toHex(),
          ackTicket.response.toHex(),
          ticket.amount.toBN().toString(),
          ticket.winProb.toHex(),
          r.toHex(),
          s.toHex(),
          v + 27
        )
      )

      await transaction.send()
      ackTicket.redeemed = true
      this.connector.account.updateLocalState(ackTicket.preImage)

      log('Successfully submitted ticket', ackTicket.response.toHex())
      return {
        status: 'SUCCESS',
        receipt: transaction.transactionHash,
        ackTicket
      }
    } catch (err) {
      log('Unexpected error when submitting ticket', ackTicket.response.toHex(), err)
      return {
        status: 'ERROR',
        error: err
      }
    }
  }
}

export default Channel
