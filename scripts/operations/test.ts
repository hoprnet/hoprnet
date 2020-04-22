import { Ganache, bash } from './utils'
const config: any = require('../../truffle-config')

export default async () => {
  const ganache = new Ganache({
    port: config.networks.test.port
  })

  await ganache.start()
  await bash(`npx truffle test --network test`)
  await ganache.stop()
}
