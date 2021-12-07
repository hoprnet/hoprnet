import type { Multiaddr } from 'multiaddr'
import type PeerId from 'peer-id'
import { ChainWrapper } from './ethereum'
import chalk from 'chalk'
import { debug, privKeyToPeerId } from '@hoprnet/hopr-utils'
import {
  AcknowledgedTicket,
  PublicKey,
  Balance,
  Address,
  NativeBalance,
  cacheNoArgAsyncFunction,
  HoprDB,
  ChannelEntry,
  ChannelStatus,
  generateChannelId,
  Hash
} from '@hoprnet/hopr-utils'
import Indexer from './indexer'
import { CONFIRMATIONS, INDEXER_BLOCK_RANGE } from './constants'
import { createChainWrapper } from './ethereum'
import { PROVIDER_CACHE_TTL } from './constants'
import { EventEmitter } from 'events'
import { initializeCommitment, findCommitmentPreImage, bumpCommitment, ChannelCommitmentInfo } from './commitment'
import { chainMock } from './index.mock'
import { useFixtures } from './indexer/index.mock'
import { sampleChainOptions } from './ethereum.mock'
import ChainWrapperSingleton from './chain'

const log = debug('hopr-core-ethereum')

export type RedeemTicketResponse =
  | {
    status: 'SUCCESS'
    receipt: string
    ackTicket: AcknowledgedTicket
  }
  | {
    status: 'FAILURE'
    message: string
  }
  | {
    status: 'ERROR'
    error: Error | string
  }

export type ChainStatus = 'UNINITIALIZED' | 'CREATING' | 'CREATED' | 'STARTING' | 'STOPPED'
export default class HoprEthereum extends EventEmitter {
  public status: ChainStatus = 'UNINITIALIZED'
  public indexer: Indexer
  private chain: ChainWrapper
  private started: Promise<HoprEthereum> | undefined
  private redeemingAll: Promise<void> | undefined = undefined

  constructor(
    //private chain: ChainWrapper, private db: HoprDB, public indexer: Indexer) {
    private db: HoprDB,
    private publicKey: PublicKey,
    private privateKey: Uint8Array,
    private options: {
      provider: string
      maxConfirmations?: number
      chainId: number
      gasPrice?: number
      network: string
      environment: string
    },
  ) {
    super()
    this.indexer = new Indexer(
      this.publicKey.toAddress(),
      this.db,
      this.options?.maxConfirmations ?? CONFIRMATIONS,
      INDEXER_BLOCK_RANGE
    )
    this.status = 'CREATING'
    log('Ready to call singleton wrapper', ChainWrapperSingleton)
    ChainWrapperSingleton.create(this.options, this.privateKey).then((chain: ChainWrapper) => {
      log('Chain instance created or passed via singleton')
      this.status = 'CREATED'
      this.chain = chain
    })
  }

  async start(): Promise<HoprEthereum> {
    if (this.started) {
      return this.started
    }

    const _start = async (): Promise<HoprEthereum> => {
      await this.chain.waitUntilReady()
      await this.indexer.start(this.chain, this.chain.getGenesisBlock())

      // Debug log used in e2e integration tests, please don't change
      log(`using blockchain address ${this.publicKey.toAddress().toHex()}`)
      log(chalk.green('Connector started'))
      return this
    }
    this.started = _start()
    return this.started
  }

  public getChain(): ChainWrapper {
    return this.chain
  }

  readonly CHAIN_NAME = 'HOPR on Ethereum'

  /**
   * Stops the connector.
   */
  async stop(): Promise<void> {
    log('Stopping connector...')
    await this.indexer.stop()
  }

  async announce(multiaddr: Multiaddr): Promise<string> {
    // promise of tx hash gets resolved when the tx is mined.
    const tx = await this.chain.announce(multiaddr)
    // event emitted by the indexer
    return this.indexer.resolvePendingTransaction('announce', tx)
  }

  async withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    // promise of tx hash gets resolved when the tx is mined.
    const tx = await this.chain.withdraw(currency, recipient, amount)
    // event emitted by the indexer
    return this.indexer.resolvePendingTransaction(currency === 'NATIVE' ? 'withdraw-native' : 'withdraw-hopr', tx)
  }

  public getOpenChannelsFrom(p: PublicKey) {
    return this.indexer.getOpenChannelsFrom(p)
  }

  public async getAccount(addr: Address) {
    return this.indexer.getAccount(addr)
  }

  public getPublicKeyOf(addr: Address) {
    return this.indexer.getPublicKeyOf(addr)
  }

  public getRandomOpenChannel() {
    return this.indexer.getRandomOpenChannel()
  }

  private uncachedGetBalance = () => this.chain.getBalance(this.publicKey.toAddress())
  private cachedGetBalance = cacheNoArgAsyncFunction<Balance>(this.uncachedGetBalance, PROVIDER_CACHE_TTL)
  /**
   * Retrieves HOPR balance, optionally uses the cache.
   * @returns HOPR balance
   */
  public async getBalance(useCache: boolean = false): Promise<Balance> {
    return useCache ? this.cachedGetBalance() : this.uncachedGetBalance()
  }

  public getPublicKey() {
    return this.publicKey
  }

  /**
   * Retrieves ETH balance, optionally uses the cache.
   * @returns ETH balance
   */
  private uncachedGetNativeBalance = () => this.chain.getNativeBalance(this.publicKey.toAddress())
  private cachedGetNativeBalance = cacheNoArgAsyncFunction<NativeBalance>(
    this.uncachedGetNativeBalance,
    PROVIDER_CACHE_TTL
  )
  public async getNativeBalance(useCache: boolean = false): Promise<NativeBalance> {
    return useCache ? this.cachedGetNativeBalance() : this.uncachedGetNativeBalance()
  }

  public smartContractInfo(): {
    network: string
    hoprTokenAddress: string
    hoprChannelsAddress: string
    channelClosureSecs: number
  } {
    return this.chain.getInfo()
  }

  public async waitForPublicNodes(): Promise<{ id: PeerId; multiaddrs: Multiaddr[] }[]> {
    return await this.indexer.getPublicNodes()
  }

  public async commitToChannel(c: ChannelEntry): Promise<void> {
    log('committing to channel', c)
    const setCommitment = async (commitment: Hash) => {
      const tx = await this.chain.setCommitment(c.source.toAddress(), commitment)
      return this.indexer.resolvePendingTransaction('channel-updated', tx)
    }
    const getCommitment = async () => (await this.db.getChannel(c.getId())).commitment

    // Get all channel information required to build the initial commitment
    const cci = new ChannelCommitmentInfo(
      this.options.chainId,
      this.smartContractInfo().hoprChannelsAddress,
      c.getId(),
      c.channelEpoch
    )

    await initializeCommitment(this.db, privKeyToPeerId(this.privateKey), cci, getCommitment, setCommitment)
  }

  public async redeemAllTickets(): Promise<void> {
    if (this.redeemingAll) {
      return this.redeemingAll
    }
    const _redeemAll = async () => {
      for (const ce of await this.db.getChannelsTo(this.publicKey.toAddress())) {
        await this.redeemTicketsInChannel(ce)
      }
      this.redeemingAll = undefined
    }
    this.redeemingAll = _redeemAll()
    return this.redeemingAll
  }

  private async redeemTicketsInChannel(channel: ChannelEntry) {
    if (!channel.destination.eq(this.getPublicKey())) {
      throw new Error('Cannot redeem ticket in channel that isnt to us')
    }
    // Because tickets are ordered and require the previous redemption to
    // have succeeded before we can redeem the next, we need to do this
    // sequentially.
    const tickets = await this.db.getAcknowledgedTickets({ channel })
    log(`redeeming ${tickets.length} tickets from ${channel.source.toB58String()}`)
    try {
      for (const ticket of tickets) {
        log('redeeming ticket', ticket)
        const result = await this.redeemTicket(channel.source, ticket)
        if (result.status !== 'SUCCESS') {
          log('Error redeeming ticket', result)
          // We need to abort as tickets require ordered redemption.
          return
        }
        log('ticket was redeemed')
      }
    } catch (e) {
      // We are going to swallow the error here, as more than one consumer may
      // be inspecting this same promise.
      log('Error when redeeming tickets, aborting', e)
    }
    log(`redemption of tickets from ${channel.source.toB58String()} is complete`)
  }

  // Private as out of order redemption will break things - redeem all at once.
  private async redeemTicket(counterparty: PublicKey, ackTicket: AcknowledgedTicket): Promise<RedeemTicketResponse> {
    if (!ackTicket.verify(counterparty)) {
      return {
        status: 'FAILURE',
        message: 'Invalid response to acknowledgement'
      }
    }

    try {
      const ticket = ackTicket.ticket

      log('Submitting ticket', ackTicket.response.toHex())
      const emptyPreImage = new Hash(new Uint8Array(Hash.SIZE).fill(0x00))
      const hasPreImage = !ackTicket.preImage.eq(emptyPreImage)
      if (!hasPreImage) {
        log(`Failed to submit ticket ${ackTicket.response.toHex()}: 'PreImage is empty.'`)
        return {
          status: 'FAILURE',
          message: 'PreImage is empty.'
        }
      }

      const isWinning = ticket.isWinningTicket(ackTicket.preImage, ackTicket.response, ticket.winProb)

      if (!isWinning) {
        log(`Failed to submit ticket ${ackTicket.response.toHex()}:  'Not a winning ticket.'`)
        return {
          status: 'FAILURE',
          message: 'Not a winning ticket.'
        }
      }

      const receipt = await this.chain.redeemTicket(counterparty.toAddress(), ackTicket, ticket)
      await this.indexer.resolvePendingTransaction('channel-updated', receipt)

      log('Successfully submitted ticket', ackTicket.response.toHex())
      await this.db.markRedeemeed(ackTicket)
      this.emit('ticket:redeemed', ackTicket)
      return {
        status: 'SUCCESS',
        receipt,
        ackTicket
      }
    } catch (err) {
      // TODO delete ackTicket -- check if it's due to gas!
      log('Unexpected error when redeeming ticket', ackTicket.response.toHex(), err)
      return {
        status: 'ERROR',
        error: err
      }
    }
  }

  async initializeClosure(dest: PublicKey): Promise<string> {
    const c = await this.db.getChannelTo(dest)
    if (c.status !== ChannelStatus.Open && c.status !== ChannelStatus.WaitingForCommitment) {
      throw Error('Channel status is not OPEN or WAITING FOR COMMITMENT')
    }
    const tx = await this.chain.initiateChannelClosure(dest.toAddress())
    return await this.indexer.resolvePendingTransaction('channel-updated', tx)
  }

  public async finalizeClosure(dest: PublicKey): Promise<string> {
    const c = await this.db.getChannelTo(dest)
    if (c.status !== ChannelStatus.PendingToClose) {
      throw Error('Channel status is not PENDING_TO_CLOSE')
    }
    return await this.chain.finalizeChannelClosure(dest.toAddress())
  }

  public async openChannel(dest: PublicKey, amount: Balance): Promise<Hash> {
    // channel may not exist, we can still open it
    let c: ChannelEntry
    try {
      c = await this.db.getChannelTo(dest)
    } catch { }
    if (c && c.status !== ChannelStatus.Closed) {
      throw Error('Channel is already opened')
    }

    const myBalance = await this.getBalance()
    if (myBalance.lt(amount)) {
      throw Error('We do not have enough balance to open a channel')
    }
    const tx = await this.chain.openChannel(this.publicKey.toAddress(), dest.toAddress(), amount)
    await this.indexer.resolvePendingTransaction('channel-updated', tx)
    return generateChannelId(this.publicKey.toAddress(), dest.toAddress())
  }

  public async fundChannel(dest: PublicKey, myFund: Balance, counterpartyFund: Balance) {
    const totalFund = myFund.add(counterpartyFund)
    const myBalance = await this.getBalance()
    if (totalFund.gt(myBalance)) {
      throw Error('We do not have enough balance to fund the channel')
    }
    const tx = await this.chain.fundChannel(this.publicKey.toAddress(), dest.toAddress(), myFund, counterpartyFund)
    return await this.indexer.resolvePendingTransaction('channel-updated', tx)
  }
}

export {
  ChannelEntry,
  ChannelCommitmentInfo,
  Indexer,
  ChainWrapperSingleton,
  chainMock,
  createChainWrapper,
  initializeCommitment,
  findCommitmentPreImage,
  bumpCommitment,
  INDEXER_BLOCK_RANGE,
  CONFIRMATIONS,
  useFixtures,
  sampleChainOptions
}
