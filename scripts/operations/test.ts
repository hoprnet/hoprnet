import { Ganache, bash } from './utils'

export default async () => {
  const ganache = new Ganache({
    port: 9545
  })

  await ganache.start()
  await bash(`npx truffle test --network test`)
  await ganache.stop()
}
