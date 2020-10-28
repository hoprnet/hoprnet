import { Ganache } from '@hoprnet/hopr-testing'
import { migrate, fund } from '@hoprnet/hopr-ethereum'

export const mochaGlobalSetup = async () => {
  let ganache = new Ganache()
  await ganache.start()
  await migrate()
  await fund(4)
}

export const mochaGlobalTeardown = async () => {
  await ganache.stop()
}
