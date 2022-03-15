/*
 * Implements HOPR fetching methods on top of REST client.
 */

import HoprClient from './client'

export default class HoprFetcher extends HoprClient {

  constructor(apiPort: number, apiToken: string) {
    super(apiPort, apiToken)
  }

  /*
 * Account API
 */
  public accountWithdraw = (jsonBody: object) => this.postReq("account/withdraw", jsonBody)

  public getBalances = () => this.getReq("account/balances")

  public getAddresses = () => this.getReq("account/addresses")

  /*
   * Aliases API
   */
  public getAliases = () => this.getReq("aliases")

  public setAliases = (peerId, alias) => this.postReq("aliases/", {"peerId": peerId, "alias": alias})


  /*
   * Channels API
   */
  public getChannels = () => this.getReq("channels")

  // DELETE /channels/{peerid}
  public closeChannel = (peerId) => this.delReq("channels/" + peerId)

  // open cmd Opens a payment channel between this node and the counter party provided.
  public setChannels = (peerId, amount) => {
    return this.postReq("channels/", {"peerId": peerId, "amount": amount})
  }

  /*
   * Tickets API
   */
  public redeemTickets = () => this.postReq(`tickets/redeem`, {})

  public getTickets = () =>  this.getReq(`tickets`)

  public getTicketStats = () => this.getReq(`tickets/statistics`)

  /*
   * Messages API
   */
  public signMessage = (msg) => this.postReq("messages/sign", {message: msg})


  public sendMessage = (body, recipient, path) => {
    return this.postReq("messages", {body: body, recipient: recipient, path: path})
  }

  /*
   * Node API
   */
  public getNodeInfo = () => this.getReq("node/info")

  public getNodeVer = () => this.getReq("node/version")

  public pingNodePeer = (peerId) => this.postReq("node/ping", {peerId: peerId})

  /*
   * Settings API
   */
  public getSettings = () => this.getReq("settings")

  public setSettings = (key: string, value: string) => this.putReq(`settings/${key}`, {key: key, value: value})
}




