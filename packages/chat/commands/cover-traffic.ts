import { encode, decode } from 'rlp'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'

const INTERVAL = 1000

export class CoverTraffic extends AbstractCommand {
  private seq: number = 0
  private timeout: NodeJS.Timeout | undefined
  private registered: boolean

  private messagesSent: number
  private messagesReceived: number
  private totalLatency: number

  private identifier: number
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
    this.messagesSent = 0
    this.messagesReceived = 0
    this.totalLatency = 0
    this.identifier = Math.random()
  }

  public name() {
    return 'cover-traffic'
  }

  public help() {
    return 'Generate chaff messages to provide cover (start/stop)'
  }

  private tick() {
    const payload = encode([this.identifier, this.seq++, Date.now()])
    this.node.sendMessage(payload, this.node.getId())
    this.messagesSent++
    setTimeout(this.tick.bind(this), INTERVAL)
  }

  private handleMessage(msg: Uint8Array) {
    const decoded = decode(msg)
    if (decoded[0] === this.identifier) {
      const ts = decoded[2]
      this.totalLatency += Date.now() - ts
      this.messagesReceived++
    }
  }

  private stats(): string {
    const reliability = ((this.messagesReceived / this.messagesSent) * 100).toFixed(2)
    const latency = this.totalLatency / this.messagesReceived
    return `${this.messagesSent} messages sent, ` + `reliability = ${reliability}%, average latency is ${latency}`
  }

  public async execute(query: string): Promise<string> {
    if (query === 'start' && !this.timeout) {
      if (!this.registered) {
        this.node.on('hopr:message', this.handleMessage.bind(this))
        this.registered = true
      }
      setTimeout(this.tick.bind(this), INTERVAL)
      return 'started'
    }
    if (query === 'stop' && this.timeout) {
      clearTimeout(this.timeout)
      return 'stopped'
    }
    if (query === 'stats') {
      return this.stats()
    }
  }
}
