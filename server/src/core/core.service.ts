import { Injectable } from '@nestjs/common'
import { ConfigService } from '@nestjs/config'
import { default as dotenvParseVariables } from 'dotenv-parse-variables'
import { default as connector } from '@hoprnet/hopr-core-ethereum'
import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { u8aToHex, moveDecimalPoint } from '@hoprnet/hopr-utils'
import { ParserService } from './parser/parser.service'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'
import { mustBeStarted } from './core.utils'

export type StartOptions = {
  debug?: boolean
  id?: number
  bootstrapNode?: boolean
  host?: string
  bootstrapServers?: string[]
}

@Injectable()
export class CoreService {
  private node: Hopr<HoprCoreConnector>

  constructor(private configService: ConfigService, private parserService: ParserService) {}

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
      provider: 'wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36',
      hosts: (await this.parserService.parseHost(envOptions.host ?? '0.0.0.0:9091')) as HoprOptions['hosts'],
      password: 'switzerland',
      output: this.parserService.outputFunctor(),
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
}
