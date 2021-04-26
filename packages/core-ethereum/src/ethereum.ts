import type { providers as IProviders, ContractTransaction } from 'ethers'
import ethers, { errors } from 'ethers'
import type { Address } from './types'
import type { HoprToken, HoprChannels } from './contracts'
import BN from 'bn.js'
import { Balance, NativeBalance, Hash } from './types'
import { durations} from '@hoprnet/hopr-utils'
import NonceTracker from './nonce-tracker'
import TransactionManager from './transaction-manager'
import { getNetworkGasPrice } from './utils'
import Debug from 'debug'
import { Networks } from '@hoprnet/hopr-ethereum'

const log = Debug('hopr:core-ethereum:chain-operations')
const abiCoder = new ethers.utils.AbiCoder()

export type Receipt = string

export function createChainWrapper(provider: IProviders.WebSocketProvider, token: HoprToken, channels: HoprChannels, network: Networks, address: Address) {

  const transactions = new TransactionManager()
  const nonceTracker = new NonceTracker({
    getLatestBlockNumber: async () => provider.getBlockNumber(),
    getTransactionCount: (address, blockNumber) => provider.getTransactionCount(address.toHex(), blockNumber),
    getConfirmedTransactions: (_addr) => Array.from(transactions.confirmed.values()),
    getPendingTransactions: (_addr) => Array.from(transactions.pending.values()),

  }, durations.minutes(15))

  async function sendTransaction<T extends (...args: any) => Promise<ContractTransaction>>(
    method: T,
    ...rest: Parameters<T>
  ): Promise<ContractTransaction> {
    const gasLimit = 300e3
    const gasPrice = getNetworkGasPrice(network)
    const nonceLock = await nonceTracker.getNonceLock(address)
    const nonce = nonceLock.nextNonce
    let transaction: ContractTransaction

    log('Sending transaction %o', {
      gasLimit,
      gasPrice,
      nonce
    })

    try {
      // send transaction to our ethereum provider
      // TODO: better type this, make it less hacky
      transaction = await method(
        ...[
          ...rest,
          {
            gasLimit,
            gasPrice,
            nonce
          }
        ]
      )
    } catch (error) {
      log('Transaction with nonce %d failed to sent: %s', nonce, error)
      nonceLock.releaseLock()
      throw Error('Could not send transaction')
    }

    log('Transaction with nonce %d successfully sent %s, waiting for confimation', nonce, transaction.hash)
    this._transactions.addToPending(transaction.hash, { nonce })
    nonceLock.releaseLock()

    // monitor transaction, this is done asynchronously
    transaction
      .wait()
      .then(() => {
        log('Transaction with nonce %d and hash %s confirmed', nonce, transaction.hash)
        this._transactions.moveToConfirmed(transaction.hash)
      })
      .catch((error) => {
        const reverted = ([errors.CALL_EXCEPTION] as string[]).includes(error)

        if (reverted) {
          log('Transaction with nonce %d and hash %s reverted: %s', nonce, transaction.hash, error)

          // this transaction failed but was confirmed as reverted
          this._transactions.moveToConfirmed(transaction.hash)
        } else {
          log('Transaction with nonce %d failed to sent: %s', nonce, error)

          const alreadyKnown = ([errors.NONCE_EXPIRED, errors.REPLACEMENT_UNDERPRICED] as string[]).includes(error)
          // if this hash is already known and we already have it included in
          // pending we can safely ignore this
          if (alreadyKnown && this._transactions.pending.has(transaction.hash)) return

          // this transaction was not confirmed so we just remove it
          this._transactions.remove(transaction.hash)
        }
      })

    return transaction
  }

  async function withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    if (currency === 'NATIVE') {
      const nonceLock = await this.account.getNonceLock()
      try {
        const transaction = await this.account.wallet.sendTransaction({
          to: recipient,
          value: ethers.BigNumber.from(amount),
          nonce: ethers.BigNumber.from(nonceLock.nextNonce)
        })
        nonceLock.releaseLock()
        return transaction.hash
      } catch (err) {
        nonceLock.releaseLock()
        throw err
      }
    } else {
      const transaction = await this.account.sendTransaction(this.hoprToken.transfer, recipient, amount)
      return transaction.hash
    }
  }

  async function fundChannel(
    token: HoprToken,
    channels: HoprChannels,
    me: Address,
    counterparty: Address,
    myFund: Balance,
    counterpartyFund: Balance
  ): Promise<Receipt> {
    const totalFund = myFund.toBN().add(counterpartyFund.toBN())
    const transaction = await sendTransaction(
      token.send,
      channels.address,
      totalFund.toString(),
      abiCoder.encode(
        ['address', 'address', 'uint256', 'uint256'],
        [me.toHex(), counterparty.toHex(), myFund.toBN().toString(), counterpartyFund.toBN().toString()]
      )
    )
    await transaction.wait()
    return transaction.hash
  }

  async function openChannel(token: HoprToken, channels: HoprChannels, me: Address, counterparty: Address, amount: Balance): Promise<Receipt> {
    const transaction = await sendTransaction(
      token.send,
      channels.address,
      amount.toBN().toString(),
      abiCoder.encode(
        ['address', 'address', 'uint256', 'uint256'],
        [me.toHex(), counterparty.toHex(), amount.toBN().toString(), '0']
      )
    )
    await transaction.wait()
    return transaction.hash
  }

  async function finalizeChannelClosure(channels: HoprChannels, counterparty: Address): Promise<Receipt> {
    const transaction = await sendTransaction(
      channels.finalizeChannelClosure,
      counterparty.toHex()
    )
    await transaction.wait()
    return transaction.hash
    // TODO: catch race-condition
  }

  async function initiateChannelClosure(channels: HoprChannels, counterparty: Address): Promise<Receipt> {
    const transaction = await sendTransaction(channels.initiateChannelClosure, counterparty.toHex())
    await transaction.wait()
    return transaction.hash
    // TODO: catch race-condition
  }

  async function redeemTicket(hoprChannels, counterparty, ackTicket, ticket): Promise<Receipt> {
    const transaction = await sendTransaction(
      hoprChannels.redeemTicket,
      counterparty.toHex(),
      ackTicket.preImage.toHex(),
      ackTicket.ticket.epoch.serialize(),
      ackTicket.ticket.index.serialize(),
      ackTicket.response.toHex(),
      ticket.amount.toBN().toString(),
      ticket.winProb.toHex(),
      ticket.signature.serialize()
    )
    await transaction.wait()
    return transaction.hash
  }

  async function setCommitment(channels: HoprChannels, commitment: Hash): Promise<Receipt> {
    const transaction = await sendTransaction(
      channels.bumpCommitment,
      commitment.toHex()
    )
    await transaction.wait()
    return transaction.hash
  }

  const api = {
    // TODO: use indexer when it's done syncing
    getBalance: (address: Address) =>
      token.balanceOf(address.toHex()).then((res) => new Balance(new BN(res.toString()))),
    getNativeBalance: (address) =>
      provider.getBalance(address.toHex()).then((res) => new NativeBalance(new BN(res.toString()))),
    withdraw: (currency: 'NATIVE' | 'HOPR', recipient: string, amount: string) => withdraw(currency, recipient, amount),
    fundChannel: (me: Address, counterparty: Address, myTotal: Balance, theirTotal: Balance) =>
      fundChannel(token, channels, me, counterparty, myTotal, theirTotal),
    openChannel: (me, counterparty, amount) => openChannel(token, channels, me, counterparty, amount),
    finalizeChannelClosure: (counterparty) => finalizeChannelClosure(channels, counterparty),
    initiateChannelClosure: (counterparty) => initiateChannelClosure(channels, counterparty),
    redeemTicket: (counterparty, ackTicket, ticket) => redeemTicket(channels, counterparty, ackTicket, ticket),
    setCommitment: (comm: Hash) => setCommitment(channels, comm)
  }


  return api
}

export type ChainWrapper = ReturnType<typeof createChainWrapper>

