import Ganache from 'ganache-core'
import { NODE_SEEDS, BOOTSTRAP_SEEDS } from '@hoprnet/hopr-demo-seeds'

const accounts = NODE_SEEDS.concat(BOOTSTRAP_SEEDS)
const balance = Number(1000000000000000000000000).toString(16)

let server:
  | {
      listen: (port: number, cb: (err: Error, blockchain: any) => void) => void
      close: (cb: (err: Error) => void) => void
    }
  | undefined

const DEFAULT_OPS: Ganache.IServerOptions = {
  ws: true,
  port: 9545,
  accounts: accounts.map((account) => ({
    secretKey: account,
    balance,
  })),
  gasLimit: 0xfffffffffff,
  gasPrice: '1',
}

class CustomGanache {
  private ops: Ganache.IServerOptions

  constructor(customOps: Ganache.IServerOptions = {}) {
    this.ops = {
      ...DEFAULT_OPS,
      ...customOps,
    }
  }

  public async start(): Promise<this> {
    return new Promise<this>((resolve, reject) => {
      console.log('Starting ganache instance')

      server = Ganache.server(this.ops)

      server.listen(this.ops.port, (err) => {
        if (err) return reject(err.message)

        const url = `${this.ops.ws ? 'ws' : 'http'}://127.0.0.1:${this.ops.port}`
        console.log(`Network ready at ${url}`)
        return resolve(this)
      })
    })
  }

  public async stop(): Promise<this> {
    return new Promise<this>((resolve, reject) => {
      console.log('Closing ganache instance')

      if (typeof server === 'undefined') {
        return resolve(this)
      }

      server.close((err) => {
        if (err) return reject(err.message)

        console.log('Network closed')
        server = undefined
        return resolve(this)
      })
    })
  }

  public async restart(): Promise<this> {
    await this.stop()
    await this.start()

    return this
  }
}

export default CustomGanache
