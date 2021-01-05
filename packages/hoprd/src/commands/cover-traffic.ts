import { encode, decode } from 'rlp'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'

const INTERVAL = 1000

/*
 * Generate Cover Traffic
 *
 * We add "useless" packets to our network at a random rate to avoid exposing
 * real packets to be distinguished and/or isolated by a node with some sort of
 * heuristic. Thus, by implementing this "decoy traffic", we reduce the attack
 * vector on our users and increase the general privacy of the network.
 *
 * This is currently a first step implementation that simply sends regular
 * messages through the network to itself, allowing it to also monitor network
 * success metrics.
 *
 */
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
    this.timeout = setTimeout(this.tick.bind(this), INTERVAL) // tick again after interval
  }

  private handleMessage(msg: Uint8Array) {
    const decoded = decode(msg)
    console.log(decoded)
    if (decoded[0] === this.identifier) {
      const ts = decoded[2]
      this.totalLatency += Date.now() - ts
      this.messagesReceived++
    }
  }

  private stats(): string {
    if (this.messagesReceived < 1) {
      return `${this.messagesSent} messages sent, no messages received`
    }
    const reliability = ((this.messagesReceived / this.messagesSent) * 100).toFixed(2)
    const latency = this.totalLatency / this.messagesReceived
    return `${this.messagesSent} messages sent, ` + `reliability = ${reliability}%, average latency is ${latency}`
  }

  public async execute(query: string): Promise<string> {
    if (query === 'start' && !this.timeout) {
      if (!this.registered) {
        // Intercept message event to monitor success rate.
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
