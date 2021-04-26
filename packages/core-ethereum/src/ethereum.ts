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

const log = Debug('hopr:core-ethereum:chain-operations')
const abiCoder = new ethers.utils.AbiCoder()

export function createChainWrapper(provider: IProviders.WebSocketProvider, token: HoprToken, channels: HoprChannels) {

  const api = {
    // TODO: use indexer when it's done syncing
    getLatestBlockNumber: async () => provider.getBlockNumber(),
    getTransactionCount: (address, blockNumber) => provider.getTransactionCount(address.toHex(), blockNumber),
    getBalance: (address: Address) => token.balanceOf(address.toHex()).then((res) => new Balance(new BN(res.toString()))),
    getNativeBalance: (address) =>
      provider.getBalance(address.toHex()).then((res) => new NativeBalance(new BN(res.toString()))),
    getConfirmedTransactions: (_addr) => Array.from(transactions.confirmed.values()),
    getPendingTransactions: (_addr) => Array.from(transactions.pending.values()),
    getNonceLock: (addr) => nonceTracker.getNonceLock(addr),
    withdraw: (currency: 'NATIVE' | 'HOPR', recipient: string, amount: string) => withdraw(currency, recipient, amount),
    fundChannel: (me: Address, counterparty: Address, myTotal: Balance, theirTotal: Balance) => fundChannel(token, channels, me, counterparty, myTotal, theirTotal),
    openChannel: (me, counterparty, amount) => openChannel(token, channels, me, counterparty, amount),
    finalizeChannelClosure: (counterparty) => finalizeChannelClosure(channels, counterparty),
    initiateChannelClosure: (counterparty) => initiateChannelClosure(channels, counterparty),
    redeemTicket: (counterparty, ackTicket, ticket) => redeemTicket(channels, counterparty, ackTicket, ticket),
    setCommitment: (comm: Hash) => setCommitment(channels, comm)
  }

  const nonceTracker = new NonceTracker(api, durations.minutes(15))
  const transactions = new TransactionManager()

  return api 
}

export type ChainWrapper = ReturnType<typeof createChainWrapper>

async function sendTransaction<T extends (...args: any) => Promise<ContractTransaction>>(
  method: T,
  ...rest: Parameters<T>
): Promise<ContractTransaction> {
  const gasLimit = 300e3
  const gasPrice = getNetworkGasPrice(this.network)
  const nonceLock = await this.api.getNonceLock(this.getAddress())
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


async function fundChannel(token: HoprToken, channels: HoprChannels, myAddress: Address, counterpartyAddress: Address, myFund: Balance, counterpartyFund: Balance) {
  const totalFund = myFund.toBN().add(counterpartyFund.toBN())
  try {
    const transaction = await sendTransaction(
      token.send,
      channels.address,
      totalFund.toString(),
      abiCoder.encode(
        ['address', 'address', 'uint256', 'uint256'],
        [
          myAddress.toHex(),
          counterpartyAddress.toHex(),
          myFund.toBN().toString(),
          counterpartyFund.toBN().toString()
        ]
      )
    )
    await transaction.wait()

    return transaction.hash
  } catch (err) {
    // TODO: catch race-condition
    log('fund error', err)
    throw Error(`Failed to fund channel`)
  }
}
async function openChannel(hoprToken, hoprChannels, myAddress, counterpartyAddress, fundAmount) {
  try {
    const transaction = await sendTransaction(
      hoprToken.send,
      hoprChannels.address,
      fundAmount.toBN().toString(),
      abiCoder.encode(
        ['address', 'address', 'uint256', 'uint256'],
        [myAddress.toHex(), counterpartyAddress.toHex(), fundAmount.toBN().toString(), '0']
      )
    )
    await transaction.wait()

    return transaction.hash
  } catch (err) {
    // TODO: catch race-condition
    console.log(err)
    throw Error(`Failed to open channel`)
  }
}

async function finalizeChannelClosure(hoprChannels, counterpartyAddress) {
  try {
    const transaction = await sendTransaction(
      hoprChannels.finalizeChannelClosure,
      counterpartyAddress.toHex()
    )
    await transaction.wait()

    return transaction.hash
  } catch (err) {
    // TODO: catch race-condition
    console.log(err)
    throw Error(`Failed to finilize channel closure`)
  }
}

async function initiateChannelClosure(hoprChannels, counterpartyAddress){
  try {
    const transaction = await sendTransaction(
      hoprChannels.initiateChannelClosure,
      counterpartyAddress.toHex()
    )
    await transaction.wait()

    return transaction.hash
  } catch (err) {
    // TODO: catch race-condition
    console.log(err)
    throw Error(`Failed to initialize channel closure`)
  }
}

async function redeemTicket(hoprChannels, counterparty, ackTicket, ticket){
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
  return transaction
}

async function setCommitment(channels: HoprChannels, commitment: Hash){
  try {
    const transaction = await sendTransaction(
      channels.bumpCommitment,
      commitment.toHex()
    )
    await transaction.wait()
    return transaction.hash
  } catch (err) {
    // TODO: catch race-condition
    log(err)
    throw Error(`Failed to initialize channel closure`)
  }
}

