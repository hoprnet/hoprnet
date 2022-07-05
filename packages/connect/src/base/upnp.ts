import NatAPI from 'nat-api'
import type { Startable } from '@libp2p/interfaces/startable'

export class UpnpManager implements Startable {
  private client: NatAPI | undefined
  private _isStarted: boolean

  constructor() {
    this._isStarted = false
  }

  public isStarted(): boolean {
    return this._isStarted
  }

  public beforeStart() {
    this.client = new NatAPI()

    // Reduce default timeout
    //@ts-ignore
    this.client._upnpClient.timeout = 700
  }

  public start(): void {
    this._isStarted = true
  }

  /**
   * Tries to get the external IP using UPnP.
   * @returns either the external IP or undefined
   */
  externalIp(): Promise<string | void> {
    return new Promise<string | void>((resolve) => {
      ;(this.client as NatAPI).externalIp((err: any, ip: any) => {
        if (err) {
          resolve()
        }
        resolve(ip)
      })
    })
  }

  /**
   * Tries to open the requested port using UPnP
   * @param port number of the port to open
   * @returns true if port was opened
   */
  async map(port: number): Promise<boolean> {
    const success = await Promise.all(
      (['UDP', 'TCP'] as ('UDP' | 'TCP')[]).map(
        (name) =>
          new Promise<boolean>((resolve) => {
            ;(this.client as NatAPI).map(
              {
                publicPort: port,
                privatePort: port,
                protocol: name,
                description: `hopr ${name}`
              },
              (err: any) => {
                if (err) {
                  return resolve(false)
                }
                resolve(true)
              }
            )
          })
      )
    )

    return success[0] && success[1]
  }

  /**
   * Frees all allocated ports and cleans all opened sockets
   * @returns a promise that resolves once done
   */
  async stop(): Promise<void> {
    if (!this._isStarted) {
      return
    }

    await new Promise<void>((resolve) => (this.client as NatAPI).destroy(resolve))

    this._isStarted = false
  }
}
