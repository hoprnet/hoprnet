import EventEmitter from 'events'
import Web3 from 'web3'
import { WebsocketProvider } from 'web3-core'
import { wait } from '../utils'

export enum Events {
  'connected' = 'connected',
  'disconnected' = 'disconnected',
  'reconnected' = 'reconnected'
}
export type IEvents = keyof typeof Events

export const isConnectionError = (err: Error) => {
  return err.message.includes('connection not open') || err.message.includes('CONNECTION ERROR')
}

interface IEventEmitter extends EventEmitter {
  on(event: IEvents, listener: () => void): this
  off(event: IEvents, listener: () => void): this
  once(event: IEvents, listener: () => void): this
  emit(event: IEvents): boolean
}

class CustomWeb3 extends Web3 implements Web3 {
  private reconnecting = false
  private manualDisconnect = false
  public events: IEventEmitter

  constructor(
    private readonly uri: string,
    private readonly ops: {
      reconnection: boolean
      reconnectionDelay: number
    } = {
      reconnection: true,
      reconnectionDelay: 1000
    }
  ) {
    super()

    this.events = new EventEmitter()
    // @TODO: find a better way to do this
    this.connect()
  }

  private disconnected(): Promise<boolean> {
    console.log('web3 disconnected!')
    this.events.emit('disconnected')

    if (this.manualDisconnect) return
    if (this.reconnecting) return
    return this.reconnect()
  }

  // @TODO: add max retries & increasing timer
  private async reconnect(): Promise<boolean> {
    try {
      // @TODO: should return promise
      if (this.reconnecting) return false

      this.reconnecting = true
      console.log('web3 reconnecting..')

      await wait(this.ops.reconnectionDelay)

      return this.connect()
    } catch (err) {
      throw err
    } finally {
      this.reconnecting = false
    }
  }

  public async isConnected(): Promise<boolean> {
    return new Promise<boolean>(async (resolve, reject) => {
      const currentProvider = this.currentProvider as WebsocketProvider | undefined
      if (!currentProvider) return resolve(false)

      try {
        const isListening = await this.eth.net.isListening()
        return resolve(isListening)
      } catch (err) {
        if (err.message.includes('connection not open')) {
          return resolve(false)
        }

        return reject(err)
      }
    })
  }

  // @TODO: add timeout to 'isConnected'
  public async connect(): Promise<boolean> {
    return new Promise<boolean>(async (resolve, reject) => {
      try {
        if (await this.isConnected()) {
          return true
        }

        const currentProvider = this.currentProvider as WebsocketProvider | undefined
        const provider = new Web3.providers.WebsocketProvider(this.uri)

        provider.on('error', this.disconnected.bind(this))
        provider.on('end', this.disconnected.bind(this))

        this.manualDisconnect = false
        this.setProvider(provider)

        while (!(await this.isConnected())) {
          wait(100)
        }

        if (typeof currentProvider !== 'undefined') {
          this.events.emit('reconnected')
        }
        this.events.emit('connected')

        return resolve(true)
      } catch (err) {
        return reject(err)
      }
    })
  }

  public async disconnect(): Promise<void> {
    const provider = this.currentProvider as WebsocketProvider
    if (!provider) return

    this.manualDisconnect = true
    provider.disconnect(0, 'client disconnected manually')
    this.events.emit('disconnected')
  }
}

export default CustomWeb3
