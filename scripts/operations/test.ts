import { Ganache } from '@hoprnet/hopr-testing'
import { bash } from './utils'
import networks from '../../truffle-networks.json'

export default async () => {
  const ganache = new Ganache({
    port: networks.test.port,
  })

  await ganache.start()
  await bash(`npx truffle test --network test`)
  await ganache.stop()
}
