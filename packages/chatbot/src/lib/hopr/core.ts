import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'
import HoprCoreConnector, { Currencies } from '@hoprnet/hopr-core-connector-interface'
import { getBootstrapAddresses, u8aToHex, parseHosts } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import { EventEmitter } from 'events'
import { encode, decode } from 'rlp'
import debug from 'debug'


const log = debug('hopr-chatbot:core')
const error = debug('hopr-chatbot:core:error')

export default class Core {
  public events: EventEmitter
  node: Hopr<HoprCoreConnector>
  options: HoprOptions
  protected started: Boolean

  static mustBeStarted(): MethodDecorator {
    return (_target: Core, _key: string, descriptor: TypedPropertyDescriptor<any>): TypedPropertyDescriptor<any> => {
      const originalFn = descriptor.value
      descriptor.value = function (...args: any[]) {
        if (!this.started) {
          throw Error('HOPR node is not started')
        }
        return originalFn.bind(this)(...args)
      }
      return descriptor
    }
  }

  private _functor(msg: Uint8Array) {
    log('- functor | Received message')
    try {
      const [decoded, time] = decode(msg) as [Buffer, Buffer]
      log('- functor | Message', decoded.toString())
      log('- functor | Latency', Date.now() - parseInt(time.toString('hex'), 16) + 'ms')
      this.events.emit('message', decoded)
    } catch (err) {
      error('- functor | Error: Could not decode message', err)
      error('- functor | Error: Message', msg.toString())
    }
  }

  constructor(
    options: HoprOptions = {
      hosts: parseHosts(),
      provider: process.env.HOPR_CHATBOT_PROVIDER || 'wss://xdai.poanetwork.dev/wss',
      network: process.env.HOPR_CHATBOT_NETWORK || 'ETHEREUM',
      debug: Boolean(process.env.HOPR_CHABOT_DEBUG) || false,
      password: process.env.HOPR_CHATBOT_PASSWORD || 'switzerland',
    },
  ) {
    this.options = { ...options, output: this._functor.bind(this) }
    this.events = new EventEmitter()
  }

  async start(): Promise<void> {
    try {
      log('- start | Creating HOPR Node')
      this.node = await Hopr.create({
        ...this.options,
        bootstrapServers: [...(await getBootstrapAddresses()).values()],
      })
      log('- start | Created HOPR Node')
      this.started = true
      log('- start | Started HOPR Node')
    } catch (err) {
      error('- start | Error: Unable to start node', err)
    }
  }

  @Core.mustBeStarted()
  async getHoprBalance(): Promise<string> {
      return (await this.node.paymentChannels.account.balance).toString()
  }

  @Core.mustBeStarted()
  async getBalance(): Promise<string> {
      return (await this.node.paymentChannels.account.nativeBalance).toString()
  }

  @Core.mustBeStarted()
  getBootstrapServers(): string {
      return this.node.bootstrapServers.map(node => node.id.toB58String()).join(',')
  }

  @Core.mustBeStarted()
  async withdraw({
    currency,
    recipient,
    amount,
  }: {
    currency: Currencies
    recipient: string
    amount: string
  }): Promise<string> {
      return await this.node.paymentChannels.withdraw(currency, recipient, amount)
  }

  @Core.mustBeStarted()
  listConnectedPeers(): number {
      return this.node.getConnectedPeers().length
  }

  @Core.mustBeStarted()
  async send({
    peerId,
    payload,
    intermediatePeerIds = [],
    includeRecipient = false
  }: {
    peerId: string
    payload: Uint8Array
    intermediatePeerIds?: string[]
    includeRecipient?: boolean
  }): Promise<{
    intermediatePeerIds: string[]
  }> {
    const message = encode([includeRecipient ? `${await this.address('hopr')}:${payload}` : payload, Date.now()])
    log(`- send | Sending message: ${payload}`)
    await this.node.sendMessage(message, PeerId.createFromB58String(peerId), async () =>
      intermediatePeerIds.map((str) => PeerId.createFromB58String(str)),
    )
    return {
      intermediatePeerIds,
    }
  }

  @Core.mustBeStarted()
  async address(type: 'native' | 'hopr'): Promise<string> {
    if (type === 'native') {
      return this.node.paymentChannels.utils.pubKeyToAccountId(this.node.getId().pubKey.marshal()).then(u8aToHex)
    } else {
      return this.node.getId().toB58String()
    }
  }
}
