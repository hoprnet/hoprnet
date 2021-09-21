import type { ContractTransaction, UnsignedTransaction } from 'ethers'
import type { Multiaddr } from 'multiaddr'
import { providers, utils, errors, Wallet, BigNumber } from 'ethers'
import {
  Networks,
  networks,
  getContractData,
  HoprToken,
  HoprChannels,
  HoprToken__factory,
  HoprChannels__factory
} from '@hoprnet/hopr-ethereum'
import {
  Address,
  Ticket,
  AcknowledgedTicket,
  Balance,
  NativeBalance,
  Hash,
  PublicKey,
  durations,
  PromiseValue
} from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import NonceTracker from './nonce-tracker'
import TransactionManager, { TransactionPayload } from './transaction-manager'
import Debug from 'debug'
import { CONFIRMATIONS } from './constants'

const log = Debug('hopr:core-ethereum:chain-operations')
const abiCoder = new utils.AbiCoder()
const knownNetworks = Object.entries(networks).map(([name, data]) => ({
  name: name as Networks,
  ...data
}))

export type Receipt = string
export type ChainWrapper = PromiseValue<ReturnType<typeof createChainWrapper>>

export async function createChainWrapper(providerURI: string, privateKey: Uint8Array, checkDuplicate: Boolean = true) {
  const provider = providerURI.startsWith('http')
    ? new providers.StaticJsonRpcProvider(providerURI)
    : new providers.WebSocketProvider(providerURI)
  const wallet = new Wallet(privateKey).connect(provider)
  const publicKey = PublicKey.fromPrivKey(privateKey)
  const address = publicKey.toAddress()
  const chainId = await provider.getNetwork().then((res) => res.chainId)
  const networkInfo = knownNetworks.find((info) => info.chainId === chainId)
  // get network's name by looking into our known networks
  const network: Networks = networkInfo?.name || 'localhost'

  const hoprTokenDeployment = getContractData(network, 'HoprToken')
  const hoprChannelsDeployment = getContractData(network, 'HoprChannels')

  const token = HoprToken__factory.connect(hoprTokenDeployment.address, wallet)
  const channels = HoprChannels__factory.connect(hoprChannelsDeployment.address, wallet)
  const genesisBlock = (await provider.getTransaction(hoprChannelsDeployment.transactionHash)).blockNumber
  const channelClosureSecs = await channels.secsClosure()

  const transactions = new TransactionManager()
  const nonceTracker = new NonceTracker(
    {
      getLatestBlockNumber: async () => provider.getBlockNumber(),
      getTransactionCount: (address, blockNumber) => provider.getTransactionCount(address.toHex(), blockNumber),
      getConfirmedTransactions: (_addr) => Array.from(transactions.confirmed.values()),
      getPendingTransactions: (_addr) => Array.from(transactions.pending.values())
    },
    durations.minutes(15)
  )

  // naive implementation, assumes transaction is not replaced
  // temporary used until https://github.com/ethers-io/ethers.js/issues/1479
  // is fixed
  async function waitForConfirmations(
    transactionHash: string,
    confimations: number,
    timeout: number,
    onMined: (nonce: number, hash: string) => void,
    onConfirmed: (nonce: number, hash: string) => void
  ): Promise<providers.TransactionResponse> {
    const wait = 1e3
    let started = 0
    let response: providers.TransactionResponse

    while (started < timeout) {
      response = await provider.getTransaction(transactionHash)
      if (response && response.confirmations == 0) onMined(response.nonce, response.hash)
      if (response && response.confirmations >= confimations) {
        onConfirmed(response.nonce, response.hash)
        break
      }
      // wait 1 sec
      await new Promise((resolve) => setTimeout(resolve, wait))
      started += wait
    }

    if (!response) throw Error(errors.TIMEOUT)
    return response
  }

  /**
   * Update nonce-tracker and transaction-manager, broadcast the transaction on chain, and listen
   * to the response until reaching block confirmation.
   * @param checkDuplicate If the flag is true (default), check if an unconfirmed (pending/mined) transaction with the same payload has been sent
   * @param method contract method
   * @param rest contract arguments
   * @returns Promise of a ContractTransaction
   */
  async function sendTransaction<T extends (...args: any) => Promise<ContractTransaction>>(
    checkDuplicate: Boolean,
    populatedTx: UnsignedTransaction,
    method: T,
    ...rest: Parameters<T>
  ): Promise<ContractTransaction> {
    const gasLimit = 300e3
    const gasPrice = networkInfo?.gas ?? (await provider.getGasPrice())
    const nonceLock = await nonceTracker.getNonceLock(address)
    const nonce = nonceLock.nextNonce
    let transaction: ContractTransaction

    log('Sending transaction %o', {
      gasLimit,
      gasPrice,
      nonce
    })

    const essentialTxPayload: TransactionPayload = {
      to: populatedTx.to,
      data: populatedTx.data as string,
      value: BigNumber.from(populatedTx.value ?? 0)
    }
    log('essentialTxPayload %o', essentialTxPayload)

    try {
      if (checkDuplicate && transactions.existInMinedOrPendingWithHigherFee(essentialTxPayload, gasPrice)) {
        // check duplicated pending/mined transaction against transaction manager
        // if transaction manager has a transaction with the same payload that is mined or is pending but with
        // a higher or equal nonce, halt.
        log(
          'checkDuplicate %s and %s',
          checkDuplicate,
          transactions.existInMinedOrPendingWithHigherFee(essentialTxPayload, gasPrice)
        )
        return
        // TODO: If the transaction manager is out of sync, check against mempool/mined blocks from provider.
      }

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
    transactions.addToPending(transaction.hash, { nonce, gasPrice }, essentialTxPayload)
    nonceLock.releaseLock()

    try {
      await waitForConfirmations(
        transaction.hash,
        CONFIRMATIONS,
        30e3,
        (nonce: number, hash: string) => {
          log('Transaction with nonce %d and hash %s mined', nonce, hash)
          transactions.moveToMined(hash)
        },
        (nonce: number, hash: string) => {
          log('Transaction with nonce %d and hash %s confirmed', nonce, hash)
          transactions.moveToConfirmed(hash)
        }
      )
    } catch (error) {
      const isRevertedErr = [error?.code, String(error)].includes(errors.CALL_EXCEPTION)
      const isAlreadyKnownErr =
        [error?.code, String(error)].includes(errors.NONCE_EXPIRED) ||
        [error?.code, String(error)].includes(errors.REPLACEMENT_UNDERPRICED)

      if (isRevertedErr) {
        log('Transaction with nonce %d and hash %s reverted: %s', nonce, transaction.hash, error)

        // this transaction failed but was confirmed as reverted
        transactions.moveToConfirmed(transaction.hash)
      } else {
        log('Transaction with nonce %d failed to sent: %s', nonce, error)

        // if this hash is already known and we already have it included in
        // pending we can safely ignore this
        if (isAlreadyKnownErr && transactions.pending.has(transaction.hash)) return

        // this transaction was not confirmed so we just remove it
        transactions.remove(transaction.hash)
      }

      throw error
    }

    return transaction
  }

  async function announce(multiaddr: Multiaddr): Promise<string> {
    const populatedTx = await channels.populateTransaction.announce(
      publicKey.toUncompressedPubKeyHex(),
      multiaddr.bytes
    )

    try {
      const confirmation = await sendTransaction(true, populatedTx, channels.announce, publicKey.toUncompressedPubKeyHex(), multiaddr.bytes)
      return confirmation.hash
    } catch {
      throw new Error('Fatal error, announce transaction failed')
    }
  }

  async function withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    if (currency === 'NATIVE') {
      const nonceLock = await nonceTracker.getNonceLock(address)
      try {
        // FIXME: track pending tx
        const transaction = await wallet.sendTransaction({
          to: recipient,
          value: BigNumber.from(amount),
          nonce: BigNumber.from(nonceLock.nextNonce)
        })
        nonceLock.releaseLock()
        return transaction.hash
      } catch (err) {
        nonceLock.releaseLock()
        throw err
      }
    } else {
      const populatedTx = await token.populateTransaction.transfer(recipient, amount)
      return (await sendTransaction(checkDuplicate, populatedTx, token.transfer, recipient, amount)).hash
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
    const populatedTx = await token.populateTransaction.send(
      channels.address,
      totalFund.toString(),
      abiCoder.encode(
        ['address', 'address', 'uint256', 'uint256'],
        [me.toHex(), counterparty.toHex(), myFund.toBN().toString(), counterpartyFund.toBN().toString()]
      )
    )
    const transaction = await sendTransaction(
      checkDuplicate,
      populatedTx,
      token.send,
      channels.address,
      totalFund.toString(),
      abiCoder.encode(
        ['address', 'address', 'uint256', 'uint256'],
        [me.toHex(), counterparty.toHex(), myFund.toBN().toString(), counterpartyFund.toBN().toString()]
      )
    )
    return transaction.hash
  }

  async function openChannel(
    token: HoprToken,
    channels: HoprChannels,
    me: Address,
    counterparty: Address,
    amount: Balance
  ): Promise<Receipt> {
    const populatedTx = await token.populateTransaction.send(
      channels.address,
      amount.toBN().toString(),
      abiCoder.encode(
        ['address', 'address', 'uint256', 'uint256'],
        [me.toHex(), counterparty.toHex(), amount.toBN().toString(), '0']
      )
    )
    const transaction = await sendTransaction(
      checkDuplicate,
      populatedTx,
      token.send,
      channels.address,
      amount.toBN().toString(),
      abiCoder.encode(
        ['address', 'address', 'uint256', 'uint256'],
        [me.toHex(), counterparty.toHex(), amount.toBN().toString(), '0']
      )
    )
    return transaction.hash
  }

  async function finalizeChannelClosure(channels: HoprChannels, counterparty: Address): Promise<Receipt> {
    const populatedTx = await channels.populateTransaction.finalizeChannelClosure(counterparty.toHex())
    const transaction = await sendTransaction(
      checkDuplicate,
      populatedTx,
      channels.finalizeChannelClosure,
      counterparty.toHex()
    )
    return transaction.hash
    // TODO: catch race-condition
  }

  async function initiateChannelClosure(channels: HoprChannels, counterparty: Address): Promise<Receipt> {
    const populatedTx = await channels.populateTransaction.initiateChannelClosure(counterparty.toHex())

    const transaction = await sendTransaction(
      checkDuplicate,
      populatedTx,
      channels.initiateChannelClosure,
      counterparty.toHex()
    )
    return transaction.hash
    // TODO: catch race-condition
  }

  async function redeemTicket(
    channels: HoprChannels,
    counterparty: Address,
    ackTicket: AcknowledgedTicket,
    ticket: Ticket
  ): Promise<Receipt> {
    const populatedTx = await channels.populateTransaction.redeemTicket(
      counterparty.toHex(),
      ackTicket.preImage.toHex(),
      ackTicket.ticket.epoch.serialize(),
      ackTicket.ticket.index.serialize(),
      ackTicket.response.toHex(),
      ticket.amount.toBN().toString(),
      ticket.winProb.toBN().toString(),
      ticket.signature.serializeEthereum()
    )

    const transaction = await sendTransaction(
      checkDuplicate,
      populatedTx,
      channels.redeemTicket,
      counterparty.toHex(),
      ackTicket.preImage.toHex(),
      ackTicket.ticket.epoch.serialize(),
      ackTicket.ticket.index.serialize(),
      ackTicket.response.toHex(),
      ticket.amount.toBN().toString(),
      ticket.winProb.toBN().toString(),
      ticket.signature.serializeEthereum()
    )
    return transaction.hash
  }

  async function setCommitment(channels: HoprChannels, counterparty: Address, commitment: Hash): Promise<Receipt> {
    const populatedTx = await channels.populateTransaction.bumpChannel(counterparty.toHex(), commitment.toHex())
    const transaction = await sendTransaction(
      checkDuplicate,
      populatedTx,
      channels.bumpChannel,
      counterparty.toHex(),
      commitment.toHex()
    )
    return transaction.hash
  }

  const api = {
    getBalance: (address: Address) =>
      token.balanceOf(address.toHex()).then((res) => new Balance(new BN(res.toString()))),
    getNativeBalance: (address: Address) =>
      provider.getBalance(address.toHex()).then((res) => new NativeBalance(new BN(res.toString()))),
    announce,
    withdraw: (currency: 'NATIVE' | 'HOPR', recipient: string, amount: string) => withdraw(currency, recipient, amount),
    fundChannel: (me: Address, counterparty: Address, myTotal: Balance, theirTotal: Balance) =>
      fundChannel(token, channels, me, counterparty, myTotal, theirTotal),
    openChannel: (me: Address, counterparty: Address, amount: Balance) =>
      openChannel(token, channels, me, counterparty, amount),
    finalizeChannelClosure: (counterparty: Address) => finalizeChannelClosure(channels, counterparty),
    initiateChannelClosure: (counterparty: Address) => initiateChannelClosure(channels, counterparty),
    redeemTicket: (counterparty: Address, ackTicket: AcknowledgedTicket, ticket: Ticket) =>
      redeemTicket(channels, counterparty, ackTicket, ticket),
    getGenesisBlock: () => genesisBlock,
    setCommitment: (counterparty: Address, comm: Hash) => setCommitment(channels, counterparty, comm),
    getWallet: () => wallet,
    waitUntilReady: async () => await provider.ready,
    getLatestBlockNumber: async () => provider.getBlockNumber(), // TODO: use indexer when it's done syncing
    subscribeBlock: (cb) => provider.on('block', cb),
    subscribeError: (cb) => {
      provider.on('error', cb)
      channels.on('error', cb)
    },
    subscribeChannelEvents: (cb) => channels.on('*', cb),
    unsubscribe: () => {
      provider.removeAllListeners()
      channels.removeAllListeners()
    },
    getChannels: () => channels,
    getPrivateKey: () => utils.arrayify(wallet.privateKey),
    getPublicKey: () => PublicKey.fromString(utils.computePublicKey(wallet.publicKey, true)),
    getInfo: () => ({
      network,
      hoprTokenAddress: hoprTokenDeployment.address,
      hoprChannelsAddress: hoprChannelsDeployment.address,
      channelClosureSecs
    })
  }

  return api
}
