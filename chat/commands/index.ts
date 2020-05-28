import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'

import CloseChannel from './closeChannel'
import Crawl from './crawl'
import ListCommands from './listCommands'
import ListConnectors from './listConnectors'
import ListOpenChannels from './listOpenChannels'
import OpenChannel from './openChannel'
import Ping from './ping'
import PrintAddress from './printAddress'
import PrintBalance from './printBalance'
import SendMessage from './sendMessage'
import StopNode from './stopNode'
import Version from './version'

export default class Commands {
  closeChannel: CloseChannel
  crawl: Crawl
  listCommands: ListCommands
  listConnectors: ListConnectors
  listOpenChannels: ListOpenChannels
  openChannel: OpenChannel
  ping: Ping
  printAddress: PrintAddress
  printBalance: PrintBalance
  sendMessage: SendMessage
  stopNode: StopNode
  version: Version

  constructor(public node: Hopr<HoprCoreConnector>) {
    this.closeChannel = new CloseChannel(node)
    this.crawl = new Crawl(node)
    this.listCommands = new ListCommands()
    this.listConnectors = new ListConnectors()
    this.listOpenChannels = new ListOpenChannels(node)
    this.openChannel = new OpenChannel(node)
    this.ping = new Ping(node)
    this.printAddress = new PrintAddress(node)
    this.printBalance = new PrintBalance(node)
    this.sendMessage = new SendMessage(node)
    this.stopNode = new StopNode(node)
    this.version = new Version()
  }
}
