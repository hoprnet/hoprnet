import { EventEmitter } from 'events'
import { Injectable } from '@nestjs/common'
import { ConfigService } from '@nestjs/config'
import { default as dotenvParseVariables } from 'dotenv-parse-variables'
import { default as connector } from '@hoprnet/hopr-core-ethereum'
import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import type { Channel } from '@hoprnet/hopr-core-connector-interface'
import { u8aToHex, moveDecimalPoint } from '@hoprnet/hopr-utils'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'
import * as rlp from 'rlp'
import { ParserService } from './parser/parser.service'
import { mustBeStarted, getMyOpenChannels } from './core.utils'
import { pubKeyToPeerId } from '@hoprnet/hopr-core/lib/utils' // @TODO: expose unofficial API

export type StartOptions = {
  debug?: boolean
  id?: number
  bootstrapNode?: boolean
  host?: string
  bootstrapServers?: string[]
  provider?: string
}

@Injectable()
export class CoreService {
  private node: Hopr<HoprCoreConnector>
  private events = new EventEmitter()

  constructor(private configService: ConfigService, private parserService: ParserService) {}

  @mustBeStarted()
  private async getChannel(channelId: string) {
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

    const envOptions = dotenvParseVariables({
      debug: this.configService.get('DEBUG'),
      id: this.configService.get('ID'),
      bootstrapNode: this.configService.get('BOOTSTRAP_NODE'),
      host: this.configService.get('CORE_HOST'),
      bootstrapServers: this.configService.get('BOOTSTRAP_SERVERS'),
      provider: this.configService.get('PROVIDER'),
    }) as StartOptions

    const options: HoprOptions = {
      id: envOptions.id,
      debug: envOptions.debug ?? true,
      bootstrapNode: envOptions.bootstrapNode ?? false,
      network: 'ethereum',
      connector,
      bootstrapServers: await Promise.all<PeerInfo>(
        (
          envOptions.bootstrapServers ?? [
            '/ip4/34.65.177.154/tcp/9091/p2p/16Uiu2HAm4FcroWGzc9yhDAsKSGC8W9yoDKiQBnAGK5aQdqJWmior',
          ]
        ).map((multiaddr) => this.parserService.parseBootstrap(multiaddr) as Promise<PeerInfo>),
      ),
      provider: envOptions.provider ?? 'wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36',
      hosts: (await this.parserService.parseHost(envOptions.host ?? '0.0.0.0:9091')) as HoprOptions['hosts'],
      password: 'switzerland',
      // @TODO: deprecate this, refactor hopr-core to not expect an output function
      output: this.parserService.outputFunctor(this.events),
    }
    console.log(':: HOPR Options ::', options)
    console.log(':: Starting HOPR Core Node ::')
    this.node = await Hopr.create(options)
    console.log(':: HOPR Core Node Started ::')
  }

  // @TODO: handle if already stopping
  async stop(): Promise<{ timestamp: number }> {
    if (!this.started) return

    console.log(':: Stopping HOPR Core Node ::')
    await this.node.stop()
    this.node = undefined
    console.log(':: HOPR Core Node Stopped ::')
    return {
      timestamp: +new Date(),
    }
  }

  @mustBeStarted()
  async getStatus(): Promise<{
    id: string
    multiAddresses: string[]
    connectedNodes: number
  }> {
    try {
      await this.node.network.crawler.crawl()
    } catch {}

    const id = this.node.peerInfo.id.toB58String()
    const multiAddresses = this.node.peerInfo.multiaddrs.toArray().map((multiaddr) => multiaddr.toString())

    const connectedNodes = this.node.network.peerStore.peers.length

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
    const latency = await this.node.ping(PeerId.createFromB58String(peerId))

    return {
      latency,
    }
  }

  @mustBeStarted()
  async getBalance(type: 'native' | 'hopr'): Promise<string> {
    const { paymentChannels } = this.node
    const { Balance, NativeBalance } = paymentChannels.types

    if (type === 'native') {
      return paymentChannels.account.nativeBalance.then((b) => {
        return moveDecimalPoint(b.toString(), NativeBalance.DECIMALS * -1)
      })
    } else {
      return paymentChannels.account.balance.then((b) => {
        return moveDecimalPoint(b.toString(), Balance.DECIMALS * -1)
      })
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

  // @TODO: rename 'getChannelInfo' to 'getChannel'
  @mustBeStarted()
  async getChannelInfo(
    channelId: string,
  ): Promise<{
    balance: string
    state: number
  }> {
    const channel = await this.getChannel(channelId)

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

    // @TODO: update proto to provide open channel funding amount
    const channelBalance = ChannelBalance.create(
      undefined,
      selfIsPartyA
        ? {
            balance: channelFunding,
            balance_a: channelFunding,
          }
        : {
            balance: channelFunding,
            // @ts-ignore
            balance_a: new Balance(0),
          },
    )

    await connector.channel.create(
      counterPartyPubKey,
      async () => pubKeyToAccountId(await this.node.interactions.payments.onChainKey.interact(counterParty)),
      channelBalance,
      (balance: Types.ChannelBalance): Promise<Types.SignedChannel> => {
        return this.node.interactions.payments.open.interact(counterParty, balance)
      },
    )

    return channelId
  }

  // @TODO: improve proto: should return txHash
  @mustBeStarted()
  async closeChannel(channelId: string): Promise<string> {
    const channel = await this.getChannel(channelId)

    await channel.instance.initiateSettlement()

    return channelId
  }

  // @TODO: improve proto: implement manual intermediate peer ids
  @mustBeStarted()
  async send({
    peerId,
    payload,
  }: {
    peerId: string
    payload: Uint8Array
  }): Promise<{
    intermediatePeerIds: string[]
  }> {
    // @TODO: this should be done by hopr-core
    const message = rlp.encode([payload, Date.now()])

    await this.node.sendMessage(message, PeerId.createFromB58String(peerId), async () => [])

    return {
      intermediatePeerIds: [],
    }
  }

  // @TODO: support filter by peerId, hopr-core needs refactor
  @mustBeStarted()
  async listen({ peerId }: { peerId?: string }): Promise<EventEmitter> {
    return this.events
  }
}
