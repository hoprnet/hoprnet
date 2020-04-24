// example how to use hopr-ethereum API
import { Ganache, migrate, fund } from '@hoprnet/hopr-ethereum'

const start = async () => {
  const ganache = new Ganache()

  await ganache.start()
  await migrate()
  await fund()

  await ganache.restart()
  await migrate()
  await fund()

  await ganache.stop()
}

start().catch(console.error)
