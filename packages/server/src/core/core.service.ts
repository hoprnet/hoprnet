import { EventEmitter } from 'events'
import { Injectable, Inject, Optional } from '@nestjs/common'
import { ConfigService } from '@nestjs/config'
import { default as dotenvParseVariables } from 'dotenv-parse-variables'
import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Channel, Types, Currencies } from '@hoprnet/hopr-core-connector-interface'
import { u8aToHex, getBootstrapAddresses } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import * as rlp from 'rlp'
import { ParserService } from './parser/parser.service'
import { mustBeStarted, getMyOpenChannels } from './core.utils'
import { pubKeyToPeerId } from '@hoprnet/hopr-core/lib/utils' // @TODO: expose unofficial API
import { PROVIDER_NAME as HOPR_NODE_PROVIDER } from '../node.module'

export type StartOptions = {
  debug_mode?: boolean
  id?: number
  bootstrapNode?: boolean
  host?: string
  bootstrapServers?: string[]
  provider?: string
}

@Injectable()
export class CoreService {
  private events = new EventEmitter()

  constructor(
    private configService: ConfigService,
    private parserService: ParserService,
    @Optional() @Inject(HOPR_NODE_PROVIDER) private node: Hopr<HoprCoreConnector>,
  ) {}

  @mustBeStarted()
  private async findChannel(channelId: string) {
    const channels = await this.getChannels()
    const channel = channels.find((channel) => {
      return channel.channelId === channelId
    })

    if (!channel) {
      throw Error(`Channel ${channelId} not found.`)
    }

    return channel
  }

  get started(): boolean {
    return !!this.node
  }

  // @TODO: handle if already starting
  async start(): Promise<void> {
    if (this.started) return
    if (typeof this.node !== 'undefined') return

    const envOptions = dotenvParseVariables({
      debug_mode: this.configService.get('DEBUG_MODE'),
      id: this.configService.get('ID'),
      bootstrapNode: this.configService.get('BOOTSTRAP_NODE'),
      host: this.configService.get('CORE_HOST'),
      bootstrapServers: this.configService.get('BOOTSTRAP_SERVERS'),
      provider: this.configService.get('PROVIDER'),
    }) as StartOptions

    // if only one server is provided, parser will parse it into a string
    if (typeof envOptions.bootstrapServers === 'string') {
      envOptions.bootstrapServers = [envOptions.bootstrapServers]
    }

    let bootstrapServers
    // At the moment, if it's run as a bootstrap node, we shouldn't add
    // boostrap nodes.
    if (!envOptions.bootstrapNode) {
      console.log(':: Starting a server ::', envOptions)
      const bootstrapServerMap = await getBootstrapAddresses(
        envOptions.bootstrapServers ? envOptions.bootstrapServers.join(',') : undefined,
      )
      bootstrapServers = [...bootstrapServerMap.values()]
    } else {
      bootstrapServers = []
    }

    const options = {
      id: envOptions.id,
      debug: envOptions.debug_mode ?? false,
      bootstrapNode: envOptions.bootstrapNode ?? false,
      network: 'ethereum',
      // using testnet bootstrap servers
      bootstrapServers: bootstrapServers,
      provider: envOptions.provider ?? 'wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36',
      host: envOptions.host ?? '0.0.0.0:9091',
      password: 'switzerland',
    }

    console.log(':: HOPR Options ::', options)
    console.log(':: Starting HOPR Core Node ::')

    try {
      this.node = await Hopr.create({
        id: options.id,
        debug: options.debug,
        bootstrapNode: options.bootstrapNode,
        network: options.network,
        bootstrapServers: options.bootstrapServers,
        provider: options.provider,
        hosts: (await this.parserService.parseHost(options.host)) as HoprOptions['hosts'],
        password: options.password,
        // @TODO: deprecate this, refactor hopr-core to not expect an output function
        output: this.parserService.outputFunctor(this.events),
      })
      console.log(':: HOPR Core Node Started ::')
    } catch (err) {
      console.log(`${err}`)
    }
  }

  // @TODO: handle if already stopping
  async stop(): Promise<{ timestamp: number }> {
    if (!this.started) return

    console.log(':: Stopping HOPR Core Node ::')
    await this.node.stop()
    this.node = undefined
    console.log(':: HOPR Core Node Stopped ::')
    return {
      timestamp: Math.floor(new Date().valueOf() / 1e3),
    }
  }

  @mustBeStarted()
  async getStatus(): Promise<{
    id: string
    multiAddresses: string[]
    connectedNodes: number
  }> {
    try {
      await this.node.crawl()
    } catch {}

    const id = this.node.peerInfo.id.toB58String()
    const multiAddresses = this.node.peerInfo.multiaddrs.toArray().map((multiaddr) => multiaddr.toString())

    const connectedNodes = this.node.getConnectedPeers().length

    return {
      id,
      multiAddresses,
      connectedNodes,
    }
  }

  @mustBeStarted()
  async getPing(
    peerId: string,
  ): Promise<{
    latency: number
  }> {
    const { latency } = await this.node.ping(PeerId.createFromB58String(peerId))

    return {
      latency,
    }
  }

  @mustBeStarted()
  async getBalance(type: 'native' | 'hopr'): Promise<string> {
    const { paymentChannels } = this.node

    if (type === 'native') {
      return (await paymentChannels.account.nativeBalance).toString()
    } else {
      return (await paymentChannels.account.balance).toString()
    }
  }

  @mustBeStarted()
  async getAddress(type: 'native' | 'hopr'): Promise<string> {
    if (type === 'native') {
      return this.node.paymentChannels.utils.pubKeyToAccountId(this.node.peerInfo.id.pubKey.marshal()).then(u8aToHex)
    } else {
      return this.node.peerInfo.id.toB58String()
    }
  }

  @mustBeStarted()
  async getChannels(): Promise<
    {
      channelId: string
      counterParty: string
      balance: string
      state: number
      instance: Channel
    }[]
  > {
    const channels = await getMyOpenChannels(this.node)

    return Promise.all(
      channels.map(async (channel) => {
        const channelId = u8aToHex(await channel.channelId)
        const counterParty = (await pubKeyToPeerId(await channel.offChainCounterparty)).toB58String()
        const balance = (await channel.balance).toString()

        return {
          channelId,
          counterParty,
          balance,
          state: 0, // @TODO: update h-c-e to provide state,
          instance: channel,
        }
      }),
    )
  }

  @mustBeStarted()
  async getChannelData(
    channelId: string,
  ): Promise<{
    balance: string
    state: number
  }> {
    const channel = await this.findChannel(channelId)

    return {
      balance: channel.balance,
      state: channel.state,
    }
  }

  // @TODO: improve proto: should return txHash
  @mustBeStarted()
  async openChannel(peerId: string): Promise<string> {
    const connector = this.node.paymentChannels
    const { isPartyA, getId, pubKeyToAccountId } = connector.utils
    const { ChannelBalance, Balance } = connector.types
    const self = this.node.peerInfo.id
    const selfPubKey = self.pubKey.marshal()
    const selfAccountId = await pubKeyToAccountId(selfPubKey)
    const counterParty = PeerId.createFromB58String(peerId)
    const counterPartyPubKey = counterParty.pubKey.marshal()
    const counterPartyAccountId = await pubKeyToAccountId(counterPartyPubKey)

    const selfIsPartyA = isPartyA(selfAccountId, counterPartyAccountId)
    const channelId = await getId(selfAccountId, counterPartyAccountId).then(u8aToHex)

    // @ts-ignore @TODO: properly export types in h-c-e
    const channelFunding = new Balance(10)
    
    await this.node.openChannel(counterParty, channelFunding)
    return channelId
  }

  // @TODO: improve proto: should return txHash
  @mustBeStarted()
  async closeChannel(channelId: string): Promise<string> {
    const channel = await this.findChannel(channelId)

    channel.instance.initiateSettlement()

    return channelId
  }

  // @TODO: improve proto: implement manual intermediate peer ids
  @mustBeStarted()
  async send({
    peerId,
    payload,
    intermediatePeerIds,
  }: {
    peerId: string
    payload: Uint8Array
    intermediatePeerIds: string[]
  }): Promise<{
    intermediatePeerIds: string[]
  }> {
    // @TODO: should this be done by hopr-core?
    const message = rlp.encode([payload, Date.now()])

    await this.node.sendMessage(message, PeerId.createFromB58String(peerId), async () =>
      intermediatePeerIds.map((str) => PeerId.createFromB58String(str)),
    )

    return {
      intermediatePeerIds,
    }
  }

  @mustBeStarted()
  async listen({ peerId }: { peerId?: string }): Promise<EventEmitter> {
    return this.events
  }

  @mustBeStarted()
  async withdraw({
    currency,
    recipient,
    amount,
  }: {
    currency: Currencies
    recipient: string
    amount: string
  }): Promise<Record<string, any>> {
    await this.node.paymentChannels.withdraw(currency, recipient, amount)

    return {}
  }
}
