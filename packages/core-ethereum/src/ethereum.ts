import type { ContractTransaction, BaseContract } from 'ethers'
import type { Multiaddr } from 'multiaddr'
import { providers, utils, errors, Wallet, BigNumber, ethers } from 'ethers'
import type { HoprToken, HoprChannels, TypedEvent } from '@hoprnet/hopr-ethereum'
import { getContractData } from '@hoprnet/hopr-ethereum'
import {
  Address,
  Ticket,
  AcknowledgedTicket,
  Balance,
  NativeBalance,
  Hash,
  PublicKey,
  durations,
  DeferType
} from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import NonceTracker from './nonce-tracker'
import TransactionManager, { TransactionPayload } from './transaction-manager'
import { debug } from '@hoprnet/hopr-utils'
import { TX_CONFIRMATION_WAIT } from './constants'

const log = debug('hopr:core-ethereum:ethereum')
const abiCoder = new utils.AbiCoder()

export type Receipt = string
export type ChainWrapper = Awaited<ReturnType<typeof createChainWrapper>>
export type SendTransactionReturn = {
  code: 'SUCCESS' | 'DUPLICATE'
  tx: Partial<ContractTransaction>
}

export async function createChainWrapper(
  networkInfo: { provider: string; chainId: number; gasPrice?: string; network: string; environment: string },
  privateKey: Uint8Array,
  checkDuplicate: Boolean = true,
  timeout = TX_CONFIRMATION_WAIT
) {
  const provider = networkInfo.provider.startsWith('http')
    ? new providers.StaticJsonRpcProvider(networkInfo.provider)
    : new providers.WebSocketProvider(networkInfo.provider)
  log('Provider obtained from options', provider.network)
  const wallet = new Wallet(privateKey).connect(provider)
  const publicKey = PublicKey.fromPrivKey(privateKey)
  const address = publicKey.toAddress()
  const providerChainId = (await provider.getNetwork()).chainId

  // ensure chain id matches our expectation
  if (networkInfo.chainId !== providerChainId) {
    throw Error(`Providers chain id ${providerChainId} does not match ${networkInfo.chainId}`)
  }

  const hoprTokenDeployment = getContractData(networkInfo.network, networkInfo.environment, 'HoprToken')
  const hoprChannelsDeployment = getContractData(networkInfo.network, networkInfo.environment, 'HoprChannels')

  const token = new ethers.Contract(hoprTokenDeployment.address, hoprTokenDeployment.abi, wallet) as HoprToken

  const channels = new ethers.Contract(
    hoprChannelsDeployment.address,
    hoprChannelsDeployment.abi,
    wallet
  ) as HoprChannels

  const genesisBlock = (await provider.getTransaction(hoprChannelsDeployment.transactionHash)).blockNumber
  const channelClosureSecs = await channels.secsClosure()

  const transactions = new TransactionManager()
  const nonceTracker = new NonceTracker(
    {
      getLatestBlockNumber: provider.getBlockNumber.bind(provider),
      getTransactionCount: (address, blockNumber) => provider.getTransactionCount(address.toHex(), blockNumber),
      getPendingTransactions: (_addr) => transactions.getAllUnconfirmedTxs(),
      getConfirmedTransactions: (_addr) => Array.from(transactions.confirmed.values())
    },
    durations.minutes(15)
  )

  let gasPrice: number | BigNumber
  if (networkInfo.gasPrice) {
    const [gasPriceValue, gasPriceUnit] = networkInfo.gasPrice.split(' ')
    gasPrice = ethers.utils.parseUnits(gasPriceValue, gasPriceUnit)
  } else {
    gasPrice = await provider.getGasPrice()
  }

  /**
   * Update nonce-tracker and transaction-manager, broadcast the transaction on chain, and listen
   * to the response until reaching block confirmation.
   * @param checkDuplicate If the flag is true (default), check if an unconfirmed (pending/mined) transaction with the same payload has been sent
   * @param method contract method
   * @param rest contract arguments
   * @returns Promise of a ContractTransaction
   */
  async function sendTransaction<T extends BaseContract>(
    checkDuplicate: Boolean,
    contract: T,
    method: keyof T['functions'],
    handleTxListener: (tx: string) => DeferType<string>,
    ...rest: Parameters<T['functions'][keyof T['functions']]>
  ): Promise<SendTransactionReturn> {
    const gasLimit = 400e3
    const nonceLock = await nonceTracker.getNonceLock(address)
    const nonce = nonceLock.nextNonce
    let transaction: ContractTransaction

    log('Sending transaction %o', {
      gasLimit,
      gasPrice,
      nonce
    })

    // breakdown steps in ethersjs
    // https://github.com/ethers-io/ethers.js/blob/master/packages/abstract-signer/src.ts/index.ts#L122
    // 1. omit this._checkProvider("sendTransaction");
    // 2. populate transaction
    const tx = await contract.populateTransaction[method as string](...rest)
    const populatedTx = await wallet.populateTransaction({ ...tx, gasLimit, gasPrice, nonce })
    const essentialTxPayload: TransactionPayload = {
      to: populatedTx.to,
      data: populatedTx.data as string,
      value: BigNumber.from(populatedTx.value ?? 0)
    }
    log('essentialTxPayload %o', essentialTxPayload)

    let initiatedHash: string
    let deferredListener: DeferType<string>
    try {
      if (checkDuplicate) {
        const [isDuplicate, hash] = transactions.existInMinedOrPendingWithHigherFee(essentialTxPayload, gasPrice)
        // check duplicated pending/mined transaction against transaction manager
        // if transaction manager has a transaction with the same payload that is mined or is pending but with
        // a higher or equal nonce, halt.
        log('checkDuplicate checkDuplicate=%s isDuplicate=%s with hash %s', checkDuplicate, isDuplicate, hash)

        if (isDuplicate) {
          return {
            code: 'DUPLICATE',
            tx: { hash }
          }
        }
        // TODO: If the transaction manager is out of sync, check against mempool/mined blocks from provider.
      }

      // 3. sign transaction
      const signedTx = await wallet.signTransaction(populatedTx)
      // compute tx hash and save to initiated tx list in tx manager
      initiatedHash = utils.keccak256(signedTx)
      transactions.addToQueuing(initiatedHash, { nonce, gasPrice }, essentialTxPayload)
      // with let indexer to listen to the tx
      deferredListener = handleTxListener(initiatedHash)
      // 4. send transaction to our ethereum provider
      transaction = await provider.sendTransaction(signedTx)
    } catch (error) {
      log('Transaction with nonce %d failed to sent: %s', nonce, error)
      deferredListener && deferredListener.reject()
      transactions.remove(initiatedHash)
      nonceLock.releaseLock()
      throw error
    }

    log('Transaction with nonce %d successfully sent %s, waiting for confimation', nonce, transaction.hash)
    transactions.moveFromQueuingToPending(transaction.hash)
    nonceLock.releaseLock()

    try {
      // wait for the tx to be mined
      await provider.waitForTransaction(transaction.hash, 1, timeout)
    } catch (error) {
      log(error)
      // remove listener but not throwing error message
      deferredListener.reject()
      // this transaction was not confirmed so we just remove it
      transactions.remove(transaction.hash)
      const isRevertedErr = [error?.code, String(error)].includes(errors.CALL_EXCEPTION)
      const isAlreadyKnownErr =
        [error?.code, String(error)].includes(errors.NONCE_EXPIRED) ||
        [error?.code, String(error)].includes(errors.REPLACEMENT_UNDERPRICED)

      if (isRevertedErr) {
        log('Transaction with nonce %d and hash %s reverted due to call exception: %s', nonce, transaction.hash, error)
      } else if (isAlreadyKnownErr) {
        log('Transaction with nonce %d and hash %s reverted due to known error: %s', nonce, transaction.hash, error)
      } else {
        log('Transaction with nonce %d and hash failed to send: %s', nonce, transaction.hash, error)
      }

      throw new Error(`Failed in mining transaction. ${error}`)
    }
    try {
      await deferredListener.promise
      return {
        code: 'SUCCESS',
        tx: { hash: transaction.hash }
      }
    } catch (error) {
      log('error: transaction with nonce %d and hash failed to send: %s', nonce, transaction.hash, error)
      throw error
    }
  }

  async function announce(multiaddr: Multiaddr, txHandler: (tx: string) => DeferType<string>): Promise<string> {
    try {
      const confirmation = await sendTransaction(
        checkDuplicate,
        channels,
        'announce',
        txHandler,
        publicKey.toUncompressedPubKeyHex(),
        multiaddr.bytes
      )
      return confirmation.tx.hash
    } catch (error) {
      throw new Error(`Failed in sending announce transaction ${error}`)
    }
  }

  async function withdraw(
    currency: 'NATIVE' | 'HOPR',
    recipient: string,
    amount: string,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<string> {
    if (currency === 'NATIVE') {
      const nonceLock = await nonceTracker.getNonceLock(address)
      try {
        // FIXME: track pending tx
        const transaction = await wallet.sendTransaction({
          to: recipient,
          value: BigNumber.from(amount),
          nonce: BigNumber.from(nonceLock.nextNonce),
          gasPrice
        })
        nonceLock.releaseLock()
        return transaction.hash
      } catch (err) {
        nonceLock.releaseLock()
        throw err
      }
    }

    // withdraw HOPR
    try {
      const transaction = await sendTransaction(checkDuplicate, token, 'transfer', txHandler, recipient, amount)
      return transaction.tx.hash
    } catch (error) {
      throw new Error(`Failed in sending withdraw transaction ${error}`)
    }
  }

  async function fundChannel(
    token: HoprToken,
    channels: HoprChannels,
    me: Address,
    counterparty: Address,
    myFund: Balance,
    counterpartyFund: Balance,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> {
    try {
      const totalFund = myFund.toBN().add(counterpartyFund.toBN())
      const transaction = await sendTransaction(
        checkDuplicate,
        token,
        'send',
        txHandler,
        channels.address,
        totalFund.toString(),
        abiCoder.encode(
          ['address', 'address', 'uint256', 'uint256'],
          [me.toHex(), counterparty.toHex(), myFund.toBN().toString(), counterpartyFund.toBN().toString()]
        )
      )
      return transaction.tx.hash
    } catch (error) {
      throw new Error(`Failed in sending fundChannel transaction ${error}`)
    }
  }

  async function openChannel(
    token: HoprToken,
    channels: HoprChannels,
    me: Address,
    counterparty: Address,
    amount: Balance,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> {
    try {
      const transaction = await sendTransaction(
        checkDuplicate,
        token,
        'send',
        txHandler,
        channels.address,
        amount.toBN().toString(),
        abiCoder.encode(
          ['address', 'address', 'uint256', 'uint256'],
          [me.toHex(), counterparty.toHex(), amount.toBN().toString(), '0']
        )
      )
      return transaction.tx.hash
    } catch (error) {
      throw new Error(`Failed in sending openChannel transaction ${error}`)
    }
  }

  async function finalizeChannelClosure(
    channels: HoprChannels,
    counterparty: Address,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> {
    try {
      const transaction = await sendTransaction(
        checkDuplicate,
        channels,
        'finalizeChannelClosure',
        txHandler,
        counterparty.toHex()
      )
      return transaction.tx.hash
    } catch (error) {
      throw new Error(`Failed in sending finalizeChannelClosure transaction ${error}`)
    }
    // TODO: catch race-condition
  }

  async function initiateChannelClosure(
    channels: HoprChannels,
    counterparty: Address,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> {
    try {
      const transaction = await sendTransaction(
        checkDuplicate,
        channels,
        'initiateChannelClosure',
        txHandler,
        counterparty.toHex()
      )
      return transaction.tx.hash
    } catch (error) {
      throw new Error(`Failed in sending initiateChannelClosure transaction ${error}`)
    }
    // TODO: catch race-condition
  }

  async function redeemTicket(
    channels: HoprChannels,
    counterparty: Address,
    ackTicket: AcknowledgedTicket,
    ticket: Ticket,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> {
    try {
      const transaction = await sendTransaction(
        checkDuplicate,
        channels,
        'redeemTicket',
        txHandler,
        counterparty.toHex(),
        ackTicket.preImage.toHex(),
        ackTicket.ticket.epoch.serialize(),
        ackTicket.ticket.index.serialize(),
        ackTicket.response.toHex(),
        ticket.amount.toBN().toString(),
        ticket.winProb.toBN().toString(),
        ticket.signature.serialize()
      )
      return transaction.tx.hash
    } catch (error) {
      throw new Error(`Failed in sending redeemticket transaction ${error}`)
    }
  }

  async function setCommitment(
    channels: HoprChannels,
    counterparty: Address,
    commitment: Hash,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> {
    try {
      const transaction = await sendTransaction(
        checkDuplicate,
        channels,
        'bumpChannel',
        txHandler,
        counterparty.toHex(),
        commitment.toHex()
      )
      return transaction.tx.hash
    } catch (error) {
      throw new Error(`Failed in sending setCommitment transaction ${error}`)
    }
  }

  async function getNativeTokenTransactionInBlock(
    blockNumber: number,
    isOutgoing: boolean = true
  ): Promise<Array<string>> {
    const blockWithTx = await provider.getBlockWithTransactions(blockNumber)
    const txs = blockWithTx.transactions.filter(
      (tx) => tx.value.gt(BigNumber.from(0)) && (isOutgoing ? tx.from : tx.to) === wallet.address
    )
    return txs.length === 0 ? [] : txs.map((tx) => tx.hash)
  }

  const api = {
    getBalance: (address: Address) =>
      token.balanceOf(address.toHex()).then((res) => new Balance(new BN(res.toString()))),
    getNativeBalance: (address: Address) =>
      provider.getBalance(address.toHex()).then((res) => new NativeBalance(new BN(res.toString()))),
    getNativeTokenTransactionInBlock: (blockNumber: number, isOutgoing: boolean = true) =>
      getNativeTokenTransactionInBlock(blockNumber, isOutgoing),
    announce,
    withdraw: (
      currency: 'NATIVE' | 'HOPR',
      recipient: string,
      amount: string,
      txHandler: (tx: string) => DeferType<string>
    ) => withdraw(currency, recipient, amount, txHandler),
    fundChannel: (
      me: Address,
      counterparty: Address,
      myTotal: Balance,
      theirTotal: Balance,
      txHandler: (tx: string) => DeferType<string>
    ) => fundChannel(token, channels, me, counterparty, myTotal, theirTotal, txHandler),
    openChannel: (me: Address, counterparty: Address, amount: Balance, txHandler: (tx: string) => DeferType<string>) =>
      openChannel(token, channels, me, counterparty, amount, txHandler),
    finalizeChannelClosure: (counterparty: Address, txHandler: (tx: string) => DeferType<string>) =>
      finalizeChannelClosure(channels, counterparty, txHandler),
    initiateChannelClosure: (counterparty: Address, txHandler: (tx: string) => DeferType<string>) =>
      initiateChannelClosure(channels, counterparty, txHandler),
    redeemTicket: (
      counterparty: Address,
      ackTicket: AcknowledgedTicket,
      ticket: Ticket,
      txHandler: (tx: string) => DeferType<string>
    ) => redeemTicket(channels, counterparty, ackTicket, ticket, txHandler),
    getGenesisBlock: () => genesisBlock,
    setCommitment: (counterparty: Address, comm: Hash, txHandler: (tx: string) => DeferType<string>) =>
      setCommitment(channels, counterparty, comm, txHandler),
    getWallet: () => wallet,
    waitUntilReady: async () => await provider.ready,
    getLatestBlockNumber: provider.getBlockNumber.bind(provider), // TODO: use indexer when it's done syncing
    subscribeBlock: (cb: (blockNumber: number) => void | Promise<void>): (() => void) => {
      provider.on('block', cb)

      return () => {
        provider.off('block', cb)
      }
    },
    subscribeError: (cb: (err: any) => void | Promise<void>): (() => void) => {
      provider.on('error', cb)
      channels.on('error', cb)
      token.on('error', cb)

      return () => {
        provider.off('error', cb)
        channels.off('error', cb)
        token.off('error', cb)
      }
    },
    subscribeChannelEvents: (cb: (event: TypedEvent<any, any>) => void | Promise<void>): (() => void) => {
      channels.on('*', cb)

      return () => {
        channels.off('*', cb)
      }
    },
    // Cannot directly apply filters here because it does not return a full event object
    subscribeTokenEvents: (cb: (event: TypedEvent<any, any>) => void | Promise<void>): (() => void) => {
      token.on('*', cb)

      return () => {
        token.off('*', cb)
      }
    }, // subscribe all the Transfer events from current nodes in HoprToken.
    unsubscribe: () => {
      provider.removeAllListeners()
      channels.removeAllListeners()
      token.removeAllListeners()
    },
    getChannels: () => channels,
    getPrivateKey: () => utils.arrayify(wallet.privateKey),
    getPublicKey: () => PublicKey.fromString(utils.computePublicKey(wallet.publicKey, true)),
    getInfo: () => ({
      network: networkInfo.network,
      hoprTokenAddress: hoprTokenDeployment.address,
      hoprChannelsAddress: hoprChannelsDeployment.address,
      channelClosureSecs
    }),
    updateConfirmedTransaction: (hash: string) => transactions.moveToConfirmed(hash),
    getAllQueuingTransactionRequests: () => transactions.getAllQueuingTxs()
  }

  return api
}
