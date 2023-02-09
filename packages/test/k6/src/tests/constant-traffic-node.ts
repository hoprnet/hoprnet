import { check, fail } from 'k6'
import { RefinedResponse } from 'k6/http'
import { Counter } from 'k6/metrics'
import { randomString } from './test-helpers'
import { ChannelResponse, HoprNode, Peer } from '../api/hoprd.types'
import {
  AccountApi,
  ChannelsApi,
  MesssagesApi,
  NodeApi,
  Addresses,
  NodeBalance,
  SendMessageRequest,
  TOKEN_HOPR_MIN,
  TOKEN_NATIVE_MIN
} from '../api/index'

export class ConstantTrafficNode {
  private node: HoprNode
  private accountApi: AccountApi
  private nodeApi: NodeApi
  private channelsApi: ChannelsApi
  private messagesApi: MesssagesApi

  public addresses: Addresses
  private peers: Peer[] = []

  public constructor(
    node: HoprNode,
    dataPool?: { addresses: { [key: string]: Addresses }; peers: { [key: string]: Peer[] } }
  ) {
    this.node = node
    this.accountApi = new AccountApi(node)
    this.nodeApi = new NodeApi(node)
    this.channelsApi = new ChannelsApi(node)
    this.messagesApi = new MesssagesApi(node)

    if (dataPool !== undefined) {
      this.addresses = dataPool.addresses[this.node.alias]
      this.peers = dataPool.peers[this.node.alias]
    } else {
      this.addresses = { native: '', hopr: '' }
    }
  }

  public checkHealth() {
    this.checkAddresses()
    this.checkBalance()
    this.checkConnectivity()
    this.checkChannels()
  }

  public getQualityPeers(peerType: string): Peer[] {
    return this.getPeers(peerType).filter((peer) => peer.quality >= this.node.thresholds.peersQuality)
  }

  public sendHopMessage(hops: number, successCounter: Counter, failedCounter: Counter) {
    const messageRequest: SendMessageRequest = {
      body: randomString(15),
      recipient: this.getRandomPeer().peerId,
      path: this.getPath(hops)
    }
    const messageResponse = this.messagesApi.sendMessage(messageRequest)

    if (check(messageResponse, { 'Message sent': () => messageResponse.status === 202 })) {
      console.log(`Message sent from node '${this.node.alias}' using path ${JSON.stringify(messageRequest.path)}} `)
      successCounter.add(1)
    } else {
      console.error(`Failed to send message. Details: ${JSON.stringify(messageResponse)}`)
      failedCounter.add(1, { request: JSON.stringify(messageRequest), response: JSON.stringify(messageResponse) })
    }
  }

  private getPath(hops: number): string[] {
    let path: string[] = []
    while (hops > 1) {
      let pathPeer = this.getRandomPeer()
      if (!(hops == 2 && this.addresses?.hopr == pathPeer.peerId)) {
        path.push(pathPeer.peerId)
        hops--
      }
    }
    return path
  }

  private getRandomPeer(): Peer {
    return this.peers[Math.floor(Math.random() * this.peers.length)]
  }

  private checkAddresses() {
    // Get Addresses
    const responseAddresses: RefinedResponse<'text'> = this.accountApi.getAddresses()
    if (!check(responseAddresses, { 'addresses retrieved': (r) => r.status === 200 })) {
      fail(`Unable to get node addresses for '${this.node.alias}`)
    }
    this.addresses = JSON.parse(responseAddresses.body)
  }

  private checkBalance() {
    // Get Balance
    const responseBalance: RefinedResponse<'text'> = this.accountApi.getBalance()
    if (!check(responseBalance, { 'balance retrieved': (r) => r.status === 200 })) {
      console.error(responseBalance.body)
      fail(`Unable to get node balance for '${this.node.alias}`)
    }
    const balance: NodeBalance = new NodeBalance(JSON.parse(responseBalance.body))
    if (balance.hopr < TOKEN_HOPR_MIN) {
      fail(`Node '${this.node.alias} does not have enough hopr tokens (current balance is ${balance.hopr})`)
    }
    if (balance.native < TOKEN_NATIVE_MIN) {
      fail(`Node '${this.node.alias} does not have enough native tokens (current balance is ${balance.native})`)
    }
  }

  private checkConnectivity() {
    // Get connectivity
    const nodeInfoResponse = this.nodeApi.getInfo()
    if (!check(nodeInfoResponse, { 'node info received': (r) => r.status === 200 })) {
      fail(`Unable to get node info for '${this.node.alias}`)
    }
    const nodeInfo: any = JSON.parse(nodeInfoResponse.body)
    if (
      !check(nodeInfo, {
        'green node connectivity': (ni) => ni.connectivityStatus === this.node.thresholds.connectivityStatus.valueOf()
      })
    ) {
      fail(`Node '${this.node.alias}' is not in green network connectivity`)
    }
  }

  private checkChannels() {
    // Get Channels
    const getChannelsResponse = this.channelsApi.getChannels()
    if (!check(getChannelsResponse, { 'channels received': (r) => r.status === 200 })) {
      console.log(`Channels ${JSON.stringify(getChannelsResponse)}`)
      fail(`Unable to get open channels for node '${this.node.alias}`)
    }
    const channelsResponse: ChannelResponse = JSON.parse(getChannelsResponse.body)
    if (
      !check(channelsResponse.incoming, {
        'Not enough incomming channels opened': (ic) => ic.length >= this.node.thresholds.incommingOpenChannels
      })
    ) {
      fail(`Node '${this.node.alias}' does not have enough incomming channels opened`)
    }
    if (
      !check(channelsResponse.outgoing, {
        'Not enough outgoing channels opened': (oc) => oc.length >= this.node.thresholds.outgoingOpenChannels
      })
    ) {
      fail(`Node '${this.node.alias}' does not have enough outgoing channels opened`)
    }
  }

  private getPeers(peerType: string): Peer[] {
    const peersResponse = this.nodeApi.getPeers()
    if (!check(peersResponse, { 'peers received': (r) => r.status === 200 })) {
      fail(`Node '${this.node.alias} has failed to received peers`)
    }
    return peerType === 'connected'
      ? JSON.parse(peersResponse.body).connected
      : JSON.parse(peersResponse.body).announced
  }
}
