import { setImmediate as setImmediatePromise } from 'timers/promises'
import type { Multiaddr } from 'multiaddr'
import {
  providers,
  utils,
  errors,
  Wallet,
  BigNumber,
  ethers,
  type ContractTransaction,
  type BaseContract
} from 'ethers'
import { getContractData, type HoprToken, type HoprChannels } from '@hoprnet/hopr-ethereum'
import {
  Address,
  Ticket,
  Balance,
  NativeBalance,
  PublicKey,
  durations,
  type AcknowledgedTicket,
  type DeferType,
  type Hash
} from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import NonceTracker from './nonce-tracker'
import TransactionManager, { type TransactionPayload } from './transaction-manager'
import { debug } from '@hoprnet/hopr-utils'
import { TX_CONFIRMATION_WAIT } from './constants'
import type { BlockWithTransactions } from '@ethersproject/abstract-provider'

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

  const genesisBlock = parseInt(hoprChannelsDeployment.blockNumber)
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
  const sendTransaction = async <T extends BaseContract>(
    checkDuplicate: Boolean,
    contract: T,
    method: keyof T['functions'],
    handleTxListener: (tx: string) => DeferType<string>,
    ...rest: Parameters<T['functions'][keyof T['functions']]>
  ): Promise<SendTransactionReturn> => {
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
      // throws various exceptions if tx gets rejected
      transaction = await provider.sendTransaction(signedTx)
    } catch (error) {
      log('Transaction with nonce %d failed to sent: %s', nonce, error)
      deferredListener && deferredListener.reject()
      // @TODO what if signing the transaction failed and initiatedHash is undefined?
      initiatedHash && transactions.remove(initiatedHash)
      nonceLock.releaseLock()

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

      throw new Error(`Failed in publishing transaction. ${error}`)
    }

    log('Transaction with nonce %d successfully sent %s, waiting for confimation', nonce, transaction.hash)
    transactions.moveFromQueuingToPending(transaction.hash)
    nonceLock.releaseLock()

    try {
      // wait for the tx to be mined - mininal and scheduled implementation
      // only fails if tx does not get mined within the specified timeout
      await new Promise<void>((resolve, reject) => {
        let done = false
        const cleanUp = (err?: string) => {
          if (done) {
            return
          }
          done = true

          provider.off(transaction.hash, onTransaction)
          // Give other tasks time to get scheduled before
          // processing the result
          if (err) {
            setImmediate(reject, Error(err))
          } else {
            setImmediate(resolve)
          }
        }

        const onTransaction = (receipt: providers.TransactionReceipt) => {
          if (receipt.confirmations >= 1) {
            cleanUp()
          }
        }
        setTimeout(cleanUp, timeout, `Timeout while waiting for transaction ${transaction.hash}`)

        provider.on(transaction.hash, onTransaction)
      })
    } catch (error) {
      log(`Error while waiting for transaction ${transaction.hash}`, error)
      // remove listener but not throwing error message
      deferredListener.reject()
      // this transaction was not confirmed so we just remove it
      transactions.remove(transaction.hash)

      throw error
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

  /*
   * Sends announce transaction on-chain synchronously.
   * Throws when an error during transaction sending is encountered.
   *
   * @param the address to be announced
   * @param callback transaction handler
   * @returns a Promise that resolve to the hash of the on-chain transaction
   */
  const announce = async (multiaddr: Multiaddr, txHandler: (tx: string) => DeferType<string>): Promise<string> => {
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

  const withdraw = async (
    currency: 'NATIVE' | 'HOPR',
    recipient: string,
    amount: string,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<string> => {
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

  const fundChannel = async (
    me: Address,
    counterparty: Address,
    myFund: Balance,
    counterpartyFund: Balance,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> => {
    const totalFund = myFund.toBN().add(counterpartyFund.toBN())

    try {
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

  const openChannel = async (
    me: Address,
    counterparty: Address,
    amount: Balance,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> => {
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

  const finalizeChannelClosure = async (
    counterparty: Address,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> => {
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

  const initiateChannelClosure = async (
    counterparty: Address,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> => {
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

  const redeemTicket = async (
    counterparty: Address,
    ackTicket: AcknowledgedTicket,
    ticket: Ticket,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> => {
    try {
      const transaction = await sendTransaction(
        checkDuplicate,
        channels,
        'redeemTicket',
        txHandler,
        counterparty.toHex(),
        ackTicket.preImage.serialize(),
        ackTicket.ticket.epoch.serialize(),
        ackTicket.ticket.index.serialize(),
        ackTicket.response.serialize(),
        ticket.amount.toBN().toString(),
        ticket.winProb.toBN().toString(),
        ticket.signature.serialize()
      )
      return transaction.tx.hash
    } catch (error) {
      throw new Error(`Failed in sending redeemticket transaction ${error}`)
    }
  }

  const setCommitment = async (
    counterparty: Address,
    commitment: Hash,
    txHandler: (tx: string) => DeferType<string>
  ): Promise<Receipt> => {
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

  const getNativeTokenTransactionInBlock = async (
    blockNumber: number,
    isOutgoing: boolean = true
  ): Promise<Array<string>> => {
    let blockWithTxs: BlockWithTransactions
    const RETRIES = 3
    for (let i = 0; i < RETRIES; i++) {
      try {
        blockWithTxs = await provider.getBlockWithTransactions(blockNumber)
      } catch (err) {
        if (i + 1 < RETRIES) {
          // Give other tasks CPU time to happen
          // Push next provider query to end of next event loop iteration
          await setImmediatePromise()
          continue
        } else {
          log(
            `could not retrieve native token transaction from block ${blockNumber} using the provider, assuming there are none`,
            err
          )
          return []
        }
      }
    }

    const txs = blockWithTxs.transactions.filter(
      (tx) => tx.value.gt(BigNumber.from(0)) && (isOutgoing ? tx.from : tx.to) === wallet.address
    )
    return txs.length === 0 ? [] : txs.map((tx) => tx.hash)
  }

  const getLatestBlockNumber = async (): Promise<number> => {
    let latestOnChainBlock: number
    const RETRIES = 3
    for (let i = 0; i < RETRIES; i++) {
      try {
        latestOnChainBlock = await provider.getBlockNumber()
        return latestOnChainBlock
      } catch (err) {
        if (i + 1 < RETRIES) {
          await setImmediatePromise()
          continue
        } else {
          log(`Could not determine latest on-chain block. Now waiting for next block.`)
        }
      }
    }

    if (latestOnChainBlock == undefined) {
      await new Promise<void>((resolve) => {
        const unsubscribeBlock = api.subscribeBlock((blockNumber: number) => {
          latestOnChainBlock = blockNumber
          unsubscribeBlock()
          resolve()
        })
      })
    }
  }

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
        } else {
          log(`Could not determine current on-chain token balance using the provider.`)
          throw Error(`Could not determine on-chain token balance`)
        }
      }
    }

    return new Balance(new BN(rawBalance.toString()))
  }

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
        } else {
          log(`Could not determine current on-chain native balance using the provider.`)
          throw Error(`Could not determine on-chain native balance`)
        }
      }
    }

    return new NativeBalance(new BN(rawNativeBalance.toString()))
  }

  const api = {
    getBalance,
    getNativeBalance,
    getNativeTokenTransactionInBlock,
    announce,
    withdraw,
    fundChannel,
    openChannel,
    finalizeChannelClosure,
    initiateChannelClosure,
    redeemTicket,
    getGenesisBlock: () => genesisBlock,
    setCommitment,
    getWallet: () => wallet,
    waitUntilReady: async () => await provider.ready,
    getLatestBlockNumber, // TODO: use indexer when it's done syncing
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
    unsubscribe: () => {
      provider.removeAllListeners()
      channels.removeAllListeners()
      token.removeAllListeners()
    },
    getChannels: () => channels,
    getToken: () => token,
    getPrivateKey: () => privateKey,
    getPublicKey: () => PublicKey.fromPrivKey(privateKey),
    getInfo: () => ({
      network: networkInfo.network,
      hoprTokenAddress: hoprTokenDeployment.address,
      hoprChannelsAddress: hoprChannelsDeployment.address,
      channelClosureSecs
    }),
    updateConfirmedTransaction: (hash: string) => transactions.moveToConfirmed(hash),
    getAllQueuingTransactionRequests: transactions.getAllQueuingTxs.bind(
      transactions
    ) as TransactionManager['getAllQueuingTxs']
  }

  return api
}
