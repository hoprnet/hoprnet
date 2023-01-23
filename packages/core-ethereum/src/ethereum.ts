import { setImmediate as setImmediatePromise } from 'timers/promises'
import type { Multiaddr } from '@multiformats/multiaddr'
import {
  providers,
  utils,
  errors,
  BigNumber,
  ethers,
  type UnsignedTransaction,
  type ContractTransaction,
  type BaseContract
} from 'ethers'
import {
  Address,
  Balance,
  NativeBalance,
  PublicKey,
  durations,
  type AcknowledgedTicket,
  type DeferType,
  type Hash,
  create_counter
} from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import NonceTracker from './nonce-tracker.js'
import TransactionManager, { type TransactionPayload } from './transaction-manager.js'
import { debug } from '@hoprnet/hopr-utils'
import { CORE_ETHEREUM_CONSTANTS } from '../lib/core_ethereum_misc.js'
import type { Block } from '@ethersproject/abstract-provider'

// @ts-ignore untyped library
import retimer from 'retimer'
import {
  HOPR_CHANNELS_ABI,
  HOPR_NETWORK_REGISTRY_ABI,
  HOPR_TOKEN_ABI,
  HoprChannels,
  HoprNetworkRegistry,
  HoprToken,
  DeploymentExtract
} from './utils/index.js'

// Exported from Rust
const constants = CORE_ETHEREUM_CONSTANTS()

const log = debug('hopr:core-ethereum:ethereum')
const abiCoder = new utils.AbiCoder()

// Metrics
const metric_countSendTransaction = create_counter(
  'core_ethereum_counter_num_send_transactions',
  'The number of sendTransaction calls'
)

export type Receipt = string
export type ChainWrapper = Awaited<ReturnType<typeof createChainWrapper>>

export enum SendTransactionStatus {
  SUCCESS = 'SUCCESS',
  DUPLICATE = 'DUPLICATE'
}
export type SendTransactionReturn =
  | {
      code: SendTransactionStatus.SUCCESS
      tx: Partial<ContractTransaction>
    }
  | {
      code: SendTransactionStatus.DUPLICATE
    }

export async function createChainWrapper(
  deploymentExtract: DeploymentExtract,
  networkInfo: {
    provider: string
    chainId: number
    maxFeePerGas: string
    maxPriorityFeePerGas: string
    network: string
    environment: string
  },
  privateKey: Uint8Array,
  checkDuplicate: Boolean = true,
  txTimeout = constants.TX_CONFIRMATION_WAIT
) {
  log(`[DEBUG] networkInfo.provider ${JSON.stringify(networkInfo.provider, null, 2)}`)
  const provider = networkInfo.provider.startsWith('http')
    ? new providers.StaticJsonRpcProvider(networkInfo.provider)
    : new providers.WebSocketProvider(networkInfo.provider)
  log(`[DEBUG] provider ${provider}`)
  const publicKey = PublicKey.fromPrivKey(privateKey)
  log(`[DEBUG] publicKey ${publicKey}`)
  const address = publicKey.toAddress()
  log(`[DEBUG] address ${address}`)
  const providerChainId = (await provider.getNetwork()).chainId
  log(`[DEBUG] providerChainId ${providerChainId}`)

  // ensure chain id matches our expectation
  if (networkInfo.chainId !== providerChainId) {
    throw Error(`Providers chain id ${providerChainId} does not match ${networkInfo.chainId}`)
  }

  log(`[DEBUG] deploymentExtract ${JSON.stringify(deploymentExtract, null, 2)}`)

  const token = new ethers.Contract(deploymentExtract.hoprTokenAddress, HOPR_TOKEN_ABI, provider) as any as HoprToken

  const channels = new ethers.Contract(
    deploymentExtract.hoprChannelsAddress,
    HOPR_CHANNELS_ABI,
    provider
  ) as any as HoprChannels

  const networkRegistry = new ethers.Contract(
    deploymentExtract.hoprNetworkRegistryAddress,
    HOPR_NETWORK_REGISTRY_ABI,
    provider
  ) as any as HoprNetworkRegistry

  //getGenesisBlock, taking the earlier deployment block between the channel and network Registery
  const genesisBlock = deploymentExtract.indexerStartBlockNumber
  const channelClosureSecs = await channels.secsClosure()

  const transactions = new TransactionManager()

  const subscribeBlock = (cb: (blockNumber: number) => void | Promise<void>): (() => void) => {
    provider.on('block', cb)

    return () => {
      provider.off('block', cb)
    }
  }

  /**
   * Gets the latest block number by explicitly querying the provider
   * @returns a Promise that resolves with the latest block number
   */
  const getLatestBlockNumber = async (): Promise<number> => {
    const RETRIES = 3
    for (let i = 0; i < RETRIES; i++) {
      try {
        return await provider.getBlockNumber()
      } catch (err) {
        if (i + 1 < RETRIES) {
          await setImmediatePromise()
          continue
        }

        log(`Could not determine latest on-chain block. Now waiting for next block.`)
      }
    }

    // Waits for next block and returns the blockNumber
    return new Promise<number>((resolve) => {
      provider.once('block', resolve)
    })
  }

  /**
   * Gets the number of previous transactions
   * @param address account to query for
   * @param blockNumber [optional] number of the block to consider
   * @returns a Promise that resolves with the transaction count
   */
  const getTransactionCount = async (address: Address, blockNumber?: number): Promise<number> => {
    const RETRIES = 3
    for (let i = 0; i < RETRIES; i++) {
      try {
        return await provider.getTransactionCount(address.toHex(), blockNumber)
      } catch (err) {
        if (i + 1 < RETRIES) {
          await setImmediatePromise()
        }
      }
    }

    log(`Could not determine latest transaction count using the given provider.`)
    throw Error(`Could not get latest transaction count using the given provider`)
  }

  const nonceTracker = new NonceTracker(
    {
      getLatestBlockNumber,
      getTransactionCount,
      getPendingTransactions: (_addr) => transactions.getAllUnconfirmedTxs(),
      getConfirmedTransactions: (_addr) => Array.from(transactions.confirmed.values())
    },
    durations.minutes(15)
  )

  const [defaultMaxFeePerGasValue, defaultMaxFeePerGasUnit] = networkInfo.maxFeePerGas.split(' ')
  const defaultMaxFeePerGas = ethers.utils.parseUnits(defaultMaxFeePerGasValue, defaultMaxFeePerGasUnit)
  const [defaultMaxPriorityFeePerGasValue, defaultMaxPriorityFeePerGasUnit] =
    networkInfo.maxPriorityFeePerGas.split(' ')
  const defaultMaxPriorityFeePerGas = ethers.utils.parseUnits(
    defaultMaxPriorityFeePerGasValue,
    defaultMaxPriorityFeePerGasUnit
  )

  const waitForTransaction = (txHash: string, removeListener: () => void) => {
    return new Promise<void>((resolve, reject) => {
      let done = false
      let timer: any
      const cleanUp = (err?: string) => {
        if (done) {
          return
        }
        done = true
        timer?.clear()

        // delete all listeners for this particular tx
        provider.off(txHash)

        // Give other tasks time to get scheduled before
        // processing the result
        if (err) {
          log(`Error while waiting for transaction ${txHash}`, err)
          // remove listener but not throwing error message
          removeListener()
          // this transaction was not confirmed so we just remove it
          transactions.remove(txHash)

          setImmediate(reject, Error(err))
        } else {
          setImmediate(resolve)
        }
      }

      const onTransaction = (receipt: providers.TransactionReceipt) => {
        if (receipt.confirmations >= 1) {
          transactions.moveFromPendingToMined(receipt.transactionHash)
          cleanUp()
        } else {
          log(`Received tx receipt for ${txHash} with 0 confirmations, continue listening`)
        }
      }

      // Subscribe to all tx events, unsubscription is handled in cleanup
      provider.on(txHash, onTransaction)

      // Schedule clean up if the timeout is reached
      timer = retimer(cleanUp, txTimeout, `Timed out after waiting ${txTimeout}ms for transaction ${txHash}`)
    })
  }

  /**
   * Build an essential transaction payload from contract parameters
   * @param value amount of native token to send
   * @param contract destination to send funds to or contract to execute the requested method
   * @param method contract method
   * @param rest contract method arguments
   * @returns TransactionPayload
   */
  const buildEssentialTxPayload = <T extends BaseContract>(
    value: BigNumber | string | number,
    contract: T | string,
    method: keyof T['functions'],
    ...rest: Parameters<T['functions'][keyof T['functions']]>
  ): TransactionPayload => {
    if (rest.length > 0 && typeof contract === 'string') {
      throw Error(`sendTransaction: passing arguments to non-contract instances is not implemented`)
    }

    const essentialTxPayload: TransactionPayload = {
      to: typeof contract === 'string' ? contract : contract.address,
      data:
        rest.length > 0 && typeof contract !== 'string'
          ? contract.interface.encodeFunctionData(method as string, rest)
          : '',
      value: BigNumber.from(value ?? 0)
    }
    log('essentialTxPayload %o', essentialTxPayload)
    return essentialTxPayload
  }

  /**
   * Update nonce-tracker and transaction-manager, broadcast the transaction on chain, and listen
   * to the response until reaching block confirmation.
   * Transaction is built on essential transaction payload
   * @param checkDuplicate If the flag is true (default), check if an unconfirmed (pending/mined) transaction with the same payload has been sent
   * @param handleTxListener build listener to transaction hash
   * @returns Promise of a ContractTransaction
   */
  const sendTransaction = async (
    checkDuplicate: Boolean,
    essentialTxPayload: TransactionPayload,
    handleTxListener: (tx: string) => DeferType<string>
  ): Promise<SendTransactionReturn> => {
    const gasLimit = 400e3
    const nonceLock = await nonceTracker.getNonceLock(address)
    const nonce = nonceLock.nextNonce

    let feeData: providers.FeeData

    try {
      feeData = await provider.getFeeData()
    } catch (error) {
      log('Transaction with nonce %d failed to getFeeData', nonce, error)
      // TODO: find an API for fee data per environment
      feeData = {
        lastBaseFeePerGas: null,
        maxFeePerGas: defaultMaxFeePerGas,
        maxPriorityFeePerGas: defaultMaxPriorityFeePerGas,
        gasPrice: null
      }
    }

    log('Sending transaction %o', {
      gasLimit,
      maxFeePerGas: feeData.maxFeePerGas,
      maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
      nonce
    })

    // breakdown steps in ethersjs
    // https://github.com/ethers-io/ethers.js/blob/master/packages/abstract-signer/src.ts/index.ts#L122
    // 1. omit this._checkProvider("sendTransaction");
    // 2. populate transaction, from essential tx payload
    const populatedTx: UnsignedTransaction = {
      to: essentialTxPayload.to,
      value: essentialTxPayload.value,
      type: 2,
      nonce,
      gasLimit,
      maxFeePerGas: feeData.maxFeePerGas,
      maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
      chainId: providerChainId,
      data: essentialTxPayload.data
    }

    if (checkDuplicate) {
      const [isDuplicate, hash] = transactions.existInMinedOrPendingWithHigherFee(
        essentialTxPayload,
        BigNumber.from(populatedTx.maxPriorityFeePerGas)
      )
      // check duplicated pending/mined transaction against transaction manager
      // if transaction manager has a transaction with the same payload that is mined or is pending but with
      // a higher or equal nonce, halt.
      log('checkDuplicate checkDuplicate=%s isDuplicate=%s with hash %s', checkDuplicate, isDuplicate, hash)

      if (isDuplicate) {
        nonceLock.releaseLock()
        return {
          code: SendTransactionStatus.DUPLICATE
        }
      }
      // TODO: If the transaction manager is out of sync, check against mempool/mined blocks from provider.
    }

    // 3. sign transaction
    const signingKey = new utils.SigningKey(privateKey)
    const signature = signingKey.signDigest(utils.keccak256(utils.serializeTransaction(populatedTx)))

    const signedTx = utils.serializeTransaction(populatedTx, signature)
    // compute tx hash and save to initiated tx list in tx manager
    const initiatedHash = utils.keccak256(signedTx)
    const addedToQueue = transactions.addToQueuing(
      initiatedHash,
      {
        nonce: populatedTx.nonce,
        maxPriority: BigNumber.from(populatedTx.maxPriorityFeePerGas),
        maxFeePerGas: BigNumber.from(populatedTx.maxFeePerGas),
        gasLimit: BigNumber.from(populatedTx.gasLimit)
      },
      essentialTxPayload
    )

    if (!addedToQueue) {
      nonceLock.releaseLock()
      return { code: SendTransactionStatus.DUPLICATE }
    }
    // with let indexer to listen to the tx
    const deferredListener = handleTxListener(initiatedHash)

    let transaction: ContractTransaction
    try {
      // 4. send transaction to our ethereum provider
      // throws various exceptions if tx gets rejected
      transaction = await provider.sendTransaction(signedTx)
      // when transaction is sent to the provider, it is moved from queuing to pending
      transactions.moveFromQueuingToPending(initiatedHash)
    } catch (error) {
      nonceLock.releaseLock()
      log('Transaction with nonce %d failed to sent: %s', populatedTx.nonce, error)
      deferredListener.reject()
      // @TODO what if signing the transaction failed and initiatedHash is undefined?
      initiatedHash && transactions.remove(initiatedHash)

      const isRevertedErr = [error?.code, String(error)].includes(errors.CALL_EXCEPTION)
      const isAlreadyKnownErr =
        [error?.code, String(error)].includes(errors.NONCE_EXPIRED) ||
        [error?.code, String(error)].includes(errors.REPLACEMENT_UNDERPRICED)

      if (isRevertedErr) {
        log(
          'Transaction with nonce %d and hash %s reverted due to call exception: %s',
          populatedTx.nonce,
          transaction.hash,
          error
        )
      } else if (isAlreadyKnownErr) {
        log(
          'Transaction with nonce %d and hash %s reverted due to known error: %s',
          populatedTx.nonce,
          transaction.hash,
          error
        )
      } else {
        log('Transaction with nonce %d and hash failed to send: %s', populatedTx.nonce, transaction.hash, error)
      }

      throw new Error(`Failed in publishing transaction. ${error}`)
    }

    log('Transaction with nonce %d successfully sent %s, waiting for confimation', populatedTx.nonce, transaction.hash)
    metric_countSendTransaction.increment()
    nonceLock.releaseLock()

    // wait for the tx to be mined - mininal and scheduled implementation
    // only fails if tx does not get mined within the specified timeout
    await waitForTransaction(transaction.hash, deferredListener.reject.bind(deferredListener))

    try {
      await deferredListener.promise
      transactions.moveFromMinedToConfirmed(transaction.hash)
      return {
        code: SendTransactionStatus.SUCCESS,
        tx: { hash: transaction.hash }
      }
    } catch (error) {
      log('error: transaction with nonce %d and hash failed to send: %s', populatedTx.nonce, transaction.hash, error)
      throw error
    }
  }

  /**
   * Initiates a transaction that announces nodes on-chain.
   * @param multiaddr the address to be announced
   * @param txHandler handler to call once the transaction has been published
   * @returns a Promise that resolves with the transaction hash
   */
  const announce = async (multiaddr: Multiaddr, txHandler: (tx: string) => DeferType<string>): Promise<string> => {
    log('Announcing on-chain with %s', multiaddr.toString())
    let sendResult: SendTransactionReturn
    let error: unknown
    try {
      const confirmationEssentialTxPayload = buildEssentialTxPayload(
        0,
        channels,
        'announce',
        publicKey.toUncompressedPubKeyHex(),
        multiaddr.bytes
      )
      sendResult = await sendTransaction(checkDuplicate, confirmationEssentialTxPayload, txHandler)
    } catch (err) {
      error = err
    }

    switch (sendResult.code) {
      case SendTransactionStatus.SUCCESS:
        return sendResult.tx.hash
      case SendTransactionStatus.DUPLICATE:
        throw new Error(`Failed in sending announce transaction because transaction is a duplicate`)
      default:
        throw new Error(`Failed in sending announce transaction due to ${error}`)
    }
  }

  /**
   * Initiates a transaction that withdraws funds of the node
   * @param currency either native token or Hopr token
   * @param recipient recipeint of the token transfer
   * @param amount amount of tokens to send
   * @param txHandler handler to call once the transaction has been published
   * @returns a Promise that resolves with the transaction hash
   */
  const withdraw = async (
    currency: 'NATIVE' | 'HOPR',
    recipient: string,
    amount: string,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<string> => {
    log('Withdrawing %s %s tokens', amount, currency)
    let sendResult: SendTransactionReturn
    let withdrawEssentialTxPayload: TransactionPayload
    let error: unknown
    try {
      switch (currency) {
        case 'NATIVE':
          withdrawEssentialTxPayload = buildEssentialTxPayload(amount, recipient, undefined)
          sendResult = await sendTransaction(checkDuplicate, withdrawEssentialTxPayload, txHandler)
          break
        case 'HOPR':
          withdrawEssentialTxPayload = buildEssentialTxPayload(0, token, 'transfer', recipient, amount)
          sendResult = await sendTransaction(checkDuplicate, withdrawEssentialTxPayload, txHandler)
          break
      }
    } catch (err) {
      error = err
    }

    switch (sendResult.code) {
      case SendTransactionStatus.SUCCESS:
        return sendResult.tx.hash
      case SendTransactionStatus.DUPLICATE:
        throw new Error(`Failed in sending withdraw transaction because transaction is a duplicate`)
      default:
        throw new Error(`Failed in sending withdraw transaction due to ${error}`)
    }
  }

  /**
   * Initiates a transaction that funds a payment channel
   * @param partyA first participant of the channel
   * @param partyB second participant of the channel
   * @param fundsA stake of first party
   * @param fundsB stake of second party
   * @param txHandler handler to call once the transaction has been published
   * @returns a Promise that resolves wiht the transaction hash
   */
  const fundChannel = async (
    partyA: Address,
    partyB: Address,
    fundsA: Balance,
    fundsB: Balance,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> => {
    const totalFund = fundsA.toBN().add(fundsB.toBN())
    log(
      'Funding channel from %s with %s HOPR to %s with %s HOPR',
      partyA.toHex(),
      fundsA.toFormattedString(),
      partyB.toHex(),
      fundsB.toFormattedString()
    )
    let sendResult: SendTransactionReturn
    let error: unknown
    try {
      const fundChannelEssentialTxPayload = buildEssentialTxPayload(
        0,
        token,
        'send',
        channels.address,
        totalFund.toString(),
        abiCoder.encode(
          ['address', 'address', 'uint256', 'uint256'],
          [partyA.toHex(), partyB.toHex(), fundsA.toBN().toString(), fundsB.toBN().toString()]
        )
      )
      sendResult = await sendTransaction(checkDuplicate, fundChannelEssentialTxPayload, txHandler)
    } catch (err) {
      error = err
    }

    switch (sendResult.code) {
      case SendTransactionStatus.SUCCESS:
        return sendResult.tx.hash
      case SendTransactionStatus.DUPLICATE:
        throw new Error(`Failed in sending fundChannel transaction because transaction is a duplicate`)
      default:
        throw new Error(`Failed in sending fundChannel transaction due to ${error}`)
    }
  }

  /**
   * Initiates a transaction that initiates the settlement of a payment channel
   * @param counterparty second participant of the channel
   * @param txHandler handler to call once the transaction has been published
   * @returns a Promise that resolves with the transaction hash
   */
  const initiateChannelClosure = async (
    counterparty: Address,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> => {
    log('Initiating channel closure to %s', counterparty.toHex())
    let sendResult: SendTransactionReturn
    let error: unknown

    try {
      const initiateChannelClosureEssentialTxPayload = buildEssentialTxPayload(
        0,
        channels,
        'initiateChannelClosure',
        counterparty.toHex()
      )
      sendResult = await sendTransaction(checkDuplicate, initiateChannelClosureEssentialTxPayload, txHandler)
    } catch (err) {
      error
    }

    switch (sendResult.code) {
      case SendTransactionStatus.SUCCESS:
        return sendResult.tx.hash
      case SendTransactionStatus.DUPLICATE:
        throw new Error(`Failed in sending initiateChannelClosure transaction because transaction is a duplicate`)
      default:
        throw new Error(`Failed in sending initiateChannelClosure transaction due to ${error}`)
    }

    // TODO: catch race-condition
  }

  /**
   * Initiates a transaction that performs the second step to settle
   * a payment channel.
   * @param counterparty second participant of the payment channel
   * @param txHandler handler to call once the transaction has been published
   * @returns a Promise that resolves with the transaction hash
   */
  const finalizeChannelClosure = async (
    counterparty: Address,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> => {
    log('Finalizing channel closure to %s', counterparty.toHex())
    let sendResult: SendTransactionReturn
    let error: unknown

    try {
      const finalizeChannelClosureEssentialTxPayload = buildEssentialTxPayload(
        0,
        channels,
        'finalizeChannelClosure',
        counterparty.toHex()
      )
      sendResult = await sendTransaction(checkDuplicate, finalizeChannelClosureEssentialTxPayload, txHandler)
    } catch (err) {
      error = err
    }

    switch (sendResult.code) {
      case SendTransactionStatus.SUCCESS:
        return sendResult.tx.hash
      case SendTransactionStatus.DUPLICATE:
        throw new Error(`Failed in sending finalizeChannelClosure transaction because transaction is a duplicate`)
      default:
        throw new Error(`Failed in sending finalizeChannelClosure transaction due to ${error}`)
    }
    // TODO: catch race-condition
  }

  /**
   * Initiates a transaction that redeems an acknowledged ticket
   * @param counterparty second participant
   * @param ackTicket the acknowledged ticket to reedeem
   * @param txHandler handler to call once the transaction has been published
   * @returns a Promise that resolve with the transaction hash
   */
  const redeemTicket = async (
    counterparty: Address,
    ackTicket: AcknowledgedTicket,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> => {
    log('Redeeming ticket for challenge %s in channel to %s', ackTicket.ticket.challenge.toHex(), counterparty.toHex())

    let sendResult: SendTransactionReturn
    let error: unknown
    try {
      const redeemTicketEssentialTxPayload = buildEssentialTxPayload(
        0,
        channels,
        'redeemTicket',
        counterparty.toHex(),
        ackTicket.preImage.toHex(),
        ackTicket.ticket.epoch.toHex(),
        ackTicket.ticket.index.toHex(),
        ackTicket.response.toHex(),
        ackTicket.ticket.amount.toBN().toString(),
        ackTicket.ticket.winProb.toBN().toString(),
        ackTicket.ticket.signature.toHex()
      )
      sendResult = await sendTransaction(checkDuplicate, redeemTicketEssentialTxPayload, txHandler)
    } catch (err) {
      error = err
    }

    switch (sendResult.code) {
      case SendTransactionStatus.SUCCESS:
        return sendResult.tx.hash
      case SendTransactionStatus.DUPLICATE:
        throw new Error(`Failed in sending redeemticket transaction because transaction is a duplicate`)
      default:
        throw new Error(`Failed in sending redeemticket transaction due to ${error}`)
    }
  }

  /**
   * Initiates a transaction that sets a commitment
   * @param counterparty second participant of the payment channel
   * @param commitment value to deposit on-chain
   * @param txHandler handler to call once the transaction has been published
   * @returns a Promise that resolves with the transaction hash
   */
  const setCommitment = async (
    counterparty: Address,
    commitment: Hash,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> => {
    log('Setting commitment %s in channel to %s', commitment.toHex(), counterparty.toHex())
    let sendResult: SendTransactionReturn
    let error: unknown

    try {
      const setCommitmentEssentialTxPayload = buildEssentialTxPayload(
        0,
        channels,
        'bumpChannel',
        counterparty.toHex(),
        commitment.toHex()
      )
      sendResult = await sendTransaction(checkDuplicate, setCommitmentEssentialTxPayload, txHandler)
    } catch (err) {
      error = err
    }

    switch (sendResult.code) {
      case SendTransactionStatus.SUCCESS:
        return sendResult.tx.hash
      case SendTransactionStatus.DUPLICATE:
        throw new Error(`Failed in sending commitment transaction because transaction is a duplicate`)
      default:
        throw new Error(`Failed in sending commitment transaction due to ${error}`)
    }
  }

  /**
   * Gets the transaction hashes of a specific block
   * @param blockNumber block number to look for
   * @returns a Promise that resolves with the transaction hashes of the requested block
   */
  const getTransactionsInBlock = async (blockNumber: number): Promise<string[]> => {
    let block: Block
    const RETRIES = 3
    for (let i = 0; i < RETRIES; i++) {
      try {
        block = await provider.getBlock(blockNumber)
      } catch (err) {
        if (i + 1 < RETRIES) {
          // Give other tasks CPU time to happen
          // Push next provider query to end of next event loop iteration
          await setImmediatePromise()
          continue
        }

        log(`could not retrieve native token transactions from block ${blockNumber} using the provider.`, err)
        throw err
      }
    }

    return block.transactions
  }

  /**
   * Gets the timestamp of a block
   * @param blockNumber block number to look for
   * @returns a Promise that resolves with the transaction hashes of the requested block
   */
  const getTimestamp = async function (blockNumber: number): Promise<number> {
    let block: Block

    const RETRIES = 3
    for (let i = 0; i < RETRIES; i++) {
      try {
        block = await provider.getBlock(blockNumber)
      } catch (err) {
        if (i + 1 < RETRIES) {
          // Give other tasks CPU time to happen
          // Push next provider query to end of next event loop iteration
          await setImmediatePromise()
          continue
        }

        log(`could not retrieve native token transactions from block ${blockNumber} using the provider.`, err)
        throw err
      }
    }

    return block.timestamp
  }

  /**
   * Gets the token balance of a specific account
   * @param accountAddress account to query for
   * @returns a Promise that resolves with the token balance
   */
  const getBalance = async (accountAddress: Address): Promise<Balance> => {
    const RETRIES = 3
    let rawBalance: BigNumber
    for (let i = 0; i < RETRIES; i++) {
      try {
        rawBalance = await token.balanceOf(accountAddress.toHex())
      } catch (err) {
        if (i + 1 < RETRIES) {
          await setImmediatePromise()
          continue
        }

        log(`Could not determine current on-chain token balance using the provider.`)
        throw Error(`Could not determine on-chain token balance`)
      }
    }

    return new Balance(new BN(rawBalance.toString()))
  }

  /**
   * Gets the native balance of a specific account
   * @param accountAddress account to query for
   * @returns a Promise that resolves with the native balance of the account
   */
  const getNativeBalance = async (accountAddress: Address): Promise<Balance> => {
    const RETRIES = 3
    let rawNativeBalance: BigNumber
    for (let i = 0; i < RETRIES; i++) {
      try {
        rawNativeBalance = await provider.getBalance(accountAddress.toHex())
      } catch (err) {
        if (i + 1 < RETRIES) {
          await setImmediatePromise()
          continue
        }

        log(`Could not determine current on-chain native balance using the provider.`)
        throw Error(`Could not determine on-chain native balance`)
      }
    }

    return new NativeBalance(new BN(rawNativeBalance.toString()))
  }

  return {
    getBalance,
    getNativeBalance,
    getTransactionsInBlock,
    getTimestamp,
    announce,
    withdraw,
    fundChannel,
    finalizeChannelClosure,
    initiateChannelClosure,
    redeemTicket,
    getGenesisBlock: () => genesisBlock,
    setCommitment,
    sendTransaction, //: provider.sendTransaction.bind(provider) as typeof provider['sendTransaction'],
    waitUntilReady: async () => await provider.ready,
    getLatestBlockNumber, // TODO: use indexer when it's done syncing
    subscribeBlock,
    subscribeError: (cb: (err: any) => void | Promise<void>): (() => void) => {
      provider.on('error', cb)
      channels.on('error', cb)
      token.on('error', cb)
      networkRegistry.on('error', cb)

      return () => {
        provider.off('error', cb)
        channels.off('error', cb)
        token.off('error', cb)
        networkRegistry.off('error', cb)
      }
    },
    unsubscribe: () => {
      provider.removeAllListeners()
      channels.removeAllListeners()
      token.removeAllListeners()
      networkRegistry.removeAllListeners()
    },
    getChannels: () => channels,
    getToken: () => token,
    getNetworkRegistry: () => networkRegistry,
    getPrivateKey: () => privateKey,
    getPublicKey: () => publicKey,
    getInfo: () => ({
      network: networkInfo.network,
      hoprTokenAddress: deploymentExtract.hoprTokenAddress,
      hoprChannelsAddress: deploymentExtract.hoprChannelsAddress,
      hoprNetworkRegistryAddress: deploymentExtract.hoprNetworkRegistryAddress,
      channelClosureSecs
    }),
    updateConfirmedTransaction: transactions.moveToConfirmed.bind(
      transactions
    ) as TransactionManager['moveToConfirmed'],
    getAllUnconfirmedHash: transactions.getAllUnconfirmedHash.bind(
      transactions
    ) as TransactionManager['getAllUnconfirmedHash'],
    getAllQueuingTransactionRequests: transactions.getAllQueuingTxs.bind(
      transactions
    ) as TransactionManager['getAllQueuingTxs']
  }
}
