import { Ganache } from '@hoprnet/hopr-testing'
import { migrate, fund } from '@hoprnet/hopr-ethereum'

let ganache

export const mochaGlobalSetup = async () => {
  ganache = new Ganache()
  await ganache.start()
  await migrate()
  await fund(4)
}

export const mochaGlobalTeardown = async () => {
  await ganache.stop()
}
