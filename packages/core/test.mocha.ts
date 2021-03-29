import { Ganache } from '@hoprnet/hopr-testing'
import { getAddresses, compile, migrate, fund } from '@hoprnet/hopr-ethereum'

let ganache: Ganache

export const mochaGlobalSetup = async () => {
  ganache = new Ganache()
  await ganache.start()

  await compile()
  await migrate()
  await fund(`--address ${getAddresses()?.localhost?.HoprToken} --accounts-to-fund 5`)
}

export const mochaGlobalTeardown = async () => {
  await ganache.stop()
}
