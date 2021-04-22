import { Ganache } from '@hoprnet/hopr-testing'
import { getContracts, compile, migrate, fund } from '@hoprnet/hopr-ethereum'

let ganache: Ganache

export const mochaGlobalSetup = async () => {
  ganache = new Ganache()
  await ganache.start()

  await compile()
  await migrate()
  await fund(`--address ${getContracts().localhost.HoprToken.address} --accounts-to-fund 5`)
}

export const mochaGlobalTeardown = async () => {
  await ganache.stop()
}
