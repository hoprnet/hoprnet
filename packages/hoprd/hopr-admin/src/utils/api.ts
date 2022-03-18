import { type ApiPath } from '.'

export type Channel = {
  type: string
  channelId: string
  peerId: string
  status: string
  balance: string
}

/**
 * Wrapper around fetch that allows you to call various HOPRd API endpoints.
 */
export default class API {
  constructor(private apiEndpoint: string, private apiToken?: string) {}

  private getEndpoint(apiPath: ApiPath): string {
    return new URL(apiPath, this.apiEndpoint).href
  }

  private getHeaders(): Headers {
    const headers = new Headers()
    headers.set('Content-Type', 'application/json')
    headers.set('Accept-Content', 'application/json')
    if (this.apiToken) headers.set('Authorization', 'Basic ' + btoa(this.apiToken))
    return headers
  }

  public updateConfig(apiEndpoint: string, apiToken?: string): void {
    this.apiEndpoint = apiEndpoint
    this.apiToken = apiToken
  }

  public async getReq(apiPath: ApiPath): Promise<Response> {
    return fetch(this.getEndpoint(apiPath), {
      headers: this.getHeaders()
    })
  }

  public async postReq(apiPath: ApiPath, payload: any): Promise<Response> {
    return fetch(this.getEndpoint(apiPath), {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify(payload)
    })
  }

  public async putReq(apiPath: ApiPath, payload: any): Promise<Response> {
    return fetch(this.getEndpoint(apiPath), {
      method: 'PUT',
      headers: this.getHeaders(),
      body: JSON.stringify(payload)
    })
  }

  public async delReq(apiPath: ApiPath): Promise<Response> {
    return fetch(this.getEndpoint(apiPath), {
      method: 'DELETE',
      headers: this.getHeaders()
    })
  }

  // account API
  public async withdraw(amount: string, currency: string, recipient: string) {
    return this.postReq('/api/v2/account/withdraw', { amount, currency, recipient })
  }
  public async getBalances(): Promise<{
    hopr: string
    native: string
  }> {
    return this.getReq('/api/v2/account/balances').then((res) => res.json())
  }
  public async getAddresses(): Promise<{
    hopr: string
    native: string
  }> {
    return this.getReq('/api/v2/account/addresses').then((res) => res.json())
  }

  // aliases API
  public async getAliases(): Promise<Record<string, string>> {
    return this.getReq('/api/v2/aliases').then((res) => res.json())
  }
  public async setAlias(peerId: string, alias: string) {
    return this.postReq('/api/v2/aliases', { peerId, alias })
  }

  // channels API
  public async getChannels(): Promise<{
    incoming: Channel[]
    outgoing: Channel[]
  }> {
    return this.getReq('/api/v2/channels').then((res) => res.json())
  }
  public async closeChannel(peerId: string): Promise<
    {
      json: () => Promise<{
        receipt: string
        channelStatus: string
      }>
    } & Response
  > {
    return this.delReq(`/api/v2/channels/${peerId}`)
  }
  public async openChannel(peerId: string, amount: string) {
    return this.postReq('/api/v2/channels', { peerId, amount })
  }

  // tickets API
  public async redeemTickets() {
    return this.postReq('/api/v2/tickets/redeem', {})
  }
  public async getTickets() {
    return this.getReq('/api/v2/tickets')
  }
  public async getTicketStats() {
    return this.getReq('/api/v2/tickets/statistics')
  }

  // messages API
  public async signMessage(msg: string) {
    return this.postReq('/api/v2/messages/sign', { message: msg })
  }
  public async sendMessage(body: string, recipient: string, path: string[]) {
    return this.postReq('/api/v2/messages', { body: body, recipient: recipient, path: path })
  }

  // node API
  public async getInfo(): Promise<{
    channelClosurePeriod: number
    announcedAddress: string[]
    listeningAddress: string[]
    environment: string
    network: string
    hoprToken: string
    hoprChannels: string
  }> {
    return this.getReq('/api/v2/node/info').then((res) => res.json())
  }
  public async getVersion() {
    return this.getReq('/api/v2/node/version')
  }
  public async ping(peerId: string) {
    return this.postReq('/api/v2/node/ping', { peerId })
  }
  public async getPeers(): Promise<{
    connected: {
      peerId: string
      quality: number
    }[]
    announced: {
      peerId: string
      quality: number
    }[]
  }> {
    return this.getReq('/api/v2/node/peers').then((res) => res.json())
  }

  // settings API
  public async getSettings(): Promise<{
    strategy: string
    includeRecipient: boolean
  }> {
    return this.getReq('/api/v2/settings').then((res) => res.json())
  }
  public async setSetting(key: string, value: string) {
    return this.putReq(`/api/v2/settings/${key}`, { key: key, value: value })
  }
}
