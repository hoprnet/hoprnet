export default class HoprClient {
  private readonly apiEndpoint: string
  private readonly apiPort: number
  private readonly apiToken: string

  constructor(apiPort: number, apiToken: string) {
    this.apiPort = apiPort
    this.apiToken = apiToken
    this.apiEndpoint = `http://localhost:${apiPort}/api/v2/`
  }

  public getReq = (apiSuffix: string) => {
    return fetch(this.apiEndpoint + apiSuffix, {
      headers: {
        Accept: 'application/json',
        'x-auth-token': this.apiToken
      }
    }).then((res) => res.json())
  }

  postReq = (apiSuffix: string, jsonBody: object) => {
    return fetch(this.apiEndpoint + apiSuffix, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-auth-token': this.apiToken
      },
      body: JSON.stringify(jsonBody)
    })
  }

  putReq = (apiSuffix: string, jsonBody: object) => {
    return fetch(this.apiEndpoint + apiSuffix, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'x-auth-token': this.apiToken
      },
      body: JSON.stringify(jsonBody)
    })
  }

  delReq = (apiSuffix: string) => {
    return fetch(this.apiEndpoint + apiSuffix, {
      method: 'DELETE',
      headers: {
        Accept: 'application/json',
        'x-auth-token': this.apiToken
      }
    })
  }
}
