import { Ganache } from '@hoprnet/hopr-testing'
import { bash } from './utils'
import networks from '../../truffle-networks'

export default async (...args: any[]) => {
  const ganache = new Ganache({
    port: networks.test.port,
  })

  let command = 'npx truffle test --network test'
  if (args.length > 0) {
    command += ` ${args.join(' ')}`
  }

  await ganache.start()
  await bash(command)
  await ganache.stop()
}
