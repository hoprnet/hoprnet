import Ganache from 'ganache-core'
import { NODE_SEEDS, BOOTSTRAP_SEEDS } from '@hoprnet/hopr-demo-seeds'

const accounts = NODE_SEEDS.concat(BOOTSTRAP_SEEDS)
const balance = Number(1000000000000000000000000).toString(16)

const DEFAULT_OPS: Ganache.IServerOptions = {
  ws: true,
  port: 8545,
  accounts: accounts.map((account) => ({
    secretKey: account,
    balance
  })),
  gasLimit: 0xfffffffffff,
  gasPrice: '1'
}

class CustomGanache {
  private server: ReturnType<typeof Ganache.server>
  private ops: Ganache.IServerOptions

  constructor(customOps: Ganache.IServerOptions = {}) {
    this.ops = {
      ...DEFAULT_OPS,
      ...customOps
    }
  }

  public async start(): Promise<this> {
    return new Promise<this>((resolve, reject) => {
      console.log('Starting ganache instance')

      this.server = Ganache.server(this.ops)

      // @ts-ignore
      this.server.listen(this.ops.port, (err) => {
        const url = `${this.ops.ws ? 'ws' : 'http'}://127.0.0.1:${this.ops.port}`
        console.log(`Network ready at ${url}`)
        if (err) {
          console.log('Error creating ganache', err)
          return reject()
        }
        return resolve(this)
      })
    })
  }

  public async stop(): Promise<this> {
    return new Promise<this>((resolve, reject) => {
      console.log('Closing ganache instance')

      if (typeof this.server === 'undefined') {
        return resolve(this)
      }

      this.server.close((err) => {
        if (err) return reject(err.message)

        console.log('Network closed')
        this.server = undefined
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
